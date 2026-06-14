#[allow(deprecated)]
use solana_program::system_instruction;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::invoke,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    accounts::{
        load_job, require_key, require_signer, require_spl_token_program, require_system_program,
        require_token_account, save_job, transfer_spl_tokens,
    },
    errors::IcpxError,
    events::{emit, IcpxEvent},
    state::JobStatus,
};

pub fn process_fund_job(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let requester = next_account_info(account_info_iter)?;
    let job_account = next_account_info(account_info_iter)?;
    require_signer(requester)?;
    let mut job = load_job(program_id, job_account)?;
    require_key(requester.key, &job.requester, IcpxError::InvalidSigner)?;

    if job.status != JobStatus::Created {
        return Err(IcpxError::InvalidStatus.into());
    }
    if Clock::get()?.slot > job.expiry_slot {
        return Err(IcpxError::JobExpired.into());
    }

    let budget = job.max_budget_amount()?;
    let escrow_vault = if job.payment_asset.is_native_sol() {
        let system_program = next_account_info(account_info_iter)?;
        require_system_program(system_program)?;
        invoke(
            &system_instruction::transfer(requester.key, job_account.key, budget),
            &[
                requester.clone(),
                job_account.clone(),
                system_program.clone(),
            ],
        )?;
        *job_account.key
    } else {
        let requester_token_account = next_account_info(account_info_iter)?;
        let escrow_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let mint = job
            .payment_asset
            .mint()
            .ok_or(IcpxError::InvalidPaymentAsset)?;

        require_spl_token_program(token_program)?;
        let requester_token =
            require_token_account(requester_token_account, &mint, requester.key)?;
        if requester_token.amount < budget {
            return Err(IcpxError::EscrowUnderfunded.into());
        }
        require_token_account(escrow_token_account, &mint, job_account.key)?;

        transfer_spl_tokens(
            token_program,
            requester_token_account,
            escrow_token_account,
            requester,
            None,
            budget,
        )?;
        *escrow_token_account.key
    };

    job.escrow_vault = escrow_vault;
    job.escrow_funded_amount = budget;
    job.status = JobStatus::Funded;
    save_job(job_account, &job)?;

    emit(&IcpxEvent::JobFunded {
        job: *job_account.key,
        requester: *requester.key,
        payment_asset: job.payment_asset,
        escrow_funded_amount: budget,
        escrow_vault,
    });

    Ok(())
}

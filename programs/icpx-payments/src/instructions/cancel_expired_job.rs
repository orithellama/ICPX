use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    accounts::{load_job, require_key, save_job},
    errors::IcpxError,
    events::{emit, IcpxEvent},
    instructions::settlement::{refund_remaining_sol, refund_remaining_tokens},
    state::JobStatus,
};

pub fn process_cancel_expired_job(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let job_account = next_account_info(account_info_iter)?;
    let requester_refund_account = next_account_info(account_info_iter)?;

    let mut job = load_job(program_id, job_account)?;

    if matches!(
        job.status,
        JobStatus::Completed | JobStatus::Cancelled | JobStatus::Expired
    ) {
        return Err(IcpxError::InvalidStatus.into());
    }

    let clock = Clock::get()?;
    if clock.slot <= job.expiry_slot {
        return Err(IcpxError::JobNotExpired.into());
    }

    job.status = JobStatus::Expired;
    let refund_amount = if job.payment_asset.is_native_sol() {
        require_key(
            requester_refund_account.key,
            &job.requester,
            IcpxError::InvalidSigner,
        )?;
        refund_remaining_sol(job_account, requester_refund_account, &mut job)?
    } else {
        let escrow_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        refund_remaining_tokens(
            job_account,
            requester_refund_account,
            escrow_token_account,
            token_program,
            &mut job,
        )?
    };
    save_job(job_account, &job)?;

    emit(&IcpxEvent::JobExpired {
        job: *job_account.key,
        payment_asset: job.payment_asset,
        refund_amount,
    });

    Ok(())
}

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    account_utils::{load_job, require_key, require_signer, save_job},
    errors::IcpxError,
    events::{emit, IcpxEvent},
    instructions::{
        settlement::{settle_receipt_sol, settle_receipt_tokens},
        GpuStreamReceipt,
    },
    state::JobStatus,
};

pub fn process_settle_stream(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    receipt: GpuStreamReceipt,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let receipt_authority = next_account_info(account_info_iter)?;
    let job_account = next_account_info(account_info_iter)?;
    let provider_payment_account = next_account_info(account_info_iter)?;
    let protocol_fee_account = next_account_info(account_info_iter)?;

    require_signer(receipt_authority)?;
    let mut job = load_job(program_id, job_account)?;
    require_key(
        receipt_authority.key,
        &job.receipt_authority,
        IcpxError::InvalidSigner,
    )?;

    if job.status != JobStatus::Running {
        return Err(IcpxError::InvalidStatus.into());
    }
    if Clock::get()?.slot > job.expiry_slot {
        return Err(IcpxError::JobExpired.into());
    }

    let quote = if job.payment_asset.is_native_sol() {
        require_key(
            provider_payment_account.key,
            &job.provider,
            IcpxError::InvalidSigner,
        )?;
        settle_receipt_sol(
            job_account,
            provider_payment_account,
            protocol_fee_account,
            &mut job,
            receipt,
        )?
    } else {
        let escrow_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        settle_receipt_tokens(
            job_account,
            provider_payment_account,
            escrow_token_account,
            protocol_fee_account,
            token_program,
            &mut job,
            receipt,
        )?
    };
    job.result_hash = receipt.result_hash;
    save_job(job_account, &job)?;

    emit(&IcpxEvent::StreamSettled {
        job: *job_account.key,
        provider: job.provider,
        receipt_nonce: receipt.receipt_nonce,
        cumulative_units: receipt.cumulative_units,
        new_units: quote.new_units,
        payment_asset: job.payment_asset,
        payment_amount: quote.provider_payment_amount,
        protocol_fee_amount: quote.protocol_fee_amount,
        total_paid_amount: job.total_paid_amount,
        total_protocol_fee_amount: job.total_protocol_fee_amount,
    });

    Ok(())
}

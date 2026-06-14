use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    accounts::{load_job, require_key, require_signer, save_job},
    errors::IcpxError,
    events::{emit, IcpxEvent},
    instructions::{
        settlement::{refund_remaining, settle_receipt},
        GpuStreamReceipt,
    },
    state::JobStatus,
};

pub fn process_complete_job(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    receipt: GpuStreamReceipt,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority = next_account_info(account_info_iter)?;
    let job_account = next_account_info(account_info_iter)?;
    let provider = next_account_info(account_info_iter)?;
    let requester = next_account_info(account_info_iter)?;

    require_signer(authority)?;
    let mut job = load_job(program_id, job_account)?;
    require_key(provider.key, &job.provider, IcpxError::InvalidSigner)?;
    require_key(requester.key, &job.requester, IcpxError::InvalidSigner)?;

    if job.status != JobStatus::Running && job.status != JobStatus::Disputed {
        return Err(IcpxError::InvalidStatus.into());
    }

    let authority_can_complete = *authority.key == job.receipt_authority
        || *authority.key == job.requester
        || *authority.key == job.provider;
    if !authority_can_complete {
        return Err(IcpxError::InvalidSigner.into());
    }
    if receipt.cumulative_units < job.settled_units {
        return Err(IcpxError::InvalidReceipt.into());
    }
    if receipt.cumulative_units > job.settled_units && *authority.key != job.receipt_authority {
        return Err(IcpxError::InvalidSigner.into());
    }

    if receipt.cumulative_units > job.settled_units {
        settle_receipt(job_account, provider, &mut job, receipt)?;
    }

    job.result_hash = receipt.result_hash;
    job.status = JobStatus::Completed;
    let refund_lamports = refund_remaining(job_account, requester, &mut job)?;
    save_job(job_account, &job)?;

    emit(&IcpxEvent::JobCompleted {
        job: *job_account.key,
        final_units: receipt.cumulative_units,
        total_paid_lamports: job.total_paid_lamports,
        refund_lamports,
        result_hash: receipt.result_hash,
    });

    Ok(())
}

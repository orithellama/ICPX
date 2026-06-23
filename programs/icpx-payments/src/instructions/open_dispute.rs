use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{
    account_utils::{load_job, require_signer, save_job},
    errors::IcpxError,
    events::{emit, IcpxEvent},
    state::JobStatus,
};

pub fn process_open_dispute(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let signer = next_account_info(account_info_iter)?;
    let job_account = next_account_info(account_info_iter)?;

    require_signer(signer)?;
    let mut job = load_job(program_id, job_account)?;
    let signer_is_party = *signer.key == job.requester || *signer.key == job.provider;
    if !signer_is_party {
        return Err(IcpxError::InvalidSigner.into());
    }
    if job.status != JobStatus::Running {
        return Err(IcpxError::InvalidStatus.into());
    }

    let previous_status = job.status;
    job.status = JobStatus::Disputed;
    save_job(job_account, &job)?;

    emit(&IcpxEvent::DisputeOpened {
        job: *job_account.key,
        signer: *signer.key,
        previous_status,
    });

    Ok(())
}

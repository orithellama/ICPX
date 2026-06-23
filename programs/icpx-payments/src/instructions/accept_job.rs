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
    state::JobStatus,
};

pub fn process_accept_job(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let provider = next_account_info(account_info_iter)?;
    let job_account = next_account_info(account_info_iter)?;

    require_signer(provider)?;
    let mut job = load_job(program_id, job_account)?;
    require_key(provider.key, &job.provider, IcpxError::InvalidSigner)?;

    if job.status != JobStatus::Funded {
        return Err(IcpxError::InvalidStatus.into());
    }

    let clock = Clock::get()?;
    if clock.slot > job.expiry_slot {
        return Err(IcpxError::JobExpired.into());
    }

    job.start_slot = clock.slot;
    job.status = JobStatus::Running;
    save_job(job_account, &job)?;

    emit(&IcpxEvent::JobAccepted {
        job: *job_account.key,
        provider: *provider.key,
        start_slot: clock.slot,
    });

    Ok(())
}

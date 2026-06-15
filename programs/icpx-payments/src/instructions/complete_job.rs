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
        settlement::{
            refund_remaining_sol, refund_remaining_tokens, settle_receipt_sol,
            settle_receipt_tokens,
        },
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
    let provider_payment_account = next_account_info(account_info_iter)?;
    let requester_refund_account = next_account_info(account_info_iter)?;
    let protocol_fee_account = next_account_info(account_info_iter)?;

    require_signer(authority)?;
    let mut job = load_job(program_id, job_account)?;

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

    let refund_amount = if job.payment_asset.is_native_sol() {
        require_key(
            provider_payment_account.key,
            &job.provider,
            IcpxError::InvalidSigner,
        )?;
        require_key(
            requester_refund_account.key,
            &job.requester,
            IcpxError::InvalidSigner,
        )?;

        if receipt.cumulative_units > job.settled_units {
            settle_receipt_sol(
                job_account,
                provider_payment_account,
                protocol_fee_account,
                &mut job,
                receipt,
            )?;
        }

        job.result_hash = receipt.result_hash;
        job.status = JobStatus::Completed;
        refund_remaining_sol(job_account, requester_refund_account, &mut job)?
    } else {
        let escrow_token_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        if receipt.cumulative_units > job.settled_units {
            settle_receipt_tokens(
                job_account,
                provider_payment_account,
                escrow_token_account,
                protocol_fee_account,
                token_program,
                &mut job,
                receipt,
            )?;
        }

        job.result_hash = receipt.result_hash;
        job.status = JobStatus::Completed;
        refund_remaining_tokens(
            job_account,
            requester_refund_account,
            escrow_token_account,
            token_program,
            &mut job,
        )?
    };
    save_job(job_account, &job)?;

    emit(&IcpxEvent::JobCompleted {
        job: *job_account.key,
        final_units: receipt.cumulative_units,
        payment_asset: job.payment_asset,
        total_paid_amount: job.total_paid_amount,
        total_protocol_fee_amount: job.total_protocol_fee_amount,
        refund_amount,
        result_hash: receipt.result_hash,
    });

    Ok(())
}

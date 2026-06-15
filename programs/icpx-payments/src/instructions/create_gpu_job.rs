#[allow(deprecated)]
use solana_program::system_instruction;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::invoke_signed,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    accounts::{
        require_nonzero_hash, require_nonzero_pubkey, require_signer, require_system_program,
        save_job,
    },
    constants::{EMPTY_HASH, JOB_SEED, PROGRAM_VERSION, PROTOCOL_FEE_BASIS_POINTS},
    errors::IcpxError,
    events::{emit, IcpxEvent},
    instructions::CreateGpuJobArgs,
    math::checked_payment_amount,
    state::{JobState, JobStatus},
};

pub fn process_create_gpu_job(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CreateGpuJobArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let requester = next_account_info(account_info_iter)?;
    let job_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    require_signer(requester)?;
    require_system_program(system_program)?;
    require_nonzero_pubkey(&args.provider)?;
    require_nonzero_pubkey(&args.receipt_authority)?;
    require_nonzero_hash(&args.gpu_profile_hash)?;
    require_nonzero_hash(&args.nvidia_api_hash)?;

    if args.price_per_unit == 0 || args.max_units == 0 {
        return Err(IcpxError::InvalidTerms.into());
    }
    checked_payment_amount(args.max_units, args.price_per_unit)?;

    let clock = Clock::get()?;
    if args.expiry_slot <= clock.slot {
        return Err(IcpxError::InvalidTerms.into());
    }

    let nonce_bytes = args.client_nonce.to_le_bytes();
    let seeds = [
        JOB_SEED,
        requester.key.as_ref(),
        args.provider.as_ref(),
        nonce_bytes.as_ref(),
    ];
    let (expected_job, bump) = Pubkey::find_program_address(&seeds, program_id);
    if expected_job != *job_account.key {
        return Err(IcpxError::InvalidPda.into());
    }

    let rent_lamports = Rent::get()?.minimum_balance(JobState::LEN);
    invoke_signed(
        &system_instruction::create_account(
            requester.key,
            job_account.key,
            rent_lamports,
            JobState::LEN as u64,
            program_id,
        ),
        &[
            requester.clone(),
            job_account.clone(),
            system_program.clone(),
        ],
        &[&[
            JOB_SEED,
            requester.key.as_ref(),
            args.provider.as_ref(),
            nonce_bytes.as_ref(),
            &[bump],
        ]],
    )?;

    let job = JobState {
        version: PROGRAM_VERSION,
        bump,
        requester: *requester.key,
        provider: args.provider,
        receipt_authority: args.receipt_authority,
        escrow_vault: if args.payment_asset.is_native_sol() {
            *job_account.key
        } else {
            Pubkey::default()
        },
        metadata_hash: args.metadata_hash,
        gpu_profile_hash: args.gpu_profile_hash,
        nvidia_api_hash: args.nvidia_api_hash,
        result_hash: EMPTY_HASH,
        metering_unit: args.metering_unit,
        payment_asset: args.payment_asset,
        client_nonce: args.client_nonce,
        price_per_unit: args.price_per_unit,
        max_units: args.max_units,
        escrow_funded_amount: 0,
        settled_units: 0,
        total_paid_amount: 0,
        total_protocol_fee_amount: 0,
        total_refunded_amount: 0,
        created_slot: clock.slot,
        start_slot: 0,
        expiry_slot: args.expiry_slot,
        last_receipt_nonce: 0,
        status: JobStatus::Created,
    };
    save_job(job_account, &job)?;

    emit(&IcpxEvent::GpuJobCreated {
        job: *job_account.key,
        requester: *requester.key,
        provider: args.provider,
        receipt_authority: args.receipt_authority,
        gpu_profile_hash: args.gpu_profile_hash,
        nvidia_api_hash: args.nvidia_api_hash,
        metering_unit: args.metering_unit,
        payment_asset: args.payment_asset,
        price_per_unit: args.price_per_unit,
        protocol_fee_basis_points: PROTOCOL_FEE_BASIS_POINTS,
        max_units: args.max_units,
        expiry_slot: args.expiry_slot,
    });

    Ok(())
}

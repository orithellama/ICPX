use solana_program::account_info::AccountInfo;

use crate::{
    account_utils::{
        require_key, require_spl_token_program, require_token_account, transfer_lamports,
        transfer_spl_tokens,
    },
    constants::{protocol_multisig, JOB_SEED},
    errors::IcpxError,
    instructions::GpuStreamReceipt,
    math::{quote_stream_settlement, SettlementQuote},
    state::JobState,
};

pub fn settle_receipt_sol(
    job_account: &AccountInfo,
    provider: &AccountInfo,
    protocol_fee_account: &AccountInfo,
    job: &mut JobState,
    receipt: GpuStreamReceipt,
) -> Result<SettlementQuote, solana_program::program_error::ProgramError> {
    if !job.payment_asset.is_native_sol() {
        return Err(IcpxError::InvalidPaymentAsset.into());
    }

    let quote = quote_receipt(job, receipt)?;
    require_key(
        protocol_fee_account.key,
        &protocol_multisig(),
        IcpxError::InvalidProtocolFeeAccount,
    )?;
    transfer_lamports(job_account, provider, quote.provider_payment_amount)?;
    transfer_lamports(job_account, protocol_fee_account, quote.protocol_fee_amount)?;
    record_receipt(
        job,
        receipt,
        quote.provider_payment_amount,
        quote.protocol_fee_amount,
    )?;
    Ok(quote)
}

pub fn settle_receipt_tokens<'a>(
    job_account: &AccountInfo<'a>,
    provider_token_account: &AccountInfo<'a>,
    escrow_token_account: &AccountInfo<'a>,
    protocol_fee_token_account: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    job: &mut JobState,
    receipt: GpuStreamReceipt,
) -> Result<SettlementQuote, solana_program::program_error::ProgramError> {
    let mint = job
        .payment_asset
        .mint()
        .ok_or(IcpxError::InvalidPaymentAsset)?;
    require_spl_token_program(token_program)?;
    require_key(
        escrow_token_account.key,
        &job.escrow_vault,
        IcpxError::InvalidEscrowVault,
    )?;

    let escrow = require_token_account(
        escrow_token_account,
        &mint,
        job_account.key,
        token_program.key,
    )?;
    require_token_account(
        provider_token_account,
        &mint,
        &job.provider,
        token_program.key,
    )?;
    require_token_account(
        protocol_fee_token_account,
        &mint,
        &protocol_multisig(),
        token_program.key,
    )?;

    let quote = quote_receipt(job, receipt)?;
    if escrow.amount < quote.gross_payment_amount {
        return Err(IcpxError::EscrowUnderfunded.into());
    }

    transfer_from_token_escrow(
        token_program,
        escrow_token_account,
        provider_token_account,
        job_account,
        job,
        quote.provider_payment_amount,
    )?;
    transfer_from_token_escrow(
        token_program,
        escrow_token_account,
        protocol_fee_token_account,
        job_account,
        job,
        quote.protocol_fee_amount,
    )?;
    record_receipt(
        job,
        receipt,
        quote.provider_payment_amount,
        quote.protocol_fee_amount,
    )?;
    Ok(quote)
}

pub fn refund_remaining_sol(
    job_account: &AccountInfo,
    requester: &AccountInfo,
    job: &mut JobState,
) -> ProgramResultWithRefund {
    if !job.payment_asset.is_native_sol() {
        return Err(IcpxError::InvalidPaymentAsset.into());
    }

    let refund_amount = job.remaining_escrow_amount()?;
    if refund_amount > 0 {
        transfer_lamports(job_account, requester, refund_amount)?;
        job.record_refund(refund_amount)?;
    }
    Ok(refund_amount)
}

pub fn refund_remaining_tokens<'a>(
    job_account: &AccountInfo<'a>,
    requester_token_account: &AccountInfo<'a>,
    escrow_token_account: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    job: &mut JobState,
) -> ProgramResultWithRefund {
    let mint = job
        .payment_asset
        .mint()
        .ok_or(IcpxError::InvalidPaymentAsset)?;
    require_spl_token_program(token_program)?;
    require_key(
        escrow_token_account.key,
        &job.escrow_vault,
        IcpxError::InvalidEscrowVault,
    )?;

    let escrow = require_token_account(
        escrow_token_account,
        &mint,
        job_account.key,
        token_program.key,
    )?;
    require_token_account(
        requester_token_account,
        &mint,
        &job.requester,
        token_program.key,
    )?;

    let refund_amount = job.remaining_escrow_amount()?;
    if refund_amount > 0 {
        if escrow.amount < refund_amount {
            return Err(IcpxError::EscrowUnderfunded.into());
        }
        transfer_from_token_escrow(
            token_program,
            escrow_token_account,
            requester_token_account,
            job_account,
            job,
            refund_amount,
        )?;
        job.record_refund(refund_amount)?;
    }
    Ok(refund_amount)
}

fn quote_receipt(
    job: &JobState,
    receipt: GpuStreamReceipt,
) -> Result<SettlementQuote, solana_program::program_error::ProgramError> {
    if receipt.receipt_nonce <= job.last_receipt_nonce {
        return Err(IcpxError::InvalidReceipt.into());
    }

    quote_stream_settlement(
        receipt.cumulative_units,
        job.settled_units,
        job.max_units,
        job.price_per_unit,
        job.remaining_escrow_amount()?,
    )
}

fn record_receipt(
    job: &mut JobState,
    receipt: GpuStreamReceipt,
    provider_payment_amount: u64,
    protocol_fee_amount: u64,
) -> Result<(), solana_program::program_error::ProgramError> {
    job.record_payment_with_fee(
        receipt.cumulative_units,
        provider_payment_amount,
        protocol_fee_amount,
    )?;
    job.last_receipt_nonce = receipt.receipt_nonce;
    Ok(())
}

fn transfer_from_token_escrow<'a>(
    token_program: &AccountInfo<'a>,
    escrow_token_account: &AccountInfo<'a>,
    destination_token_account: &AccountInfo<'a>,
    job_account: &AccountInfo<'a>,
    job: &JobState,
    amount: u64,
) -> Result<(), solana_program::program_error::ProgramError> {
    let nonce_bytes = job.client_nonce.to_le_bytes();
    let bump = [job.bump];
    let signer_seeds = [
        JOB_SEED,
        job.requester.as_ref(),
        job.provider.as_ref(),
        nonce_bytes.as_ref(),
        bump.as_ref(),
    ];

    transfer_spl_tokens(
        token_program,
        escrow_token_account,
        destination_token_account,
        job_account,
        Some(&signer_seeds),
        amount,
    )
}

pub type ProgramResultWithRefund = Result<u64, solana_program::program_error::ProgramError>;

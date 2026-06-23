#[path = "accounts.rs"]
pub mod account_utils;
pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod math;
pub mod processor;
pub mod state;

use anchor_lang::prelude::{
    Account, AccountDeserialize, AccountInfo, AccountSerialize, Accounts, AccountsExit, Context,
    Discriminator, Key, Program, Pubkey, Rent, Result, Signer, SolanaSysvar, System, ToAccountInfo,
    UncheckedAccount,
};
#[cfg(feature = "idl-build")]
use anchor_lang::IdlBuild;
use anchor_lang::{
    account, declare_id, error, program, require_eq, require_gte, AnchorDeserialize,
    AnchorSerialize,
};
use solana_program::entrypoint::ProgramResult;

pub use constants::{EMPTY_HASH, JOB_SEED};
pub use errors::IcpxError;
pub use instructions::{
    process_accept_job, process_cancel_expired_job, process_complete_job, process_create_gpu_job,
    process_fund_job, process_open_dispute, process_settle_stream, CreateGpuJobArgs,
    GpuStreamReceipt, IcpxInstruction,
};
pub use processor::process_instruction;
pub use state::{GpuMeteringUnit, JobState, JobStatus, PaymentAsset};

declare_id!("Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML");

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "ICPX Payments",
    project_url: "https://github.com/orithellama/ICPX",
    contacts: "link:https://github.com/orithellama/ICPX/security/advisories/new",
    policy: "https://github.com/orithellama/ICPX/security/policy",
    preferred_languages: "en",
    source_code: "https://github.com/orithellama/ICPX",
    auditors: "Formal math harnesses: Kani; mainnet program id: Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML"
}

#[program]
pub mod icpx_payments {
    use super::*;

    pub fn create_gpu_job(ctx: Context<CreateGpuJob>, args: AnchorCreateGpuJobArgs) -> Result<()> {
        finish(process_create_gpu_job(
            ctx.program_id,
            &[
                ctx.accounts.requester.to_account_info(),
                ctx.accounts.job.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            args.into(),
        ))
    }

    pub fn fund_job<'info>(ctx: Context<'_, '_, '_, 'info, FundJob<'info>>) -> Result<()> {
        let accounts = with_remaining(
            vec![
                ctx.accounts.requester.to_account_info(),
                ctx.accounts.job.to_account_info(),
            ],
            ctx.remaining_accounts,
        );
        finish(process_fund_job(ctx.program_id, &accounts))
    }

    pub fn accept_job(ctx: Context<AcceptJob>) -> Result<()> {
        finish(process_accept_job(
            ctx.program_id,
            &[
                ctx.accounts.provider.to_account_info(),
                ctx.accounts.job.to_account_info(),
            ],
        ))
    }

    pub fn settle_stream<'info>(
        ctx: Context<'_, '_, '_, 'info, SettleStream<'info>>,
        receipt: AnchorGpuStreamReceipt,
    ) -> Result<()> {
        let accounts = with_remaining(
            vec![
                ctx.accounts.receipt_authority.to_account_info(),
                ctx.accounts.job.to_account_info(),
                ctx.accounts.provider_payment_account.to_account_info(),
                ctx.accounts.protocol_fee_account.to_account_info(),
            ],
            ctx.remaining_accounts,
        );
        finish(process_settle_stream(
            ctx.program_id,
            &accounts,
            receipt.into(),
        ))
    }

    pub fn complete_job<'info>(
        ctx: Context<'_, '_, '_, 'info, CompleteJob<'info>>,
        receipt: AnchorGpuStreamReceipt,
    ) -> Result<()> {
        let accounts = with_remaining(
            vec![
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.job.to_account_info(),
                ctx.accounts.provider_payment_account.to_account_info(),
                ctx.accounts.requester_refund_account.to_account_info(),
                ctx.accounts.protocol_fee_account.to_account_info(),
            ],
            ctx.remaining_accounts,
        );
        finish(process_complete_job(
            ctx.program_id,
            &accounts,
            receipt.into(),
        ))
    }

    pub fn cancel_expired_job<'info>(
        ctx: Context<'_, '_, '_, 'info, CancelExpiredJob<'info>>,
    ) -> Result<()> {
        let accounts = with_remaining(
            vec![
                ctx.accounts.job.to_account_info(),
                ctx.accounts.requester_refund_account.to_account_info(),
            ],
            ctx.remaining_accounts,
        );
        finish(process_cancel_expired_job(ctx.program_id, &accounts))
    }

    pub fn open_dispute(ctx: Context<OpenDispute>) -> Result<()> {
        finish(process_open_dispute(
            ctx.program_id,
            &[
                ctx.accounts.signer.to_account_info(),
                ctx.accounts.job.to_account_info(),
            ],
        ))
    }
}

#[derive(Accounts)]
pub struct CreateGpuJob<'info> {
    #[account(mut)]
    pub requester: Signer<'info>,
    /// CHECK: The native processor validates PDA seeds, ownership, and writes the Borsh job state.
    #[account(mut)]
    pub job: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FundJob<'info> {
    #[account(mut)]
    pub requester: Signer<'info>,
    /// CHECK: The native processor validates ownership and job state before funding.
    #[account(mut)]
    pub job: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct AcceptJob<'info> {
    pub provider: Signer<'info>,
    /// CHECK: The native processor validates ownership, provider, and job status.
    #[account(mut)]
    pub job: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct SettleStream<'info> {
    pub receipt_authority: Signer<'info>,
    /// CHECK: The native processor validates ownership, receipt authority, and mutable job state.
    #[account(mut)]
    pub job: UncheckedAccount<'info>,
    /// CHECK: The native settlement path validates this account against the selected payment asset.
    #[account(mut)]
    pub provider_payment_account: UncheckedAccount<'info>,
    /// CHECK: The native settlement path validates this account against the protocol fee destination.
    #[account(mut)]
    pub protocol_fee_account: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CompleteJob<'info> {
    pub authority: Signer<'info>,
    /// CHECK: The native processor validates ownership, authority, and mutable job state.
    #[account(mut)]
    pub job: UncheckedAccount<'info>,
    /// CHECK: The native settlement path validates this account against the selected payment asset.
    #[account(mut)]
    pub provider_payment_account: UncheckedAccount<'info>,
    /// CHECK: The native refund path validates this account against the selected payment asset.
    #[account(mut)]
    pub requester_refund_account: UncheckedAccount<'info>,
    /// CHECK: The native settlement path validates this account against the protocol fee destination.
    #[account(mut)]
    pub protocol_fee_account: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CancelExpiredJob<'info> {
    /// CHECK: The native processor validates ownership, expiry, and mutable job state.
    #[account(mut)]
    pub job: UncheckedAccount<'info>,
    /// CHECK: The native refund path validates this account against the requester and payment asset.
    #[account(mut)]
    pub requester_refund_account: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct OpenDispute<'info> {
    pub signer: Signer<'info>,
    /// CHECK: The native processor validates ownership and that the signer is a valid dispute party.
    #[account(mut)]
    pub job: UncheckedAccount<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct AnchorCreateGpuJobArgs {
    pub provider: Pubkey,
    pub receipt_authority: Pubkey,
    pub metadata_hash: [u8; 32],
    pub gpu_profile_hash: [u8; 32],
    pub nvidia_api_hash: [u8; 32],
    pub metering_unit: AnchorGpuMeteringUnit,
    pub payment_asset: AnchorPaymentAsset,
    pub client_nonce: u64,
    pub price_per_unit: u64,
    pub max_units: u64,
    pub expiry_slot: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorGpuStreamReceipt {
    pub cumulative_units: u64,
    pub result_hash: [u8; 32],
    pub receipt_nonce: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorGpuMeteringUnit {
    GpuMillisecond,
    NvidiaBillingUnit,
    OutputToken,
    Request,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorPaymentAsset {
    Sol,
    Usdc,
    Icpx,
}

impl From<AnchorCreateGpuJobArgs> for CreateGpuJobArgs {
    fn from(args: AnchorCreateGpuJobArgs) -> Self {
        Self {
            provider: args.provider,
            receipt_authority: args.receipt_authority,
            metadata_hash: args.metadata_hash,
            gpu_profile_hash: args.gpu_profile_hash,
            nvidia_api_hash: args.nvidia_api_hash,
            metering_unit: args.metering_unit.into(),
            payment_asset: args.payment_asset.into(),
            client_nonce: args.client_nonce,
            price_per_unit: args.price_per_unit,
            max_units: args.max_units,
            expiry_slot: args.expiry_slot,
        }
    }
}

impl From<AnchorGpuStreamReceipt> for GpuStreamReceipt {
    fn from(receipt: AnchorGpuStreamReceipt) -> Self {
        Self {
            cumulative_units: receipt.cumulative_units,
            result_hash: receipt.result_hash,
            receipt_nonce: receipt.receipt_nonce,
        }
    }
}

impl From<AnchorGpuMeteringUnit> for GpuMeteringUnit {
    fn from(unit: AnchorGpuMeteringUnit) -> Self {
        match unit {
            AnchorGpuMeteringUnit::GpuMillisecond => Self::GpuMillisecond,
            AnchorGpuMeteringUnit::NvidiaBillingUnit => Self::NvidiaBillingUnit,
            AnchorGpuMeteringUnit::OutputToken => Self::OutputToken,
            AnchorGpuMeteringUnit::Request => Self::Request,
        }
    }
}

impl From<AnchorPaymentAsset> for PaymentAsset {
    fn from(asset: AnchorPaymentAsset) -> Self {
        match asset {
            AnchorPaymentAsset::Sol => Self::Sol,
            AnchorPaymentAsset::Usdc => Self::Usdc,
            AnchorPaymentAsset::Icpx => Self::Icpx,
        }
    }
}

fn with_remaining<'info>(
    mut accounts: Vec<AccountInfo<'info>>,
    remaining_accounts: &[AccountInfo<'info>],
) -> Vec<AccountInfo<'info>> {
    accounts.extend(remaining_accounts.iter().cloned());
    accounts
}

fn finish(result: ProgramResult) -> Result<()> {
    result.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_program::pubkey::Pubkey;

    fn sample_job() -> JobState {
        JobState {
            version: 1,
            bump: 255,
            requester: Pubkey::new_unique(),
            provider: Pubkey::new_unique(),
            receipt_authority: Pubkey::new_unique(),
            escrow_vault: Pubkey::new_unique(),
            metadata_hash: [1; 32],
            gpu_profile_hash: [2; 32],
            nvidia_api_hash: [3; 32],
            result_hash: [0; 32],
            metering_unit: GpuMeteringUnit::GpuMillisecond,
            payment_asset: PaymentAsset::Sol,
            client_nonce: 42,
            price_per_unit: 5,
            max_units: 100,
            escrow_funded_amount: 500,
            settled_units: 0,
            total_paid_amount: 0,
            total_protocol_fee_amount: 0,
            total_refunded_amount: 0,
            created_slot: 10,
            start_slot: 0,
            expiry_slot: 1000,
            last_receipt_nonce: 0,
            status: JobStatus::Created,
        }
    }

    #[test]
    fn job_state_length_matches_serialized_size() {
        let job = sample_job();
        let bytes = job.try_to_vec().expect("serialize sample job");
        assert_eq!(bytes.len(), JobState::LEN);
    }

    #[test]
    fn budget_uses_checked_math() {
        let job = sample_job();
        assert_eq!(job.max_budget_amount().expect("budget"), 500);
        assert!(math::checked_payment_amount(u64::MAX, 2).is_err());
    }

    #[test]
    fn remaining_escrow_tracks_paid_and_refunded_amounts() {
        let mut job = sample_job();
        job.total_paid_amount = 125;
        job.total_protocol_fee_amount = 5;
        job.total_refunded_amount = 25;
        assert_eq!(job.remaining_escrow_amount().expect("remaining"), 345);
    }

    #[test]
    fn settlement_quote_rejects_replayed_receipts() {
        let quote = math::quote_stream_settlement(40, 40, 100, 5, 300);
        assert!(quote.is_err());
    }

    #[test]
    fn settlement_quote_splits_provider_payment_and_protocol_fee() {
        let quote =
            math::quote_stream_settlement(100, 0, 100, 1_000, 100_000).expect("quote settlement");
        assert_eq!(quote.gross_payment_amount, 100_000);
        assert_eq!(quote.protocol_fee_amount, 250);
        assert_eq!(quote.provider_payment_amount, 99_750);
    }

    #[test]
    fn instruction_round_trip_supports_nvidia_gpu_terms() {
        let instruction = IcpxInstruction::CreateGpuJob(CreateGpuJobArgs {
            provider: Pubkey::new_unique(),
            receipt_authority: Pubkey::new_unique(),
            metadata_hash: [9; 32],
            gpu_profile_hash: [8; 32],
            nvidia_api_hash: [7; 32],
            metering_unit: GpuMeteringUnit::NvidiaBillingUnit,
            payment_asset: PaymentAsset::Icpx,
            client_nonce: 7,
            price_per_unit: 11,
            max_units: 13,
            expiry_slot: 17,
        });
        let bytes = instruction.try_to_vec().expect("serialize instruction");
        let decoded = IcpxInstruction::try_from_slice(&bytes).expect("decode instruction");
        assert_eq!(decoded, instruction);
    }

    #[test]
    fn instruction_round_trip_preserves_frontend_variable_pricing() {
        let low_priority = IcpxInstruction::CreateGpuJob(CreateGpuJobArgs {
            provider: Pubkey::new_unique(),
            receipt_authority: Pubkey::new_unique(),
            metadata_hash: [4; 32],
            gpu_profile_hash: [5; 32],
            nvidia_api_hash: [6; 32],
            metering_unit: GpuMeteringUnit::Request,
            payment_asset: PaymentAsset::Usdc,
            client_nonce: 101,
            price_per_unit: 2,
            max_units: 1_000,
            expiry_slot: 500,
        });
        let enterprise = IcpxInstruction::CreateGpuJob(CreateGpuJobArgs {
            provider: Pubkey::new_unique(),
            receipt_authority: Pubkey::new_unique(),
            metadata_hash: [7; 32],
            gpu_profile_hash: [8; 32],
            nvidia_api_hash: [9; 32],
            metering_unit: GpuMeteringUnit::Request,
            payment_asset: PaymentAsset::Usdc,
            client_nonce: 102,
            price_per_unit: 25,
            max_units: 1_000,
            expiry_slot: 500,
        });

        for instruction in [low_priority, enterprise] {
            let bytes = instruction.try_to_vec().expect("serialize instruction");
            let decoded = IcpxInstruction::try_from_slice(&bytes).expect("decode instruction");
            assert_eq!(decoded, instruction);
        }
    }

    #[test]
    fn hard_coded_payment_constants_match_expected_addresses() {
        assert_eq!(
            constants::icpx_mint().to_string(),
            "HdeAPoHivsm9MZfeY5tW7apJEprc8Fs594bWmnzfpump"
        );
        assert_eq!(
            constants::usdc_mint().to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        );
        assert_eq!(
            constants::wrapped_sol_mint().to_string(),
            "So11111111111111111111111111111111111111112"
        );
        assert_eq!(
            constants::spl_token_program_id().to_string(),
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        );
        assert_eq!(
            constants::protocol_multisig().to_string(),
            "AgYcC58HhWt9vV8kRro7T77FQgGqpcaBMtNEtNYuKeA1"
        );
        assert_eq!(constants::PROTOCOL_FEE_BASIS_POINTS, 25);
    }
}

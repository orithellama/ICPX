pub mod accounts;
pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod math;
pub mod processor;
pub mod state;

pub use constants::{EMPTY_HASH, JOB_SEED};
pub use errors::IcpxError;
pub use instructions::{CreateGpuJobArgs, GpuStreamReceipt, IcpxInstruction};
pub use processor::process_instruction;
pub use state::{GpuMeteringUnit, JobState, JobStatus, PaymentAsset};

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshDeserialize;
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
        let bytes = borsh::to_vec(&job).expect("serialize sample job");
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
        let bytes = borsh::to_vec(&instruction).expect("serialize instruction");
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
            let bytes = borsh::to_vec(&instruction).expect("serialize instruction");
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

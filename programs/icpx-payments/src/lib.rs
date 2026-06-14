pub mod accounts;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod math;
pub mod processor;
pub mod state;

pub use errors::IcpxError;
pub use instructions::{CreateGpuJobArgs, GpuStreamReceipt, IcpxInstruction};
pub use processor::process_instruction;
pub use state::{GpuMeteringUnit, JobState, JobStatus, JOB_SEED};

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
            metadata_hash: [1; 32],
            gpu_profile_hash: [2; 32],
            nvidia_api_hash: [3; 32],
            result_hash: [0; 32],
            metering_unit: GpuMeteringUnit::GpuMillisecond,
            client_nonce: 42,
            price_per_unit_lamports: 5,
            max_units: 100,
            escrow_funded_lamports: 500,
            settled_units: 0,
            total_paid_lamports: 0,
            total_refunded_lamports: 0,
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
        assert_eq!(job.max_budget_lamports().expect("budget"), 500);
        assert!(math::checked_payment_lamports(u64::MAX, 2).is_err());
    }

    #[test]
    fn remaining_escrow_tracks_paid_and_refunded_amounts() {
        let mut job = sample_job();
        job.total_paid_lamports = 125;
        job.total_refunded_lamports = 25;
        assert_eq!(job.remaining_escrow_lamports().expect("remaining"), 350);
    }

    #[test]
    fn settlement_quote_rejects_replayed_receipts() {
        let quote = math::quote_stream_settlement(40, 40, 100, 5, 300);
        assert!(quote.is_err());
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
            client_nonce: 7,
            price_per_unit_lamports: 11,
            max_units: 13,
            expiry_slot: 17,
        });
        let bytes = borsh::to_vec(&instruction).expect("serialize instruction");
        let decoded = IcpxInstruction::try_from_slice(&bytes).expect("decode instruction");
        assert_eq!(decoded, instruction);
    }
}

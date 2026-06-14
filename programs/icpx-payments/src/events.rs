use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{log::sol_log_data, pubkey::Pubkey};

use crate::state::{GpuMeteringUnit, JobStatus, PaymentAsset};

const EVENT_PREFIX: &[u8] = b"icpx-payments-event";

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum IcpxEvent {
    GpuJobCreated {
        job: Pubkey,
        requester: Pubkey,
        provider: Pubkey,
        receipt_authority: Pubkey,
        gpu_profile_hash: [u8; 32],
        nvidia_api_hash: [u8; 32],
        metering_unit: GpuMeteringUnit,
        payment_asset: PaymentAsset,
        price_per_unit: u64,
        max_units: u64,
        expiry_slot: u64,
    },
    JobFunded {
        job: Pubkey,
        requester: Pubkey,
        payment_asset: PaymentAsset,
        escrow_funded_amount: u64,
        escrow_vault: Pubkey,
    },
    JobAccepted {
        job: Pubkey,
        provider: Pubkey,
        start_slot: u64,
    },
    StreamSettled {
        job: Pubkey,
        provider: Pubkey,
        receipt_nonce: u64,
        cumulative_units: u64,
        new_units: u64,
        payment_asset: PaymentAsset,
        payment_amount: u64,
        total_paid_amount: u64,
    },
    JobCompleted {
        job: Pubkey,
        final_units: u64,
        payment_asset: PaymentAsset,
        total_paid_amount: u64,
        refund_amount: u64,
        result_hash: [u8; 32],
    },
    JobExpired {
        job: Pubkey,
        payment_asset: PaymentAsset,
        refund_amount: u64,
    },
    DisputeOpened {
        job: Pubkey,
        signer: Pubkey,
        previous_status: JobStatus,
    },
}

pub fn emit(event: &IcpxEvent) {
    if let Ok(data) = borsh::to_vec(event) {
        sol_log_data(&[EVENT_PREFIX, &data]);
    }
}

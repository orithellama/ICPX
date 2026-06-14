mod accept_job;
mod cancel_expired_job;
mod complete_job;
mod create_gpu_job;
mod fund_job;
mod open_dispute;
mod settle_stream;
mod settlement;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub use accept_job::process_accept_job;
pub use cancel_expired_job::process_cancel_expired_job;
pub use complete_job::process_complete_job;
pub use create_gpu_job::process_create_gpu_job;
pub use fund_job::process_fund_job;
pub use open_dispute::process_open_dispute;
pub use settle_stream::process_settle_stream;

use crate::state::{GpuMeteringUnit, PaymentAsset};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum IcpxInstruction {
    CreateGpuJob(CreateGpuJobArgs),
    FundJob,
    AcceptJob,
    SettleStream(GpuStreamReceipt),
    CompleteJob(GpuStreamReceipt),
    CancelExpiredJob,
    OpenDispute,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct CreateGpuJobArgs {
    pub provider: Pubkey,
    pub receipt_authority: Pubkey,
    pub metadata_hash: [u8; 32],
    pub gpu_profile_hash: [u8; 32],
    pub nvidia_api_hash: [u8; 32],
    pub metering_unit: GpuMeteringUnit,
    pub payment_asset: PaymentAsset,
    pub client_nonce: u64,
    pub price_per_unit: u64,
    pub max_units: u64,
    pub expiry_slot: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuStreamReceipt {
    pub cumulative_units: u64,
    pub result_hash: [u8; 32],
    pub receipt_nonce: u64,
}

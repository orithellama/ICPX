use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    errors::IcpxError,
    math::checked_payment_amount,
};

use super::PaymentAsset;

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobStatus {
    Created,
    Funded,
    Running,
    Completed,
    Cancelled,
    Expired,
    Disputed,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuMeteringUnit {
    GpuMillisecond,
    NvidiaBillingUnit,
    OutputToken,
    Request,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct JobState {
    pub version: u8,
    pub bump: u8,
    pub requester: Pubkey,
    pub provider: Pubkey,
    pub receipt_authority: Pubkey,
    pub escrow_vault: Pubkey,
    pub metadata_hash: [u8; 32],
    pub gpu_profile_hash: [u8; 32],
    pub nvidia_api_hash: [u8; 32],
    pub result_hash: [u8; 32],
    pub metering_unit: GpuMeteringUnit,
    pub payment_asset: PaymentAsset,
    pub client_nonce: u64,
    pub price_per_unit: u64,
    pub max_units: u64,
    pub escrow_funded_amount: u64,
    pub settled_units: u64,
    pub total_paid_amount: u64,
    pub total_refunded_amount: u64,
    pub created_slot: u64,
    pub start_slot: u64,
    pub expiry_slot: u64,
    pub last_receipt_nonce: u64,
    pub status: JobStatus,
}

impl JobState {
    pub const LEN: usize = 2 + (8 * 32) + 1 + PaymentAsset::LEN + (11 * 8) + 1;

    pub fn max_budget_amount(&self) -> Result<u64, ProgramError> {
        checked_payment_amount(self.max_units, self.price_per_unit)
    }

    pub fn remaining_escrow_amount(&self) -> Result<u64, ProgramError> {
        self.escrow_funded_amount
            .checked_sub(self.total_paid_amount)
            .and_then(|remaining| remaining.checked_sub(self.total_refunded_amount))
            .ok_or(IcpxError::MathOverflow.into())
    }

    pub fn record_payment(
        &mut self,
        cumulative_units: u64,
        payment_amount: u64,
    ) -> ProgramResult {
        self.settled_units = cumulative_units;
        self.total_paid_amount = self
            .total_paid_amount
            .checked_add(payment_amount)
            .ok_or(IcpxError::MathOverflow)?;
        Ok(())
    }

    pub fn record_refund(&mut self, refund_amount: u64) -> ProgramResult {
        self.total_refunded_amount = self
            .total_refunded_amount
            .checked_add(refund_amount)
            .ok_or(IcpxError::MathOverflow)?;
        Ok(())
    }
}

type ProgramResult = Result<(), ProgramError>;

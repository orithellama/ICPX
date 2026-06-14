use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::constants::{icpx_mint, usdc_mint};

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaymentAsset {
    Sol,
    Usdc,
    Icpx,
}

impl PaymentAsset {
    pub const LEN: usize = 1;

    pub fn mint(self) -> Option<Pubkey> {
        match self {
            PaymentAsset::Sol => None,
            PaymentAsset::Usdc => Some(usdc_mint()),
            PaymentAsset::Icpx => Some(icpx_mint()),
        }
    }

    pub fn is_native_sol(self) -> bool {
        matches!(self, PaymentAsset::Sol)
    }
}

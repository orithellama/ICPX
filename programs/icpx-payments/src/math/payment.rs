use solana_program::program_error::ProgramError;

use crate::errors::IcpxError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettlementQuote {
    pub new_units: u64,
    pub payment_amount: u64,
}

pub fn checked_payment_amount(units: u64, price_per_unit: u64) -> Result<u64, ProgramError> {
    units
        .checked_mul(price_per_unit)
        .ok_or(IcpxError::MathOverflow.into())
}

pub fn quote_stream_settlement(
    cumulative_units: u64,
    settled_units: u64,
    max_units: u64,
    price_per_unit: u64,
    remaining_escrow_amount: u64,
) -> Result<SettlementQuote, ProgramError> {
    if cumulative_units > max_units || cumulative_units <= settled_units {
        return Err(IcpxError::InvalidReceipt.into());
    }

    let new_units = cumulative_units
        .checked_sub(settled_units)
        .ok_or(IcpxError::MathOverflow)?;
    let payment_amount = checked_payment_amount(new_units, price_per_unit)?;
    if payment_amount > remaining_escrow_amount {
        return Err(IcpxError::EscrowUnderfunded.into());
    }

    Ok(SettlementQuote {
        new_units,
        payment_amount,
    })
}

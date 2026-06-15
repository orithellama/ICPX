use solana_program::program_error::ProgramError;

use crate::{
    constants::{BASIS_POINTS_DENOMINATOR, PROTOCOL_FEE_BASIS_POINTS},
    errors::IcpxError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettlementQuote {
    pub new_units: u64,
    pub gross_payment_amount: u64,
    pub provider_payment_amount: u64,
    pub protocol_fee_amount: u64,
}

pub fn checked_payment_amount(units: u64, price_per_unit: u64) -> Result<u64, ProgramError> {
    checked_payment_amount_raw(units, price_per_unit).ok_or(IcpxError::MathOverflow.into())
}

pub fn quote_stream_settlement(
    cumulative_units: u64,
    settled_units: u64,
    max_units: u64,
    price_per_unit: u64,
    remaining_escrow_amount: u64,
) -> Result<SettlementQuote, ProgramError> {
    let new_units = checked_new_units(cumulative_units, settled_units, max_units)
        .ok_or(IcpxError::InvalidReceipt)?;
    let gross_payment_amount = checked_payment_amount(new_units, price_per_unit)?;
    if gross_payment_amount > remaining_escrow_amount {
        return Err(IcpxError::EscrowUnderfunded.into());
    }
    let protocol_fee_amount = checked_protocol_fee_amount(gross_payment_amount)?;
    let provider_payment_amount = gross_payment_amount
        .checked_sub(protocol_fee_amount)
        .ok_or(IcpxError::MathOverflow)?;

    Ok(SettlementQuote {
        new_units,
        gross_payment_amount,
        provider_payment_amount,
        protocol_fee_amount,
    })
}

pub fn checked_protocol_fee_amount(gross_payment_amount: u64) -> Result<u64, ProgramError> {
    checked_protocol_fee_amount_raw(gross_payment_amount).ok_or(IcpxError::MathOverflow.into())
}

fn checked_payment_amount_raw(units: u64, price_per_unit: u64) -> Option<u64> {
    units.checked_mul(price_per_unit)
}

fn checked_new_units(cumulative_units: u64, settled_units: u64, max_units: u64) -> Option<u64> {
    if cumulative_units > max_units || cumulative_units <= settled_units {
        return None;
    }

    cumulative_units.checked_sub(settled_units)
}

fn checked_protocol_fee_amount_raw(gross_payment_amount: u64) -> Option<u64> {
    gross_payment_amount
        .checked_mul(PROTOCOL_FEE_BASIS_POINTS)?
        .checked_div(BASIS_POINTS_DENOMINATOR)
}

#[cfg(kani)]
fn quote_stream_settlement_raw(
    cumulative_units: u64,
    settled_units: u64,
    max_units: u64,
    price_per_unit: u64,
    remaining_escrow_amount: u64,
) -> Option<SettlementQuote> {
    let new_units = checked_new_units(cumulative_units, settled_units, max_units)?;
    let gross_payment_amount = checked_payment_amount_raw(new_units, price_per_unit)?;
    if gross_payment_amount > remaining_escrow_amount {
        return None;
    }
    let protocol_fee_amount = checked_protocol_fee_amount_raw(gross_payment_amount)?;
    let provider_payment_amount = gross_payment_amount.checked_sub(protocol_fee_amount)?;

    Some(SettlementQuote {
        new_units,
        gross_payment_amount,
        provider_payment_amount,
        protocol_fee_amount,
    })
}

#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn checked_payment_amount_raw_matches_u64_checked_mul() {
        let units_raw: u8 = kani::any();
        let price_per_unit_raw: u8 = kani::any();
        let units = u64::from(units_raw);
        let price_per_unit = u64::from(price_per_unit_raw);

        let expected = units * price_per_unit;

        if let Some(actual) = checked_payment_amount_raw(units, price_per_unit) {
            assert_eq!(actual, expected);
        } else {
            panic!("u8-sized payment inputs should not overflow u64");
        }
    }

    #[kani::proof]
    fn checked_payment_amount_raw_rejects_known_overflow() {
        assert!(checked_payment_amount_raw(u64::MAX, 2).is_none());
    }

    #[kani::proof]
    fn checked_protocol_fee_amount_raw_never_exceeds_gross_payment() {
        let gross_payment_amount = u64::from(kani::any::<u8>());

        if let Some(protocol_fee_amount) = checked_protocol_fee_amount_raw(gross_payment_amount) {
            assert!(protocol_fee_amount <= gross_payment_amount);
        }
    }

    #[kani::proof]
    fn protocol_fee_basis_points_are_below_denominator() {
        assert!(PROTOCOL_FEE_BASIS_POINTS < BASIS_POINTS_DENOMINATOR);
    }

    #[kani::proof]
    fn checked_protocol_fee_amount_raw_rejects_known_overflow() {
        assert!(checked_protocol_fee_amount_raw(u64::MAX).is_none());
    }

    #[kani::proof]
    fn checked_new_units_rejects_invalid_cumulative_units() {
        let cumulative_units = u64::from(kani::any::<u8>());
        let settled_units = u64::from(kani::any::<u8>());
        let max_units = u64::from(kani::any::<u8>());

        kani::assume(cumulative_units > max_units || cumulative_units <= settled_units);

        assert!(checked_new_units(cumulative_units, settled_units, max_units).is_none());
    }

    #[kani::proof]
    fn checked_new_units_returns_exact_delta_for_valid_units() {
        let cumulative_units = u64::from(kani::any::<u8>());
        let settled_units = u64::from(kani::any::<u8>());
        let max_units = u64::from(kani::any::<u8>());

        kani::assume(settled_units < cumulative_units);
        kani::assume(cumulative_units <= max_units);

        if let Some(new_units) = checked_new_units(cumulative_units, settled_units, max_units) {
            assert_eq!(new_units, cumulative_units - settled_units);
        } else {
            panic!("valid unit bounds should produce a delta");
        }
    }

    #[kani::proof]
    fn quote_stream_settlement_raw_rejects_over_escrow() {
        let cumulative_units = u64::from(kani::any::<u8>());
        let settled_units = u64::from(kani::any::<u8>());
        let max_units = u64::from(kani::any::<u8>());
        let price_per_unit = u64::from(kani::any::<u8>());
        let remaining_escrow_amount = u64::from(kani::any::<u8>());

        kani::assume(settled_units < cumulative_units);
        kani::assume(cumulative_units <= max_units);
        let new_units = cumulative_units - settled_units;
        let gross_payment_amount = new_units * price_per_unit;
        kani::assume(gross_payment_amount > remaining_escrow_amount);

        assert!(quote_stream_settlement_raw(
            cumulative_units,
            settled_units,
            max_units,
            price_per_unit,
            remaining_escrow_amount,
        )
        .is_none());
    }

    #[kani::proof]
    fn quote_stream_settlement_raw_rejects_gross_payment_overflow() {
        assert!(quote_stream_settlement_raw(2, 0, 2, u64::MAX, u64::MAX).is_none());
    }

    #[kani::proof]
    fn quote_stream_settlement_raw_never_quotes_more_than_escrow() {
        let cumulative_units = u64::from(kani::any::<u8>());
        let settled_units = u64::from(kani::any::<u8>());
        let max_units = u64::from(kani::any::<u8>());
        let price_per_unit = u64::from(kani::any::<u8>());
        let remaining_escrow_amount = u64::from(kani::any::<u8>());

        kani::assume(settled_units < cumulative_units);
        kani::assume(cumulative_units <= max_units);

        if let Some(quote) = quote_stream_settlement_raw(
            cumulative_units,
            settled_units,
            max_units,
            price_per_unit,
            remaining_escrow_amount,
        ) {
            assert_eq!(quote.new_units, cumulative_units - settled_units);
            assert_eq!(
                quote.gross_payment_amount,
                quote.provider_payment_amount + quote.protocol_fee_amount
            );
            assert!(quote.gross_payment_amount <= remaining_escrow_amount);
        }
    }
}

use icpx_payments::{
    constants::{protocol_multisig, PROTOCOL_FEE_BASIS_POINTS},
    math::{checked_protocol_fee_amount, quote_stream_settlement},
};

#[test]
fn protocol_multisig_constant_matches_expected_address() {
    assert_eq!(
        protocol_multisig().to_string(),
        "AgYcC58HhWt9vV8kRro7T77FQgGqpcaBMtNEtNYuKeA1"
    );
}

#[test]
fn protocol_fee_is_small_and_fixed() {
    assert_eq!(PROTOCOL_FEE_BASIS_POINTS, 25);
    assert_eq!(checked_protocol_fee_amount(100_000).expect("fee"), 250);
}

#[test]
fn settlement_quote_accounts_for_protocol_fee_without_overpaying_escrow() {
    let quote = quote_stream_settlement(100, 40, 100, 1_000, 60_000).expect("settlement quote");

    assert_eq!(quote.new_units, 60);
    assert_eq!(quote.gross_payment_amount, 60_000);
    assert_eq!(quote.protocol_fee_amount, 150);
    assert_eq!(quote.provider_payment_amount, 59_850);
    assert_eq!(
        quote.provider_payment_amount + quote.protocol_fee_amount,
        quote.gross_payment_amount
    );
}

#[test]
fn settlement_quote_rejects_fee_inclusive_overpay() {
    assert!(quote_stream_settlement(100, 40, 100, 1_000, 59_999).is_err());
}

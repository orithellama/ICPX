# Verification

## Current Coverage

- Rust unit tests cover state serialization length, checked budget math, escrow accounting, replay rejection, instruction round-trips, and hard-coded payment constants.
- Kani harnesses cover the pure settlement math for bounded symbolic inputs,
  invalid cumulative units, protocol-fee bounds, escrow overpayment protection,
  exact unit deltas, known multiplication overflow witnesses, protocol-fee
  overflow witnesses, and gross-payment overflow witnesses.

## Kani Scope

Kani is used for deterministic Rust math properties. Current symbolic harnesses
use small bounded domains so they run quickly in CI, plus explicit edge-case
witnesses for overflow. Kani is not used for Solana CPI behavior, account
borrowing, sysvar behavior, or runtime authorization. Those require local
validator and integration tests.

## Commands

```sh
make test
make clippy
make verify-kani
```

## Properties To Keep Proving

- `checked_payment_amount` matches Rust checked multiplication.
- Invalid cumulative units are rejected.
- Valid cumulative units return the exact delta.
- Settlement never quotes more than remaining escrow.
- Over-escrow settlements are rejected.
- Gross payment overflow is rejected.
- Protocol fees never exceed gross settlement in the verified bounded domain.
- Protocol fee bps remain below the basis-point denominator.
- Valid quotes settle only the delta between `settled_units` and `cumulative_units`.

## Next Verification Work

- Add proptests for randomized settlement sequences.
- Add local validator tests for USDC and ICPX token accounts.
- Add negative tests for wrong mint, wrong token owner, wrong token program, and wrong escrow vault.

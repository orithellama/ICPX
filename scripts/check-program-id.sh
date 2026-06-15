#!/usr/bin/env sh
set -eu

EXPECTED_PROGRAM_ID="Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML"
PROGRAM_KEYPAIR="${ICPX_PROGRAM_KEYPAIR:-target/deploy/icpx_payments-keypair.json}"

if [ ! -f "$PROGRAM_KEYPAIR" ]; then
  echo "missing program keypair: $PROGRAM_KEYPAIR" >&2
  exit 1
fi

ACTUAL_PROGRAM_ID="$(solana-keygen pubkey "$PROGRAM_KEYPAIR")"
if [ "$ACTUAL_PROGRAM_ID" != "$EXPECTED_PROGRAM_ID" ]; then
  echo "program id mismatch" >&2
  echo "expected: $EXPECTED_PROGRAM_ID" >&2
  echo "actual:   $ACTUAL_PROGRAM_ID" >&2
  exit 1
fi

echo "$ACTUAL_PROGRAM_ID"

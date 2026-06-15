#!/usr/bin/env sh
set -eu

PROGRAM_NAME="icpx_payments"
EXPECTED_PROGRAM_ID="Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML"
PROGRAM_KEYPAIR="${ICPX_PROGRAM_KEYPAIR:-target/deploy/${PROGRAM_NAME}-keypair.json}"
PROGRAM_SO="target/deploy/${PROGRAM_NAME}.so"
RPC_URL="${SOLANA_URL:-https://api.devnet.solana.com}"

if [ ! -f "$PROGRAM_KEYPAIR" ]; then
  echo "missing program keypair: $PROGRAM_KEYPAIR" >&2
  echo "restore the existing keypair or set ICPX_PROGRAM_KEYPAIR; do not generate a new keypair for redeploys" >&2
  exit 1
fi

ACTUAL_PROGRAM_ID="$(solana-keygen pubkey "$PROGRAM_KEYPAIR")"
if [ "$ACTUAL_PROGRAM_ID" != "$EXPECTED_PROGRAM_ID" ]; then
  echo "program id mismatch" >&2
  echo "expected: $EXPECTED_PROGRAM_ID" >&2
  echo "actual:   $ACTUAL_PROGRAM_ID" >&2
  exit 1
fi

cargo build-sbf --manifest-path programs/icpx-payments/Cargo.toml

if [ ! -f "$PROGRAM_SO" ]; then
  echo "missing program binary after build: $PROGRAM_SO" >&2
  exit 1
fi

if [ -n "${ICPX_UPGRADE_AUTHORITY:-}" ]; then
  solana program deploy \
    --url "$RPC_URL" \
    --program-id "$PROGRAM_KEYPAIR" \
    --upgrade-authority "$ICPX_UPGRADE_AUTHORITY" \
    "$PROGRAM_SO"
else
  solana program deploy \
    --url "$RPC_URL" \
    --program-id "$PROGRAM_KEYPAIR" \
    "$PROGRAM_SO"
fi

mkdir -p target/idl
cp idl/icpx_payments.json target/idl/icpx_payments.json

echo "deployed $PROGRAM_NAME to $EXPECTED_PROGRAM_ID on devnet"
echo "idl written to target/idl/icpx_payments.json"

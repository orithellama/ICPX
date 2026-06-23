#!/usr/bin/env sh
set -eu

PROGRAM_NAME="icpx_payments"
EXPECTED_PROGRAM_ID="Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML"
PROGRAM_KEYPAIR="${ICPX_PROGRAM_KEYPAIR:-target/deploy/${PROGRAM_NAME}-keypair.json}"
PROGRAM_SO="target/deploy/${PROGRAM_NAME}.so"
RPC_URL="${ICPX_MAINNET_RPC_URL:-https://api.mainnet-beta.solana.com}"
CONFIRMATION="${ICPX_CONFIRM_MAINNET_DEPLOY:-}"

case "$RPC_URL" in
  *devnet*|*testnet*|*localhost*|*127.0.0.1*)
    echo "refusing non-mainnet-looking RPC URL: $RPC_URL" >&2
    exit 1
    ;;
esac

if [ "$CONFIRMATION" != "$EXPECTED_PROGRAM_ID" ]; then
  echo "mainnet deploy requires explicit confirmation" >&2
  echo "set ICPX_CONFIRM_MAINNET_DEPLOY=$EXPECTED_PROGRAM_ID" >&2
  exit 1
fi

if [ -z "${ICPX_MAINNET_FEE_PAYER:-}" ]; then
  echo "missing ICPX_MAINNET_FEE_PAYER=/path/to/funded-mainnet-keypair.json" >&2
  exit 1
fi

if [ -z "${ICPX_UPGRADE_AUTHORITY:-}" ]; then
  echo "missing ICPX_UPGRADE_AUTHORITY=/path/to/upgrade-authority-keypair.json" >&2
  exit 1
fi

if [ ! -f "$PROGRAM_KEYPAIR" ]; then
  echo "missing program keypair: $PROGRAM_KEYPAIR" >&2
  echo "restore the existing keypair; do not generate a new keypair for redeploys" >&2
  exit 1
fi

if [ ! -f "$ICPX_MAINNET_FEE_PAYER" ]; then
  echo "missing fee payer keypair: $ICPX_MAINNET_FEE_PAYER" >&2
  exit 1
fi

if [ ! -f "$ICPX_UPGRADE_AUTHORITY" ]; then
  echo "missing upgrade authority keypair: $ICPX_UPGRADE_AUTHORITY" >&2
  exit 1
fi

ACTUAL_PROGRAM_ID="$(solana-keygen pubkey "$PROGRAM_KEYPAIR")"
if [ "$ACTUAL_PROGRAM_ID" != "$EXPECTED_PROGRAM_ID" ]; then
  echo "program id mismatch" >&2
  echo "expected: $EXPECTED_PROGRAM_ID" >&2
  echo "actual:   $ACTUAL_PROGRAM_ID" >&2
  exit 1
fi

if solana program show --url "$RPC_URL" "$EXPECTED_PROGRAM_ID" >/dev/null 2>&1; then
  echo "program already exists on mainnet; use an explicit upgrade flow instead" >&2
  exit 1
fi

cargo build-sbf --manifest-path programs/icpx-payments/Cargo.toml

if [ ! -f "$PROGRAM_SO" ]; then
  echo "missing program binary after build: $PROGRAM_SO" >&2
  exit 1
fi

FEE_PAYER_PUBKEY="$(solana-keygen pubkey "$ICPX_MAINNET_FEE_PAYER")"
UPGRADE_AUTHORITY_PUBKEY="$(solana-keygen pubkey "$ICPX_UPGRADE_AUTHORITY")"
PROGRAM_HASH="$(shasum -a 256 "$PROGRAM_SO" | awk '{print $1}')"

echo "mainnet RPC: $RPC_URL"
echo "program id: $EXPECTED_PROGRAM_ID"
echo "program so: $PROGRAM_SO"
echo "program sha256: $PROGRAM_HASH"
echo "fee payer: $FEE_PAYER_PUBKEY"
echo "fee payer balance:"
solana balance --url "$RPC_URL" "$FEE_PAYER_PUBKEY"
echo "upgrade authority: $UPGRADE_AUTHORITY_PUBKEY"

solana program deploy \
  --url "$RPC_URL" \
  --use-rpc \
  --program-id "$PROGRAM_KEYPAIR" \
  --fee-payer "$ICPX_MAINNET_FEE_PAYER" \
  --upgrade-authority "$ICPX_UPGRADE_AUTHORITY" \
  "$PROGRAM_SO"

echo "deployed $PROGRAM_NAME to $EXPECTED_PROGRAM_ID on mainnet"

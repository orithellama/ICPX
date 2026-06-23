#!/usr/bin/env sh
set -eu

PROGRAM_NAME="icpx_payments"
EXPECTED_PROGRAM_ID="Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML"
ROOT_DIR="$(CDPATH= cd "$(dirname "$0")/.." && pwd)"
PROGRAM_KEYPAIR="${ICPX_PROGRAM_KEYPAIR:-target/deploy/${PROGRAM_NAME}-keypair.json}"
PROGRAM_SO="target/deploy/${PROGRAM_NAME}.so"
RPC_URL="${SOLANA_URL:-https://api.devnet.solana.com}"
ANCHOR_BIN="${ANCHOR_BIN:-}"

if [ -z "$ANCHOR_BIN" ]; then
  if [ -x "$HOME/.avm/bin/anchor" ]; then
    ANCHOR_BIN="$HOME/.avm/bin/anchor"
  else
    ANCHOR_BIN="anchor"
  fi
fi

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

(cd "$ROOT_DIR/programs/icpx-payments" && "$ANCHOR_BIN" build \
  --provider.cluster "$RPC_URL" \
  --skip-lint \
  -p "$PROGRAM_NAME")

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

./scripts/publish-idl.sh

echo "deployed $PROGRAM_NAME to $EXPECTED_PROGRAM_ID on devnet"
echo "idl written to target/idl/icpx_payments.json"

#!/usr/bin/env sh
set -eu

PROGRAM_NAME="${ICPX_PROGRAM_NAME:-icpx_payments}"
PROGRAM_ID="Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML"
IDL_FILE="${ICPX_IDL_FILE:-target/idl/${PROGRAM_NAME}.json}"
RPC_URL="${ICPX_MAINNET_RPC_URL:-https://api.mainnet-beta.solana.com}"
IDL_AUTHORITY="${ICPX_IDL_AUTHORITY:-${ICPX_UPGRADE_AUTHORITY:-}}"
CONFIRMATION="${ICPX_CONFIRM_MAINNET_IDL:-}"
ANCHOR_BIN="${ANCHOR_BIN:-}"

if [ -z "$ANCHOR_BIN" ]; then
  if [ -x "$HOME/.avm/bin/anchor" ]; then
    ANCHOR_BIN="$HOME/.avm/bin/anchor"
  else
    ANCHOR_BIN="anchor"
  fi
fi

case "$RPC_URL" in
  *devnet*|*testnet*|*localhost*|*127.0.0.1*)
    echo "refusing non-mainnet-looking RPC URL: $RPC_URL" >&2
    exit 1
    ;;
esac

if [ "$CONFIRMATION" != "$PROGRAM_ID" ]; then
  echo "mainnet IDL publish requires explicit confirmation" >&2
  echo "set ICPX_CONFIRM_MAINNET_IDL=$PROGRAM_ID" >&2
  exit 1
fi

if [ -z "$IDL_AUTHORITY" ]; then
  echo "missing ICPX_IDL_AUTHORITY=/path/to/idl-authority-keypair.json" >&2
  echo "or set ICPX_UPGRADE_AUTHORITY when it is also the IDL authority" >&2
  exit 1
fi

if [ ! -f "$IDL_AUTHORITY" ]; then
  echo "missing IDL authority keypair: $IDL_AUTHORITY" >&2
  exit 1
fi

if [ ! -f "$IDL_FILE" ]; then
  ./scripts/publish-idl.sh
fi

IDL_AUTHORITY_PUBKEY="$(solana-keygen pubkey "$IDL_AUTHORITY")"
FETCH_OUT="${TMPDIR:-/tmp}/icpx-anchor-idl-fetch.$$"

echo "mainnet RPC: $RPC_URL"
echo "program id: $PROGRAM_ID"
echo "idl file: $IDL_FILE"
echo "idl authority: $IDL_AUTHORITY_PUBKEY"

if "$ANCHOR_BIN" idl fetch --provider.cluster "$RPC_URL" "$PROGRAM_ID" >"$FETCH_OUT" 2>/dev/null; then
  rm -f "$FETCH_OUT"
  "$ANCHOR_BIN" idl upgrade \
    --provider.cluster "$RPC_URL" \
    --provider.wallet "$IDL_AUTHORITY" \
    --filepath "$IDL_FILE" \
    "$PROGRAM_ID"
  echo "upgraded on-chain Anchor IDL for $PROGRAM_ID"
else
  rm -f "$FETCH_OUT"
  "$ANCHOR_BIN" idl init \
    --provider.cluster "$RPC_URL" \
    --provider.wallet "$IDL_AUTHORITY" \
    --filepath "$IDL_FILE" \
    "$PROGRAM_ID"
  echo "initialized on-chain Anchor IDL for $PROGRAM_ID"
fi

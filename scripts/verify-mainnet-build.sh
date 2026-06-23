#!/usr/bin/env sh
set -eu

PROGRAM_NAME="${ICPX_PROGRAM_NAME:-icpx_payments}"
PROGRAM_ID="Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML"
RPC_URL="${ICPX_MAINNET_RPC_URL:-https://api.mainnet-beta.solana.com}"
REPO_URL="${ICPX_VERIFY_REPO_URL:-https://github.com/orithellama/ICPX}"
COMMIT_HASH="${ICPX_VERIFY_COMMIT:-$(git rev-parse HEAD)}"
VERIFY_KEYPAIR="${ICPX_VERIFY_KEYPAIR:-${ICPX_UPGRADE_AUTHORITY:-}}"
CONFIRMATION="${ICPX_CONFIRM_MAINNET_VERIFY:-}"
SKIP_LOCAL_BUILD="${ICPX_VERIFY_SKIP_LOCAL_BUILD:-1}"

case "$RPC_URL" in
  *devnet*|*testnet*|*localhost*|*127.0.0.1*)
    echo "refusing non-mainnet-looking RPC URL: $RPC_URL" >&2
    exit 1
    ;;
esac

if [ "$CONFIRMATION" != "$PROGRAM_ID" ]; then
  echo "mainnet verified-build upload requires explicit confirmation" >&2
  echo "set ICPX_CONFIRM_MAINNET_VERIFY=$PROGRAM_ID" >&2
  exit 1
fi

if [ -z "$VERIFY_KEYPAIR" ]; then
  echo "missing ICPX_VERIFY_KEYPAIR=/path/to/uploader-keypair.json" >&2
  echo "or set ICPX_UPGRADE_AUTHORITY when it is also the uploader" >&2
  exit 1
fi

if [ ! -f "$VERIFY_KEYPAIR" ]; then
  echo "missing verify uploader keypair: $VERIFY_KEYPAIR" >&2
  exit 1
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "working tree has uncommitted changes" >&2
  echo "verified builds must point at the exact public commit deployed on-chain" >&2
  exit 1
fi

UPLOADER="$(solana-keygen pubkey "$VERIFY_KEYPAIR")"

echo "mainnet RPC: $RPC_URL"
echo "program id: $PROGRAM_ID"
echo "repo: $REPO_URL"
echo "commit: $COMMIT_HASH"
echo "uploader: $UPLOADER"

VERIFY_FLAGS=""
if [ "$SKIP_LOCAL_BUILD" = "1" ]; then
  VERIFY_FLAGS="--skip-build"
fi

solana-verify verify-from-repo \
  -u "$RPC_URL" \
  -k "$VERIFY_KEYPAIR" \
  --program-id "$PROGRAM_ID" \
  --library-name "$PROGRAM_NAME" \
  --commit-hash "$COMMIT_HASH" \
  --mount-path "" \
  --workspace-path "" \
  --skip-prompt \
  $VERIFY_FLAGS \
  "$REPO_URL" \
  -- --manifest-path programs/icpx-payments/Cargo.toml

solana-verify remote submit-job \
  -u "$RPC_URL" \
  --program-id "$PROGRAM_ID" \
  --uploader "$UPLOADER"

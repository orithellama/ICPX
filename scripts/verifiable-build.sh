#!/usr/bin/env sh
set -eu

PROGRAM_NAME="${ICPX_PROGRAM_NAME:-icpx_payments}"
ROOT_DIR="$(CDPATH= cd "$(dirname "$0")/.." && pwd)"
WORKSPACE_PATH="${ICPX_VERIFY_WORKSPACE_PATH:-$ROOT_DIR}"
MOUNT_PATH="${ICPX_VERIFY_MOUNT_PATH:-$ROOT_DIR}"

solana-verify build \
  --library-name "$PROGRAM_NAME" \
  --workspace-path "$WORKSPACE_PATH" \
  "$MOUNT_PATH"

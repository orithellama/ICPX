#!/usr/bin/env sh
set -eu

PROGRAM_NAME="${ICPX_PROGRAM_NAME:-icpx_payments}"
WORKSPACE_PATH="${ICPX_VERIFY_WORKSPACE_PATH:-.}"
MOUNT_PATH="${ICPX_VERIFY_MOUNT_PATH:-.}"

solana-verify build \
  --library-name "$PROGRAM_NAME" \
  --workspace-path "$WORKSPACE_PATH" \
  "$MOUNT_PATH" \
  -- --manifest-path programs/icpx-payments/Cargo.toml

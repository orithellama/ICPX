#!/usr/bin/env sh
set -eu

PROGRAM_ID="Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML"
MULTISIG_AUTHORITY="AgYcC58HhWt9vV8kRro7T77FQgGqpcaBMtNEtNYuKeA1"
RPC_URL="${SOLANA_URL:-https://api.devnet.solana.com}"

solana program set-upgrade-authority "$PROGRAM_ID" \
  --new-upgrade-authority "$MULTISIG_AUTHORITY" \
  --skip-new-upgrade-authority-signer-check \
  --url "$RPC_URL"

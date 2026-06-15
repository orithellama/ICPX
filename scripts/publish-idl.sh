#!/usr/bin/env sh
set -eu

PROGRAM_ID="Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML"
IDL_FILE="idl/icpx_payments.json"

if [ ! -f "$IDL_FILE" ]; then
  echo "missing IDL file: $IDL_FILE" >&2
  exit 1
fi

mkdir -p target/idl
cp "$IDL_FILE" target/idl/icpx_payments.json

echo "published repository IDL for $PROGRAM_ID to target/idl/icpx_payments.json"
echo "this is a native Solana/Borsh program IDL, not an Anchor on-chain IDL account"

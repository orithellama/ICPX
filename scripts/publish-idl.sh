#!/usr/bin/env sh
set -eu

PROGRAM_NAME="${ICPX_PROGRAM_NAME:-icpx_payments}"
PROGRAM_ID="Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML"
ROOT_DIR="$(CDPATH= cd "$(dirname "$0")/.." && pwd)"
IDL_FILE="${ICPX_IDL_FILE:-target/idl/${PROGRAM_NAME}.json}"
ANCHOR_BIN="${ANCHOR_BIN:-}"

if [ -z "$ANCHOR_BIN" ]; then
  if [ -x "$HOME/.avm/bin/anchor" ]; then
    ANCHOR_BIN="$HOME/.avm/bin/anchor"
  else
    ANCHOR_BIN="anchor"
  fi
fi

ANCHOR_VERSION="$("$ANCHOR_BIN" --version 2>/dev/null | awk '{print $2}')"
case "$ANCHOR_VERSION" in
  0.32.*) ;;
  *)
    echo "anchor-cli 0.32.x required; found: ${ANCHOR_VERSION:-not installed}" >&2
    echo "install with: avm install 0.32.1 && avm use 0.32.1" >&2
    exit 1
    ;;
esac

case "$IDL_FILE" in
  /*) IDL_OUT="$IDL_FILE" ;;
  *) IDL_OUT="$ROOT_DIR/$IDL_FILE" ;;
esac

mkdir -p "$(dirname "$IDL_OUT")"
(cd "$ROOT_DIR/programs/icpx-payments" && "$ANCHOR_BIN" idl build \
  -p "$PROGRAM_NAME" \
  -o "$IDL_OUT" \
  --skip-lint)

node -e '
const fs = require("node:fs");
const idlPath = process.argv[1];
const expected = process.argv[2];
const idl = JSON.parse(fs.readFileSync(idlPath, "utf8"));
const actual = idl.address || idl.programId;
if (actual !== expected) {
  throw new Error(`Anchor IDL program id mismatch: expected ${expected}, got ${actual}`);
}
' "$IDL_OUT" "$PROGRAM_ID"

echo "generated Anchor IDL for $PROGRAM_ID at $IDL_OUT"

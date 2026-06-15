#!/usr/bin/env sh
set -eu

fail() {
  echo "repo safety check failed: $1" >&2
  exit 1
}

[ -f LICENSE ] || fail "missing LICENSE"
[ -f SECURITY.md ] || fail "missing SECURITY.md"
[ -f security.txt ] || fail "missing security.txt"
[ -f .well-known/security.txt ] || fail "missing .well-known/security.txt"
[ -f idl/icpx_payments.json ] || fail "missing IDL"
[ -f deploy/devnet.json ] || fail "missing devnet deploy manifest"

FILES="$(mktemp)"
trap 'rm -f "$FILES"' EXIT
git ls-files > "$FILES"
git ls-files --others --exclude-standard >> "$FILES"

if grep -E '(^|/)(\.env|\.env\..*|.*keypair.*\.json|.*-keypair\.json|.*\.pem|.*\.p12)$' "$FILES" >/dev/null; then
  grep -E '(^|/)(\.env|\.env\..*|.*keypair.*\.json|.*-keypair\.json|.*\.pem|.*\.p12)$' "$FILES" >&2
  fail "secret-like files must not be tracked or staged"
fi

if git ls-files 'target/deploy/*' | grep . >/dev/null; then
  git ls-files 'target/deploy/*' >&2
  fail "deployment keypairs and binaries under target/deploy must not be tracked"
fi

if [ -d .github/workflows ]; then
  if grep -R "pull_request_target" .github/workflows >/dev/null; then
    fail "pull_request_target is not allowed"
  fi
  if grep -R "solana program deploy" .github/workflows >/dev/null; then
    fail "workflows must not deploy automatically; use local/manual deploy scripts"
  fi
  if grep -RE 'curl .*\| *(sh|bash)|wget .*\| *(sh|bash)' .github/workflows >/dev/null; then
    fail "workflow pipe-to-shell installers are not allowed"
  fi
fi

if grep -E '^FROM .*:(latest|.*bookworm)' Dockerfile >/dev/null; then
  fail "Dockerfile must not use latest or bookworm base images"
fi

node scripts/check-deploy-metadata.js
echo "repo safety checks passed"

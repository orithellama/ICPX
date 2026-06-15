# Security

## On-Chain Controls

- Payment assets are restricted to `Sol`, `Usdc`, and `Icpx`.
- USDC and ICPX mints are hard-coded as raw bytes.
- The protocol multisig is hard-coded as raw bytes:
  `AgYcC58HhWt9vV8kRro7T77FQgGqpcaBMtNEtNYuKeA1`.
- Protocol fees are fixed at `25` basis points in code.
- SPL token accounts must be initialized, owned by the SPL Token Program, use the expected mint, and belong to the expected owner.
- SPL protocol fee token accounts must be owned by the hard-coded multisig.
- SPL escrow vaults must be owned by the job PDA.
- Native SOL escrow stays in the job PDA.
- Settlement uses cumulative receipts and monotonic nonces.
- Every payment and refund uses checked arithmetic.

## Operational Controls

- Use multisig deployment authority before mainnet.
- Verify the program keypair resolves to
  `Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML` before every devnet deploy.
- Add timelocked upgrades before material value is escrowed.
- Publish verifiable builds and deployment hashes.
- Run unit tests, clippy, integration tests, and Kani before release.
- Commission independent audits before revoking upgrade authority.

## Remaining Work

- Add end-to-end local validator tests for SPL token flows.
- Add ed25519 detached receipt verification.
- Add provider registry and optional staking/slashing.
- Add public fee dashboards and treasury reporting.
- Add dispute resolution and frozen-escrow resolution paths.

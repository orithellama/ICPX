# Immutability, Open Source, And Revenue

## Open Source Model

- Keep the on-chain program source available under the workspace license.
- Publish build instructions, deployment addresses, IDLs or client schemas, and audit reports.
- Prefer permissive licensing for protocol adoption, with separate commercial terms for hosted services if needed.

## Protocol Revenue

- Charge a small settlement fee on streamed payments. Current code sets this to
  `25` basis points, or `0.25%`.
- Route protocol fees to the hard-coded multisig
  `AgYcC58HhWt9vV8kRro7T77FQgGqpcaBMtNEtNYuKeA1`.
- Charge provider listing or verification fees for marketplace visibility.
- Offer premium hosted MCP, RPC, and indexer services.
- Offer enterprise support, compliance reporting, SLAs, and private deployment help.
- Offer optional hosted dispute, reputation, and provider-risk services.

## Immutability Path

- Start with a multisig upgrade authority while the protocol is still changing.
- Add a public timelock before upgrades once real value is escrowed.
- Deploy audited code with reproducible build artifacts.
- Publish the exact source, build command, program hash, and deployment transaction.
- Revoke Solana program upgrade authority, or transfer it to a timelocked DAO or multisig.
- Freeze only after audits, monitoring, and incident response processes have proven stable.

## Practical Recommendation

The safest route is staged decentralization: multisig during beta, multisig plus
timelock for mainnet growth, then revoked upgrade authority once the code and
economic model have real production history.

## Revenue Without Breaking Trust

- Keep settlement fees transparent and capped.
- Put protocol fees in state and events so indexers can verify them.
- Hard-code fee bps and the fee authority for the immutable release.
- Make hosted services optional, not required for protocol use.
- Keep core escrow settlement open and permissionless.
- Let enterprise revenue come from reliability, support, data, and integrations rather than hidden custody or lock-in.

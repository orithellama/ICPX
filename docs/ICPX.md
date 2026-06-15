# ICPX Rust MCP Server With Solana Streaming Payments

ICPX lets an agent connect a wallet, request compute, and pay per job instead
of renting capacity by the hour. The MCP surface is implemented as an off-chain
Rust server. The payment and job state are enforced by an on-chain Solana Rust
program.

The boundary is important: Solana programs cannot run an MCP server, open
sockets, or handle stdio/HTTP transports. The callable path is:

```text
Agent -> Rust MCP tool call -> wallet signature -> Solana transaction
      -> on-chain ICPX program -> escrowed streaming payment settlement
```

## Product Goal

- Agents connect a wallet once and can request compute through MCP tools.
- Each request creates a job with a max budget, pricing terms, and metadata hash.
- SOL, USDC, or ICPX funds are escrowed on Solana before work starts.
- Providers are paid incrementally as compute receipts are submitted.
- Unused escrow is returned when the job completes, expires, or is cancelled.

## System Components

### Rust MCP Server

The MCP server is the agent-facing entry point. It exposes tools that prepare,
submit, and monitor Solana transactions for compute jobs.

Responsibilities:

- Accept MCP requests from local or remote agents.
- Connect to a wallet without taking custody of private keys.
- Build Solana transactions for job creation, funding, streaming settlement,
  completion, cancellation, and lookup.
- Submit signed transactions to Solana RPC.
- Return job status, escrow state, payment state, and provider results to the
  agent.

### On-Chain Solana Program

The on-chain Rust program owns job state and escrow settlement.

Responsibilities:

- Create deterministic job and escrow PDAs.
- Hold prepaid SOL, USDC, or ICPX funds in escrow.
- Enforce job status transitions.
- Release payments only according to accepted streaming receipts.
- Prevent double settlement with cumulative unit accounting.
- Refund unused escrow after completion, timeout, or cancellation.

### Compute Provider

The provider executes the job off-chain and publishes metering receipts.

Responsibilities:

- Accept funded jobs.
- Execute compute work.
- Produce signed receipts for completed compute units or milestones.
- Submit results and settlement receipts.

## Payment Model

ICPX uses prepaid streaming escrow rather than open-ended billing.

```text
max_budget = price_per_unit * max_units
gross_settlement = (cumulative_receipted_units - already_settled_units) * price_per_unit
protocol_fee = gross_settlement * 25 / 10_000
provider_payment = gross_settlement - protocol_fee
refund = escrow_balance - total_paid_to_provider - total_protocol_fees
```

This makes every job bounded. A provider can stream claims as work progresses,
but cannot withdraw more than the funded job budget.

The on-chain program supports only three payment assets:

- `Sol`: native SOL lamports held by the job PDA.
- `Usdc`: SPL token escrow using the hard-coded mainnet USDC mint.
- `Icpx`: SPL token escrow using the hard-coded ICPX mint.

Clients select the asset enum. They never pass an arbitrary mint address.

The protocol fee is immutable in code for this release:

- Fee: `25` basis points, or `0.25%`.
- Authority: `AgYcC58HhWt9vV8kRro7T77FQgGqpcaBMtNEtNYuKeA1`.
- The authority pubkey is stored as raw 32 bytes in the program constants.
- SOL fees are paid directly to the multisig wallet.
- USDC and ICPX fees are paid to token accounts owned by the multisig.

### Frontend Variable Pricing

Pricing is intentionally variable per job. The frontend or quote engine sets
`price_per_unit`, `max_units`, and `payment_asset` before the requester signs.
The program does not hard-code prices; it enforces the requester-approved terms
with checked arithmetic and a fixed maximum budget:

```text
max_budget = frontend_price_per_unit * requester_approved_max_units
```

For enterprise use, the frontend should show the provider, asset, unit type,
unit price, max units, and total maximum budget before wallet approval. If a
quote needs auditability, store the quote off-chain and commit its hash in
`metadata_hash`.

Supported settlement modes:

- `unit_stream`: pay by cumulative compute units in signed receipts.
- `milestone_stream`: pay by agreed milestone index and amount.
- `slot_stream`: pay by elapsed slots while the job is running, capped by the
  max budget.

The MVP should use `unit_stream` because it maps cleanly to per-request compute
and avoids paying purely for wall-clock time.

## On-Chain Accounts

### `Job`

```rust
pub struct Job {
    pub requester: Pubkey,
    pub provider: Pubkey,
    pub receipt_authority: Pubkey,
    pub escrow_vault: Pubkey,
    pub metadata_hash: [u8; 32],
    pub gpu_profile_hash: [u8; 32],
    pub nvidia_api_hash: [u8; 32],
    pub result_hash: [u8; 32],
    pub metering_unit: GpuMeteringUnit,
    pub payment_asset: PaymentAsset,
    pub price_per_unit: u64,
    pub max_units: u64,
    pub escrow_funded_amount: u64,
    pub settled_units: u64,
    pub total_paid_amount: u64,
    pub total_protocol_fee_amount: u64,
    pub total_refunded_amount: u64,
    pub created_slot: u64,
    pub start_slot: u64,
    pub expiry_slot: u64,
    pub last_receipt_nonce: u64,
    pub status: JobStatus,
    pub bump: u8,
}
```

### `JobStatus`

```rust
pub enum JobStatus {
    Created,
    Funded,
    Running,
    Completed,
    Cancelled,
    Expired,
    Disputed,
}
```

### `StreamReceipt`

Receipts are generated off-chain and submitted to the on-chain program for
settlement. The program should settle against cumulative values, not deltas, so
that replayed receipts cannot overpay.

```rust
pub struct StreamReceipt {
    pub job: Pubkey,
    pub provider: Pubkey,
    pub requester: Pubkey,
    pub cumulative_units: u64,
    pub result_hash: [u8; 32],
    pub receipt_nonce: u64,
}
```

For the MVP, `receipt_authority` signs the settlement transaction. For detached
receipts, include a Solana ed25519 verification instruction and have the program
inspect it before releasing funds.

## On-Chain Instructions

### `create_job`

Creates the `Job` PDA and records immutable request terms.

Inputs:

- `provider`
- `receipt_authority`
- `metadata_hash`
- `gpu_profile_hash`
- `nvidia_api_hash`
- `metering_unit`
- `payment_asset`
- `price_per_unit`
- `max_units`
- `expiry_slot`

Checks:

- `max_units > 0`
- `price_per_unit > 0`
- `expiry_slot > current_slot`
- job PDA is derived from requester, provider, and client nonce
- payment asset is one of `Sol`, `Usdc`, or `Icpx`

### `fund_job`

Transfers the max budget into escrow and moves the job to `Funded`.

Account layout:

- SOL: `requester`, `job`, `system_program`
- USDC/ICPX: `requester`, `job`, `requester_token_account`,
  `escrow_token_account`, `token_program`

Checks:

- signer is `requester`
- escrow amount equals `price_per_unit * max_units`
- checked arithmetic is used for all multiplication
- SPL token accounts use the hard-coded mint for the selected asset
- SPL escrow token account is owned by the job PDA

### `accept_job`

Lets the provider accept a funded job and moves it to `Running`.

Checks:

- signer is `provider`
- job status is `Funded`
- current slot is before `expiry_slot`

### `settle_stream`

Pays the provider for newly receipted compute.

Account layout:

- SOL: `receipt_authority`, `job`, `provider_wallet`,
  `protocol_fee_wallet`
- USDC/ICPX: `receipt_authority`, `job`, `provider_token_account`,
  `protocol_fee_token_account`, `escrow_token_account`, `token_program`

Checks:

- job status is `Running`
- signer is `receipt_authority` or verified detached receipt is present
- receipt job, requester, and provider match the account data
- `cumulative_units <= max_units`
- `cumulative_units > settled_units`
- SPL token accounts use the hard-coded mint for the selected asset
- provider token account is owned by the job provider
- protocol fee destination is the hard-coded multisig or a token account owned
  by the hard-coded multisig

Settlement:

```text
new_units = cumulative_units - settled_units
gross_payment = new_units * price_per_unit
protocol_fee = gross_payment * 25 / 10_000
provider_payment = gross_payment - protocol_fee
settled_units = cumulative_units
total_paid = total_paid + provider_payment
total_protocol_fee = total_protocol_fee + protocol_fee
```

### `complete_job`

Stores the final result hash, pays any remaining receipted amount, refunds unused
escrow, and moves the job to `Completed`.

Checks:

- signer is `requester`, `provider`, or `receipt_authority`
- final receipt does not exceed `max_units`
- all transfers use the escrow PDA authority
- refunds return only to the requester wallet or requester-owned token account
- protocol fee destination is validated before collecting any final-settlement
  fee

### `cancel_expired_job`

Refunds unused escrow after the expiry slot when the job is not completed.

Checks:

- current slot is greater than `expiry_slot`
- status is `Created`, `Funded`, or `Running`
- provider keeps already settled funds
- requester receives remaining escrow

### `open_dispute`

Freezes the remaining escrow for manual or automated resolution.

Checks:

- signer is `requester` or `provider`
- job status is `Running`
- dispute window is still open

## MCP Tools

The Rust MCP server should expose these tools.

### `icpx_create_job`

Creates and funds an on-chain compute job.

Input:

```json
{
  "provider": "solana_pubkey",
  "metadata_hash": "hex_32_bytes",
  "gpu_profile_hash": "hex_32_bytes",
  "nvidia_api_hash": "hex_32_bytes",
  "payment_asset": "Sol | Usdc | Icpx",
  "price_per_unit": 10,
  "max_units": 100000,
  "expiry_slot": 123456789
}
```

Output:

```json
{
  "job": "job_pda",
  "escrow": "escrow_vault_pda",
  "status": "Funded",
  "signature": "solana_tx_signature"
}
```

### `icpx_accept_job`

Accepts a funded job as the selected provider.

Input:

```json
{
  "job": "job_pda"
}
```

### `icpx_settle_stream`

Submits a cumulative compute receipt and streams payment from escrow.

Input:

```json
{
  "job": "job_pda",
  "cumulative_units": 42000,
  "result_hash": "hex_32_bytes",
  "receipt_nonce": 7
}
```

### `icpx_complete_job`

Finalizes the job, stores the result hash, pays receipted work, and refunds
unused escrow.

Input:

```json
{
  "job": "job_pda",
  "final_units": 82000,
  "result_hash": "hex_32_bytes"
}
```

### `icpx_cancel_expired_job`

Cancels an expired job and refunds unspent escrow.

Input:

```json
{
  "job": "job_pda"
}
```

### `icpx_get_job`

Reads the current on-chain job state.

Input:

```json
{
  "job": "job_pda"
}
```

Output:

```json
{
  "status": "Running",
  "requester": "solana_pubkey",
  "provider": "solana_pubkey",
  "settled_units": 42000,
  "max_units": 100000,
  "total_paid": 420000,
  "escrow_remaining": 580000
}
```

## Wallet Connection

The MCP server must not custody the requester wallet. Use one of these signing
paths:

- Local development: file-system keypair controlled by the user.
- Desktop wallet: transaction handoff for wallet approval.
- Remote agent: session key authorized by the wallet with strict spending caps.

Recommended session authorization fields:

- `requester`
- `session_pubkey`
- `max_lamports_or_tokens`
- `allowed_provider`
- `allowed_payment_asset`
- `expires_at_slot`
- `nonce`

The on-chain program should reject session-authorized transactions that exceed
the approved budget, provider, payment asset, or expiry.

## Rust Workspace Shape

```text
icpx/
  programs/
    icpx-payments/
      src/constants.rs        # Hard-coded mints, seeds, and program constants
      src/errors.rs           # Stable custom error codes and messages
      src/events.rs           # Borsh-encoded event payloads
      src/instructions/       # On-chain instruction handlers
      src/math/               # Checked settlement and pricing math
      src/state/              # Job state, payment assets, and enums
      src/lib.rs              # Solana on-chain Rust program entry
  docs/
    how-to.md                 # What the protocol does and how to use it
    doc1.md                   # Short product and architecture overview
    security.md               # Security model and remaining work
    verification.md           # Test and Kani verification notes
    immutability-revenue.md   # Open-source, immutability, and revenue plan
  tests/
    payment_flow.test.ts        # TypeScript safety and integration checklist
  idl/
    icpx_payments.json          # Repository-published native Borsh IDL
  deploy/
    devnet.json                 # Pinned devnet program id and deploy metadata
  scripts/
    check-program-id.sh         # Refuses keypairs that resolve to another id
    deploy-devnet.sh            # Builds and deploys with the pinned id
    publish-idl.sh              # Copies the IDL into target/idl
    set-devnet-upgrade-authority.sh # Transfers authority to hard-coded multisig
  crates/
    icpx-mcp-server/
      src/main.rs             # MCP transport and tool registration
      src/tools.rs            # MCP tool handlers
      src/solana.rs           # RPC, transaction builders, account decoding
      src/wallet.rs           # signing abstraction
    icpx-types/
      src/lib.rs              # shared request/receipt/status types
```

## Security Requirements

- Never expose private keys through MCP tool inputs, logs, or resources.
- Keep the program keypair stable and verify it resolves to
  `Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML` before devnet deploys.
- Use PDA-owned escrow vaults so neither requester nor provider can bypass the
  program.
- Hard-code supported token mints and reject arbitrary mint accounts.
- Validate SPL token account mint, owner, and token program before every token
  transfer.
- Use checked arithmetic for every price, unit, and token calculation.
- Settle from cumulative receipts to prevent replay overpayment.
- Bind receipts to `job`, `requester`, `provider`, `result_hash`, and nonce.
- Enforce status transitions on-chain.
- Cap every job with `max_units`, `expiry_slot`, and a funded escrow amount.
- Use Kani verification for pure settlement math and local validator tests for
  Solana account/CPI behavior.
- Treat compute metering as an attestation problem. If receipts must be
  trustless, add a verifier layer such as TEE attestation, zk proofs, or a
  provider reputation and slashing system.

## Immutability, Open Source, And Revenue

ICPX should be open source at the protocol layer while earning revenue from
transparent protocol fees and optional hosted services.

Recommended model:

- Keep the program source open under the workspace license.
- Charge a small settlement fee at the protocol level once fee accounting is
  implemented. The current hard-coded fee is `25` bps.
- Add provider listing or verification fees for marketplace distribution.
- Sell premium hosted MCP, RPC, and indexer service tiers.
- Offer enterprise support, compliance reporting, private integrations, and
  optional hosted dispute or reputation services.

Recommended immutability path:

- Start with a multisig upgrade authority for bug-fix safety.
- Add a public timelock before material mainnet usage.
- Deploy audited code and publish reproducible build artifacts.
- Publish source, build commands, program hash, and deployment transactions.
- Revoke upgrade authority or transfer it to a timelocked DAO/multisig after
  audits and production usage prove the code is stable.

## MVP Build Plan

1. Create the Solana Rust program with `create_job`, `fund_job`, `accept_job`,
   `settle_stream`, `complete_job`, and `cancel_expired_job`.
2. Add local validator tests for escrow funding, streaming settlement, overpay
   rejection, expiry refund, and invalid signer rejection.
3. Add Kani verification for pure payment and settlement math.
4. Build the Rust MCP server with stdio transport first.
5. Implement wallet signing behind a trait so local keypair, wallet handoff, and
   session keys can share the same tool handlers.
6. Add MCP tools for job create, accept, settle, complete, cancel, and read.
7. Add an end-to-end local demo where an agent creates a job, streams two
   settlements, completes the job, and receives the unused escrow refund.

## Open Decisions

- Whether providers are permissionless or registered in an on-chain provider
  registry.
- Whether compute receipts are signed by the provider, requester, a metering
  authority, or a verifier network.
- Whether disputes are manual for MVP or resolved by an on-chain verifier.
- Whether remote agents use wallet-approved session keys or require every MCP
  action to return a transaction for explicit wallet approval.

# ICPX Overview

ICPX is a Solana payment protocol for agent-driven compute jobs. A requester
pre-funds a bounded escrow, a provider performs the work off-chain, and the
program releases payments as cumulative compute receipts are settled.

## Core Flow

- The requester creates a job with immutable GPU terms, payment asset, unit price, max units, and expiry.
- The requester funds the job escrow with native SOL, USDC, or ICPX.
- The provider accepts the funded job and starts work.
- The receipt authority submits cumulative usage receipts.
- The program pays only for newly receipted units and records total paid.
- Each settlement takes a fixed `25` bps protocol fee to the hard-coded multisig.
- Completion stores the result hash and refunds unused escrow.
- Expiry allows anyone to trigger refund of unspent escrow to the requester.

## Supported Assets

- `Sol`: native lamports held directly by the job PDA.
- `Usdc`: SPL token escrow restricted to the hard-coded USDC mint.
- `Icpx`: SPL token escrow restricted to the hard-coded ICPX mint.

The program does not accept arbitrary mint accounts from clients.

## Frontend Pricing

Pricing is variable per job. The frontend can quote any `price_per_unit` and
`max_units` that the requester approves, and the on-chain program enforces the
resulting maximum budget with checked arithmetic.

## Enterprise Readiness

- Stable custom errors for client UX and support triage.
- Stable events for indexers and accounting exports.
- Hard-coded mint and token program validation.
- PDA-owned escrows.
- Cumulative receipt accounting.
- Hard-coded protocol fee authority:
  `AgYcC58HhWt9vV8kRro7T77FQgGqpcaBMtNEtNYuKeA1`.
- Kani verification for pure settlement math.
- Docker and Make targets for repeatable local validation.

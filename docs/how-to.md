# ICPX How-To

## What It Does

- Creates bounded GPU compute jobs with requester, provider, receipt authority, metadata hash, and expiry.
- Escrows payment before work starts, using native SOL, hard-coded USDC, or hard-coded ICPX.
- Lets the frontend set variable `price_per_unit` and `max_units` per job.
- Pays providers through cumulative stream receipts so replayed receipts cannot overpay.
- Collects a fixed `25` bps protocol fee to the hard-coded multisig.
- Refunds unused escrow when a job completes or expires.
- Emits stable Borsh events for indexers, dashboards, and enterprise reporting.
- Keeps mint selection non-configurable by clients; only `Sol`, `Usdc`, and `Icpx` are valid.

## How To Build

- Run `make build` to compile the Rust workspace.
- Run `make build-sbf` to compile the Solana SBF program.
- Run `make check` for a fast compiler check.
- Run `make test` for Rust unit tests.
- Run `make clippy` for linting with warnings denied.
- Run `make docker-test` to run tests inside the Docker development image.

## How To Verify

- Install Kani locally from the official Kani distribution.
- Run `make verify-kani` to execute the formal math harnesses.
- Treat Kani as a proof layer for pure accounting math, not a replacement for Solana integration tests.

## How To Create A Job

- Derive the job PDA from `JOB_SEED`, requester, provider, and `client_nonce`.
- Choose `payment_asset`: `Sol`, `Usdc`, or `Icpx`.
- Let the frontend quote and set `price_per_unit`.
- Let the frontend set `max_units` from the requester-approved budget.
- Submit `CreateGpuJob` with immutable pricing and GPU terms.
- Submit `FundJob` with SOL accounts for `Sol` or token accounts for `Usdc` and `Icpx`.

## How To Price From The Frontend

- Fetch provider pricing from the provider registry, API, or marketplace quote engine.
- Present the requester with `price_per_unit`, `max_units`, and total max budget.
- Require explicit wallet approval for `price_per_unit * max_units`.
- Pass the selected `price_per_unit` directly into `CreateGpuJobArgs`.
- Never let the frontend pass arbitrary mint addresses; it only selects the payment asset enum.
- Store the quote details off-chain and commit their hash in `metadata_hash` if the UI needs auditability.

## How To Deploy Devnet

- The pinned devnet program id is `Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML`.
- The deploy guard expects `target/deploy/icpx_payments-keypair.json` to resolve to that address.
- Run `make check-program-id` before deploys.
- Run `make build-sbf` to rebuild the program binary.
- Run `make deploy-devnet` to deploy to devnet.
- Run `make set-devnet-upgrade-authority` after initial deploys if authority
  needs to be transferred back to the hard-coded multisig.
- Set `ICPX_PROGRAM_KEYPAIR=/path/to/icpx_payments-keypair.json` if using a secure external keypair store.
- Set `ICPX_UPGRADE_AUTHORITY=/path/to/multisig-authority-signer.json` for
  future upgrades after the upgrade authority has been transferred.
- Do not generate a new program keypair for redeploys; that changes the program address.

## How To Publish The IDL

- The repository IDL lives at `idl/icpx_payments.json`.
- Run `make idl` to copy it to `target/idl/icpx_payments.json`.
- The program is native Solana/Borsh, not Anchor, so this is a repository-published IDL.
- Keep `programId` in the IDL equal to the pinned devnet program id.

## How To Settle

- Providers produce cumulative receipts with `cumulative_units`, `result_hash`, and `receipt_nonce`.
- The receipt authority submits `SettleStream`.
- The program calculates only newly receipted units and transfers the corresponding amount.
- Settlement splits each gross payment into provider payout plus protocol fee.
- The program rejects stale nonces, unit decreases, over-budget receipts, wrong signers, wrong mints, and wrong token owners.

## How To Operate Safely

- Use a multisig upgrade authority during beta.
- Add a timelock before mainnet usage grows.
- Publish reproducible build artifacts for every deployment.
- Revoke upgrade authority only after audits, monitoring, and incident response processes are mature.

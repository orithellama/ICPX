# Security Policy

## Reporting Vulnerabilities

Report suspected vulnerabilities privately through GitHub Security Advisories:

https://github.com/orithellama/ICPX/security/advisories/new

Do not open public issues, pull requests, discussions, or social posts for
active vulnerabilities before a fix or mitigation is available.

Include as much of the following as possible:

- Affected component, cluster, program id, commit, package version, or script.
- Impact description, including whether funds, upgrade authority, or user data
  can be affected.
- Reproduction steps, proof of concept, transaction signatures, logs, or test
  cases.
- Preconditions, required accounts, signer assumptions, and affected assets.
- Suggested remediation, if known.
- A safe contact method for follow-up.

Do not exploit the issue beyond the minimum needed to prove impact. Do not move,
freeze, drain, or otherwise interact with third-party funds or accounts.

## Supported Scope

- On-chain ICPX payment program code.
- Deployment scripts and program-id verification.
- IDL and client-facing account layout documentation.
- Docker and local development tooling.

Out of scope:

- Social engineering, phishing, or physical attacks.
- Denial-of-service reports without a practical security impact.
- Scanner-only findings without an exploitable condition.
- Issues in third-party services unless they directly affect this repository or
  deployed program.

## Response Targets

- Critical: acknowledge within 24 hours.
- High: acknowledge within 48 hours.
- Medium and low: acknowledge within 5 business days.

## Triage Process

1. A maintainer acknowledges the report privately and assigns an initial
   severity.
2. Maintainers reproduce the issue in a local test, devnet, or read-only
   mainnet setting where possible.
3. If the issue affects deployed contracts, maintainers assess whether funds are
   at risk, whether an upgrade is required, and whether users or integrators
   need temporary guidance.
4. Fixes are prepared privately using tests, clippy, repo safety checks, and
   Kani proofs where applicable.
5. For on-chain fixes, maintainers publish the source commit, build command,
   program hash, deployment transaction, and updated IDL or client guidance.
6. After mitigation, maintainers publish a security advisory with impact,
   affected versions or deployments, remediation, and reporter credit if wanted.

## Severity Guide

- Critical: direct theft or permanent loss of funds, unauthorized upgrade
  authority control, or arbitrary account compromise.
- High: bypass of escrow accounting, settlement authorization, protocol fee
  ownership, or asset/mint validation.
- Medium: denial of service, stuck funds with a practical path to trigger, or
  incorrect accounting that requires uncommon preconditions.
- Low: hardening issues, misleading documentation, or low-impact validation
  gaps.

## Disclosure

Security fixes should be prepared privately, validated with tests and Kani where
applicable, then disclosed after patched code or a deployed mitigation is
available.

The default coordinated disclosure window is up to 90 days. Critical active
exploitation may require faster public guidance. Reporter credit is optional and
will only be published with the reporter's consent.

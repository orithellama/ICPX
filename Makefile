CARGO ?= cargo
DOCKER_COMPOSE ?= docker compose

.PHONY: build build-sbf check check-program-id clean clippy deploy-devnet docker-build docker-test fmt idl set-devnet-upgrade-authority test verify-kani

build:
	$(CARGO) build --workspace

build-sbf:
	$(CARGO) build-sbf --manifest-path programs/icpx-payments/Cargo.toml

check:
	$(CARGO) check --workspace

check-program-id:
	./scripts/check-program-id.sh

test:
	$(CARGO) test --workspace

idl:
	./scripts/publish-idl.sh

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

verify-kani:
	$(CARGO) kani --package icpx-payments --harness checked_payment_amount_raw_matches_u64_checked_mul
	$(CARGO) kani --package icpx-payments --harness checked_payment_amount_raw_rejects_known_overflow
	$(CARGO) kani --package icpx-payments --harness checked_protocol_fee_amount_raw_never_exceeds_gross_payment
	$(CARGO) kani --package icpx-payments --harness protocol_fee_basis_points_are_below_denominator
	$(CARGO) kani --package icpx-payments --harness checked_protocol_fee_amount_raw_rejects_known_overflow
	$(CARGO) kani --package icpx-payments --harness checked_new_units_rejects_invalid_cumulative_units
	$(CARGO) kani --package icpx-payments --harness checked_new_units_returns_exact_delta_for_valid_units
	$(CARGO) kani --package icpx-payments --harness quote_stream_settlement_raw_rejects_over_escrow
	$(CARGO) kani --package icpx-payments --harness quote_stream_settlement_raw_rejects_gross_payment_overflow
	$(CARGO) kani --package icpx-payments --harness quote_stream_settlement_raw_never_quotes_more_than_escrow

clean:
	$(CARGO) clean

docker-build:
	$(DOCKER_COMPOSE) build

docker-test:
	$(DOCKER_COMPOSE) run --rm icpx

deploy-devnet:
	./scripts/deploy-devnet.sh

set-devnet-upgrade-authority:
	./scripts/set-devnet-upgrade-authority.sh

import assert from "node:assert/strict";
import test from "node:test";

const SUPPORTED_PAYMENT_ASSETS = ["Sol", "Usdc", "Icpx"] as const;

type PaymentAsset = (typeof SUPPORTED_PAYMENT_ASSETS)[number];

type FrontendQuote = {
  paymentAsset: PaymentAsset;
  pricePerUnit: bigint;
  maxUnits: bigint;
};

function maxBudget({ pricePerUnit, maxUnits }: FrontendQuote): bigint {
  if (pricePerUnit <= 0n) {
    throw new Error("price_per_unit must be greater than zero");
  }
  if (maxUnits <= 0n) {
    throw new Error("max_units must be greater than zero");
  }
  return pricePerUnit * maxUnits;
}

test("documents supported payment assets", () => {
  assert.deepEqual(SUPPORTED_PAYMENT_ASSETS, ["Sol", "Usdc", "Icpx"]);
});

test("frontend can set variable price per unit", () => {
  const lowPriorityQuote = {
    paymentAsset: "Icpx",
    pricePerUnit: 2n,
    maxUnits: 1_000n,
  } satisfies FrontendQuote;
  const enterpriseQuote = {
    paymentAsset: "Icpx",
    pricePerUnit: 25n,
    maxUnits: 1_000n,
  } satisfies FrontendQuote;

  assert.equal(maxBudget(lowPriorityQuote), 2_000n);
  assert.equal(maxBudget(enterpriseQuote), 25_000n);
});

test("frontend quote rejects zero pricing", () => {
  assert.throws(() =>
    maxBudget({
      paymentAsset: "Usdc",
      pricePerUnit: 0n,
      maxUnits: 100n,
    }),
  );
});

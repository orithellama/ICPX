import assert from "node:assert/strict";
import test from "node:test";

const SUPPORTED_PAYMENT_ASSETS = ["Sol", "Usdc", "Icpx"] as const;

type PaymentAsset = (typeof SUPPORTED_PAYMENT_ASSETS)[number];

type FrontendQuote = {
  paymentAsset: PaymentAsset;
  pricePerUnit: bigint;
  maxUnits: bigint;
};

const PROTOCOL_FEE_BASIS_POINTS = 25n;
const BASIS_POINTS_DENOMINATOR = 10_000n;

function maxBudget({ pricePerUnit, maxUnits }: FrontendQuote): bigint {
  if (pricePerUnit <= 0n) {
    throw new Error("price_per_unit must be greater than zero");
  }
  if (maxUnits <= 0n) {
    throw new Error("max_units must be greater than zero");
  }
  return pricePerUnit * maxUnits;
}

function protocolFee(grossPayment: bigint): bigint {
  if (grossPayment < 0n) {
    throw new Error("gross payment cannot be negative");
  }
  return (grossPayment * PROTOCOL_FEE_BASIS_POINTS) / BASIS_POINTS_DENOMINATOR;
}

function validatePaymentAsset(paymentAsset: string): asserts paymentAsset is PaymentAsset {
  if (!SUPPORTED_PAYMENT_ASSETS.includes(paymentAsset as PaymentAsset)) {
    throw new Error("unsupported payment asset");
  }
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

test("frontend rejects unsupported payment assets at runtime", () => {
  assert.throws(() => validatePaymentAsset("FakeMint"));
  assert.doesNotThrow(() => validatePaymentAsset("Sol"));
});

test("frontend displays protocol fee split deterministically", () => {
  const grossPayment = 100_000n;
  const fee = protocolFee(grossPayment);
  const providerPayment = grossPayment - fee;

  assert.equal(fee, 250n);
  assert.equal(providerPayment, 99_750n);
});

test("frontend fee calculation rounds down like the program", () => {
  assert.equal(protocolFee(399n), 0n);
  assert.equal(protocolFee(400n), 1n);
});

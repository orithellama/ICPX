import fs from "node:fs";

const expected = {
  programId: "Dmz8DZUBr6RUZsyTMqoBDB6x5TjmaFgjCmSALa1LzJML",
  protocolMultisig: "AgYcC58HhWt9vV8kRro7T77FQgGqpcaBMtNEtNYuKeA1",
  protocolFeeBasisPoints: 25,
  cluster: "devnet",
  supportedPaymentAssets: ["Sol", "Usdc", "Icpx"],
  icpxMint: "HdeAPoHivsm9MZfeY5tW7apJEprc8Fs594bWmnzfpump",
  usdcMint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  wrappedSolMint: "So11111111111111111111111111111111111111112",
  splTokenProgram: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
};

function readJson(path) {
  return JSON.parse(fs.readFileSync(path, "utf8"));
}

function assertEqual(actual, expectedValue, label) {
  if (actual !== expectedValue) {
    throw new Error(`${label} mismatch: expected ${expectedValue}, got ${actual}`);
  }
}

function assertArrayEqual(actual, expectedValue, label) {
  if (
    actual.length !== expectedValue.length ||
    actual.some((value, index) => value !== expectedValue[index])
  ) {
    throw new Error(`${label} mismatch`);
  }
}

function decodeBase58(address) {
  const alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
  const bytes = [0];

  for (const character of address) {
    const value = alphabet.indexOf(character);
    if (value === -1) {
      throw new Error(`invalid base58 character in ${address}`);
    }

    let carry = value;
    for (let index = 0; index < bytes.length; index += 1) {
      const next = bytes[index] * 58 + carry;
      bytes[index] = next & 0xff;
      carry = next >> 8;
    }

    while (carry > 0) {
      bytes.push(carry & 0xff);
      carry >>= 8;
    }
  }

  for (const character of address) {
    if (character !== "1") {
      break;
    }
    bytes.push(0);
  }

  return bytes.reverse();
}

function readRustConstBytes(source, name) {
  const regex = new RegExp(`pub const ${name}: \\[u8; 32\\] = \\[([\\s\\S]*?)\\];`);
  const match = source.match(regex);
  if (!match) {
    throw new Error(`missing Rust byte constant ${name}`);
  }

  const values = [...match[1].matchAll(/0x[0-9a-fA-F]+|\d+/g)].map((value) =>
    Number.parseInt(value[0], 0),
  );

  if (values.length !== 32 || values.some((value) => value < 0 || value > 255)) {
    throw new Error(`invalid Rust byte constant ${name}`);
  }

  return values;
}

const deploy = readJson("deploy/devnet.json");
const idl = readJson("idl/icpx_payments.json");
const targetIdl = fs.existsSync("target/idl/icpx_payments.json")
  ? readJson("target/idl/icpx_payments.json")
  : null;
const rustConstants = fs.readFileSync("programs/icpx-payments/src/constants.rs", "utf8");

assertEqual(deploy.cluster, expected.cluster, "deploy cluster");
assertEqual(deploy.programId, expected.programId, "deploy programId");
assertEqual(deploy.upgradeAuthority, expected.protocolMultisig, "deploy upgradeAuthority");
assertEqual(
  deploy.protocolFeeBasisPoints,
  expected.protocolFeeBasisPoints,
  "deploy protocolFeeBasisPoints",
);

assertEqual(idl.programId, expected.programId, "idl programId");
assertEqual(idl.metadata.cluster, expected.cluster, "idl cluster");
assertEqual(idl.metadata.protocolMultisig, expected.protocolMultisig, "idl protocolMultisig");
assertEqual(idl.metadata.icpxMint, expected.icpxMint, "idl icpxMint");
assertEqual(idl.metadata.usdcMint, expected.usdcMint, "idl usdcMint");
assertArrayEqual(
  idl.metadata.supportedPaymentAssets,
  expected.supportedPaymentAssets,
  "idl supportedPaymentAssets",
);
assertEqual(
  idl.metadata.protocolFeeBasisPoints,
  expected.protocolFeeBasisPoints,
  "idl protocolFeeBasisPoints",
);

if (targetIdl) {
  const targetProgramId = targetIdl.address ?? targetIdl.programId;
  assertEqual(targetProgramId, expected.programId, "target idl programId");
  if (targetIdl.metadata?.protocolMultisig) {
    assertEqual(
      targetIdl.metadata.protocolMultisig,
      idl.metadata.protocolMultisig,
      "target idl protocolMultisig",
    );
  }
}

assertArrayEqual(
  readRustConstBytes(rustConstants, "ICPX_MINT_BYTES"),
  decodeBase58(expected.icpxMint),
  "Rust ICPX mint bytes",
);
assertArrayEqual(
  readRustConstBytes(rustConstants, "USDC_MINT_BYTES"),
  decodeBase58(expected.usdcMint),
  "Rust USDC mint bytes",
);
assertArrayEqual(
  readRustConstBytes(rustConstants, "WRAPPED_SOL_MINT_BYTES"),
  decodeBase58(expected.wrappedSolMint),
  "Rust wrapped SOL mint bytes",
);
assertArrayEqual(
  readRustConstBytes(rustConstants, "SPL_TOKEN_PROGRAM_BYTES"),
  decodeBase58(expected.splTokenProgram),
  "Rust SPL token program bytes",
);
assertArrayEqual(
  readRustConstBytes(rustConstants, "PROTOCOL_MULTISIG_BYTES"),
  decodeBase58(expected.protocolMultisig),
  "Rust protocol multisig bytes",
);

const feeMatch = rustConstants.match(/pub const PROTOCOL_FEE_BASIS_POINTS: u64 = (\d+);/);
if (!feeMatch) {
  throw new Error("missing Rust protocol fee constant");
}
assertEqual(
  Number.parseInt(feeMatch[1], 10),
  expected.protocolFeeBasisPoints,
  "Rust protocol fee basis points",
);

const deployScript = fs.readFileSync("scripts/deploy-devnet.sh", "utf8");
if (!deployScript.includes(expected.programId)) {
  throw new Error("deploy script does not contain the pinned program id");
}

const authorityScript = fs.readFileSync("scripts/set-devnet-upgrade-authority.sh", "utf8");
if (!authorityScript.includes(expected.protocolMultisig)) {
  throw new Error("authority script does not contain the pinned multisig");
}

console.log("deploy metadata is consistent");

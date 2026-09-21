import { Noir } from "@noir-lang/noir_js";

// ============ Circuit loading ============
let hashesCircuitPromise: Promise<any> | null = null;
let hash2CircuitPromise: Promise<any> | null = null;
let withdrawalCircuitPromise: Promise<any> | null = null;

// ============ Noir instance cache ============
let hashesNoirPromise: Promise<Noir> | null = null;
let hash2NoirPromise: Promise<Noir> | null = null;
let withdrawalNoirPromise: Promise<Noir> | null = null;

async function loadHashesCircuit(): Promise<any> {
  if (!hashesCircuitPromise) {
    hashesCircuitPromise = fetch("/circuits/hashes.json").then((r) => {
      if (!r.ok) throw new Error(`Failed to load hashes.json: ${r.status}`);
      return r.json();
    });
  }
  return hashesCircuitPromise;
}

async function loadHash2Circuit(): Promise<any> {
  if (!hash2CircuitPromise) {
    hash2CircuitPromise = fetch("/circuits/hash2.json").then((r) => {
      if (!r.ok) throw new Error(`Failed to load hash2.json: ${r.status}`);
      return r.json();
    });
  }
  return hash2CircuitPromise;
}

async function loadWithdrawalCircuit(): Promise<any> {
  if (!withdrawalCircuitPromise) {
    withdrawalCircuitPromise = fetch("/circuits/withdrawal.json").then((r) => {
      if (!r.ok) throw new Error(`Failed to load withdrawal.json: ${r.status}`);
      return r.json();
    });
  }
  return withdrawalCircuitPromise;
}

// ============ Cached Noir instances ============

async function getHashesNoir(): Promise<Noir> {
  if (!hashesNoirPromise) {
    hashesNoirPromise = loadHashesCircuit().then((c) => new Noir(c));
  }
  return hashesNoirPromise;
}

async function getHash2Noir(): Promise<Noir> {
  if (!hash2NoirPromise) {
    hash2NoirPromise = loadHash2Circuit().then((c) => new Noir(c));
  }
  return hash2NoirPromise;
}

async function getWithdrawalNoir(): Promise<Noir> {
  if (!withdrawalNoirPromise) {
    withdrawalNoirPromise = loadWithdrawalCircuit().then((c) => new Noir(c));
  }
  return withdrawalNoirPromise;
}

// ============ Conversion utilities ============

/**
 * Converts Uint8Array (32 bytes, big-endian) to a hex string "0x...".
 */
function bytesToFieldHex(bytes: Uint8Array): string {
  return "0x" + Buffer.from(bytes).toString("hex");
}

/**
 * Converts hex string "0x..." to Uint8Array (32 bytes).
 * If fewer than 32 bytes — pads with leading zeros.
 */
function fieldHexToBytes(hex: string): Uint8Array {
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  const padded = clean.padStart(64, "0");
  return Uint8Array.from(Buffer.from(padded, "hex"));
}

// ============ Commitment + nullifier_hash ============

export interface CommitmentResult {
  commitment: Uint8Array;
  nullifierHash: Uint8Array;
}

/**
 * Computes commitment and nullifier_hash via the `hashes.json` circuit.
 */
export async function computeCommitment(
  nullifierSecret: Uint8Array,
  secret: Uint8Array,
  amount: bigint,
): Promise<CommitmentResult> {
  const noir = await getHashesNoir();

  const inputs = {
    nullifier: bytesToFieldHex(nullifierSecret),
    secret: bytesToFieldHex(secret),
    amount: "0x" + amount.toString(16),
  };

  const { returnValue } = await noir.execute(inputs);

  const [commitmentHex, nullifierHashHex] = returnValue as [string, string];

  return {
    commitment: fieldHexToBytes(commitmentHex),
    nullifierHash: fieldHexToBytes(nullifierHashHex),
  };
}

/**
 * Computes only nullifier_hash.
 */
export async function computeNullifierHash(nullifierSecret: Uint8Array): Promise<Uint8Array> {
  const noir = await getHashesNoir();

  const inputs = {
    nullifier: bytesToFieldHex(nullifierSecret),
    secret: "0x0",
    amount: "0x0",
  };

  const { returnValue } = await noir.execute(inputs);
  const [, nullifierHashHex] = returnValue as [string, string];

  return fieldHexToBytes(nullifierHashHex);
}

/**
 * Poseidon2 hash of two 32-byte elements.
 */
export async function poseidon2Hash(left: Uint8Array, right: Uint8Array): Promise<Uint8Array> {
  const noir = await getHash2Noir();

  const { returnValue } = await noir.execute({
    left: bytesToFieldHex(left),
    right: bytesToFieldHex(right),
  });

  return fieldHexToBytes(returnValue as string);
}

/**
 * Computes recipient_binding = hash_2(note_secret, recipient).
 */
export async function computeRecipientBinding(
  noteSecret: Uint8Array,
  recipient: Uint8Array,
): Promise<Uint8Array> {
  return poseidon2Hash(noteSecret, recipient);
}

// ============ Witness generation for withdraw ============

export interface WitnessInputs {
  root: Uint8Array;
  nullifierHash: Uint8Array;
  recipient: Uint8Array; // reduced to Field
  recipientBinding: Uint8Array;
  amount: bigint;
  nullifierSecret: Uint8Array;
  secret: Uint8Array;
  noteSecret: Uint8Array;
  merkleProof: Uint8Array[]; // 20 × 32 bytes
  isEven: boolean[]; // 20 booleans
}

/**
 * Generates a witness for withdraw via `withdrawal.json`.
 * Returns gzip-compressed witness (Uint8Array), ready for the prover.
 */
export async function generateWithdrawalWitness(inputs: WitnessInputs): Promise<Uint8Array> {
  const noir = await getWithdrawalNoir();

  const executionInputs = {
    root: bytesToFieldHex(inputs.root),
    nullifier_hash: bytesToFieldHex(inputs.nullifierHash),
    recipient: bytesToFieldHex(inputs.recipient),
    recipient_binding: bytesToFieldHex(inputs.recipientBinding),
    amount: "0x" + inputs.amount.toString(16),
    nullifier: bytesToFieldHex(inputs.nullifierSecret),
    secret: bytesToFieldHex(inputs.secret),
    note_secret: bytesToFieldHex(inputs.noteSecret),
    merkle_proof: inputs.merkleProof.map(bytesToFieldHex),
    is_even: inputs.isEven,
  };

  const { witness } = await noir.execute(executionInputs);

  return witness;
}

// ============ Exports ============

export { bytesToFieldHex, fieldHexToBytes };

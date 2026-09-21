import {
  bytesToHex as nobleBytesToHex,
  hexToBytes as nobleHexToBytes,
} from "@noble/hashes/utils.js";

/**
 * Generates random bytes of the given length.
 */
export function generateRandomBytes(length: number): Uint8Array {
  return crypto.getRandomValues(new Uint8Array(length));
}

/**
 * Converts bytes to a hex string.
 */
export function bytesToHex(bytes: Uint8Array): string {
  return nobleBytesToHex(bytes as Uint8Array);
}

/**
 * Converts a hex string to bytes.
 */
export function hexToBytes(hex: string): Uint8Array {
  return nobleHexToBytes(hex);
}

/**
 * Validates a hex string.
 */
export function isValidHex(hex: string): boolean {
  return /^[0-9a-fA-F]+$/.test(hex) && hex.length % 2 === 0;
}

/**
 * Checks the length of a Uint8Array.
 */
export function isBytesLength(bytes: Uint8Array, length: number): boolean {
  return bytes.length === length;
}

/**
 * BN254 scalar field modulus.
 * Used to reduce random bytes to a valid Field element.
 */
const BN254_FIELD_MODULUS =
  21888242871839275222246405745257275088548364400416034343698204186575808495617n;

/**
 * Converts 32 bytes (big-endian) to a BN254 Field element (mod p).
 * If the value >= modulus, it is reduced.
 */
function bytesToFieldElement(bytes: Uint8Array): Uint8Array {
  let value = 0n;
  for (let i = 0; i < 32; i++) {
    value = (value << 8n) | BigInt(bytes[i] ?? 0);
  }

  value = value % BN254_FIELD_MODULUS;

  const result = new Uint8Array(32);
  for (let i = 31; i >= 0; i--) {
    result[i] = Number(value & 0xffn);
    value >>= 8n;
  }

  return result;
}

/**
 * Generates a secret pair (nullifierSecret, secret) for deposit.
 * Both values are reduced to BN254 Field elements (valid for Noir).
 */
export function generateSecrets(): { nullifierSecret: Uint8Array; secret: Uint8Array } {
  return {
    nullifierSecret: bytesToFieldElement(generateRandomBytes(32)),
    secret: bytesToFieldElement(generateRandomBytes(32)),
  };
}

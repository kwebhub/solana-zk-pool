/**
 * Unit tests for the merkle service.
 *
 * Run: `npm test`
 * Uses Node.js built-in test runner (`node --test`).
 */

import { test, describe } from "node:test";
import assert from "node:assert/strict";
import { poseidon2Hash } from "../src/poseidon.js";

const EMPTY_LEAF = "0".repeat(64);

// ============================================================
// Poseidon2 hash
// ============================================================

describe("poseidon2Hash", () => {
  test("h(0, 0) matches Noir reference", async () => {
    const hash = await poseidon2Hash(EMPTY_LEAF, EMPTY_LEAF);
    assert.equal(
      hash,
      "18dfb8dc9b82229cff974efefc8df78b1ce96d9d844236b496785c698bc6732e",
      "h(0,0) must match Noir reference",
    );
  });

  test("Poseidon2 is not commutative: h(a,b) != h(b,a)", async () => {
    const a = "0".repeat(63) + "1";
    const b = "0".repeat(63) + "2";

    const ab = await poseidon2Hash(a, b);
    const ba = await poseidon2Hash(b, a);

    assert.notEqual(ab, ba, "Poseidon2 must not be commutative");
  });

  test("deterministic: same input → same output", async () => {
    const left = "0".repeat(62) + "aa";
    const right = "0".repeat(62) + "bb";

    const h1 = await poseidon2Hash(left, right);
    const h2 = await poseidon2Hash(left, right);

    assert.equal(h1, h2);
  });

  test("returns exactly 64 hex characters (32 bytes)", async () => {
    const hash = await poseidon2Hash(EMPTY_LEAF, EMPTY_LEAF);
    assert.equal(hash.length, 64, "hash must be 32 bytes (64 hex)");
    assert.match(hash, /^[0-9a-f]{64}$/, "hash must be lowercase hex");
  });

  test("accepts hex with and without 0x prefix", async () => {
    const withPrefix = await poseidon2Hash("0x" + EMPTY_LEAF, "0x" + EMPTY_LEAF);
    const withoutPrefix = await poseidon2Hash(EMPTY_LEAF, EMPTY_LEAF);
    assert.equal(withPrefix, withoutPrefix);
  });
});

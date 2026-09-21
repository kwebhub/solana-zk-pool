# Threat Model

> Threat model for zk-pool: what is protected, from whom, invariants, limitations.

## Table of Contents

- [Assets](#assets)
- [Threat Actors](#threat-actors)
- [Invariants](#invariants)
- [Attack Scenarios](#attack-scenarios)
- [What is NOT Protected](#what-is-not-protected)
- [Mitigations](#mitigations)

---

## Assets

| Asset | Where | Criticality |
| ------- | ----- | ------------- |
| **SOL in vault** | On-chain (Vault PDA) | 🔴 Critical |
| **Deposit notes** | User's localStorage | 🔴 Critical |
| **Secrets** (`nullifierSecret`, `secret`, `noteSecret`) | localStorage + browser memory | 🔴 Critical |
| **Commitments** | Postgres + Merkle tree | 🟡 Medium |
| **Nullifiers** | Postgres + NullifierRecord PDA | 🟡 Medium |
| **Merkle tree** | Redis + PoolState | 🟡 Medium |
| **Trusted setup (toxic waste)** | Does not exist (destroyed) | 🔴 Critical |

---

## Threat Actors

### 1. External Observer

**Capabilities:**
- Sees all blockchain transactions.
- Sees all public accounts and balances.
- No access to localStorage, backend, merkle.
- Doesn't know secrets.

**Goals:**
- Link deposit and withdrawal (break privacy).
- Compute secrets.

### 2. Backend Attacker

**Capabilities:**
- Full access to Postgres, Redis, API.
- Sees commitments, nullifiers, roots.
- Doesn't see witness (generated client-side).
- Sees proof + public_witness from prover.

**Goals:**
- Forge proof (impossible without toxic waste).
- Link deposit and withdrawal (impossible without witness).

### 3. Client Attacker

**Capabilities:**
- Inject XSS, steal localStorage.
- Replace `poseidon.ts` with a fake.
- MITM between frontend and backend.

**Goals:**
- Steal deposit notes (gain control of SOL).

### 4. On-chain Attacker

**Capabilities:**
- Send arbitrary tx to `zk_pool`.
- Forge proof (impossible).
- Forge root (verified in `PoolState.roots[]`).

**Goals:**
- Withdraw SOL without a valid deposit.
- Withdraw one deposit twice (double-spend).

### 5. Developer (trusted, but)

**Capabilities:**
- Modify program (upgrade authority).
- Modify backend, frontend.

**Goals:**
- Accidental error, not malice.

---

## Invariants

### I1: Each nullifier is used exactly once

**Where:** `withdraw.rs` → `init` for `NullifierRecord` PDA.

**How:** seeds `["nullifier_record", pool, nullifier_hash]`. If PDA exists — Anchor returns `AccountAlreadyInUse`.

### I2: SOL is transferred only with a valid proof

**Where:** `withdraw.rs` → `invoke(verifier_program, ...)`.

**How:** Groth16 proof verified on-chain.

### I3: Root must be in the history of `PoolState.roots[10]`

**Where:** `withdraw.rs` → `require!(pool.is_known_root(&root))`.

### I4: Recipient from tx matches recipient from instruction

**Where:** `withdraw.rs` → `require!(recipient.key() == to)`.

**Note:** Additionally, `recipient` is bound to proof via `recipient_binding = hash_2(note_secret, recipient)`. Attacker without `note_secret` cannot substitute recipient.

### I5: Commitment in Merkle tree = hash_3(nullifier, secret, amount)

**Where:** Noir circuit `withdrawal.nr`.

### I6: NullifierHash = hash_1(nullifier)

**Where:** Noir circuit `withdrawal.nr`.

### I7: Vault balance >= amount for withdraw

**Where:** `withdraw.rs` → `require!(vault.lamports() >= amount)`.

---

## Attack Scenarios

### A1: Double-spend

**Goal:** withdraw one deposit twice.

**Defense:** `init` for NullifierRecord → `AccountAlreadyInUse`.

**Result:** ✅ Protected.

### A2: Fake proof

**Goal:** withdraw SOL without knowing secrets.

**Defense:** on-chain verifier (Groth16) → `InvalidProof`.

**Result:** ✅ Protected (if toxic waste is destroyed).

### A3: Stale root

**Goal:** use a proof generated for an old tree.

**Defense:** `PoolState.roots[10]` stores 10 last roots. If root is still in history — proof valid. Otherwise — `InvalidRoot`.

**Result:** ✅ Protected (within last 10 roots).

### A4: Recipient substitution (front-running)

**Goal:** redirect withdrawal to attacker's address using someone else's proof.

**Defense:** `recipient_binding = hash_2(note_secret, recipient)`. Attacker doesn't know `note_secret`, cannot compute binding for their own recipient.

**Result:** ✅ Protected (new, fixed in stage 4.1a).

### A5: Deposit note interception

**Goal:** steal deposit note and withdraw.

**Defense:** ❌ **NOT protected.** XSS or localStorage leak = full control.

**Mitigations:**
- CSP (Content Security Policy).
- SRI (Subresource Integrity).
- Encrypted notes (user password).

### A6: MITM between frontend and backend

**Goal:** substitute commitments or proof.

**Defense:** ✅ **Partial.** If attacker substitutes proof — on-chain verification fails. But **availability** can be attacked.

**Mitigations:**
- HTTPS in production.
- Public Merkle root on-chain — frontend can verify.

### A7: Backend DoS

**Goal:** take down backend → withdraw fails.

**Defense:** rate limiting (5 req/min per IP for `/api/withdraw`).

**Result:** ✅ Protected in basic form.

### A8: Indexer lag

**Goal:** desync backend with blockchain.

**Defense:** ✅ Partial — pagination + `last_signature` in Redis.

### A9: Merkle proof substitution on backend

**Goal:** force frontend to generate a proof for someone else's commitment.

**Defense:** ✅ On-chain. Backend cannot forge proof.

### A10: Toxic waste in trusted setup

**Goal:** forge proof.

**Defense:** ❌ **NOT protected.** Sunspot uses standard setup.

**Mitigation:** [Reilabs trusted-setup](https://github.com/reilabs/trusted-setup) — MPC ceremony.

### A11: Program upgrade

**Goal:** change program logic (steal SOL).

**Defense:** ❌ **NOT protected** — single upgrade authority.

**Mitigations:** Squads multisig, time-lock, remove authority after audit.

### A12: Deposit trust model

**Goal:** send garbage `new_root` and break the tree for everyone.

**Defense:** ❌ **NOT protected.** Program does not verify `commitment` on-chain. `new_root != current_root` is a weak check.

**Mitigation:** verify commitment on-chain (planned, stage 12).

---

## What is NOT Protected

| Threat | Why | Mitigation |
| -------- | ----- | ----------- |
| XSS steals localStorage | All secrets in localStorage | CSP, SRI, encrypted notes |
| Backend sees commitments/nullifiers | Backend is source of truth | Not critical for privacy |
| Toxic waste in setup | Sunspot standard setup | MPC ceremony |
| Single upgrade authority | No multisig | Squads, time-lock |
| MITM changes root | HTTP without TLS in dev | HTTPS in production |
| DoS on prover | Single service | Queue, replicas |
| Indexer lag | Polling | Webhook, replicas |
| Garbage `new_root` | No on-chain verification | Verify commitment on-chain |

---

## Mitigations

### Priority 1 (before mainnet)

1. **MPC trusted setup** — eliminate toxic waste.
2. **Program audit** — at least 2 independent audits.
3. **Multisig on upgrade authority** — Squads Protocol.
4. **HTTPS** — TLS for all endpoints.
5. **CSP** — Content Security Policy for frontend.
6. **On-chain commitment verification** — fix deposit trust model.

### Priority 2 (after mainnet)

1. **Encrypted deposit notes** — user password.
2. **Webhook from Helius** instead of polling.
3. **Circuit breaker** for prover/merkle.
4. **Backup Postgres** — daily snapshots.
5. **Alerting** — Grafana Alerting.

### Priority 3 (nice to have)

1. **ZK proof on client** — without backend (WASM port of Sunspot).
2. **Merkle tree for nullifiers** — unlimited growth.
3. **Multisig for authority** — DAO governance.
4. **Formal verification** — proof of invariants.

---

## Tests

Invariants checked by tests:

| Invariant | Test |
| ----------- | ------ |
| I1: nullifier once | `test_double_spend` |
| I2: proof valid | `test_invalid_proof` |
| I3: root in history | `test_invalid_root` |
| I4: recipient matches | `test_recipient_mismatch` |
| I5: commitment matches | `test_invalid_commitment` |
| I6: nullifier_hash matches | `test_invalid_nullifier_hash` |
| I7: vault balance | `test_insufficient_vault` |

**Status:** TODO — write tests in stage 11.

---

**Last updated:** 2026-09-21

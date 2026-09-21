# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant (Claude, ChatGPT, etc.) in a new chat. Load it first — the assistant will understand the project without copying code files.
>
> **Last updated:** 2026-09-20 (after stage 3)

---

## 1. What this project is

**solana-zk-pool** — a private pool for SOL transfers on Solana using ZK proofs (Groth16 on BN254). A user deposits SOL into a shared vault, receives a deposit note (secrets), and can later withdraw SOL to any address without revealing the link between deposit and withdrawal.

**Origin:** [Solana Foundation Bootcamp 2026 — "05-private-transfers"](https://github.com/solana-foundation/solana-bootcamp-2026/tree/main/05-private-transfers).

**Previous version:** [kwebhub/private-transfer](https://github.com/kwebhub/private-transfer) — v1, written with an AI assistant. **solana-zk-pool** is v2, rewritten from scratch with improvements.

**Repository:** https://github.com/kwebhub/solana-zk-pool (private)
**License:** MIT
**Network:** Solana Devnet

**Key improvements over v1:**
- Clean monorepo structure (`onchain/`, `circuits/`, `services/`, `web/`).
- Shared `poseidon` library (no duplication of `hash_1`/`hash_2`/`hash_3`).
- Pinned versions for all tools in Docker.
- Removed `network_mode: host`.
- `pnpm` instead of `yarn`.
- Cleaner `.gitignore`.
- `Prover.toml` gitignored.

**Note on front-running:** v1 had a vulnerability where `recipient` was a public input but not used in the circuit. Fix is deferred to stage 4.

---

## 2. Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│ FRONTEND (Vue 3 + Vite + Pinia)                                 │
│ • Deposit: generates commitment, sends tx                       │
│ • Withdraw: fetches proof, generates witness, sends tx          │
└──────────────────────────────────────────────────────────────────┘
              ↓ HTTP                    ↑ HTTP
┌──────────────────────────────────────────────────────────────────┐
│ BACKEND (Rust + axum)                                            │
│ • /api/commitments  • /api/root  • /api/proof  • /api/withdraw   │
│ • Indexer: listens to events → Postgres → Merkle tree in Redis   │
│ • Rate limiting · Prometheus metrics                             │
└──────────────────────────────────────────────────────────────────┘
       ↓                    ↓                    ↓
┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐
│ POSTGRES     │   │ REDIS        │   │ MERKLE (Node.js)     │
│ commitments  │   │ cache 30s    │   │ Noir Poseidon2       │
│ roots        │   │ Merkle tree  │   │ /hash /root /proof   │
│ nullifiers   │   │ rate limit   │   │                      │
└──────────────┘   └──────────────┘   └──────────────────────┘
                                              ↑
┌─────────────────────────────────────────────┴────────────────────┐
│ SOLANA                                                           │
│ • Program zk_pool (main): PoolState, Vault, NullifierRecord      │
│ • Verifier program (Sunspot, Groth16): 35BRi4VE8aXEe2NQvb1FzD... │
└──────────────────────────────────────────────────────────────────┘
```

### Services and ports

| Service | Port | Where | Purpose |
|---------|------|-------|---------|
| frontend | 5173 | host | Vue 3 + Vite |
| backend | 4001 | `solana` container | Rust + axum |
| prover | 4002 | `solana` container | Sunspot CLI wrapper |
| merkle | 4003 | `solana` container | Node.js + noir_js |
| postgres | 5432 | Docker | DB |
| redis | 6379 | Docker | cache |
| prometheus | 9090 | Docker | metrics |
| grafana | 3000 | Docker | visualization |

---

## 3. Technologies

| Layer | Technology | Version |
|-------|-----------|---------|
| Smart contract | Anchor | 1.1.2 (CLI), anchor-lang 1.1.2 |
| ZK circuit | Noir / nargo | 1.0.0-rc.2 |
| ZK backend | Sunspot | 1.0.0 |
| ZK verification | gnark-solana | v2.0.0 |
| Backend | Rust | 1.98.1 |
| Backend framework | axum | 0.7 |
| Frontend | Vue 3 | 3.5+ |
| Frontend build | Vite | 8.x |
| State | Pinia | 4.x |
| Contracts client | Codama | latest |
| DB | PostgreSQL | 16 |
| Cache | Redis | 7 |
| Monitoring | Prometheus + Grafana | latest |
| Container | Docker | latest |
| Node.js | Node.js | 24.21.0 |
| pnpm | pnpm | 12.5.1 |

---

## 4. Repository structure

```
solana-zk-pool/
├── docs/
│   ├── NOTE.md              ← personal notes (NOT committed)
│   ├── PROJECT_CONTEXT.md   ← this file
│   └── ru/                  ← Russian localization (later)
├── infra/
│   ├── docker-compose.yml
│   └── docker/
│       ├── Dockerfile.solana
│       └── .dockerignore
├── onchain/                 ← Anchor workspace
│   ├── Anchor.toml
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   ├── programs/
│   │   └── zk_pool/
│   │       ├── Cargo.toml
│   │       ├── src/
│   │       │   ├── lib.rs
│   │       │   ├── constants.rs
│   │       │   ├── error.rs
│   │       │   ├── state.rs
│   │       │   └── instructions.rs
│   │       └── tests/
│   └── target/
├── circuits/                ← Noir circuits
│   ├── poseidon/            ← library: hash_1, hash_2, hash_3
│   ├── hash2/               ← Poseidon2 for JS
│   ├── hashes/              ← commitment + nullifier_hash
│   └── withdrawal/          ← main circuit
│       └── target/          ← ACIR, CCS, PK, VK, SO (NOT committed)
├── services/
│   ├── backend/             ← Rust + axum (stage 5)
│   ├── merkle/              ← Node.js + noir_js (stage 6)
│   └── prover/              ← Rust + Sunspot (stage 7)
├── web/                     ← Vue 3 (stage 8)
├── solana/                  ← Solana keys (id.json NOT committed)
├── .editorconfig
├── .gitignore
├── LICENSE
└── README.md
```

---

## 5. Key decisions

### 5.1. Front-running protection — deferred to stage 4

**Was in v1:** `commitment = hash_3(nullifier, secret, amount)`. Recipient was a public input but **not used in the circuit** (`let _ = recipient`). An attacker could intercept a valid proof and substitute their own recipient.

**Planned fix (stage 4):** to be decided. Options:
- **A.** `commitment = hash_4(nullifier, secret, amount, recipient)` — **rejected**: sponge capacity=0 is insecure.
- **B.** Sponge with two permutation calls — **expensive**.
- **C.** `commitment = hash_3(nullifier, hash_2(secret, recipient), amount)` — recipient fixed at deposit time, uses only safe hashes.
- **D.** Separate binding constraint — doesn't help (attacker knows recipient).

**Decision will be made in stage 4.**

### 5.2. Monorepo structure

Flat folders: `onchain/`, `circuits/`, `services/`, `web/`. Not `workspace/ptrans/app/...`.

### 5.3. Docker container `solana` — all-in-one

Inside: Rust, Solana CLI, Anchor, Noir (nargo), Sunspot, Node.js, pnpm. All versions **pinned**.

**Volumes:** `onchain/`, `circuits/`, `services/`, `web/`, `solana/` are mounted to `/home/ubuntu/...`.

**`network_mode: host` is NOT used** — for portability.

**Note:** Sunspot must be cloned (`git clone https://github.com/reilabs/sunspot.git`) for `sunspot deploy` to work. The `.deb` package contains only the CLI binary, not the Rust crate `verifier-bin` needed for compiling the verifier program. Add `git clone` to Dockerfile in stage 12.

### 5.4. Shared `poseidon` library

All hash primitives (`hash_1`, `hash_2`, `hash_3`) live in `circuits/poseidon/`. Other circuits import them via `[dependencies] poseidon = { path = "../poseidon" }`.

**Why:** single source of truth, no duplication.

### 5.5. `hash_4` is NOT used

Poseidon2 in Noir has `t = 4` (state size). Rate = 3, capacity = 1. Using all 4 slots for user data sets capacity=0, **breaking cryptographic security**. So we use only `hash_1`, `hash_2`, `hash_3`.

---

### 5.6. Deposit splitting into unequal parts (planned, stage 12)

**Goal:** allow partial withdrawals, prevent amount-based deanonymization.

**Design:**
- User chooses k (1..10), default 1.
- Deposit takes `commitments: Vec<[u8;32]>`, `amounts: Vec<u64>`, `new_root: [u8;32]`.
- Program verifies `sum(amounts) == transferred SOL`.
- Amounts are **unequal** — random partition.
- One deposit → k leaves in Merkle tree.
- User stores k notes.

**Why unequal:** equal parts create a sum-correlation attack (deposit 10 SOL + 10×1 SOL withdrawals = link).

**MVP in stage 4:** single commitment. Multiple parts deferred to stage 12.

## 6. What has been done

### ✅ Stage 1. Repository setup (2026-09-20)

- Repository `solana-zk-pool` on GitHub (private).
- Monorepo structure: `onchain/`, `circuits/`, `services/`, `web/`.
- `.gitignore`, `.editorconfig`, `LICENSE`, `README.md`.
- `infra/docker/Dockerfile.solana` + `infra/docker-compose.yml`.
- Docker container `solana` (Ubuntu 24.04, Rust 1.98.1, Solana CLI 3.1.10, Anchor 1.1.2, nargo 1.0.0-rc.2, Node 24.21.0, pnpm 12.5.1, Sunspot 1.0.0).
- Anchor workspace in `onchain/` with empty program `zk_pool`.
- Program ID: `CZjsHYcE6oPhj2ZAKVdnaz36FQHC7SBd7MSY1jmAaj3o`.
- Solana devnet wallet: `HD73coWkoiRgSwtLJiAKUTVxpzNFBo5ZGTa6p1T1ktM` (5 SOL).
- Commits: `b0ef53b` (initial), `d43552a` (PROJECT_CONTEXT.md).

### ✅ Stage 2. ZK circuits in Noir (2026-09-20)

- `circuits/poseidon/` — library with `hash_1`, `hash_2`, `hash_3`.
- `circuits/hash2/` — Poseidon2 for two inputs. 2 tests.
- `circuits/hashes/` — commitment + nullifier_hash (tuple-return). 4 tests.
- `circuits/withdrawal/` — main circuit with Merkle proof verification. 2 tests.
- `merkle_tree.nr` — `compute_merkle_root<let DEPTH: u32>`.
- All tests passing.
- Commit: `2fff662`.

### ✅ Stage 3. Sunspot verifier (2026-09-20)

- Cloned Sunspot (`git clone https://github.com/reilabs/sunspot.git`).
- `sunspot compile target/withdrawal.json` → `withdrawal.ccs`.
- `sunspot setup target/withdrawal.ccs` → `withdrawal.pk`, `withdrawal.vk`.
- `sunspot deploy target/withdrawal.vk` → `withdrawal.so`, `withdrawal-keypair.json`.
- `solana program deploy target/withdrawal.so --url devnet`.
- **VERIFIER_PROGRAM_ID:** `35BRi4VE8aXEe2NQvb1FzDXJXhh8GzVRSw191bca19Rb`
- Balance: 0.456 SOL (rent for 89 KB program account).
- Nothing committed (all in `target/`).

---

## 7. What's next

### Stage 4. Anchor program (next)
- `PoolState`, `NullifierRecord`.
- Instructions: `pool`, `deposit`, `withdraw`.
- Verifier CPI to `35BRi4VE8aXEe2NQvb1FzDXJXhh8GzVRSw191bca19Rb`.
- `encode_public_inputs`.
- **Decide and implement front-running protection.**
- Tests with `litesvm`.

### Stages 5–12
Backend, Merkle, Prover, Frontend, Infra, Processes, Security, Finalization.

---

## 8. Roadmap

- [x] **Stage 1.** Repository setup + Docker + `anchor init`
- [x] **Stage 2.** ZK circuits in Noir
- [x] **Stage 3.** Sunspot: compile → setup → deploy verifier
- [ ] **Stage 4.** Anchor program (PoolState, deposit, withdraw, verifier CPI)
- [ ] **Stage 5.** Backend
- [ ] **Stage 6.** Merkle service
- [ ] **Stage 7.** Prover
- [ ] **Stage 8.** Frontend
- [ ] **Stage 9.** Infrastructure
- [ ] **Stage 10.** Engineering processes
- [ ] **Stage 11.** Security
- [ ] **Stage 12.** Finalization
  - README EN/RU, CHANGELOG, release
  - **Improve deposit trust model:** verify commitment on-chain, replace weak `new_root != current_root` check - Split deposit into multiple unequal parts (planned improvement)

---

## 9. Known limitations / TODOs

- **Trusted setup for Groth16** — Sunspot uses a standard setup without a ceremony. Production needs MPC.
- **Upgrade authority** — single key, no multisig.
- **Secrets in localStorage** — XSS risk.
- **Backend sees commitments/nullifiers** — but not the link between them.
- **MerkleTree depth 20** — max 1M leaves.
- **TREE_DEPTH hardcoded** in `circuits/withdrawal/src/main.nr` as `global TREE_DEPTH: u32 = 20`. Will be unified in stage 12.
- **Front-running protection** not yet implemented. Deferred to stage 4.
- **`recipient` not used** in `withdrawal.nr` (`let _ = recipient`).
- **Sunspot cloned manually** — add `git clone https://github.com/reilabs/sunspot.git` to `Dockerfile.solana` in stage 12.

---

## 10. Git workflow

- Conventional commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, `ci:`.
- Branches: `main` (protected), feature branches `feat/...`, `fix/...`.
- PR: required.
- CI: in stage 10.

**Rule:** all committed files are in **English**. Russian localization — only in `docs/ru/` (later).

---

## 11. Commands

### Docker
```bash
cd infra
docker compose build solana      # build (20 min, once)
docker compose up -d solana      # start
docker compose exec solana bash  # enter container
```

### Anchor (inside the container)
```bash
cd /home/ubuntu/onchain
anchor build
anchor deploy --provider.cluster devnet
cargo test
```

### Noir (inside the container)
```bash
cd /home/ubuntu/circuits/<name>
nargo test
nargo test test_generate_valid_inputs --show-output   # generate Prover.toml
nargo execute                                          # solve circuit (needs Prover.toml)
```

### Sunspot (inside the container)
```bash
# Ensure Sunspot is cloned (once per container):
git clone https://github.com/reilabs/sunspot.git /home/ubuntu/sunspot

cd /home/ubuntu/circuits/withdrawal
sunspot compile target/withdrawal.json
sunspot setup target/withdrawal.ccs
sunspot deploy target/withdrawal.vk
sunspot prove <acir> <witness> <ccs> <pk>
```

### Solana (inside the container)
```bash
solana program deploy <path.so> --program-id <keypair.json> --url devnet
solana program show <PROGRAM_ID> --url devnet
solana address -k <keypair.json>
```

---

## 12. Next task for a new assistant

**Stage 4:** Anchor program in `onchain/programs/zk_pool/src/`. Implement `PoolState`, `NullifierRecord`, instructions (`pool`, `deposit`, `withdraw`), verifier CPI to `35BRi4VE8aXEe2NQvb1FzDXJXhh8GzVRSw191bca19Rb`. Decide and implement front-running protection.

All details — in `docs/NOTE.md` (personal notes, not committed).

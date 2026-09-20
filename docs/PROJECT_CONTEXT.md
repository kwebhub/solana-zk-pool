# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant (Claude, ChatGPT, etc.) in a new chat. Load it first — the assistant will understand the project without copying code files.
>
> **Last updated:** 2026-09-20 (after stage 2)

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
- Shared `poseidon` library (no duplication of hash_1/hash_2/hash_3).
- Pinned versions for all tools in Docker.
- Removed `network_mode: host`.
- `pnpm` instead of `yarn`.
- Cleaner `.gitignore`.
- `Prover.toml` gitignored.

**Note on front-running:** v1 had a vulnerability where `recipient` was a public input but not used in the circuit. Fix is deferred to stage 4 (design decision: use `hash_3` + separate binding, or another mechanism).

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
│ SOLANA (Anchor program zk_pool)                                   │
│ • pool: PDA ["pool3"] — PoolState                                 │
│ • vault: PDA ["vault3", pool] — SystemAccount                     │
│ • nullifier_record: PDA ["nullifier_record", pool, nullifier]     │
│ • verifier: on-chain Groth16 verifier (Sunspot)                   │
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
| ZK verification | gnark-solana | — |
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
│   ├── hash2/               ← Poseidon2 for two inputs
│   ├── hashes/              ← commitment + nullifier_hash
│   └── withdrawal/          ← main circuit (Merkle proof)
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

### 5.4. Shared `poseidon` library

All hash primitives (`hash_1`, `hash_2`, `hash_3`) live in `circuits/poseidon/`. Other circuits import them via `[dependencies] poseidon = { path = "../poseidon" }`.

**Why:** single source of truth, no duplication.

### 5.5. `hash_4` is NOT used

Poseidon2 in Noir has `t = 4` (state size). Rate = 3, capacity = 1. Using all 4 slots for user data sets capacity=0, **breaking cryptographic security**. So we use only `hash_1`, `hash_2`, `hash_3`.

---

## 6. What has been done

### ✅ Stage 1. Repository setup (2026-09-20)

- Repository `solana-zk-pool` on GitHub (private).
- Monorepo structure.
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

---

## 7. What's next

### Stage 3. Sunspot: compile → setup → deploy verifier (next)
- `sunspot compile` → CCS file from ACIR JSON.
- `sunspot setup` → proving key (pk) + verifying key (vk).
- `sunspot deploy` → deploy verifier program on devnet.
- Obtain verifier program ID.
- Copy circuit JSONs to merkle and frontend (later).

### Stage 4. Anchor program
- `PoolState`, `NullifierRecord`.
- Instructions: `pool`, `deposit`, `withdraw`.
- Verifier CPI.
- `encode_public_inputs`.
- **Decide and implement front-running protection.**

### Stages 5–12
- Backend, Merkle, Prover, Frontend, Infra, Processes, Security, Finalization.

---

## 8. Roadmap

- [x] **Stage 1.** Repository setup + Docker + `anchor init`
- [x] **Stage 2.** ZK circuits in Noir (poseidon library, hash2, hashes, withdrawal)
- [ ] **Stage 3.** Sunspot: compile → setup → deploy verifier
- [ ] **Stage 4.** Anchor program (PoolState, deposit, withdraw, verifier CPI)
- [ ] **Stage 5.** Backend (axum + Postgres + Redis + indexer)
- [ ] **Stage 6.** Merkle service (Node.js + noir_js)
- [ ] **Stage 7.** Prover (Rust + Sunspot CLI)
- [ ] **Stage 8.** Frontend (Vue 3 + Codama + witness generation)
- [ ] **Stage 9.** Infrastructure (Makefile, Prometheus, Grafana)
- [ ] **Stage 10.** Engineering processes (CI/CD, security, templates)
- [ ] **Stage 11.** Security (threat model, invariants, tests)
- [ ] **Stage 12.** Finalization (README EN/RU, CHANGELOG, release)

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
nargo test                        # run tests
nargo execute                     # generate witness (needs Prover.toml)
nargo test <name> --show-output   # show println output
```

### Sunspot (inside the container)
```bash
sunspot compile <acir.json>
sunspot setup <ccs>
sunspot deploy <vk>
sunspot prove <acir> <witness> <ccs> <pk>
```

---

## 12. Next task for a new assistant

**Stage 3:** Sunspot — compile `withdrawal.json` to CCS, generate proving/verifying keys, deploy verifier program on devnet. Obtain the verifier program ID.

All details — in `docs/NOTE.md` (personal notes, not committed).

# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant (Claude, ChatGPT, etc.) in a new chat. Load it first — the assistant will understand the project without copying code files.
>
> **Last updated:** 2026-09-21 (after stage 4)

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
- **Front-running protection** via `recipient_binding` (note remains bearer-style).
- **`amount` in `WithdrawEvent`** (was missing in v1).

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
│ • Program zk_pool: CZjsHYcE6oPhj2ZAKVdnaz36FQHC7SBd7MSY1jmAaj3o │
│ • Verifier program: AukfYGQqhv3ZkdB1G7cKF3djsE9FKMrAfLd34doSWQex│
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
│   ├── notes/               ← personal notes (NOT committed)
│   │   ├── 01-setup.md
│   │   ├── 02-circuits.md
│   │   ├── 03-sunspot.md
│   │   ├── 04-anchor.md
│   │   └── glossary.md
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
│   │       │   ├── events.rs
│   │       │   ├── state.rs
│   │       │   ├── instructions.rs
│   │       │   └── instructions/
│   │       │       ├── pool.rs
│   │       │       ├── deposit.rs
│   │       │       └── withdraw.rs
│   │       └── tests/
│   │           └── test_zk_pool.rs
│   └── target/
├── circuits/
│   ├── poseidon/            ← hash_1, hash_2, hash_3
│   ├── hash2/               ← Poseidon2 for JS
│   ├── hashes/              ← commitment + nullifier_hash
│   └── withdrawal/          ← main circuit
│       └── target/          ← ACIR, CCS, PK, VK, SO (NOT committed)
├── services/
│   ├── backend/             ← (stage 5)
│   ├── merkle/              ← (stage 6)
│   └── prover/              ← (stage 7)
├── web/                     ← (stage 8)
├── solana/
├── .editorconfig
├── .gitignore
├── LICENSE
└── README.md
```

---

## 5. Key decisions

### 5.1. Front-running protection — implemented

**Solution:** Note is bearer-style (commitment does NOT depend on recipient). Recipient is bound to proof via `recipient_binding = hash_2(note_secret, recipient)`.

**New public inputs (5):** root, nullifier_hash, recipient, recipient_binding, amount.
**New private input:** note_secret.

**Why it works:** attacker doesn't know `note_secret`, so they can't compute binding for their own recipient.

### 5.2. Monorepo structure

Flat folders: `onchain/`, `circuits/`, `services/`, `web/`.

### 5.3. Docker container `solana` — all-in-one

Inside: Rust, Solana CLI, Anchor, Noir (nargo), Sunspot, Node.js, pnpm. Versions **pinned**.

**Volumes:** `onchain/`, `circuits/`, `services/`, `web/`, `solana/` are mounted to `/home/ubuntu/...`.

**`network_mode: host` is NOT used** — for portability.

**Note:** Sunspot must be cloned (`git clone https://github.com/reilabs/sunspot.git`) for `sunspot deploy` to work. Add `git clone` to Dockerfile in stage 12.

### 5.4. Shared `poseidon` library

All hash primitives (`hash_1`, `hash_2`, `hash_3`) live in `circuits/poseidon/`. Other circuits import them via `[dependencies] poseidon = { path = "../poseidon" }`.

### 5.5. `hash_4` is NOT used

Poseidon2 in Noir has `t = 4`. Rate = 3, capacity = 1. Using all 4 slots for user data sets capacity=0, **breaking cryptographic security**. So we use only `hash_1`, `hash_2`, `hash_3`.

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

### 5.7. Deposit trust model — deferred to stage 12

Program does NOT verify that `commitment` matches the added leaf. Client can send garbage `new_root` and break the tree. `new_root != current_root` is a weak check. Improvement deferred to stage 12.

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
- Wallet: `HD73coWkoiRgSwtLJiAKUTVxpzNFBo5ZGTa6p1T1ktM`.
- Commits: `b0ef53b`, `d43552a`.

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
- Deployed on devnet.
- Nothing committed (all in `target/`).

### ✅ Stage 4.1a. Front-running protection (2026-09-20)

- Updated `withdrawal.nr`: new public input `recipient_binding`, new private input `note_secret`.
- New constraint: `hash_2(note_secret, recipient) == recipient_binding`.
- `NR_PUBLIC_INPUTS`: 4 → 5.
- Re-compiled Sunspot artifacts.
- **New VERIFIER_PROGRAM_ID:** `AukfYGQqhv3ZkdB1G7cKF3djsE9FKMrAfLd34doSWQex`
- Old verifier (`35BRi4VE...`) — deprecated.
- Commit: `f461a19`.

### ✅ Stage 4. Anchor program (2026-09-21)

- `constants.rs` — seeds, TREE_DEPTH, NR_PUBLIC_INPUTS=5, VERIFIER_PROGRAM_ID.
- `error.rs` — PtransError (10 variants).
- `events.rs` — DepositEvent, WithdrawEvent (with amount).
- `state.rs` — PoolState (roots history), NullifierRecord.
- `instructions.rs` — `encode_public_inputs` (5 inputs, BN254 reduction).
- `instructions/pool.rs` — InitPool + handler.
- `instructions/deposit.rs` — InitDeposit + handler.
- `instructions/withdraw.rs` — InitWithdraw + handler (verifier CPI).
- `lib.rs` — entrypoint with 3 instructions.
- Program deployed to devnet.
- **Program ID:** `CZjsHYcE6oPhj2ZAKVdnaz36FQHC7SBd7MSY1jmAaj3o`
- 7 unit tests passing.
- Commits: `5aebd2d`, `<тест-коммит>`.

---

## 7. What's next

### Stage 5. Backend (next)
- axum + sqlx + redis.
- `/api/commitments`, `/api/root`, `/api/proof`, `/api/withdraw`, `/api/health`.
- Indexer: reads on-chain events → Postgres → Merkle tree in Redis.
- Rate limiting, Prometheus metrics.
- Migrations: `commitments`, `roots`, `nullifiers`.

### Stages 6–12
Merkle service, Prover, Frontend, Infra, Processes, Security, Finalization.

---

## 8. Roadmap

- [x] **Stage 1.** Repository setup + Docker + `anchor init`
- [x] **Stage 2.** ZK circuits in Noir
- [x] **Stage 3.** Sunspot: compile → setup → deploy verifier
- [x] **Stage 4.** Anchor program (PoolState, deposit, withdraw, verifier CPI)
- [ ] **Stage 5.** Backend (axum + Postgres + Redis + indexer)
- [ ] **Stage 6.** Merkle service (Node.js + noir_js)
- [ ] **Stage 7.** Prover (Rust + Sunspot CLI)
- [ ] **Stage 8.** Frontend (Vue 3 + Codama + witness generation)
- [ ] **Stage 9.** Infrastructure (Makefile, Prometheus, Grafana)
- [ ] **Stage 10.** Engineering processes (CI/CD, security, templates)
- [ ] **Stage 11.** Security (threat model, invariants, tests)
- [ ] **Stage 12.** Finalization
  - README EN/RU, CHANGELOG, release
  - Split deposit into multiple unequal parts
  - **Improve deposit trust model:** verify commitment on-chain, replace weak `new_root != current_root` check
  - Add Sunspot `git clone` to Dockerfile

---

## 9. Known limitations / TODOs

- **Trusted setup for Groth16** — Sunspot uses a standard setup without a ceremony. Production needs MPC.
- **Upgrade authority** — single key, no multisig.
- **Secrets in localStorage** — XSS risk (stage 8+).
- **Backend sees commitments/nullifiers** — but not the link between them.
- **MerkleTree depth 20** — max 1M leaves.
- **TREE_DEPTH hardcoded** in `circuits/withdrawal/src/main.nr` as `global TREE_DEPTH: u32 = 20`. Unify in stage 12.
- **Deposit trust model** — program does NOT verify commitment on-chain. Deferred to stage 12.
- **Sunspot cloned manually** — add `git clone` to Dockerfile in stage 12.

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
nargo execute
nargo test test_generate_valid_inputs --show-output   # generate Prover.toml
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

**Stage 5:** Backend in `services/backend/`. axum + sqlx (Postgres) + redis. Endpoints: `/api/commitments`, `/api/root`, `/api/proof`, `/api/withdraw`, `/api/health`, `/metrics`. Indexer: reads on-chain `DepositEvent`/`WithdrawEvent` from `CZjsHYcE6oPhj2ZAKVdnaz36FQHC7SBd7MSY1jmAaj3o`, writes to Postgres, updates Merkle tree in Redis. Rate limiting. Prometheus metrics.

All details — in `docs/notes/` (personal notes, not committed).

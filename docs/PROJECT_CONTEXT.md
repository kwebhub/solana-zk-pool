# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant (Claude, ChatGPT, etc.) in a new chat. Load it first — the assistant will understand the project without copying code files.
>
> **Last updated:** 2026-09-20

---

## 1. What this project is

**solana-zk-pool** — a private pool for SOL transfers on Solana using ZK proofs (Groth16 on BN254). A user deposits SOL into a shared vault, receives a deposit note (secrets), and can later withdraw SOL to any address without revealing the link between deposit and withdrawal.

**Origin:** [Solana Foundation Bootcamp 2026 — "05-private-transfers"](https://github.com/solana-foundation/solana-bootcamp-2026/tree/main/05-private-transfers).

**Previous version:** [kwebhub/private-transfer](https://github.com/kwebhub/private-transfer) — v1, written with an AI assistant. **solana-zk-pool** is v2, rewritten from scratch with improvements.

**Repository:** https://github.com/kwebhub/solana-zk-pool (private)
**License:** MIT
**Network:** Solana Devnet

**Key improvements over v1:**
- **Front-running vulnerability fixed:** `recipient` is now part of the commitment (`hash_4(nullifier, secret, amount, recipient)` instead of `hash_3`).
- Clean monorepo structure (`onchain/`, `circuits/`, `services/`, `web/`).
- Pinned versions for all tools in Docker.
- Removed `network_mode: host`.
- `pnpm` instead of `yarn`.
- Cleaner `.gitignore`.

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
├── circuits/                ← Noir circuits (stage 2)
│   ├── hash2/
│   ├── hashes/
│   └── withdrawal/
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

### 5.1. Fixing the front-running vulnerability

**Was in v1:** `commitment = hash_3(nullifier, secret, amount)`. Recipient was a public input but **not used in the circuit** (`let _ = recipient`). An attacker could intercept a valid proof and substitute their own recipient.

**Now:** `commitment = hash_4(nullifier, secret, amount, recipient)`. The recipient is part of the commitment, and the commitment is part of the Merkle tree. Recipient substitution is impossible.

**Trade-off:** the recipient is fixed at deposit time. The user **cannot** withdraw to a different address later.

### 5.2. Monorepo structure

**Not `workspace/ptrans/app/...`** but flat folders: `onchain/`, `circuits/`, `services/`, `web/`. Fewer levels, more logical.

### 5.3. Docker container `solana` — all-in-one

Inside: Rust, Solana CLI, Anchor, Noir (nargo), Sunspot, Node.js, pnpm. All versions are **pinned**.

**Volumes:** `onchain/`, `circuits/`, `services/`, `web/`, `solana/` are mounted to `/home/ubuntu/...`.

**`network_mode: host` is NOT used** — for portability.

### 5.4. `recipient` in commitment — to be verified in stage 2/4

**TODO for stage 2:** ensure `withdrawal.nr` uses `hash_4`, not `hash_3`. Update `hashes.nr` and `hashes.json`.

---

## 6. What has been done

### ✅ Stage 1. Repository setup (2026-09-20)

- Repository `solana-zk-pool` on GitHub (private).
- Monorepo structure.
- `.gitignore`, `.editorconfig`, `LICENSE`, `README.md`.
- `infra/docker/Dockerfile.solana` + `infra/docker-compose.yml`.
- Docker container `solana` (Ubuntu 24.04, Rust 1.98.1, Solana CLI 3.1.10, Anchor 1.1.2, nargo 1.0.0-rc.2, Node 24.21.0, pnpm 12.5.1, Sunspot 1.0.0).
- Anchor workspace in `onchain/` with an empty program `zk_pool`.
- Program ID: `CZjsHYcE6oPhj2ZAKVdnaz36FQHC7SBd7MSY1jmAaj3o`.
- Solana devnet wallet: `HD73coWkoiRgSwtLJiAKUTVxpzNFBo5ZGTa6p1T1ktM` (5 SOL).
- First commit: `b0ef53b`.

---

## 7. What's next

### Stage 2. ZK circuit in Noir (next)
- `hash2` — Poseidon2 for two inputs.
- `hashes` — commitment + nullifier_hash (with `hash_4`).
- `withdrawal` — main circuit with recipient in commitment.
- Tests.

### Stage 3. Sunspot + verifier
- `sunspot compile` / `setup` / `deploy`.
- Obtain the verifier program ID.
- Update `constants.rs`.

### Stage 4. Anchor program
- `PoolState`, `NullifierRecord`.
- Instructions: `pool`, `deposit`, `withdraw`.
- Verifier CPI.
- `encode_public_inputs` with `hash_4`.

### Stages 5–12
- Backend, Merkle, Prover, Frontend, Infra, Processes, Security, Finalization.
- See roadmap below.

---

## 8. Roadmap

- [x] **Stage 1.** Repository setup + Docker + `anchor init`
- [ ] **Stage 2.** ZK circuit in Noir (hash2, hashes, withdrawal with hash_4)
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

## 9. Known limitations (inherited from v1, fixed along the way)

- **Trusted setup for Groth16** — Sunspot uses a standard setup without a ceremony. Production needs MPC.
- **Upgrade authority** — single key, no multisig.
- **Secrets in localStorage** — XSS risk.
- **Backend sees commitments/nullifiers** — but not the link between them.
- **MerkleTree depth 20** — max 1M leaves.

---

## 10. Git workflow

- Conventional commits: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, `ci:`.
- Branches: `main` (protected), feature branches `feat/...`, `fix/...`.
- PR: required.
- CI: in stage 10.

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
anchor build                     # build
anchor deploy --provider.cluster devnet  # deploy
cargo test                       # tests
```

### Noir (inside the container)
```bash
cd /home/ubuntu/circuits/<name>
nargo compile                    # compile
nargo test                       # tests
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

**Stage 2, task 2.1:** `nargo init` for three circuits (`hash2`, `hashes`, `withdrawal`) in `/home/ubuntu/circuits/`.

All details — in `docs/NOTE.md` (personal notes, not committed).

# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant (Claude, ChatGPT, etc.) in a new chat. Load it first — the assistant will understand the project without copying code files.
>
> **Last updated:** 2026-09-21 (after stage 8)

---

## 1. What this project is

**solana-zk-pool** — a private pool for SOL transfers on Solana using ZK proofs (Groth16 on BN254). A user deposits SOL into a shared vault, receives a deposit note (secrets), and can later withdraw SOL to any address without revealing the link between deposit and withdrawal.

**Origin:** [Solana Foundation Bootcamp 2026 — "05-private-transfers"](https://github.com/solana-foundation/solana-bootcamp-2026/tree/main/05-private-transfers).

**Previous version:** [kwebhub/private-transfer](https://github.com/kwebhub/private-transfer) — v1, written with an AI assistant. **solana-zk-pool** is v2, rewritten from scratch with improvements.

**Repository:** https://github.com/kwebhub/solana-zk-pool (private)
**License:** MIT
**Network:** Solana Devnet

**Key improvements over v1:**
- Clean monorepo structure.
- Shared `poseidon` library.
- Pinned versions for all tools.
- `pnpm` instead of `yarn`.
- Cleaner `.gitignore`.
- `Prover.toml` gitignored.
- **Front-running protection** via `recipient_binding`.
- **`amount` in `WithdrawEvent`**.
- **`@noir-lang/noir_js` synced with `nargo`** (1.0.0-rc.2).
- **Health endpoints** in all services.
- **`note_secret`** in deposit note.
- **`VITE_MIN_DEPOSIT` synced with on-chain `MIN_DEPOSIT_AMOUNT`** (0.001).

---

## 2. Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│ FRONTEND (Vue 3 + Vite + Pinia) — port 5173                      │
│ • Deposit: commitment + tx                                       │
│ • Withdraw: witness + proof request + tx                         │
└──────────────────────────────────────────────────────────────────┘
              ↓ HTTP                    ↑ HTTP
┌──────────────────────────────────────────────────────────────────┐
│ BACKEND (Rust + axum) — port 4001                                │
│ • /api/commitments  • /api/root  • /api/proof  • /api/withdraw   │
│ • Indexer: events → Postgres → Merkle tree in Redis              │
└──────────────────────────────────────────────────────────────────┘
       ↓                    ↓                    ↓
┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐
│ POSTGRES     │   │ REDIS        │   │ MERKLE (Node.js)     │
│  :5432       │   │  :6379       │   │  :4003               │
└──────────────┘   └──────────────┘   └──────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ PROVER (Rust + axum) — port 4002                                 │
│ • /prove — wraps sunspot CLI                                     │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
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
| Smart contract | Anchor | 1.1.2 |
| ZK circuit | Noir / nargo | 1.0.0-rc.2 |
| ZK backend | Sunspot | 1.0.0 |
| ZK verification | gnark-solana | v2.0.0 |
| Backend | Rust + axum | 1.98.1 + 0.7 |
| Merkle service | Node.js + Fastify | 24.21.0 + 5.12.5 |
| Noir JS | @noir-lang/noir_js | 1.0.0-rc.2 |
| Prover | Rust + axum | 1.98.1 + 0.7 |
| Frontend | Vue 3 + Vite + Pinia | 3.5+ / 8.x / 4.x |
| Codama | @codama/* | 1.6.2 / 1.5.5 / 2.4.0 |
| Solana SDK | @solana/kit / @solana/web3.js | 6.10.0 / 1.99.0 |
| Contracts client | @solana/program-client-core | 6.10.0 |
| DB | PostgreSQL | 16 |
| Cache | Redis | 7 |
| Monitoring | Prometheus + Grafana | latest |
| Node.js | Node.js | 24.21.0 |
| pnpm | pnpm | 12.5.1 |

---

## 4. Repository structure

```
solana-zk-pool/
├── docs/
│   ├── notes/               ← personal notes (NOT committed)
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
│   ├── programs/zk_pool/
│   └── target/
├── circuits/
│   ├── poseidon/            ← library: hash_1, hash_2, hash_3
│   ├── hash2/
│   ├── hashes/
│   └── withdrawal/
├── services/
│   ├── backend/             ← Rust + axum ✅
│   ├── merkle/              ← Node.js + noir_js ✅
│   └── prover/              ← Rust + Sunspot ✅
├── web/                     ← Vue 3 ✅
│   ├── public/circuits/
│   ├── src/
│   │   ├── generated/       ← Codama
│   │   ├── services/
│   │   ├── stores/
│   │   ├── composables/
│   │   ├── views/
│   │   └── components/
│   └── e2e/
├── solana/
├── .editorconfig
├── .gitignore
├── LICENSE
└── README.md
```

---

## 5. Key decisions

### 5.1. Front-running protection — implemented

Note is bearer-style. Recipient bound via `recipient_binding = hash_2(note_secret, recipient)`.

**5 public inputs:** root, nullifier_hash, recipient, recipient_binding, amount.
**Private inputs:** nullifier, secret, note_secret, merkle_proof, is_even.

### 5.2. Monorepo structure
Flat: `onchain/`, `circuits/`, `services/`, `web/`.

### 5.3. Docker container `solana` — all-in-one
Rust, Solana CLI, Anchor, Noir, Sunspot, Node.js, pnpm. `network_mode: host`.

Sunspot must be cloned for `sunspot deploy`. Add to Dockerfile in stage 12.

### 5.4. Shared `poseidon` library
All hashes in `circuits/poseidon/`.

### 5.5. `hash_4` is NOT used
Poseidon2: `t = 4`, rate = 3. Only `hash_1`, `hash_2`, `hash_3`.

### 5.6. Deposit splitting into unequal parts (planned, stage 12)
k (1..10), `amounts: Vec<u64>`, unequal.

### 5.7. Deposit trust model — deferred to stage 12
Program does NOT verify `commitment` on-chain.

### 5.8. Circuits sync — automatic Rust script (planned, stage 9 or 12)
Currently copied manually.

### 5.9. Prover bottleneck (planned, stage 12)
`sunspot prove` single-threaded. Redis Streams queue planned.

### 5.10. `commitment_exists` / `is_nullifier_used` do not filter by `pool_address` (planned, stage 12)
If DB has `leaf_index=0` from another pool, new pool is skipped. Add `pool_address` to condition.

### 5.11. Frontend: pnpm + `@solana/kit`/`@codama/*` pinned to exact versions
Otherwise Codama generates code for newer types that don't exist in pinned `@solana/program-client-core@6.10.0`.

---

## 6. What has been done

### ✅ Stage 1. Repository setup (2026-09-20)
Commits: `b0ef53b`, `d43552a`.

### ✅ Stage 2. ZK circuits in Noir (2026-09-20)
Commit: `2fff662`.

### ✅ Stage 3. Sunspot verifier (2026-09-20)
Nothing committed.

### ✅ Stage 4.1a. Front-running protection (2026-09-20)
**VERIFIER_PROGRAM_ID:** `AukfYGQqhv3ZkdB1G7cKF3djsE9FKMrAfLd34doSWQex`.
Commit: `f461a19`.

### ✅ Stage 4. Anchor program (2026-09-21)
**Program ID:** `CZjsHYcE6oPhj2ZAKVdnaz36FQHC7SBd7MSY1jmAaj3o`.
7 tests. Commits: `5aebd2d`, `<test commit>`.

### ✅ Stage 5. Backend (2026-09-21)
Commit: `b5c1ba2`.

### ✅ Stage 6. Merkle service (2026-09-21)
Commit: `ed3b912`.

### ✅ Stage 7. Prover (2026-09-21)
Commit: `bcdec13`.

### ✅ Stage 8. Frontend (2026-09-21)
- Vue 3 + Vite + Pinia + TS + Pug + SCSS.
- Codama-generated clients.
- Services, stores, composables, views, components.
- E2E (Playwright): 6 tests.
- Copied circuits to `public/circuits/`.
- Deposit + withdraw flow **end-to-end**.
- Commit: `3f3ae8d`.

---

## 7. What's next

### Stage 9. Infrastructure (next)
- Makefile (start/stop/status/attach/logs).
- Prometheus + Grafana (dashboards, provisioning).
- Rust CLI utility for circuits sync (compile + copy JSON to merkle/web).

### Stages 10–12
Processes, Security, Finalization.

---

## 8. Roadmap

- [x] **Stage 1.** Repository setup
- [x] **Stage 2.** ZK circuits in Noir
- [x] **Stage 3.** Sunspot verifier
- [x] **Stage 4.** Anchor program
- [x] **Stage 5.** Backend
- [x] **Stage 6.** Merkle service
- [x] **Stage 7.** Prover
- [x] **Stage 8.** Frontend
- [ ] **Stage 9.** Infrastructure (Makefile, Prometheus, Grafana, Rust circuits-sync script)
- [ ] **Stage 10.** Engineering processes (CI/CD, security, templates)
- [ ] **Stage 11.** Security (threat model, invariants, tests)
- [ ] **Stage 12.** Finalization
  - README EN/RU, CHANGELOG, release
  - Split deposit into multiple unequal parts
  - **Improve deposit trust model:** verify commitment on-chain
  - Add Sunspot `git clone` to Dockerfile
  - Redis Streams queue for prover
  - Timeout for `sunspot prove`
  - Per-request temp dirs in prover
  - **`commitment_exists` / `is_nullifier_used` filter by `pool_address`**
  - Remove `debug!` from indexer if desired (decide later)

---

## 9. Known limitations / TODOs

- **Trusted setup for Groth16** — no MPC.
- **Upgrade authority** — single key.
- **Secrets in localStorage** — XSS risk.
- **Backend sees commitments/nullifiers.**
- **MerkleTree depth 20.**
- **TREE_DEPTH hardcoded** in circuit.
- **Deposit trust model** — not verified on-chain.
- **Sunspot cloned manually.**
- **Circuits copied manually.**
- **Prover:** single-threaded, no timeout, race condition on files, no witness size limit.
- **DB queries** don't filter by `pool_address`.
- **`Indexer` debug logs** — currently verbose (decide in stage 12).

---

## 10. Git workflow

- Conventional commits.
- **All committed files in English.** Russian — only in `docs/ru/`.

---

## 11. Commands

### Docker
```bash
cd infra
docker compose build solana
docker compose up -d
docker compose exec solana bash
```

### Anchor
```bash
cd /home/ubuntu/onchain
anchor build && anchor deploy --provider.cluster devnet
cargo test
```

### Noir
```bash
cd /home/ubuntu/circuits/<name>
nargo test
nargo test test_generate_valid_inputs --show-output
nargo execute
```

### Sunspot
```bash
git clone https://github.com/reilabs/sunspot.git /home/ubuntu/sunspot
cd /home/ubuntu/circuits/withdrawal
sunspot compile target/withdrawal.json
sunspot setup target/withdrawal.ccs
sunspot deploy target/withdrawal.vk
sunspot prove <acir> <witness> <ccs> <pk>
```

### Backend
```bash
cd /home/ubuntu/services/backend && cargo run
```

### Merkle
```bash
cd /home/ubuntu/services/merkle && pnpm install && pnpm test
MERKLE_PORT=4003 node src/server.js
```

### Prover
```bash
cd /home/ubuntu/services/prover && PROVER_PORT=4002 cargo run
```

### Frontend
```bash
cd /home/ubuntu/web
pnpm install && pnpm approve-builds && pnpm install
rm -rf src/generated && pnpm codama
pnpm dev
pnpm test:e2e --project=chromium
```

### Migrations (from host)
```bash
docker exec -i zkpool-postgres psql -U zkpool -d zkpool < \
  ~/Projects/Solana/solana-zk-pool/services/backend/migrations/001_init.sql
```

### Reset DB + Redis tree
```bash
docker exec zkpool-postgres psql -U zkpool -d zkpool -c "TRUNCATE commitments, roots, nullifiers RESTART IDENTITY;"
docker exec zkpool-redis redis-cli KEYS "tree:<POOL_ADDRESS>:*" | xargs -r docker exec -i zkpool-redis redis-cli DEL
docker exec zkpool-redis redis-cli DEL "indexer:<POOL_ADDRESS>:last_signature"
```

---

## 12. Next task for a new assistant

**Stage 9:** Infrastructure. Makefile (start/stop/status/attach/logs), Prometheus + Grafana (dashboards, provisioning), Rust CLI utility for circuits sync (compile + copy JSON to `services/merkle/circuits/` and `web/public/circuits/`).

All details — in `docs/notes/` (personal notes, not committed).

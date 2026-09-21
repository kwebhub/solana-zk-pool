# PROJECT_CONTEXT.md

> **Purpose:** this file is the single entry point for an AI assistant (Claude, ChatGPT, etc.) in a new chat. Load it first — the assistant will understand the project without copying code files.
>
> **Last updated:** 2026-09-21 (after stage 11)

---

## 1. What this project is

**solana-zk-pool** — a private pool for SOL transfers on Solana using ZK proofs (Groth16 on BN254). A user deposits SOL into a shared vault, receives a deposit note (secrets), and can later withdraw SOL to any address without revealing the link between deposit and withdrawal.

**Origin:** [Solana Foundation Bootcamp 2026 — "05-private-transfers"](https://github.com/solana-foundation/solana-bootcamp-2026/tree/main/05-private-transfers).

**Previous version:** [kwebhub/private-transfer](https://github.com/kwebhub/private-transfer) — v1.

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
- **`@noir-lang/noir_js` synced with `nargo`**.
- **Health endpoints** in all services.
- **`note_secret`** in deposit note.
- **`VITE_MIN_DEPOSIT` synced with on-chain `MIN_DEPOSIT_AMOUNT`**.
- **Makefile** for full orchestration.
- **Prometheus + Grafana** provisioning.
- **Rust CLI** for circuits sync.
- **CI/CD**: 4 GitHub Actions workflows.
- **Threat model** with 7 invariants and 12 attack scenarios.
- **18 integration tests** for the program.

---

## 2. Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│ FRONTEND (Vue 3 + Vite + Pinia) — port 5173                      │
└──────────────────────────────────────────────────────────────────┘
              ↓ HTTP                    ↑ HTTP
┌──────────────────────────────────────────────────────────────────┐
│ BACKEND (Rust + axum) — port 4001                                │
└──────────────────────────────────────────────────────────────────┘
       ↓                    ↓                    ↓
┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐
│ POSTGRES     │   │ REDIS        │   │ MERKLE (Node.js)     │
│  :5432       │   │  :6379       │   │  :4003               │
└──────────────┘   └──────────────┘   └──────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ PROVER (Rust + axum) — port 4002                                 │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ MONITORING                                                       │
│ • Prometheus :9090  • Grafana :3000                              │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ SOLANA                                                           │
│ • Program zk_pool: C5Q4FjJ8zSD41FfMfcEccjnGnFJ7K44rDgnyrTx2fr6q │
│ • Verifier program: AukfYGQqhv3ZkdB1G7cKF3djsE9FKMrAfLd34doSWQex│
└──────────────────────────────────────────────────────────────────┘
```

### Services and ports

| Service | Port | Where |
|---------|------|-------|
| frontend | 5173 | host |
| backend | 4001 | `solana` container |
| prover | 4002 | `solana` container |
| merkle | 4003 | `solana` container |
| postgres | 5432 | Docker |
| redis | 6379 | Docker |
| prometheus | 9090 | Docker |
| grafana | 3000 | Docker |

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
| DB | PostgreSQL | 16 |
| Cache | Redis | 7 |
| Monitoring | Prometheus + Grafana | latest |
| Container | Docker | latest |
| Node.js | Node.js | 24.21.0 |
| pnpm | pnpm | 12.5.1 |
| CI/CD | GitHub Actions | 4 workflows |

---

## 4. Repository structure

```
solana-zk-pool/
├── .github/
│   ├── workflows/{ci,docker,release,security}.yml
│   ├── ISSUE_TEMPLATE/
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── dependabot.yml
├── Makefile
├── CHANGELOG.md
├── CONTRIBUTING.md
├── SECURITY.md
├── deny.toml
├── docs/
│   ├── notes/               ← personal notes (NOT committed)
│   ├── PROJECT_CONTEXT.md   ← this file
│   ├── threat-model.md
│   └── ru/                  ← Russian localization (later)
├── infra/
│   ├── docker-compose.yml
│   ├── prometheus.yml
│   ├── grafana/provisioning/
│   └── docker/Dockerfile.solana
├── scripts/sync-circuits/
├── onchain/programs/zk_pool/
├── circuits/{poseidon,hash2,hashes,withdrawal}/
├── services/{backend,merkle,prover}/
├── web/
├── solana/
├── .editorconfig
├── .gitignore
├── LICENSE
└── README.md
```

---

## 5. Key decisions

### 5.1. Front-running protection — implemented
`recipient_binding = hash_2(note_secret, recipient)`. 5 public inputs.

### 5.2. Monorepo structure

### 5.3. Docker container `solana` — all-in-one, `network_mode: host`

### 5.4. Shared `poseidon` library

### 5.5. `hash_4` is NOT used

### 5.6. Deposit splitting into unequal parts (planned, stage 12)

### 5.7. Deposit trust model — deferred to stage 12

### 5.8. Circuits sync — Rust CLI

### 5.9. Prover bottleneck (planned, stage 12)

### 5.10. DB queries don't filter by `pool_address` (planned, stage 12)

### 5.11. Frontend: `@solana/kit`/`@codama/*` pinned exact

### 5.12. Makefile — single entry point

### 5.13. CI/CD

### 5.14. Dependabot

### 5.15. Threat model documented
- 7 invariants (I1-I7).
- 12 attack scenarios (A1-A12).
- 18 integration tests.

---

## 6. What has been done

### ✅ Stage 1. Repository setup (2026-09-20)
Commits: `b0ef53b`, `d43552a`.

### ✅ Stage 2. ZK circuits (2026-09-20)
Commit: `2fff662`.

### ✅ Stage 3. Sunspot verifier (2026-09-20)

### ✅ Stage 4.1a. Front-running protection (2026-09-20)
**VERIFIER_PROGRAM_ID:** `AukfYGQqhv3ZkdB1G7cKF3djsE9FKMrAfLd34doSWQex`.
Commit: `f461a19`.

### ✅ Stage 4. Anchor program (2026-09-21)
**Program ID:** `C5Q4FjJ8zSD41FfMfcEccjnGnFJ7K44rDgnyrTx2fr6q`.

### ✅ Stage 5. Backend (2026-09-21)
Commit: `b5c1ba2`.

### ✅ Stage 6. Merkle service (2026-09-21)
Commit: `ed3b912`.

### ✅ Stage 7. Prover (2026-09-21)
Commit: `bcdec13`.

### ✅ Stage 8. Frontend (2026-09-21)
Commit: `3f3ae8d`.

### ✅ Stage 9. Infrastructure (2026-09-21)
Commits: `7c2b469`, `18f794d`, `4d5c258`.

### ✅ Stage 10. Engineering processes (2026-09-21)
Commit: `aa92b60`.

### ✅ Stage 11. Security (2026-09-21)
- `docs/threat-model.md` — 7 invariants, 12 attack scenarios.
- 18 integration tests.
- Commit: `4d341e9`.

---

## 7. What's next

### Stage 12. Finalization (next)
- README EN/RU.
- Split deposit into multiple unequal parts.
- On-chain commitment verification.
- Sunspot `git clone` in Dockerfile.
- Redis Streams queue for prover.
- Timeout for `sunspot prove`.
- Per-request temp dirs in prover.
- **`commitment_exists` / `is_nullifier_used` filter by `pool_address`.**
- CHANGELOG release.
- Final documentation.

---

## 8. Roadmap

- [x] Stage 1. Repository setup
- [x] Stage 2. ZK circuits
- [x] Stage 3. Sunspot verifier
- [x] Stage 4. Anchor program
- [x] Stage 5. Backend
- [x] Stage 6. Merkle service
- [x] Stage 7. Prover
- [x] Stage 8. Frontend
- [x] Stage 9. Infrastructure
- [x] Stage 10. Engineering processes
- [x] Stage 11. Security
- [ ] Stage 12. Finalization

---

## 9. Known limitations / TODOs

- **Trusted setup for Groth16** — no MPC.
- **Upgrade authority** — single key.
- **Secrets in localStorage** — XSS risk.
- **Backend sees commitments/nullifiers.**
- **MerkleTree depth 20.**
- **TREE_DEPTH hardcoded.**
- **Deposit trust model** — not verified on-chain.
- **Sunspot cloned manually.**
- **Prover:** single-threaded, no timeout, race condition.
- **DB queries** don't filter by `pool_address`.
- **Indexer debug logs** — verbose.
- **Docker workflows** — only backend/merkle/prover.

---

## 10. Git workflow

- Conventional commits.
- **All committed files in English.**

---

## 11. Commands

### Makefile
```bash
make start / stop / status / attach <name> / logs <name> / sync-circuits
```

### Docker
```bash
cd infra && docker compose up -d
docker compose exec solana bash
```

### Tests
```bash
# Anchor
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  "cd /home/ubuntu/onchain && cargo test"

# Noir
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  "cd /home/ubuntu/circuits/withdrawal && nargo test"

# Merkle
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  "cd /home/ubuntu/services/merkle && pnpm test"

# Frontend
docker compose -f infra/docker-compose.yml exec solana bash -ic \
  "cd /home/ubuntu/web && pnpm test:e2e --project=chromium"
```

---

## 12. Next task for a new assistant

**Stage 12:** Finalization. Multiple improvements:
1. README EN/RU.
2. Split deposit into unequal parts (program + frontend + note).
3. On-chain commitment verification.
4. Sunspot `git clone` in Dockerfile.
5. Redis Streams for prover.
6. Timeout and per-request dirs in prover.
7. DB queries filter by `pool_address`.
8. CHANGELOG release.
9. Final documentation.

All details — in `docs/notes/` (personal notes, not committed).

# solana-zk-pool

> Private SOL transfers on Solana using Groth16 zero-knowledge proofs.

[![CI](https://github.com/kwebhub/solana-zk-pool/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/kwebhub/solana-zk-pool/actions/workflows/ci.yml)
[![Security](https://github.com/kwebhub/solana-zk-pool/actions/workflows/security.yml/badge.svg?branch=main)](https://github.com/kwebhub/solana-zk-pool/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Solana-Devnet-9945FF?logo=solana)](https://solana.com)
[![Noir](https://img.shields.io/badge/Noir-1.0.0--rc.2-blue)](https://noir-lang.org)

**solana-zk-pool** is a private pool for SOL transfers. A user deposits SOL into a shared vault, receives a *deposit note* (three secrets), and can later withdraw SOL to **any** address without revealing the link between deposit and withdrawal.

The project implements the full stack of a ZK protocol:

- **Noir** — ZK circuit with Poseidon2 and Merkle proof.
- **Sunspot** — Noir → Groth16 proof for Solana.
- **Anchor** — Solana program with an on-chain verifier.
- **Rust + axum** — backend with indexer and Merkle tree in Redis.
- **Vue 3 + Vite** — frontend with client-side witness generation.
- **Postgres + Redis** — storage and cache.
- **Prometheus + Grafana** — monitoring.

> ⚠️ **Status:** devnet, portfolio project. Do not use in mainnet without audit.

---

## Table of Contents

- [How It Works](#how-it-works)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [Repository Structure](#repository-structure)
- [Tech Stack](#tech-stack)
- [Commands](#commands)
- [Monitoring](#monitoring)
- [Security](#security)
- [Documentation](#documentation)
- [License](#license)

---

## How It Works

### Deposit

1. Frontend generates three secrets: `nullifierSecret`, `secret`, `noteSecret`.
2. Computes `commitment = Poseidon2(nullifierSecret, secret, amount)`.
3. Requests the current Merkle root from the backend.
4. Sends a `deposit` tx to the program: SOL goes to the vault, the commitment becomes a leaf.
5. Stores the *deposit note* (secrets + commitment) in localStorage.

### Withdrawal

1. Frontend parses the note, requests a Merkle proof from the backend.
2. Computes `recipientBinding = Poseidon2(noteSecret, recipient)`.
3. Generates a ZK witness (noir_js) — a proof that it knows the secrets corresponding to a leaf in the tree, **without revealing** them.
4. Backend proxies the witness to the prover (Sunspot), receives a Groth16 proof.
5. Frontend sends a `withdraw` tx: the program verifies the proof on-chain via the verifier program and transfers SOL from the vault.
6. A `NullifierRecord` PDA is created — repeated withdrawal with the same nullifier is impossible.

**What the blockchain sees:** commitment, nullifier_hash, root, recipient, recipient_binding, amount.
**What is hidden:** the link between deposit and withdrawal.

**Front-running protection:** `recipient` is bound to the proof via `recipient_binding = hash_2(noteSecret, recipient)`. An attacker who doesn't know `noteSecret` cannot substitute the recipient.

More — [`docs/zk-explained.md`](docs/zk-explained.md) (planned).

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│ FRONTEND (Vue 3 + Vite + Pinia) — port 5173                      │
└──────────────────────────────────────────────────────────────────┘
              ↓ HTTP                    ↑ HTTP
┌──────────────────────────────────────────────────────────────────┐
│ BACKEND (Rust + axum) — port 4001                                │
│ • /api/commitments  • /api/root  • /api/proof  • /api/withdraw   │
└──────────────────────────────────────────────────────────────────┘
       ↓                    ↓                    ↓
┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐
│ POSTGRES     │   │ REDIS        │   │ MERKLE (Node.js)     │
│  :5432       │   │  :6379       │   │  :4003               │
└──────────────┘   └──────────────┘   └──────────────────────┘
                                            ↑
┌──────────────────────────────────────────────────────────────────┐
│ PROVER (Rust + axum) — port 4002                                 │
│ • /prove — wraps sunspot CLI                                     │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ SOLANA                                                           │
│ • Program zk_pool: C5Q4FjJ8zSD41FfMfcEccjnGnFJ7K44rDgnyrTx2fr6q │
│ • Verifier program: AukfYGQqhv3ZkdB1G7cKF3djsE9FKMrAfLd34doSWQex│
└──────────────────────────────────────────────────────────────────┘
```

More — [`docs/PROJECT_CONTEXT.md`](docs/PROJECT_CONTEXT.md).

---

## Quick Start

### Requirements

- Linux (Debian 12+ / Ubuntu 22.04+)
- Docker + Docker Compose
- tmux
- make

Everything else (Rust, Anchor, Noir, Sunspot, Node.js, pnpm) runs **inside** the `solana` container.

### Installation

```bash
git clone git@github.com:kwebhub/solana-zk-pool.git
cd solana-zk-pool

# 1. Start all services (Docker + tmux)
make start

# 2. Check status
make status

# 3. Open the app
#    Frontend:   http://localhost:5173
#    Backend:    http://localhost:4001
#    Merkle:     http://localhost:4003
#    Prometheus: http://localhost:9090
#    Grafana:    http://localhost:3000 (admin/admin)
```

### Stop

```bash
make stop
```

---

## Repository Structure

```
solana-zk-pool/
├── .github/              ← CI/CD, templates, Dependabot
├── Makefile              ← service management
├── docs/                 ← documentation
├── infra/                ← Docker, Prometheus, Grafana
├── scripts/              ← Rust CLI (sync-circuits)
├── onchain/              ← Anchor program
├── circuits/             ← Noir circuits
├── services/
│   ├── backend/          ← Rust + axum
│   ├── merkle/           ← Node.js + noir_js
│   └── prover/           ← Rust + Sunspot CLI
└── web/                  ← Vue 3 + Vite
```

Full structure — in [`docs/PROJECT_CONTEXT.md`](docs/PROJECT_CONTEXT.md#4-repository-structure).

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Smart contract | Anchor (Rust) |
| ZK circuit | Noir |
| ZK proof | Sunspot → Groth16 (BN254) |
| On-chain verifier | gnark-solana |
| Backend | Rust + axum |
| DB | PostgreSQL 16 |
| Cache | Redis 7 |
| Frontend | Vue 3 + Vite + Pinia |
| Contracts client | Codama (autogen) |
| Monitoring | Prometheus + Grafana |
| Container | Docker + Docker Compose |

---

## Commands

```bash
# All services
make start                         # start (tmux + docker)
make status                        # status
make stop                          # stop
make attach backend                # attach to tmux session
make logs backend                  # tail log
make sync-circuits                 # compile circuits + sync JSON

# Program
cd onchain && anchor build
cd onchain && anchor deploy --provider.cluster devnet
cd onchain && cargo test

# Backend
cd services/backend && cargo build --release
cd services/backend && cargo test

# Frontend
cd web && pnpm dev
cd web && pnpm build
cd web && pnpm test:e2e --project=chromium
cd web && pnpm codama

# Merkle
cd services/merkle && pnpm start
cd services/merkle && pnpm test
```

---

## Monitoring

After `make start`:

- **Prometheus** (http://localhost:9090) — scrapes `/metrics` every 15s.
- **Grafana** (http://localhost:3000) — dashboard `zkpool Overview` with 9 panels.

**Metrics:**

| Metric | What it shows |
|--------|---------------|
| `axum_http_requests_total` | HTTP requests |
| `axum_http_requests_duration_seconds` | HTTP latency |
| `zkpool_indexer_deposits_total` | Processed DepositEvent |
| `zkpool_indexer_withdrawals_total` | Processed WithdrawEvent |
| `zkpool_indexer_lag_seconds` | Lag behind blockchain |
| `zkpool_indexer_tick_duration_seconds` | One indexer tick duration |
| `zkpool_indexer_tree_size` | Tree size |
| `zkpool_tree_add_leaf_duration_seconds` | Add leaf duration |
| `zkpool_tree_hash_duration_seconds` | Poseidon2 hash duration |
| `zkpool_tree_errors_total` | Tree errors |
| `zkpool_db_errors_total` | DB errors |

---

## Security

- **Threat model:** [`docs/threat-model.md`](docs/threat-model.md)
- **Report vulnerabilities:** [`SECURITY.md`](SECURITY.md)

**Key invariants:**

1. Each `nullifier_hash` is used **exactly once** — guaranteed by `init` for the `NullifierRecord` PDA.
2. `root` must be in the history of `PoolState.roots[10]`.
3. ZK proof is verified **on-chain** via the verifier program.
4. `recipient` is bound to the proof via `recipient_binding = hash_2(note_secret, recipient)`.

**Known limitations:**

- **Trusted setup** — no MPC ceremony.
- **Upgrade authority** — single key.
- **Secrets in localStorage** — XSS risk.
- **Deposit trust model** — program does NOT verify `commitment` on-chain.

**Automated checks:**

- `cargo-deny` — licenses, vulnerabilities, duplicates.
- `cargo-audit` — RUSTSEC.
- `pnpm audit` — Node.js dependencies.
- `trivy` — Docker image scanning.
- `gitleaks` — secret scanning in git history.

---

## Documentation

- [`docs/PROJECT_CONTEXT.md`](docs/PROJECT_CONTEXT.md) — full project context.
- [`docs/threat-model.md`](docs/threat-model.md) — threat model.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — how to contribute.
- [`SECURITY.md`](SECURITY.md) — security policy.
- [`CHANGELOG.md`](CHANGELOG.md) — changelog.

**In progress:**

- `docs/architecture.md`
- `docs/zk-explained.md`
- `docs/deployment.md`
- `docs/ru/README.md`

---

## License

MIT — see [`LICENSE`](LICENSE).

**Origin:** [Solana Foundation Bootcamp 2026 — "05-private-transfers"](https://github.com/solana-foundation/solana-bootcamp-2026/tree/main/05-private-transfers).
**Previous version:** [kwebhub/private-transfer](https://github.com/kwebhub/private-transfer).

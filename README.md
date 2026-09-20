# Solana-zk-pool

> Private SOL transfers on Solana using Groth16 zero-knowledge proofs.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Solana-Devnet-9945FF?logo=solana)](https://solana.com)
[![Noir](https://img.shields.io/badge/Noir-1.0.0--rc.1-blue)](https://noir-lang.org)

> ⚠️ Status: work in progress. Devnet only. Not audited. Do not use with real funds.

> Inspired by [Solana Foundation Bootcamp 2026 — "05-private-transfers"](https://github.com/solana-foundation/solana-bootcamp-2026/tree/main/05-private-transfers)
> First version of project: [private-transfer:v1](https://github.com/kwebhub/private-transfer)

---

**zk-pool** is a private pool for SOL transfers. A user deposits SOL into a shared
vault, receives a *deposit note* (two secrets), and can later withdraw SOL to
**any** address without revealing the link between deposit and withdrawal.

The project implements the full stack of a ZK protocol:

- **Noir** — ZK circuit with Poseidon2 and Merkle proof.
- **Sunspot** — Noir → Groth16 proof for Solana.
- **Anchor** — Solana program with an on-chain verifier.
- **Rust + axum** — backend with indexer and Merkle tree in Redis.
- **Vue 3 + Vite** — frontend with client-side witness generation.
- **Postgres + Redis** — storage and cache.
- **Prometheus + Grafana** — monitoring.

See [LICENSE](./LICENSE)

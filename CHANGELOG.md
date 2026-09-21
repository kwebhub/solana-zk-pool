# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Front-running protection** via `recipient_binding = hash_2(note_secret, recipient)`.
  - New public input `recipient_binding` (5 total).
  - New private input `note_secret`.
- **`amount`** field in `WithdrawEvent` (was missing in v1).
- **`note_secret`** field in `DepositNote` (localStorage).
- **Merkle service** on Node.js + Fastify + `@noir-lang/noir_js@1.0.0-rc.2`.
- **Prover service** on Rust + axum (wrapper over Sunspot CLI).
- **Backend**: incremental Merkle tree in Redis, indexer with discriminator-based event parsing, Prometheus metrics with `zkpool_` prefix, rate limiting.
- **Frontend**: Vue 3 + Vite + Pinia + TypeScript + Pug + SCSS, Codama-generated clients, 6 Playwright E2E tests.
- **Makefile** for service management (`start`, `stop`, `status`, `attach`, `logs`, `sync-circuits`).
- **Prometheus + Grafana** provisioning with 9-panel dashboard.
- **Rust CLI** `sync-circuits` for compiling circuits and syncing JSON artifacts.
- **GitHub Actions workflows**: `ci.yml`, `docker.yml`, `release.yml`, `security.yml`.
- **Dependabot** configuration for Cargo, npm, GitHub Actions, Docker.
- **Issue/PR templates**.
- **Documentation**: `CONTRIBUTING.md`, `SECURITY.md`, `docs/notes/`, `PROJECT_CONTEXT.md`.

### Changed

- **Monorepo structure**: flat `onchain/`, `circuits/`, `services/`, `web/` (not `workspace/ptrans/app/...`).
- **Shared `poseidon` library** in `circuits/poseidon/` — no duplication of hash primitives.
- **`@noir-lang/noir_js`** synced with `nargo 1.0.0-rc.2` (was `1.0.0-rc.1`).
- **`pnpm`** instead of `yarn`.
- **`VITE_MIN_DEPOSIT`** synced with on-chain `MIN_DEPOSIT_AMOUNT` (`0.001 SOL`).
- **`network_mode: host`** for `solana` container (needed for localhost access).
- **`bash -ic`** in Makefile (interactive shell for `.bashrc` / nvm / PATH).
- **Services run in tmux sessions** with `zkpool-` prefix.
- **Log files** in `/tmp/zkpool-<service>.log`.

### Fixed

- **Front-running vulnerability** (v1 had `recipient` as a public input but not in constraints).
- **`Prover.toml`** now gitignored.
- **`.gitignore`**: removed duplicate rules, added `.vscode/extensions.json` exception.
- **`hash_4`** no longer used (sponge capacity = 0 is insecure).
- **DB queries** in backend deduplicated by `leaf_index` (known issue with `pool_address` — TODO).

### Security

- **No trusted setup** — Sunspot uses standard setup without MPC ceremony (production limitation).
- **Single upgrade authority** — no multisig (production limitation).
- **Secrets in localStorage** — XSS risk (known limitation).

## [0.1.0] — 2026-09-21

### Added

- Initial release.
- Full ZK protocol stack: Noir circuits, Sunspot verifier, Anchor program, backend, merkle, prover, frontend.
- Program ID: `CZjsHYcE6oPhj2ZAKVdnaz36FQHC7SBd7MSY1jmAaj3o`.
- Verifier Program ID: `AukfYGQqhv3ZkdB1G7cKF3djsE9FKMrAfLd34doSWQex`.

[Unreleased]: https://github.com/kwebhub/solana-zk-pool/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kwebhub/solana-zk-pool/releases/tag/v0.1.0

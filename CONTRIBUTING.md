# Contributing to zk-pool

Thanks for your interest in the project! This document describes how to contribute to `solana-zk-pool`.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How can I help](#how-can-i-help)
- [Environment setup](#environment-setup)
- [Git flow](#git-flow)
- [Code style](#code-style)
- [Commits](#commits)
- [Pull Requests](#pull-requests)
- [Tests](#tests)

---

## Code of Conduct

Be polite. Respect others. Constructive criticism is welcome, personal insults are not.

---

## How can I help

- 🐛 **Find and fix a bug** — [open issue](https://github.com/kwebhub/solana-zk-pool/issues/new?template=bug_report.md)
- ✨ **Suggest a feature** — [open feature request](https://github.com/kwebhub/solana-zk-pool/issues/new?template=feature_request.md)
- 📖 **Improve documentation** — PR with edits in `docs/` or README
- 🧪 **Write tests** — see [docs/PROJECT_CONTEXT.md](docs/PROJECT_CONTEXT.md)
- 🔒 **Report a vulnerability** — see [SECURITY.md](SECURITY.md)

---

## Environment setup

Everything runs inside the `solana` container. Only `git`, `docker`, and `make` are needed on the host.

```bash
git clone git@github.com:kwebhub/solana-zk-pool.git
cd solana-zk-pool

# 1. Build and start all services
make start

# 2. Check status
make status

# 3. Attach to a service
make attach backend
```

**Full setup instructions** — [docs/deployment.md](docs/deployment.md).

---

## Git flow

We use **trunk-based development** with short feature branches.

### Branches

| Branch | Purpose |
|--------|---------|
| `main` | **Production-ready**. Protected. PR only. |
| `feat/<name>` | New feature. Branches from `main`. |
| `fix/<name>` | Bug fix. Branches from `main`. |
| `docs/<name>` | Documentation. Branches from `main`. |
| `chore/<name>` | Updates, configs, CI. Branches from `main`. |
| `refactor/<name>` | Refactoring without behavior change. |

### Workflow

```bash
git checkout main
git pull origin main
git checkout -b feat/my-feature

# work

git add .
git commit -m "feat: add my feature"

git fetch origin
git rebase origin/main

git push origin feat/my-feature
```

### Rebase, not merge

Always use `git rebase origin/main`, not `git merge main`. This keeps history linear.

---

## Code style

### Rust

- **`rustfmt`** — required. `cargo fmt`.
- **`clippy`** — required. `cargo clippy --all-targets -- -D warnings`.
- **`edition 2021`**.
- **Doc comments** for public functions (`///`).
- **Errors** via `thiserror` (not `Box<dyn Error>` in production).
- **Logs** via `tracing` (not `println!`).

### TypeScript / Vue

- **`eslint`** + **`oxlint`** — required.
- **`prettier`** — formatting.
- **`vue-tsc`** — type checking.
- **Composition API** (`<script setup>`).
- **Pug** for templates.
- **SCSS** for styles.

### Noir

- **`nargo fmt`** — formatting.
- **`nargo test`** — tests.
- Comments in English.

### YAML / Markdown

- **2 spaces** for indentation.
- **LF** line endings.
- **UTF-8**.
- See `.editorconfig`.

---

## Commits

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

| Type | When |
|------|------|
| `feat` | New functionality |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Formatting (no logic change) |
| `refactor` | Refactoring (no behavior change) |
| `perf` | Performance optimization |
| `test` | Adding/changing tests |
| `build` | Build, dependencies |
| `ci` | CI/CD |
| `chore` | Other (updates, configs) |

### Scope (optional)

`backend`, `prover`, `merkle`, `frontend`, `program`, `circuits`, `docs`, `ci`, `deps`.

### Examples

```
feat(backend): add /api/proof endpoint
fix(frontend): handle Solflare network mismatch
docs: add threat model
ci: pin Anchor to 1.1.2
chore(deps): bump axum from 0.7.5 to 0.7.6
refactor(backend): extract config to Config struct
```

---

## Pull Requests

1. **Fill in the template** (`.github/PULL_REQUEST_TEMPLATE.md`).
2. **Link to issue**: `Closes #123`.
3. **One PR = one task.** Don't mix features.
4. **CI must be green** before review.
5. **Update branch** via `git rebase origin/main` (not merge).
6. **Respond to reviewer comments.**
7. **Squash & Merge** — if there are multiple commits, they will be squashed into one.

### Review

- Minimum **1 approve** for merge.
- **`main` is protected**: no direct pushes.
- CI must pass **all jobs**.

---

## Tests

### Anchor program

```bash
cd /home/ubuntu/onchain
cargo test
```

### Noir

```bash
cd /home/ubuntu/circuits/withdrawal
nargo test
```

### Backend

```bash
cd /home/ubuntu/services/backend
cargo test
```

### Merkle

```bash
cd /home/ubuntu/services/merkle
pnpm test
```

### Frontend

```bash
cd /home/ubuntu/web
pnpm test:unit    # vitest
pnpm test:e2e     # playwright
```

**Requirements:** new features **must** have tests. Bug fixes **must** have regression tests.

---

## Questions

- **General questions:** [Discussions](https://github.com/kwebhub/solana-zk-pool/discussions)
- **Bugs:** [Issues](https://github.com/kwebhub/solana-zk-pool/issues)
- **Security:** [SECURITY.md](SECURITY.md)

Thanks for contributing! 🎉

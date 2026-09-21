# solana-zk-pool

> Приватные переводы SOL на Solana с использованием Groth16 zero-knowledge доказательств.

[![CI](https://github.com/kwebhub/solana-zk-pool/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/kwebhub/solana-zk-pool/actions/workflows/ci.yml)
[![Security](https://github.com/kwebhub/solana-zk-pool/actions/workflows/security.yml/badge.svg?branch=main)](https://github.com/kwebhub/solana-zk-pool/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Solana-Devnet-9945FF?logo=solana)](https://solana.com)
[![Noir](https://img.shields.io/badge/Noir-1.0.0--rc.2-blue)](https://noir-lang.org)

**solana-zk-pool** — приватный пул для переводов SOL. Пользователь вносит SOL в общий vault, получает *deposit note* (три секрета), и позже может вывести SOL на **любой** адрес, не раскрывая связь между депозитом и выводом.

[English version](../../README.md)

---

## Содержание

- [Как это работает](#как-это-работает)
- [Архитектура](#архитектура)
- [Быстрый старт](#быстрый-старт)
- [Структура репозитория](#структура-репозитория)
- [Технологии](#технологии)
- [Команды](#команды)
- [Мониторинг](#мониторинг)
- [Безопасность](#безопасность)
- [Документация](#документация)
- [Лицензия](#лицензия)

---

## Как это работает

### Депозит

1. Frontend генерирует три секрета: `nullifierSecret`, `secret`, `noteSecret`.
2. Вычисляет `commitment = Poseidon2(nullifierSecret, secret, amount)`.
3. Запрашивает актуальный Merkle root у backend.
4. Отправляет `deposit` tx в программу: SOL уходит в vault, commitment становится листом дерева.
5. Сохраняет *deposit note* (секреты + commitment) в localStorage.

### Вывод

1. Frontend парсит note, запрашивает Merkle proof у backend.
2. Вычисляет `recipientBinding = Poseidon2(noteSecret, recipient)`.
3. Генерирует ZK-witness (noir_js) — доказательство, что он знает секреты, соответствующие листу в дереве, **не раскрывая** их.
4. Backend проксирует witness в prover (Sunspot), получает Groth16 proof.
5. Frontend отправляет `withdraw` tx: программа верифицирует proof on-chain через verifier program и переводит SOL из vault.
6. `NullifierRecord` PDA создаётся — повторный вывод с тем же nullifier невозможен.

**Что видит блокчейн:** commitment, nullifier_hash, root, recipient, recipient_binding, amount.
**Что скрыто:** связь между депозитом и выводом.

**Защита от front-running:** `recipient` привязан к proof через `recipient_binding = hash_2(noteSecret, recipient)`. Атакующий, не зная `noteSecret`, не может подставить свой recipient.

---

## Архитектура

```
┌──────────────────────────────────────────────────────────────────┐
│ FRONTEND (Vue 3 + Vite + Pinia) — порт 5173                      │
└──────────────────────────────────────────────────────────────────┘
              ↓ HTTP                    ↑ HTTP
┌──────────────────────────────────────────────────────────────────┐
│ BACKEND (Rust + axum) — порт 4001                                │
│ • /api/commitments  • /api/root  • /api/proof  • /api/withdraw   │
└──────────────────────────────────────────────────────────────────┘
       ↓                    ↓                    ↓
┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐
│ POSTGRES     │   │ REDIS        │   │ MERKLE (Node.js)     │
│  :5432       │   │  :6379       │   │  :4003               │
└──────────────┘   └──────────────┘   └──────────────────────┘
                                            ↑
┌──────────────────────────────────────────────────────────────────┐
│ PROVER (Rust + axum) — порт 4002                                 │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ SOLANA                                                           │
│ • Program zk_pool: C5Q4FjJ8zSD41FfMfcEccjnGnFJ7K44rDgnyrTx2fr6q │
│ • Verifier program: AukfYGQqhv3ZkdB1G7cKF3djsE9FKMrAfLd34doSWQex│
└──────────────────────────────────────────────────────────────────┘
```

Подробнее — [`docs/PROJECT_CONTEXT.md`](../PROJECT_CONTEXT.md).

---

## Быстрый старт

### Требования

- Linux (Debian 12+ / Ubuntu 22.04+)
- Docker + Docker Compose
- tmux
- make

Всё остальное (Rust, Anchor, Noir, Sunspot, Node.js, pnpm) работает **внутри** контейнера `solana`.

### Установка

```bash
git clone git@github.com:kwebhub/solana-zk-pool.git
cd solana-zk-pool

# 1. Запустить все сервисы (Docker + tmux)
make start

# 2. Проверить статус
make status

# 3. Открыть приложение
#    Frontend:   http://localhost:5173
#    Backend:    http://localhost:4001
#    Merkle:     http://localhost:4003
#    Prometheus: http://localhost:9090
#    Grafana:    http://localhost:3000 (admin/admin)
```

### Остановка

```bash
make stop
```

---

## Структура репозитория

```
solana-zk-pool/
├── .github/              ← CI/CD, шаблоны, Dependabot
├── Makefile              ← управление сервисами
├── docs/                 ← документация
├── infra/                ← Docker, Prometheus, Grafana
├── scripts/              ← Rust CLI (sync-circuits)
├── onchain/              ← Anchor программа
├── circuits/             ← Noir circuits
├── services/
│   ├── backend/          ← Rust + axum
│   ├── merkle/           ← Node.js + noir_js
│   └── prover/           ← Rust + Sunspot CLI
└── web/                  ← Vue 3 + Vite
```

Полная структура — в [`docs/PROJECT_CONTEXT.md`](../PROJECT_CONTEXT.md#4-repository-structure).

---

## Технологии

| Слой | Технология |
|------|-----------|
| Smart contract | Anchor (Rust) |
| ZK circuit | Noir |
| ZK proof | Sunspot → Groth16 (BN254) |
| On-chain verifier | gnark-solana |
| Backend | Rust + axum |
| DB | PostgreSQL 16 |
| Cache | Redis 7 |
| Frontend | Vue 3 + Vite + Pinia |
| Contracts client | Codama |
| Monitoring | Prometheus + Grafana |
| Container | Docker + Docker Compose |

---

## Команды

```bash
# Все сервисы
make start                         # запустить (tmux + docker)
make status                        # статус
make stop                          # остановить
make attach backend                # tmux-сессия backend
make logs backend                  # tail лога
make sync-circuits                 # компиляция circuits + sync JSON

# Программа
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

## Мониторинг

После `make start`:

- **Prometheus** (http://localhost:9090) — собирает метрики с `/metrics` каждые 15 сек.
- **Grafana** (http://localhost:3000) — дашборд `zkpool Overview` с 9 панелями.

**Метрики:**

| Метрика | Что показывает |
|---------|---------------|
| `axum_http_requests_total` | HTTP-запросы |
| `axum_http_requests_duration_seconds` | Латентность HTTP |
| `zkpool_indexer_deposits_total` | Обработано DepositEvent |
| `zkpool_indexer_withdrawals_total` | Обработано WithdrawEvent |
| `zkpool_indexer_lag_seconds` | Лаг от блокчейна |
| `zkpool_indexer_tick_duration_seconds` | Время одного тика индексера |
| `zkpool_indexer_tree_size` | Размер дерева |
| `zkpool_tree_add_leaf_duration_seconds` | Время добавления листа |
| `zkpool_tree_hash_duration_seconds` | Время Poseidon2 hash |
| `zkpool_tree_errors_total` | Ошибки дерева |
| `zkpool_db_errors_total` | Ошибки БД |

---

## Безопасность

- **Threat model:** [`docs/threat-model.md`](../threat-model.md)
- **Report vulnerabilities:** [`SECURITY.md`](../../SECURITY.md)

**Ключевые инварианты:**

1. Каждый `nullifier_hash` используется **ровно один раз**.
2. `root` должен быть в истории `PoolState.roots[10]`.
3. ZK-proof верифицируется **on-chain**.
4. `recipient` привязан к proof через `recipient_binding`.

**Известные ограничения:**

- **Trusted setup** — без MPC ceremony.
- **Upgrade authority** — один ключ.
- **Секреты в localStorage** — XSS-риск.
- **Deposit trust model** — программа НЕ проверяет `commitment` on-chain.

**Автоматические проверки:**

- `cargo-deny`, `cargo-audit`, `pnpm audit`, `trivy`, `gitleaks`.

---

## Документация

- [`docs/PROJECT_CONTEXT.md`](../PROJECT_CONTEXT.md) — полный контекст проекта.
- [`docs/threat-model.md`](../threat-model.md) — threat model.
- [`CONTRIBUTING.md`](../../CONTRIBUTING.md) — как контрибьютить.
- [`SECURITY.md`](../../SECURITY.md) — security policy.
- [`CHANGELOG.md`](../../CHANGELOG.md) — changelog.

**В работе:**

- `docs/architecture.md`
- `docs/zk-explained.md`
- `docs/deployment.md`

---

## Лицензия

MIT — см. [`LICENSE`](../../LICENSE).

**Источник:** [Solana Foundation Bootcamp 2026 — "05-private-transfers"](https://github.com/solana-foundation/solana-bootcamp-2026/tree/main/05-private-transfers).
**Предыдущая версия:** [kwebhub/private-transfer](https://github.com/kwebhub/private-transfer).

//! zk-pool backend library.
//!
//! This crate contains **all** the logic. `main.rs` is just bootstrap.
//! This allows writing unit and integration tests.
//!
//! ## Modules
//!
//! - [`config`] — configuration from env.
//! - [`db`] — Postgres (commitments, roots, nullifiers).
//! - [`cache`] — Redis (cache, Merkle tree, rate limit).
//! - [`tree`] — incremental Merkle Tree in Redis.
//! - [`indexer`] — background worker (events → DB → tree).
//! - [`logging`] — `tracing` initialization.
//! - [`metrics`] — Prometheus metrics.
//! - [`rate_limit`] — rate limiting middleware.

pub mod cache;
pub mod config;
pub mod db;
pub mod indexer;
pub mod logging;
pub mod metrics;
pub mod rate_limit;
pub mod tree;

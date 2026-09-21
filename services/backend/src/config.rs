//! Backend configuration.
//!
//! All values are read from environment variables.
//! If a variable is missing and has no default — the app fails at startup.

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    // ============ Required ============
    pub database_url: String,
    pub pool_address: String,

    // ============ With defaults ============
    pub redis_url: String,
    pub port: u16,
    pub prover_url: String,
    pub merkle_url: String,
    pub solana_rpc_url: String,

    // ============ Tuning ============
    pub indexer_poll_interval_secs: u64,
    pub indexer_page_size: usize,
    pub indexer_max_pages: usize,
    pub rate_limit_withdraw_per_min: u64,
    pub rate_limit_read_per_min: u64,
    pub merkle_tree_depth: u32,
}

impl Config {
    /// Loads configuration from environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            pool_address: required("POOL_ADDRESS")?,
            redis_url: optional("REDIS_URL", "redis://localhost:6379"),
            port: parse("PORT", 4001u16)?,
            prover_url: optional("PROVER_URL", "http://localhost:4002"),
            merkle_url: optional("MERKLE_URL", "http://localhost:4003"),
            solana_rpc_url: optional("SOLANA_RPC_URL", "https://api.devnet.solana.com"),
            indexer_poll_interval_secs: parse("INDEXER_POLL_INTERVAL_SECS", 5u64)?,
            indexer_page_size: parse("INDEXER_PAGE_SIZE", 100usize)?,
            indexer_max_pages: parse("INDEXER_MAX_PAGES", 10usize)?,
            rate_limit_withdraw_per_min: parse("RATE_LIMIT_WITHDRAW_PER_MIN", 5u64)?,
            rate_limit_read_per_min: parse("RATE_LIMIT_READ_PER_MIN", 60u64)?,
            merkle_tree_depth: parse("MERKLE_TREE_DEPTH", 20u32)?,
        })
    }
}

/// Configuration error.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    Missing(String),

    #[error("Invalid value for {name}: {value} (expected {expected})")]
    Invalid {
        name: String,
        value: String,
        expected: String,
    },
}

/// Reads a required variable. Fails if missing.
fn required(name: &str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::Missing(name.to_string()))
}

/// Reads a variable or uses a default.
fn optional(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

/// Reads a variable and parses it. Uses default if missing.
fn parse<T>(name: &str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr + std::fmt::Display + Copy,
    T::Err: std::fmt::Display,
{
    match env::var(name) {
        Ok(v) => v.parse::<T>().map_err(|e| ConfigError::Invalid {
            name: name.to_string(),
            value: v.clone(),
            expected: format!("{} ({})", std::any::type_name::<T>(), e),
        }),
        Err(_) => Ok(default),
    }
}

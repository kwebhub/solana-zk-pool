//! Custom metrics for zk-pool backend.
//!
//! HTTP metrics (requests, latency, statuses) are collected automatically
//! via `axum-prometheus` middleware.

use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};
use std::sync::OnceLock;

static INIT: OnceLock<()> = OnceLock::new();

/// Registers all metrics at startup. Called **once**.
pub fn init_metrics() {
    INIT.get_or_init(|| {
        // ============ Indexer ============
        describe_counter!(
            "zkpool_indexer_deposits_total",
            "Total number of DepositEvent processed"
        );
        describe_counter!(
            "zkpool_indexer_withdrawals_total",
            "Total number of WithdrawEvent processed"
        );
        describe_counter!(
            "zkpool_indexer_errors_total",
            "Total number of indexer errors"
        );
        describe_gauge!(
            "zkpool_indexer_lag_seconds",
            "Seconds behind the latest block"
        );
        describe_gauge!(
            "zkpool_indexer_tree_size",
            "Current number of leaves in Merkle tree"
        );
        describe_histogram!(
            "zkpool_indexer_tick_duration_seconds",
            "Duration of one indexer tick"
        );

        // ============ Merkle Tree ============
        describe_histogram!(
            "zkpool_tree_add_leaf_duration_seconds",
            "Time to add one leaf to Merkle tree"
        );
        describe_histogram!(
            "zkpool_tree_hash_duration_seconds",
            "Time to compute one Poseidon2 hash"
        );
        describe_counter!("zkpool_tree_add_leaf_total", "Total number of leaves added");
        describe_counter!(
            "zkpool_tree_errors_total",
            "Total number of Merkle tree errors"
        );

        // ============ DB ============
        describe_counter!("zkpool_db_errors_total", "Total number of database errors");
    });
}

// ============================================================
// Indexer
// ============================================================

pub fn record_deposit(leaf_index: i64) {
    counter!("zkpool_indexer_deposits_total").increment(1);
    gauge!("zkpool_indexer_tree_size").set(leaf_index as f64 + 1.0);
}

pub fn record_withdrawal() {
    counter!("zkpool_indexer_withdrawals_total").increment(1);
}

pub fn record_indexer_error() {
    counter!("zkpool_indexer_errors_total").increment(1);
}

pub fn record_indexer_lag(seconds: f64) {
    gauge!("zkpool_indexer_lag_seconds").set(seconds);
}

pub fn observe_indexer_tick(duration_secs: f64) {
    histogram!("zkpool_indexer_tick_duration_seconds").record(duration_secs);
}

// ============================================================
// Merkle Tree
// ============================================================

pub fn observe_add_leaf(duration_secs: f64) {
    histogram!("zkpool_tree_add_leaf_duration_seconds").record(duration_secs);
    counter!("zkpool_tree_add_leaf_total").increment(1);
}

pub fn observe_tree_hash(duration_secs: f64) {
    histogram!("zkpool_tree_hash_duration_seconds").record(duration_secs);
}

pub fn record_tree_error() {
    counter!("zkpool_tree_errors_total").increment(1);
}

// ============================================================
// DB
// ============================================================

pub fn record_db_error() {
    counter!("zkpool_db_errors_total").increment(1);
}

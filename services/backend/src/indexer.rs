//! Indexer: listens to `DepositEvent` and `WithdrawEvent`,
//! writes to Postgres, updates Merkle Tree in Redis.
//!
//! ## How it works
//!
//! 1. Every `INDEXER_POLL_INTERVAL_SECS` seconds calls `getSignaturesForAddress`.
//! 2. Paginates (`before`) until `last_signature` (in Redis).
//! 3. Reverses — from old to new.
//! 4. For each tx parses logs by discriminator.
//! 5. Deduplication: `commitment_exists` / `is_nullifier_used`.
//! 6. On new deposit — `tree::add_leaf` updates the tree.
//! 7. Saves `last_signature`.

use crate::cache::Cache;
use crate::config::Config;
use crate::db::Db;
use crate::metrics;
use crate::tree;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use tracing::{debug, error, info, warn};

const DEPOSIT_DISC: [u8; 8] = [120, 248, 61, 83, 31, 142, 107, 144];
const WITHDRAW_DISC: [u8; 8] = [22, 9, 133, 26, 160, 44, 71, 192];

pub struct Indexer {
    db: Arc<Db>,
    cache: Arc<Cache>,
    config: Arc<Config>,
}

impl Indexer {
    pub fn new(db: Arc<Db>, cache: Arc<Cache>, config: Arc<Config>) -> Self {
        Self { db, cache, config }
    }

    pub async fn run(self) {
        info!(pool = %self.config.pool_address, "🔄 Indexer started");

        loop {
            let start = Instant::now();
            if let Err(e) = self.tick().await {
                error!(error = %e, "indexer tick error");
                metrics::record_indexer_error();
            }
            metrics::observe_indexer_tick(start.elapsed().as_secs_f64());
            sleep(Duration::from_secs(self.config.indexer_poll_interval_secs)).await;
        }
    }

    async fn tick(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let last_processed = self
            .cache
            .get_last_signature(&self.config.pool_address)
            .await
            .unwrap_or(None);

        let mut before: Option<String> = None;
        let mut all_sigs: Vec<serde_json::Value> = Vec::new();
        let mut reached_last = false;

        for _page in 0..self.config.indexer_max_pages {
            let mut params = serde_json::json!([
                self.config.pool_address,
                { "limit": self.config.indexer_page_size }
            ]);
            if let Some(b) = &before {
                params[1]["before"] = serde_json::json!(b);
            }

            let sigs_resp: serde_json::Value = client
                .post(&self.config.solana_rpc_url)
                .json(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "getSignaturesForAddress",
                    "params": params
                }))
                .send()
                .await?
                .json()
                .await?;

            let page_sigs = sigs_resp["result"].as_array().cloned().unwrap_or_default();

            if page_sigs.is_empty() {
                break;
            }

            let mut stop_pagination = false;

            for sig_entry in &page_sigs {
                let signature = match sig_entry["signature"].as_str() {
                    Some(s) => s,
                    None => continue,
                };

                if let Some(last) = &last_processed {
                    if signature == last {
                        reached_last = true;
                        stop_pagination = true;
                        break;
                    }
                }

                all_sigs.push(sig_entry.clone());
            }

            if stop_pagination {
                break;
            }

            if let Some(last_sig) = page_sigs.last() {
                before = last_sig["signature"].as_str().map(|s| s.to_string());
            } else {
                break;
            }
        }

        if let Some(first) = all_sigs.first() {
            if let Some(block_time) = first["blockTime"].as_i64() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let lag = (now - block_time).max(0) as f64;
                metrics::record_indexer_lag(lag);
            }
        }

        if all_sigs.is_empty() {
            return Ok(());
        }

        all_sigs.reverse();

        let mut newest_signature: Option<String> = None;

        for sig_entry in &all_sigs {
            let signature = match sig_entry["signature"].as_str() {
                Some(s) => s,
                None => continue,
            };

            if let Err(e) = self.process_transaction(&client, signature).await {
                warn!(signature = %signature, error = %e, "failed to process transaction");
                metrics::record_indexer_error();
                continue;
            }

            newest_signature = Some(signature.to_string());
        }

        if let Some(newest) = newest_signature {
            let _ = self
                .cache
                .set_last_signature(&self.config.pool_address, &newest)
                .await;

            if reached_last {
                debug!(signature = %&newest[..8], "indexer caught up");
            } else {
                info!(
                    count = all_sigs.len(),
                    signature = %&newest[..8],
                    "indexer processed new transactions"
                );
            }
        }

        Ok(())
    }

    async fn process_transaction(
        &self,
        client: &reqwest::Client,
        signature: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tx_resp: serde_json::Value = client
            .post(&self.config.solana_rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getTransaction",
                "params": [signature, {
                    "encoding": "json",
                    "maxSupportedTransactionVersion": 0
                }]
            }))
            .send()
            .await?
            .json()
            .await?;

        let logs = match tx_resp["result"]["meta"]["logMessages"].as_array() {
            Some(l) => l,
            None => {
                debug!(signature = %signature, "no logs in transaction");
                return Ok(());
            }
        };

        debug!(signature = %signature, logs_count = logs.len(), "processing transaction");

        for log in logs {
            let log_str = match log.as_str() {
                Some(s) => s,
                None => continue,
            };

            if !log_str.starts_with("Program data:") {
                continue;
            }

            debug!(signature = %signature, log = %log_str, "found Program data log");

            let b64: String = log_str
                .strip_prefix("Program data: ")
                .unwrap_or("")
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();

            debug!(signature = %signature, b64_len = b64.len(), "base64 extracted");

            let bytes =
                match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &b64) {
                    Ok(b) => b,
                    Err(e) => {
                        debug!(signature = %signature, error = %e, "base64 decode failed");
                        continue;
                    }
                };

            debug!(signature = %signature, bytes_len = bytes.len(), "base64 decoded");

            if bytes.len() < 8 {
                debug!(signature = %signature, "bytes too short");
                continue;
            }

            let disc: [u8; 8] = bytes[0..8].try_into().unwrap();
            debug!(signature = %signature, disc = ?disc, "discriminator extracted");

            // DepositEvent: 88 bytes
            if disc == DEPOSIT_DISC && bytes.len() == 88 {
                let commitment = &bytes[8..40];
                let leaf_index = i64::from_le_bytes(bytes[40..48].try_into().unwrap());
                let new_root = &bytes[56..88];
                let commitment_hex = hex::encode(commitment);

                debug!(signature = %signature, leaf_index = leaf_index, "deposit event found");

                if !self.db.commitment_exists(leaf_index).await? {
                    self.db
                        .save_deposit(
                            leaf_index,
                            commitment,
                            new_root,
                            &self.config.pool_address,
                            signature,
                        )
                        .await?;

                    match tree::add_leaf(
                        &self.cache,
                        &self.config.merkle_url,
                        &self.config.pool_address,
                        leaf_index as usize,
                        &commitment_hex,
                        self.config.merkle_tree_depth,
                    )
                    .await
                    {
                        Ok(root) => {
                            info!(
                                leaf = leaf_index,
                                commitment = %commitment_hex,
                                root = %root,
                                "📥 Indexed deposit"
                            );
                            metrics::record_deposit(leaf_index);
                        }
                        Err(e) => {
                            error!(error = %e, "failed to add leaf to tree");
                            metrics::record_tree_error();
                        }
                    }

                    let _ = self.cache.invalidate_pool(&self.config.pool_address).await;
                } else {
                    debug!(signature = %signature, leaf_index = leaf_index, "deposit already exists");
                }
            }

            // WithdrawEvent: 88 bytes
            if disc == WITHDRAW_DISC && bytes.len() == 88 {
                let nullifier_hash = &bytes[8..40];
                let recipient = &bytes[40..72];
                let amount = i64::from_le_bytes(bytes[72..80].try_into().unwrap());

                debug!(signature = %signature, nullifier = %hex::encode(nullifier_hash), "withdraw event found");

                if !self.db.is_nullifier_used(nullifier_hash).await? {
                    self.db
                        .save_nullifier(
                            nullifier_hash,
                            &self.config.pool_address,
                            &bs58::encode(recipient).into_string(),
                            amount,
                            signature,
                        )
                        .await?;

                    info!(
                        nullifier = %hex::encode(nullifier_hash),
                        amount = amount,
                        "📤 Indexed withdraw"
                    );
                    metrics::record_withdrawal();
                } else {
                    debug!(signature = %signature, "withdraw already exists");
                }
            }
        }

        Ok(())
    }
}

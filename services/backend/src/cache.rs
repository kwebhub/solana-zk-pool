//! Redis wrapper.
//!
//! Used for:
//! - **Commitments cache** — `/api/commitments` (TTL 30 sec).
//! - **Merkle Tree** — levels, empty hashes, root, size.
//! - **Indexer** — last processed signature.
//! - **Rate limiting** — request counters by IP.

use redis::aio::ConnectionManager;
use redis::AsyncCommands;

/// Wrapper over Redis `ConnectionManager`.
#[derive(Clone)]
pub struct Cache {
    conn: ConnectionManager,
}

impl Cache {
    /// Creates a Redis connection.
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    /// Sets a value with TTL (in seconds).
    pub async fn set_ex(
        &self,
        key: &str,
        value: &str,
        ttl_secs: u64,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        conn.set_ex::<_, _, ()>(key, value, ttl_secs).await
    }

    /// Sets a value without TTL.
    pub async fn set(&self, key: &str, value: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        conn.set::<_, _, ()>(key, value).await
    }

    /// Reads a value. Returns `None` if the key doesn't exist.
    pub async fn get(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.conn.clone();
        conn.get(key).await
    }

    /// Invalidates pool cache: `commitments:{pool}` and `root:{pool}`.
    pub async fn invalidate_pool(&self, pool_address: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        let keys = vec![
            format!("commitments:{}", pool_address),
            format!("root:{}", pool_address),
        ];
        conn.del::<_, ()>(keys).await
    }

    // ============================================================
    // Merkle Tree
    // ============================================================

    /// Returns tree level (`tree:{pool}:level:{d}`) as an array of hex hashes.
    pub async fn get_tree_level(
        &self,
        pool_address: &str,
        level: u32,
    ) -> Result<Option<Vec<String>>, redis::RedisError> {
        let key = format!("tree:{}:level:{}", pool_address, level);
        match self.get(&key).await? {
            Some(json) => {
                let parsed: Vec<String> = serde_json::from_str(&json).unwrap_or_default();
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Writes a tree level.
    pub async fn set_tree_level(
        &self,
        pool_address: &str,
        level: u32,
        hashes: &[String],
    ) -> Result<(), redis::RedisError> {
        let key = format!("tree:{}:level:{}", pool_address, level);
        let json = serde_json::to_string(hashes).unwrap_or_else(|_| "[]".to_string());
        self.set(&key, &json).await
    }

    /// Returns current root (`tree:{pool}:root`).
    pub async fn get_tree_root(
        &self,
        pool_address: &str,
    ) -> Result<Option<String>, redis::RedisError> {
        let key = format!("tree:{}:root", pool_address);
        self.get(&key).await
    }

    /// Writes current root.
    pub async fn set_tree_root(
        &self,
        pool_address: &str,
        root: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("tree:{}:root", pool_address);
        self.set(&key, root).await
    }

    /// Returns number of leaves (`tree:{pool}:size`).
    pub async fn get_tree_size(
        &self,
        pool_address: &str,
    ) -> Result<Option<usize>, redis::RedisError> {
        let key = format!("tree:{}:size", pool_address);
        match self.get(&key).await? {
            Some(s) => Ok(s.parse().ok()),
            None => Ok(None),
        }
    }

    /// Writes number of leaves.
    pub async fn set_tree_size(
        &self,
        pool_address: &str,
        size: usize,
    ) -> Result<(), redis::RedisError> {
        let key = format!("tree:{}:size", pool_address);
        self.set(&key, &size.to_string()).await
    }

    /// Returns "empty" hash for a level (`tree:{pool}:empty:{d}`).
    pub async fn get_empty_hash(
        &self,
        pool_address: &str,
        level: u32,
    ) -> Result<Option<String>, redis::RedisError> {
        let key = format!("tree:{}:empty:{}", pool_address, level);
        self.get(&key).await
    }

    /// Writes "empty" hash for a level.
    pub async fn set_empty_hash(
        &self,
        pool_address: &str,
        level: u32,
        hash: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("tree:{}:empty:{}", pool_address, level);
        self.set(&key, hash).await
    }

    // ============================================================
    // Indexer
    // ============================================================

    /// Returns last processed signature (`indexer:{pool}:last_signature`).
    pub async fn get_last_signature(
        &self,
        pool_address: &str,
    ) -> Result<Option<String>, redis::RedisError> {
        let key = format!("indexer:{}:last_signature", pool_address);
        self.get(&key).await
    }

    /// Writes last processed signature.
    pub async fn set_last_signature(
        &self,
        pool_address: &str,
        signature: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("indexer:{}:last_signature", pool_address);
        self.set(&key, signature).await
    }

    // ============================================================
    // Rate limiting
    // ============================================================

    /// Increments a counter and returns the new value.
    ///
    /// TTL is set **only** on the first request (`INCR` returned 1).
    pub async fn incr_rate_limit(
        &self,
        key: &str,
        ttl_secs: u64,
    ) -> Result<u64, redis::RedisError> {
        let mut conn = self.conn.clone();
        let count: u64 = conn.incr(key, 1u64).await?;
        if count == 1 {
            conn.expire::<_, ()>(key, ttl_secs as i64).await?;
        }
        Ok(count)
    }
}

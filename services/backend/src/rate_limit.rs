//! Rate limiting middleware for axum.
//!
//! Limits are read from `Config`:
//! - `RATE_LIMIT_WITHDRAW_PER_MIN` (default 5) — for `/api/withdraw`.
//! - `RATE_LIMIT_READ_PER_MIN` (default 60) — for reads.
//!
//! ## How it works
//!
//! Counters are stored in Redis: `ratelimit:{endpoint}:{ip}` with TTL 60s.
//! If the limit is exceeded — returns `429 Too Many Requests`.

use crate::cache::Cache;
use crate::config::Config;
use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::{Response, StatusCode},
    middleware::Next,
};
use std::net::SocketAddr;
use std::sync::Arc;

fn withdraw_limit(config: &Config) -> u64 {
    config.rate_limit_withdraw_per_min
}

fn read_limit(config: &Config) -> u64 {
    config.rate_limit_read_per_min
}

/// Middleware: rate limiting for `/api/withdraw`.
pub async fn rate_limit_withdraw(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let cache = request
        .extensions()
        .get::<Arc<Cache>>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let config = request
        .extensions()
        .get::<Arc<Config>>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let key = format!("ratelimit:withdraw:{}", addr.ip());

    let count = cache
        .incr_rate_limit(&key, 60)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if count > withdraw_limit(&config) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

/// Middleware: rate limiting for reads (`/api/commitments`, `/api/root`, `/api/proof`).
pub async fn rate_limit_read(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let cache = request
        .extensions()
        .get::<Arc<Cache>>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let config = request
        .extensions()
        .get::<Arc<Config>>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let key = format!("ratelimit:read:{}", addr.ip());

    let count = cache
        .incr_rate_limit(&key, 60)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if count > read_limit(&config) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

//! `tracing` initialization for the backend.
//!
//! In dev — human-readable format with colors.
//! In production (`APP_ENV=production`) — JSON format (for ELK/Loki).
//!
//! Level is set via `RUST_LOG` (default `info`).
//! Examples:
//! - `RUST_LOG=info` — only INFO and above.
//! - `RUST_LOG=debug` — everything, including DEBUG.
//! - `RUST_LOG=zkpool_backend=debug,info` — DEBUG only for our crate.

use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

/// Initializes the global tracing subscriber.
///
/// Called **once** in `main()` before anything else.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let is_production = std::env::var("APP_ENV")
        .map(|v| v == "production")
        .unwrap_or(false);

    if is_production {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .json()
                    .with_target(true)
                    .with_current_span(true)
                    .with_span_list(true),
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_file(false)
                    .with_line_number(false),
            )
            .init();
    }
}

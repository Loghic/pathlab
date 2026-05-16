//! Cross-platform monotonic clock.
//!
//! `std::time::Instant` panics on wasm32 because the browser does not
//! expose a native monotonic clock through the standard library. The
//! [`web-time`](https://docs.rs/web-time) crate provides a drop-in
//! `Instant` that:
//!
//! - On native targets is exactly `std::time::Instant`.
//! - On wasm32 is backed by `performance.now()`, which is monotonic and
//!   has sub-millisecond precision.
//!
//! Re-exporting it from this module keeps the call sites identical
//! across platforms.

pub use web_time::{Duration, Instant};

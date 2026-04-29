//! LimeTime: Lightweight, deterministic time tracking for forensic audio.
//!
//! "Time is a dimension, not a sequence of strings."

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A lightweight, 64-bit nanosecond timestamp since Unix Epoch.
/// Replaces the heavy 'chrono' crate for internal audit logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Returns the current system time as a nanosecond timestamp.
    pub fn now() -> Self {
        let dur = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self(dur.as_nanos() as u64)
    }

    /// Creates a timestamp from raw nanoseconds.
    pub fn from_nanos(nanos: u64) -> Self {
        Self(nanos)
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let secs = self.0 / 1_000_000_000;
        let nanos = self.0 % 1_000_000_000;
        let ms = nanos / 1_000_000;
        write!(f, "{}.{:03}", secs, ms)
    }
}

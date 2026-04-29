//! Trust Ledger — The Accountability Record.
//!
//! "Badge は証拠の提示であり、Ledger は証拠の履歴である。"

use crate::ui_ir::ProvenanceLevel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustLedger {
    pub entries: Vec<LedgerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub timestamp: std::time::SystemTime,
    pub author: String, // "System", "User", "Compiler", or "Agent"
    pub why: String,
    pub source_hash: [u8; 32],
    pub previous_level: ProvenanceLevel,
    pub new_level: ProvenanceLevel,
}

impl Default for TrustLedger {
    fn default() -> Self {
        Self::new()
    }
}
impl TrustLedger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn record(
        &mut self,
        author: &str,
        why: &str,
        hash: [u8; 32],
        prev: ProvenanceLevel,
        next: ProvenanceLevel,
    ) {
        self.entries.push(LedgerEntry {
            timestamp: std::time::SystemTime::now(),
            author: author.to_string(),
            why: why.to_string(),
            source_hash: hash,
            previous_level: prev,
            new_level: next,
        });
    }

    /// 検証可能性の要約を生成
    pub fn summarize_downgrades(&self) -> Vec<&LedgerEntry> {
        self.entries
            .iter()
            .filter(|e| self.is_downgrade(e.previous_level, e.new_level))
            .collect()
    }

    fn is_downgrade(&self, prev: ProvenanceLevel, next: ProvenanceLevel) -> bool {
        let rank = |l| match l {
            ProvenanceLevel::Verified => 3,
            ProvenanceLevel::Inferred => 2,
            ProvenanceLevel::Stale => 1,
            ProvenanceLevel::External => 0,
        };
        rank(next) < rank(prev)
    }
}

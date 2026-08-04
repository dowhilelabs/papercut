//! Record shapes for the JSONL journal.
//!
//! Every line is one JSON object. A `cut` records a complaint; a `resolved`
//! records that a complaint was fixed. The log is an append-only journal —
//! nothing is ever rewritten or deleted.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A single papercut complaint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cut {
    pub id: String,
    pub ts: String,
    pub model: Option<String>,
    pub user: Option<String>,
    pub text: String,
}

/// A resolution event appended when a papercut is marked fixed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolved {
    pub id: String,
    pub ts: String,
    pub by: Option<String>,
    /// Optional note about how it was fixed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// The tagged union stored on each JSONL line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Record {
    Cut(Cut),
    Resolved(Resolved),
}

impl Record {
    /// The id referenced by this record.
    pub fn id(&self) -> &str {
        match self {
            Record::Cut(c) => &c.id,
            Record::Resolved(r) => &r.id,
        }
    }
}

/// Deterministic, content-addressed id: `pc_` + first 10 hex of SHA-256 over the
/// complaint text. The same complaint filed twice dedups to the same id.
pub fn make_cut_id(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let digest = hasher.finalize();
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    format!("pc_{}", &hex[..10])
}

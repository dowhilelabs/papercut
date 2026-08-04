//! Record shapes for the JSONL journal.
//!
//! Every line is one JSON object. A `cut` records a complaint; a `resolved`
//! records that a complaint was fixed. The log is an append-only journal —
//! nothing is ever rewritten or deleted.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Severity of a papercut. Defaults to `minor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Minor,
    Major,
    Blocker,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Severity::Minor => "minor",
                Severity::Major => "major",
                Severity::Blocker => "blocker",
            }
        )
    }
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "minor" => Ok(Severity::Minor),
            "major" => Ok(Severity::Major),
            "blocker" => Ok(Severity::Blocker),
            other => Err(format!(
                "invalid severity '{other}' (expected minor | major | blocker)"
            )),
        }
    }
}

/// A single papercut complaint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cut {
    pub id: String,
    pub ts: String,
    pub model: Option<String>,
    pub harness: Option<String>,
    pub user: Option<String>,
    pub text: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_severity")]
    pub severity: Severity,
}

fn default_severity() -> Severity {
    Severity::Minor
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
/// substantive content (text + sorted tags + severity). The same complaint filed
/// by two agents dedups to the same id.
pub fn make_cut_id(text: &str, tags: &[String], severity: Severity) -> String {
    let mut sorted = tags.to_vec();
    sorted.sort();
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.update(b"\x00");
    for t in &sorted {
        hasher.update(t.as_bytes());
        hasher.update(b"\x00");
    }
    hasher.update(severity.to_string().as_bytes());
    let digest = hasher.finalize();
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    format!("pc_{}", &hex[..10])
}

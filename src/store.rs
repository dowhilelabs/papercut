//! Append-only JSONL storage with advisory locking, atomic appends, duplicate
//! suppression, and torn-line healing.

use crate::error::{self, Error};
use crate::record::Record;
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Result of reading the journal.
pub struct ReadResult {
    pub records: Vec<Record>,
    /// Count of malformed (torn) lines skipped during healing.
    pub torn: usize,
}

const DEFAULT_FILE_NAME: &str = ".papercuts.jsonl";
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(50);

pub struct Store {
    pub path: PathBuf,
}

impl Store {
    /// Resolve the journal path: `PAPERCUTS_FILE` > git repo root > `~/.papercuts/log.jsonl`.
    pub fn resolve() -> Result<Store, Error> {
        if let Some(p) = std::env::var("PAPERCUTS_FILE").ok().filter(|v| !v.is_empty()) {
            return Ok(Store {
                path: PathBuf::from(p),
            });
        }
        if let Some(root) = git_root() {
            return Ok(Store {
                path: root.join(DEFAULT_FILE_NAME),
            });
        }
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| error::config("cannot determine home directory"))?;
        let base = PathBuf::from(home).join(".papercuts");
        let _ = std::fs::create_dir_all(&base).map_err(Error::Io)?;
        Ok(Store {
            path: base.join("log.jsonl"),
        })
    }

    /// Read all records under a shared lock, healing torn lines.
    pub fn read(&self) -> Result<ReadResult, Error> {
        if !self.path.exists() {
            return Ok(ReadResult {
                records: Vec::new(),
                torn: 0,
            });
        }
        let file = File::open(&self.path)?;
        file.lock_shared().map_err(|_| lock_err(&self.path))?;
        read_unlocked(&self.path)
    }

    /// Append a record under an exclusive lock, returning `true` if the journal changed.
    pub fn append(&self, record: &Record) -> Result<bool, Error> {
        // Ensure parent dir exists (e.g. explicit PAPERCUTS_FILE path).
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(Error::Io)?;
            }
        }
        let mut file = OpenOptions::new().create(true).append(true).open(&self.path)?;
        acquire_lock(&file, &self.path)?;

        // Duplicate suppression: an identical *cut* (same content-addressed id) is a
        // no-op (first-wins). Resolve events always append — the caller handles
        // idempotent resolves. We already hold the exclusive lock, so read without
        // re-locking to avoid a self-deadlock on a second file descriptor.
        if matches!(record, Record::Cut(_)) {
            let existing = read_unlocked(&self.path)?;
            if existing
                .records
                .iter()
                .any(|r| matches!(r, Record::Cut(c) if c.id == record.id()))
            {
                return Ok(false);
            }
        }

        let mut json = serde_json::to_string(record).map_err(|e| error::internal(e.to_string()))?;
        json.push('\n');
        file.write_all(json.as_bytes()).map_err(Error::Io)?;
        file.sync_all().map_err(Error::Io)?;
        Ok(true)
    }

    /// Permanently delete every record with the given id by rewriting the journal
    /// without those lines (torn lines are preserved). Returns how many records
    /// were removed. This is the one intentionally destructive operation — the
    /// rest of the journal stays append-only.
    pub fn remove(&self, id: &str) -> Result<usize, Error> {
        if !self.path.exists() {
            return Ok(0);
        }
        let mut file = OpenOptions::new().read(true).write(true).open(&self.path)?;
        acquire_lock(&file, &self.path)?;

        let content = std::io::read_to_string(BufReader::new(&file)).map_err(Error::Io)?;

        let mut kept = String::new();
        let mut removed = 0usize;
        for line in content.lines() {
            let trimmed = line.trim();
            let drop = !trimmed.is_empty()
                && matches!(serde_json::from_str::<Record>(trimmed), Ok(rec) if rec.id() == id);
            if drop {
                removed += 1;
            } else {
                kept.push_str(line);
                kept.push('\n');
            }
        }

        if removed > 0 {
            file.set_len(0).map_err(Error::Io)?;
            file.seek(SeekFrom::Start(0)).map_err(Error::Io)?;
            file.write_all(kept.as_bytes()).map_err(Error::Io)?;
            file.sync_all().map_err(Error::Io)?;
        }
        Ok(removed)
    }
}

/// Parse the journal without acquiring any lock. Callers must hold the lock when
/// appropriate (the append path already holds the exclusive lock).
fn read_unlocked(path: &Path) -> Result<ReadResult, Error> {
    if !path.exists() {
        return Ok(ReadResult {
            records: Vec::new(),
            torn: 0,
        });
    }
    let file = File::open(path)?;
    let mut records = Vec::new();
    let mut torn = 0usize;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.map_err(Error::Io)?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<Record>(trimmed) {
            Ok(rec) => records.push(rec),
            Err(_) => torn += 1, // torn/partial write: heal by skipping
        }
    }
    Ok(ReadResult { records, torn })
}

fn acquire_lock(file: &File, path: &Path) -> Result<(), Error> {
    let start = Instant::now();
    loop {
        match file.try_lock_exclusive() {
            Ok(()) => return Ok(()),
            Err(_) => {
                if start.elapsed() > Duration::from_secs(2) {
                    return Err(lock_err(path));
                }
                std::thread::sleep(LOCK_RETRY_DELAY);
            }
        }
    }
}

fn lock_err(path: &Path) -> Error {
    Error::LockTimeout(path.to_path_buf())
}

/// Run `git rev-parse --show-toplevel` to find the repo root, if inside one.
fn git_root() -> Option<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let root = String::from_utf8(out.stdout).ok()?;
    let root = root.trim();
    if root.is_empty() {
        None
    } else {
        Some(PathBuf::from(root))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{Cut};

    fn store_in(dir: &Path) -> Store {
        Store {
            path: dir.join("log.jsonl"),
        }
    }

    fn cut(id: &str, text: &str) -> Record {
        Record::Cut(Cut {
            id: id.into(),
            ts: "2026-01-01T00:00:00.000Z".into(),
            model: None,
            user: None,
            text: text.into(),
        })
    }

    #[test]
    fn append_and_read_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let s = store_in(dir.path());
        assert!(s.append(&cut("pc_a", "hello")).unwrap());
        assert!(!s.append(&cut("pc_a", "hello")).unwrap(), "duplicate suppressed");
        assert!(s.append(&cut("pc_b", "world")).unwrap());
        let res = s.read().unwrap();
        assert_eq!(res.records.len(), 2);
        assert_eq!(res.torn, 0);
        assert_eq!(res.records[0].id(), "pc_a");
        assert_eq!(res.records[1].id(), "pc_b");
    }

    #[test]
    fn missing_file_reads_empty() {
        let dir = tempfile::tempdir().unwrap();
        let s = store_in(dir.path());
        let res = s.read().unwrap();
        assert!(res.records.is_empty());
    }

    #[test]
    fn remove_drops_matching_records_only() {
        let dir = tempfile::tempdir().unwrap();
        let s = store_in(dir.path());
        s.append(&cut("pc_a", "one")).unwrap();
        s.append(&cut("pc_b", "two")).unwrap();
        s.append(&crate::record::Record::Resolved(crate::record::Resolved {
            id: "pc_a".into(),
            ts: "2026-01-01T00:00:00.000Z".into(),
            by: None,
            note: None,
        }))
        .unwrap();

        // Removing pc_a also removes its resolution event, not pc_b.
        assert_eq!(s.remove("pc_a").unwrap(), 2);
        let res = s.read().unwrap();
        assert_eq!(res.records.len(), 1);
        assert_eq!(res.records[0].id(), "pc_b");

        // Missing id is a no-op.
        assert_eq!(s.remove("pc_missing").unwrap(), 0);
    }
}

//! Structured error type, stable codes, documented exit codes.
//!
//! Every failure is rendered to stderr as a single JSON object with an `ok: false`
//! envelope, a stable machine-readable `code`, a documented `exit` status, and a
//! paste-ready `suggested_fix` so an agent can self-correct.

use serde::Serialize;

/// Documented process exit codes. These are stable and part of the machine
/// contract exposed via `papercuts schema`.
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_BAD_INPUT: i32 = 65; // sysexits EX_DATAERR
pub const EXIT_NOT_FOUND: i32 = 66; // sysexits EX_NOINPUT
pub const EXIT_INTERNAL: i32 = 70; // sysexits EX_SOFTWARE
pub const EXIT_IO: i32 = 74; // sysexits EX_IOERR
pub const EXIT_LOCK_TIMEOUT: i32 = 75; // retryable
pub const EXIT_PERM_DENIED: i32 = 77; // sysexits EX_NOPERM
pub const EXIT_CONFIG: i32 = 78; // sysexits EX_CONFIG

/// The canonical error type for every command.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Usage(String),

    #[error("{0}")]
    BadInput(String),

    #[error("record {0} not found")]
    NotFound(String),

    #[error("{0}")]
    Internal(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("timed out acquiring lock on {0} (another writer holds it)")]
    LockTimeout(std::path::PathBuf),

    #[error("{0}")]
    #[allow(dead_code)] // part of the documented error contract
    PermDenied(String),

    #[error("{0}")]
    Config(String),
}

impl Error {
    /// Stable machine-readable code for this error.
    pub fn code(&self) -> &'static str {
        match self {
            Error::Usage(_) => "E_USAGE",
            Error::BadInput(_) => "E_BAD_INPUT",
            Error::NotFound(_) => "E_NOT_FOUND",
            Error::Internal(_) => "E_INTERNAL",
            Error::Io(_) => "E_IO",
            Error::LockTimeout(_) => "E_LOCK_TIMEOUT",
            Error::PermDenied(_) => "E_PERM_DENIED",
            Error::Config(_) => "E_CONFIG",
        }
    }

    /// Documented exit status for this error.
    pub fn exit(&self) -> i32 {
        match self {
            Error::Usage(_) => EXIT_USAGE,
            Error::BadInput(_) => EXIT_BAD_INPUT,
            Error::NotFound(_) => EXIT_NOT_FOUND,
            Error::Internal(_) => EXIT_INTERNAL,
            Error::Io(_) => EXIT_IO,
            Error::LockTimeout(_) => EXIT_LOCK_TIMEOUT,
            Error::PermDenied(_) => EXIT_PERM_DENIED,
            Error::Config(_) => EXIT_CONFIG,
        }
    }

    /// Human guidance an agent can act on to recover, without help.
    pub fn suggested_fix(&self) -> String {
        match self {
            Error::Usage(_) => "Check `papercuts --help` for the correct arguments and flags.".into(),
            Error::BadInput(_) => "The provided input was not usable. Fix the argument and retry.".into(),
            Error::NotFound(id) => format!(
                "No record matched '{id}'. Use `papercuts list` to see current open IDs, then retry with a full or unique-prefix ID."
            ),
            Error::Internal(_) => "Unexpected internal failure. Report the output to the maintainers.".into(),
            Error::Io(_) => "The file could not be read or written. Check permissions, disk space, and that the path is not on a read-only volume.".into(),
            Error::LockTimeout(path) => format!(
                "Another process is writing to {}. Retry the command in a moment (this error is retryable).",
                path.display()
            ),
            Error::PermDenied(_) => "Permission was denied. Check file and directory permissions.".into(),
            Error::Config(_) => "Configuration is invalid. Fix the referenced setting or unset it and retry.".into(),
        }
    }
}

/// Render an error to stderr as a single JSON envelope.
#[derive(Serialize)]
pub struct ErrorEnvelope {
    ok: bool,
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    exit: i32,
    message: String,
    suggested_fix: String,
}

impl Error {
    /// Print this error as a JSON envelope to stderr and return the exit code.
    pub fn print_exit(&self) -> i32 {
        let body = ErrorBody {
            code: self.code(),
            exit: self.exit(),
            message: self.to_string(),
            suggested_fix: self.suggested_fix(),
        };
        let env = ErrorEnvelope {
            ok: false,
            error: body,
        };
        let out = serde_json::to_string(&env).unwrap_or_else(|_| {
            r#"{"ok":false,"error":{"code":"E_INTERNAL","exit":70,"message":"failed to serialize error"}}"#.into()
        });
        eprintln!("{out}");
        self.exit()
    }
}

/// Convenience constructors so call sites read cleanly.
pub fn usage<S: Into<String>>(s: S) -> Error {
    Error::Usage(s.into())
}

pub fn bad_input<S: Into<String>>(s: S) -> Error {
    Error::BadInput(s.into())
}

pub fn internal<S: Into<String>>(s: S) -> Error {
    Error::Internal(s.into())
}

pub fn not_found<S: Into<String>>(s: S) -> Error {
    Error::NotFound(s.into())
}

pub fn config<S: Into<String>>(s: S) -> Error {
    Error::Config(s.into())
}

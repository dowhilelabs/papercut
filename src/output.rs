//! Output helpers. Most commands emit a single JSON envelope on stdout; `list`
//! defaults to human-readable text (`--format json` is the data-only envelope).
//! Errors go to stderr via `crate::error`.

use serde::Serialize;

pub const CONTRACT_VERSION: u32 = 2;

#[derive(Serialize)]
pub struct Meta {
    pub contract: u32,
    pub file: String,
}

#[derive(Serialize)]
pub struct Envelope<T: Serialize> {
    pub ok: bool,
    pub data: T,
    pub meta: Meta,
}

/// Print a successful envelope to stdout.
pub fn print_envelope<T: Serialize>(data: T, file: &str) {
    let env = Envelope {
        ok: true,
        data,
        meta: Meta {
            contract: CONTRACT_VERSION,
            file: file.to_string(),
        },
    };
    let out = serde_json::to_string(&env).unwrap_or_else(|_| {
        r#"{"ok":true,"data":{},"meta":{"contract":1,"file":"?"}}"#.into()
    });
    println!("{out}");
}

//! `papercut schema` — the full machine contract, returned as JSON so agents can
//! self-orient without reading prose.

use crate::output::CONTRACT_VERSION;
use serde_json::{json, Value};

/// Build the complete machine contract as a JSON value.
pub fn contract() -> Value {
    json!({
        "contract": CONTRACT_VERSION,
        "tool": "papercut",
        "journal": {
            "default": "<repo-root>/.papercuts.jsonl",
            "env_override": "PAPERCUTS_FILE",
            "outside_git": "~/.papercuts/log.jsonl",
            "format": "append-only JSONL, one record per line",
            "history": "append-only journal; resolve appends an event, never rewrites"
        },
        "env": {
            "PAPERCUTS_FILE": { "read": true, "write": false, "desc": "Override journal path" },
            "PAPERCUTS_MODEL": { "read": true, "write": false, "desc": "Override detected model" },
            "PAPERCUTS_HARNESS": { "read": true, "write": false, "desc": "Override detected harness" },
            "PAPERCUTS_USER": { "read": true, "write": false, "desc": "Override detected user" },
            "PAPERCUTS_NOW": { "read": true, "write": false, "desc": "Deterministic RFC3339 timestamp override for tests" }
        },
        "commands": {
            "add": {
                "alias": "log",
                "default": "implicit when the first arg is a message or a flag (-m/--model)",
                "write": true,
                "args": {
                    "text": "positional; '-' or omitted reads stdin",
                    "--tag/-t": "repeatable area tag",
                    "--severity": "minor | major | blocker (default minor)",
                    "-m/--model, --harness, --user": "override detected context"
                },
                "exit": { "0": "appended", "no-op": "duplicate suppressed (first-wins)" }
            },
            "list": {
                "read": true,
                "default": "human-readable text table",
                "args": {
                    "--format": "text (default) | json | md",
                    "--all": "include resolved records",
                    "--tag": "filter by tag"
                }
            },
            "resolve": {
                "write": true,
                "args": { "id": "full or unique-prefix id", "--note": "how it was fixed" }
            },
            "schema": { "read": true, "desc": "this contract" },
            "doctor": { "read": true, "desc": "validate the journal" }
        },
        "record": {
            "kind_cut": {
                "kind": "cut",
                "id": "pc_<sha256-hex-10> content-addressed",
                "ts": "RFC3339 UTC, PAPERCUTS_NOW override",
                "model": "string|null, detected/override",
                "harness": "string|null, detected/override",
                "user": "string|null, git config or env",
                "text": "string, unbounded",
                "tags": ["string"],
                "severity": "minor|major|blocker"
            },
            "kind_resolved": {
                "kind": "resolved",
                "id": "pc_... target id",
                "ts": "RFC3339 UTC",
                "by": "string|null",
                "note": "string|null"
            }
        },
        "error_codes": {
            "E_USAGE": 2,
            "E_BAD_INPUT": 65,
            "E_NOT_FOUND": 66,
            "E_INTERNAL": 70,
            "E_IO": 74,
            "E_LOCK_TIMEOUT": 75,
            "E_PERM_DENIED": 77,
            "E_CONFIG": 78
        },
        "exit_codes": {
            "0": "success (incl. empty results)",
            "2": "usage",
            "65": "bad input",
            "66": "not found",
            "70": "internal",
            "74": "I/O",
            "75": "lock timeout (retryable)",
            "77": "permission denied",
            "78": "config"
        },
        "stdout": "list defaults to human-readable text; --format json is data-only. Other commands emit one JSON envelope",
        "stderr": "errors only; one JSON envelope with code, exit, message, suggested_fix"
    })
}

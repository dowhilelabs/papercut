//! Detect who/what is filing a papercut from the ambient environment.
//!
//! Precedence is always: explicit CLI flag > `PAPERCUTS_*` env override > auto-detection.
//! Detection is deliberately best-effort — missing fields are `None`, never an error.

use std::process::Command;

/// Detect the model, harness, and user for a new record.
pub struct Context {
    pub model: Option<String>,
    pub harness: Option<String>,
    pub user: Option<String>,
}

/// Resolve context from explicit flags, env overrides, and detection.
pub fn resolve(
    flag_model: Option<&str>,
    flag_harness: Option<&str>,
    flag_user: Option<&str>,
) -> Context {
    let model = flag_model
        .map(str::to_string)
        .or_else(|| env_var("PAPERCUTS_MODEL"))
        .or_else(detect_model);

    let harness = flag_harness
        .map(str::to_string)
        .or_else(|| env_var("PAPERCUTS_HARNESS"))
        .or_else(detect_harness);

    let user = flag_user
        .map(str::to_string)
        .or_else(|| env_var("PAPERCUTS_USER"))
        .or_else(detect_user);

    Context {
        model,
        harness,
        user,
    }
}

fn env_var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

/// Best-effort model name from common agent environment variables.
fn detect_model() -> Option<String> {
    let keys = [
        "ANTHROPIC_MODEL",
        "ANTHROPIC_SMALL_FAST_MODEL",
        "OPENAI_MODEL",
        "OPENROUTER_MODEL",
        "GEMINI_MODEL",
        "CODEX_MODEL",
        "MODEL",
    ];
    keys.iter().find_map(|name| env_var(name))
}

/// Best-effort harness name from well-known env signals.
fn detect_harness() -> Option<String> {
    if env_var("CLAUDE_CODE_ENTRYPOINT").is_some() {
        return Some("claude-code".into());
    }
    if env_var("OPENAI_CODE").is_some() || env_var("CODEX_HOME").is_some() {
        return Some("codex".into());
    }
    if env_var("CURSOR_TRACE_ID").is_some() {
        return Some("cursor".into());
    }
    if env_var("OPENCODE_DIR").is_some() {
        return Some("opencode".into());
    }
    if env_var("AI_ASSISTANT").is_some() {
        return Some("ai-assistant".into());
    }
    // Fall back to the binary that launched us, if it looks like an agent.
    std::env::args()
        .next()
        .and_then(|a| std::path::Path::new(&a).file_stem().map(|s| s.to_string_lossy().into_owned()))
        .filter(|name| name != "papercut")
}

/// Resolve the filing user: git identity (repo-local over global), else $USER.
fn detect_user() -> Option<String> {
    git_config("user.email")
        .or_else(|| git_config("user.name"))
        .or_else(|| env_var("USER"))
        .or_else(|| env_var("LOGNAME"))
        .or_else(|| Some("unknown".into()))
}

/// `git config --get <key>` — reads the effective merged config (repo-local
/// overrides global), working from any directory if global config exists.
fn git_config(key: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["config", "--get", key])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let value = String::from_utf8(out.stdout).ok()?;
    let value = value.trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_beats_env_beats_detection() {
        // No way to reliably set detection in a test, but precedence ordering is:
        std::env::set_var("PAPERCUTS_USER", "env@example.com");
        let ctx = resolve(None, None, Some("flag@example.com"));
        assert_eq!(ctx.user.as_deref(), Some("flag@example.com"));
        let ctx = resolve(None, None, None);
        assert_eq!(ctx.user.as_deref(), Some("env@example.com"));
        std::env::remove_var("PAPERCUTS_USER");
    }

    #[test]
    fn harness_known_signals() {
        std::env::set_var("CLAUDE_CODE_ENTRYPOINT", "/usr/bin/claude");
        let ctx = resolve(None, None, None);
        assert_eq!(ctx.harness.as_deref(), Some("claude-code"));
        std::env::remove_var("CLAUDE_CODE_ENTRYPOINT");
    }
}

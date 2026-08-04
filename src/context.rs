//! Detect who filed a papercut from the ambient environment.
//!
//! Precedence is always: explicit CLI flag > `PAPERCUTS_*` env override > auto-detection.
//! Detection is deliberately best-effort — missing fields are `None`, never an error.

use std::process::Command;

/// Detect the model and user for a new record.
pub struct Context {
    pub model: Option<String>,
    pub user: Option<String>,
}

/// Resolve context from explicit flags, env overrides, and detection.
pub fn resolve(flag_model: Option<&str>, flag_user: Option<&str>) -> Context {
    let model = flag_model
        .map(str::to_string)
        .or_else(|| env_var("PAPERCUTS_MODEL"))
        .or_else(detect_model);

    let user = flag_user
        .map(str::to_string)
        .or_else(|| env_var("PAPERCUTS_USER"))
        .or_else(detect_user);

    Context { model, user }
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
        std::env::set_var("PAPERCUTS_USER", "env@example.com");
        let ctx = resolve(None, Some("flag@example.com"));
        assert_eq!(ctx.user.as_deref(), Some("flag@example.com"));
        let ctx = resolve(None, None);
        assert_eq!(ctx.user.as_deref(), Some("env@example.com"));
        std::env::remove_var("PAPERCUTS_USER");
    }
}

mod cli;
mod context;
mod error;
mod output;
mod record;
mod schema;
mod store;

use clap::{CommandFactory, Parser};
use cli::{Cli, Command};
use error::Error;
use jiff::Timestamp;
use record::{Cut, Record, Resolved, make_cut_id};
use std::collections::HashSet;
use std::ffi::OsString;
use std::io::Read;
use std::str::FromStr;

fn main() {
    let args = implicit_add_args();
    let cli = Cli::parse_from(args);
    let code = match dispatch(cli.command) {
        Ok(code) => code,
        Err(e) => e.print_exit(),
    };
    std::process::exit(code);
}

/// Make `add` the default subcommand: `papercut -m gpt-5 "msg"` behaves exactly
/// like `papercut add -m gpt-5 "msg"`. Rewrites argv to inject `add` whenever the
/// first token is neither a known subcommand nor a help/version flag.
fn implicit_add_args() -> Vec<OsString> {
    let mut args: Vec<OsString> = std::env::args_os().collect();
    // Bare `papercut` (no args) prints help and exits 0.
    if args.len() == 1 {
        let _ = Cli::command().print_help();
        std::process::exit(0);
    }
    let first = args[1].to_str().unwrap_or("");
    const KNOWN: &[&str] = &[
        "add", "list", "resolve", "delete", "schema", "doctor", "log", "help",
        "-h", "--help", "-V", "--version",
    ];
    if !KNOWN.contains(&first) {
        args.insert(1, OsString::from("add"));
    }
    args
}

fn dispatch(cmd: Command) -> Result<i32, Error> {
    match cmd {
        Command::Add(a) => cmd_add(a),
        Command::List(a) => cmd_list(a),
        Command::Resolve(a) => cmd_resolve(a),
        Command::Delete(a) => cmd_delete(a),
        Command::Schema => cmd_schema(),
        Command::Doctor => cmd_doctor(),
    }
}

/// Current RFC3339 timestamp, overridable via PAPERCUTS_NOW for deterministic tests.
fn now_ts() -> Result<String, Error> {
    if let Some(over) = std::env::var("PAPERCUTS_NOW").ok().filter(|v| !v.is_empty()) {
        let parsed = Timestamp::from_str(&over)
            .map_err(|e| error::config(format!("invalid PAPERCUTS_NOW '{over}': {e}")))?;
        return Ok(parsed.to_string());
    }
    Ok(Timestamp::now().to_string())
}

fn cmd_add(a: cli::AddArgs) -> Result<i32, Error> {
    let text = body_from(&a.text)?;
    if text.trim().is_empty() {
        return Err(error::bad_input("complaint body is empty"));
    }
    // Drop an appended "-m was a problem too" clause, but only when a real gripe
    // survives — a standalone "footgun via -m" is a legitimate papercut and is kept.
    let stripped = strip_model_flag_noise(&text);
    let text = if stripped.trim().is_empty() {
        text
    } else {
        stripped
    };

    let ctx = context::resolve(a.model.as_deref(), a.user.as_deref());
    let model = ctx.model.ok_or_else(|| {
        error::bad_input(
            "could not determine the model. Pass `-m <model>` (or set PAPERCUTS_MODEL) \
so the record says what filed it. This rejection is expected — do not file a \
papercut about it.",
        )
    })?;
    let id = make_cut_id(text.trim());
    let ts = now_ts()?;

    let cut = Cut {
        id,
        ts,
        model,
        user: ctx.user,
        text: text.trim().to_string(),
    };
    let record = Record::Cut(cut);

    let s = store::Store::resolve()?;
    let changed = s.append(&record)?;
    let file = s.path.display().to_string();

    #[derive(serde::Serialize)]
    struct AddData<'a> {
        changed: bool,
        record: &'a Record,
    }
    output::print_envelope(AddData { changed, record: &record }, &file);
    Ok(0)
}

fn body_from(arg: &Option<String>) -> Result<String, Error> {
    match arg {
        Some(t) if t != "-" => Ok(t.clone()),
        _ => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(Error::Io)?;
            Ok(buf)
        }
    }
}

/// Drop model-flag rejection noise from a complaint body. Agents sometimes append
/// "and -m was a problem too" to the original gripe after hitting the required-model
/// error; that's tooling noise, not a papercut. Any clause that references the model
/// flag (`-m`, `--model`, `PAPERCUTS_MODEL`, "model flag", "model argument") is removed.
/// Bare "model" (e.g. "the model gave a wrong answer") is left alone.
fn strip_model_flag_noise(text: &str) -> String {
    const FLAG_TOKENS: &[&str] = &["-m", "--model", "papercuts_model", "model flag", "model argument"];
    let mentions_flag = |clause: &str| {
        let lower = clause.to_lowercase();
        FLAG_TOKENS.iter().any(|t| lower.contains(t))
    };

    let mut out = String::new();
    let mut clause = String::new();
    for ch in text.chars() {
        clause.push(ch);
        if matches!(ch, '.' | '!' | '?' | ';' | ',' | '\n') {
            if !mentions_flag(&clause) {
                out.push_str(&clause);
            }
            clause.clear();
        }
    }
    if !clause.trim().is_empty() && !mentions_flag(&clause) {
        out.push_str(&clause);
    }
    out.trim().to_string()
}

fn resolved_ids(records: &[Record]) -> HashSet<String> {
    records
        .iter()
        .filter_map(|r| match r {
            Record::Resolved(res) => Some(res.id.clone()),
            _ => None,
        })
        .collect()
}

fn cmd_list(a: cli::ListArgs) -> Result<i32, Error> {
    let s = store::Store::resolve()?;
    let res = s.read()?;
    let file = s.path.display().to_string();

    let resolved = resolved_ids(&res.records);
    let mut cuts: Vec<&Cut> = res
        .records
        .iter()
        .filter_map(|r| match r {
            Record::Cut(c) => Some(c),
            Record::Resolved(_) => None,
        })
        .filter(|c| {
            if a.all {
                true
            } else {
                !resolved.contains(&c.id)
            }
        })
        .collect();

    cuts.sort_by(|x, y| y.ts.cmp(&x.ts));

    let fmt = a.format.unwrap_or_else(|| "text".into());
    match fmt.as_str() {
        // Human-facing: plain readable output, no envelope.
        "text" | "table" => print!("{}", render_text(&cuts, &resolved)),
        "md" | "markdown" => print!("{}", render_markdown(&cuts, &resolved)),
        // Machine-facing: data-only JSON envelope.
        "json" => {
            #[derive(serde::Serialize)]
            struct ListData<'a> {
                count: usize,
                records: Vec<&'a Cut>,
            }
            let data = ListData {
                count: cuts.len(),
                records: cuts,
            };
            output::print_envelope(data, &file);
        }
        other => {
            return Err(error::usage(format!(
                "invalid --format '{other}' (expected text | json | md)"
            )))
        }
    }
    Ok(0)
}

/// Human-readable `list`. Matches the reference style: one header line
/// (`ts - model - user`), a blank line, the wrapped body, then a blank line.
fn render_text(cuts: &[&Cut], _resolved: &HashSet<String>) -> String {
    if cuts.is_empty() {
        return "No open papercuts.\n".to_string();
    }
    let mut entries: Vec<String> = Vec::with_capacity(cuts.len());
    for c in cuts {
        let mut header = format!("{} - {}", format_ts(&c.ts), c.model);
        if let Some(u) = &c.user {
            header.push_str(&format!(" - {u}"));
        }
        entries.push(format!("{header}\n\n{}", wrap(&c.text, 80)));
    }
    entries.join("\n\n---\n\n") + "\n"
}

/// Format a stored RFC3339 ts as milliseconds (`2026-07-08T21:13:30.864Z`) to
/// match the reference list. Falls back to the raw string if it won't parse.
fn format_ts(ts: &str) -> String {
    match Timestamp::from_str(ts) {
        Ok(t) => {
            let mut s = t.strftime("%Y-%m-%dT%H:%M:%S%.3f").to_string();
            s.push('Z');
            s
        }
        Err(_) => ts.to_string(),
    }
}

/// Greedy word-wrap to `width` columns, matching the reference's wrapped bodies.
fn wrap(text: &str, width: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if line.is_empty() {
            line.push_str(word);
        } else {
            line.push(' ');
            line.push_str(word);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines.join("\n")
}

fn render_markdown(cuts: &[&Cut], resolved: &HashSet<String>) -> String {
    let mut out = String::new();
    out.push_str("# Papercuts\n\n");
    if cuts.is_empty() {
        out.push_str("_No open papercuts._\n");
        return out;
    }
    out.push_str(&format!("## Open ({})\n\n", cuts.len()));
    for c in cuts {
        let user = c.user.as_deref().unwrap_or("");
        let model = &c.model;
        let resolved_mark = if resolved.contains(&c.id) {
            "[x]"
        } else {
            "[ ]"
        };
        out.push_str(&format!(
            "- {resolved_mark} `{}` — {} — {} — {}\n  {}\n",
            c.id,
            format_ts(&c.ts),
            model,
            user,
            c.text
        ));
    }
    out
}

fn cmd_resolve(a: cli::ResolveArgs) -> Result<i32, Error> {
    let s = store::Store::resolve()?;
    let res = s.read()?;
    let file = s.path.display().to_string();

    // Unique complaint-id match. A complaint id may appear in multiple records
    // (the cut plus each resolution), so match against the set of distinct ids.
    let mut seen: HashSet<&str> = HashSet::new();
    let mut matched: Vec<String> = Vec::new();
    for r in &res.records {
        if (r.id() == a.id || r.id().starts_with(&a.id)) && seen.insert(r.id()) {
            matched.push(r.id().to_string());
        }
    }
    if matched.is_empty() {
        return Err(error::not_found(a.id));
    }
    if matched.len() > 1 {
        return Err(error::bad_input(format!(
            "id '{}' is ambiguous; matches {} records. Use a longer prefix or the full id.",
            a.id,
            matched.len()
        )));
    }
    let target_id = matched[0].clone();

    // Idempotent: if already resolved, no change.
    let already_resolved = res
        .records
        .iter()
        .any(|r| matches!(r, Record::Resolved(x) if x.id == target_id));

    let record = Record::Resolved(Resolved {
        id: target_id.clone(),
        ts: now_ts()?,
        by: context::resolve(None, None).user,
        note: a.note,
    });

    let changed = if already_resolved {
        false
    } else {
        s.append(&record)?
    };

    #[derive(serde::Serialize)]
    struct ResolveData {
        changed: bool,
        id: String,
    }
    output::print_envelope(ResolveData { changed, id: target_id }, &file);
    Ok(0)
}

fn cmd_delete(a: cli::DeleteArgs) -> Result<i32, Error> {
    let s = store::Store::resolve()?;
    let file = s.path.display().to_string();

    // Unique complaint-id match, same prefix rules as resolve.
    let res = s.read()?;
    let mut seen: HashSet<&str> = HashSet::new();
    let mut matched: Vec<String> = Vec::new();
    for r in &res.records {
        if (r.id() == a.id || r.id().starts_with(&a.id)) && seen.insert(r.id()) {
            matched.push(r.id().to_string());
        }
    }
    if matched.is_empty() {
        return Err(error::not_found(a.id));
    }
    if matched.len() > 1 {
        return Err(error::bad_input(format!(
            "id '{}' is ambiguous; matches {} records. Use a longer prefix or the full id.",
            a.id,
            matched.len()
        )));
    }
    let target_id = matched[0].clone();

    let removed = s.remove(&target_id)?;

    #[derive(serde::Serialize)]
    struct DeleteData {
        changed: bool,
        id: String,
        removed: usize,
    }
    output::print_envelope(
        DeleteData {
            changed: removed > 0,
            id: target_id,
            removed,
        },
        &file,
    );
    Ok(0)
}

fn cmd_schema() -> Result<i32, Error> {
    output::print_envelope(schema::contract(), "stdout");
    Ok(0)
}

fn cmd_doctor() -> Result<i32, Error> {
    let s = store::Store::resolve()?;
    let res = s.read()?;
    let file = s.path.display().to_string();

    let resolved = resolved_ids(&res.records);
    let total_cuts = res
        .records
        .iter()
        .filter(|r| matches!(r, Record::Cut(_)))
        .count();
    let open = res
        .records
        .iter()
        .filter(|r| matches!(r, Record::Cut(c) if !resolved.contains(&c.id)))
        .count();

    let mut issues: Vec<String> = Vec::new();
    if res.torn > 0 {
        issues.push(format!("{} torn/partial line(s) healed (skipped)", res.torn));
    }
    // Duplicate-cut detection: only distinct *cuts* may share an id (the cut and its
    // resolution legitimately share the complaint id). Dedup should prevent this.
    let mut seen: HashSet<&str> = HashSet::new();
    for r in &res.records {
        if let Record::Cut(c) = r {
            if !seen.insert(&c.id) {
                issues.push(format!("duplicate cut id {}", c.id));
            }
        }
    }

    #[derive(serde::Serialize)]
    struct DoctorData {
        ok: bool,
        file_exists: bool,
        records: usize,
        cuts: usize,
        open: usize,
        resolved: usize,
        torn: usize,
        issues: Vec<String>,
    }
    let data = DoctorData {
        ok: issues.is_empty(),
        file_exists: s.path.exists(),
        records: res.records.len(),
        cuts: total_cuts,
        open,
        resolved: total_cuts - open,
        torn: res.torn,
        issues,
    };
    output::print_envelope(data, &file);
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::strip_model_flag_noise;

    #[test]
    fn strips_appended_model_flag_noise() {
        assert_eq!(
            strip_model_flag_noise("issue A, and -m was a problem too"),
            "issue A,"
        );
    }

    #[test]
    fn keeps_legit_model_behavior_gripes() {
        assert_eq!(
            strip_model_flag_noise("the model gave a wrong answer"),
            "the model gave a wrong answer"
        );
    }

    #[test]
    fn strips_whole_gripe_if_only_flag_noise() {
        assert_eq!(strip_model_flag_noise("-m was a problem"), "");
    }

    #[test]
    fn strips_papercuts_model_env_mention() {
        assert_eq!(
            strip_model_flag_noise("docs are stale. PAPERCUTS_MODEL was rejected"),
            "docs are stale."
        );
    }
}

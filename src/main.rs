mod cli;
mod context;
mod error;
mod output;
mod record;
mod schema;
mod store;

use clap::Parser;
use cli::{Cli, Command};
use error::Error;
use jiff::Timestamp;
use record::{Cut, Record, Resolved, Severity, make_cut_id};
use std::collections::HashSet;
use std::io::Read;
use std::str::FromStr;

fn main() {
    let cli = Cli::parse();
    let code = match dispatch(cli.command) {
        Ok(code) => code,
        Err(e) => e.print_exit(),
    };
    std::process::exit(code);
}

fn dispatch(cmd: Command) -> Result<i32, Error> {
    match cmd {
        Command::Add(a) => cmd_add(a),
        Command::List(a) => cmd_list(a),
        Command::Resolve(a) => cmd_resolve(a),
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

    let severity: Severity = a
        .severity
        .parse()
        .map_err(|e: String| error::bad_input(e))?;

    let ctx = context::resolve(a.model.as_deref(), a.harness.as_deref(), a.user.as_deref());
    let id = make_cut_id(text.trim(), &a.tags, severity);
    let ts = now_ts()?;

    let cut = Cut {
        id,
        ts,
        model: ctx.model,
        harness: ctx.harness,
        user: ctx.user,
        text: text.trim().to_string(),
        tags: a.tags,
        severity,
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

fn severity_rank(s: &Severity) -> u8 {
    match s {
        Severity::Blocker => 3,
        Severity::Major => 2,
        Severity::Minor => 1,
    }
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
        .filter(|c| match &a.tag {
            Some(tag) => c.tags.iter().any(|t| t == tag),
            None => true,
        })
        .collect();

    cuts.sort_by(|x, y| {
        severity_rank(&y.severity)
            .cmp(&severity_rank(&x.severity))
            .then_with(|| y.ts.cmp(&x.ts))
    });

    let fmt = a.format.unwrap_or_else(|| "json".into());
    match fmt.as_str() {
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
        "md" => {
            let md = render_markdown(&cuts, &resolved);
            #[derive(serde::Serialize)]
            struct MdData {
                markdown: String,
            }
            output::print_envelope(MdData { markdown: md }, &file);
        }
        other => {
            return Err(error::usage(format!(
                "invalid --format '{other}' (expected json | md)"
            )))
        }
    }
    Ok(0)
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
        let tags = if c.tags.is_empty() {
            String::new()
        } else {
            format!(" `[{}]`", c.tags.join(", "))
        };
        let ctx = match (&c.harness, &c.model) {
            (Some(h), Some(m)) => format!(" `{h}/{m}`"),
            (Some(h), None) => format!(" `{h}`"),
            (None, Some(m)) => format!(" `{m}`"),
            (None, None) => String::new(),
        };
        let user = c
            .user
            .as_ref()
            .map(|u| format!(" — {u}"))
            .unwrap_or_default();
        let resolved_mark = if resolved.contains(&c.id) {
            "[x]"
        } else {
            "[ ]"
        };
        out.push_str(&format!(
            "- {resolved_mark} `{}` — **{}** — {}{}{}{}\n",
            c.id,
            c.severity,
            c.text,
            tags,
            ctx,
            user
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
        by: context::resolve(None, None, None).user,
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

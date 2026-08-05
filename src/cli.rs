//! Command-line surface. `list` defaults to human-readable text (--format json
//! is data-only); the schema command documents the full machine contract.
//! full machine contract for self-orienting agents.

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "papercut",
    version,
    about = "A tiny CLI that gives AI agents a complaint box.",
    long_about = "A complaint box for AI agents. When an agent hits a dead end or something \
that doesn't make sense while working, it files a one-line papercut into \
.papercuts.jsonl (local to the machine). Agents write papercuts; humans and \
agents read the backlog. `papercut list` is human-readable by default; other \
commands emit one JSON envelope; errors go to stderr with stable codes. \
Run `papercut schema` for the full machine contract."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// File a papercut complaint. Default subcommand: `papercut -m <model> "message"`.
    #[command(visible_alias = "log")]
    Add(AddArgs),
    /// List papercuts, newest first. Defaults to a human-readable digest
    /// (`ts - model - user` then the body); `--format json` for machine parsing,
    /// `md` for markdown.
    List(ListArgs),
    /// Mark a papercut fixed (appends a resolved event; never rewrites history).
    Resolve(ResolveArgs),
    /// Permanently remove a papercut (and any resolution) from the journal by
    /// rewriting the file. Use for noise that shouldn't be in the backlog at all.
    Delete(DeleteArgs),
    /// Print the machine contract (commands, env vars, shapes, exit codes).
    Schema,
    /// Validate the journal file and report issues.
    Doctor,
}

#[derive(Args)]
pub struct AddArgs {
    /// The complaint body. Pass `-` or omit (with piped stdin) to read from stdin.
    #[arg(value_name = "TEXT")]
    pub text: Option<String>,
    /// Override the detected model (e.g. `-m gpt-5`).
    #[arg(short = 'm', long)]
    pub model: Option<String>,
    /// Override the detected user.
    #[arg(long)]
    pub user: Option<String>,
}

#[derive(Args)]
pub struct ListArgs {
    /// Output format: text (default) | json | md.
    #[arg(long, value_name = "FORMAT")]
    pub format: Option<String>,
    /// Include resolved records (default: open cuts only).
    #[arg(long)]
    pub all: bool,
}

#[derive(Args)]
pub struct ResolveArgs {
    /// Full or unique-prefix record id, e.g. `pc_9f2c41d0a8`.
    #[arg(value_name = "ID")]
    pub id: String,
    /// Optional note about how it was fixed.
    #[arg(long)]
    pub note: Option<String>,
}

#[derive(Args)]
pub struct DeleteArgs {
    /// Full or unique-prefix record id, e.g. `pc_9f2c41d0a8`.
    #[arg(value_name = "ID")]
    pub id: String,
}

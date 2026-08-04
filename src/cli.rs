//! Command-line surface. stdout stays data-only; the schema command documents the
//! full machine contract for self-orienting agents.

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "papercuts",
    version,
    about = "A tiny CLI that gives AI agents a complaint box.",
    long_about = "Papercuts is an append-only journal where agents (and humans) file the friction \
they hit during work — dead-end tool calls, broken links, misleading docs, footgun configs. \
It lives in .papercuts.jsonl at the repo root so every complaint shows up in git diff and travels \
with the repo. stdout is data-only (one JSON envelope); errors go to stderr with stable codes. \
Run `papercuts schema` for the full machine contract."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// File a papercut complaint (alias: `papercuts log`).
    Add(AddArgs),
    /// List papercuts, open-first then newest (severity-first digest in `--format md`).
    List(ListArgs),
    /// Mark a papercut fixed (appends a resolved event; never rewrites history).
    Resolve(ResolveArgs),
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
    /// Area tag, repeatable: `--tag tooling --tag docs`.
    #[arg(short = 't', long = "tag", action = clap::ArgAction::Append)]
    pub tags: Vec<String>,
    /// Severity: minor | major | blocker.
    #[arg(long, default_value = "minor")]
    pub severity: String,
    /// Override the detected model.
    #[arg(long)]
    pub model: Option<String>,
    /// Override the detected harness.
    #[arg(long)]
    pub harness: Option<String>,
    /// Override the detected user.
    #[arg(long)]
    pub user: Option<String>,
}

#[derive(Args)]
pub struct ListArgs {
    /// Output format: json (default) | md.
    #[arg(long)]
    pub format: Option<String>,
    /// Include resolved records (default: open cuts only).
    #[arg(long)]
    pub all: bool,
    /// Only records carrying this tag.
    #[arg(long)]
    pub tag: Option<String>,
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

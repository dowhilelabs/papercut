# papercut

A tiny CLI that gives anyone — human or AI agent — a complaint box for a repo.
When you hit friction (a dead-end tool call, a broken link, a footgun config, a
missing helper), file it in one line instead of silently pushing through. The
backlog lives in `.papercuts.jsonl` at your repo root, local to your machine and
easily reviewed and fixed.

## Quickstart

```bash
# 1. Install (macOS, Apple silicon)
brew tap dowhilelabs/papercut https://github.com/dowhilelabs/papercut
brew install papercut

# 2. File a papercut — what got in the way + what would have prevented it
#    (add is the default subcommand; `-m` records the model explicitly)
papercut "yarn web:test with a root-relative path finds no files; the workspace test cwd is apps/web" --tag tooling

# 3. Review the backlog
papercut list
papercut list --format md   # human digest
```

That's it — your first papercut is now recorded in `.papercuts.jsonl`.

## Install

**macOS, Apple silicon.** This repo doubles as a Homebrew tap:

```bash
brew tap dowhilelabs/papercut https://github.com/dowhilelabs/papercut
brew install papercut
```

The explicit URL matters: Homebrew's tap convention looks for `homebrew-papercut`, so the short `brew tap dowhilelabs/papercut` form fails.

Or, from source: `cargo install --path .`

## Usage

```bash
papercut "text"                  # file a papercut (add is the default subcommand)
papercut -m gpt-5 "text"         # ... with the model recorded explicitly
papercut add "text"              # explicit subcommand (also: papercut log)
papercut add -                   # file from stdin (good for long bodies)
papercut list                    # human-readable digest (ts - model - user + body)
papercut list --format md        # markdown digest
papercut list --format json      # machine/agent JSON
papercut list --all              # include resolved items
papercut resolve pc_9f2c41d0     # mark one fixed (full or prefix id)
papercut doctor                  # validate the journal
papercut schema                  # full machine contract (mainly for agents)
```

Each entry records **who** (git identity), **what** (model/harness, when filed
by an agent), **when** (RFC3339 UTC), plus tags and severity. Everything is
append-only — `resolve` adds an event, it never rewrites history.

## Where papercuts live

`.papercuts.jsonl` at your repo root (falls back to `~/.papercuts/log.jsonl`
outside a git repo). It's plain, append-only JSONL — human-readable and local
to your machine (gitignored, so it stays yours; it doesn't get committed or
shared). No server, no sync, no telemetry.

## For your AI agents

Paste this into `CLAUDE.md` / `AGENTS.md` / a system prompt so your agents file
friction instead of silently pushing through:

```markdown
## Papercuts

When you hit small friction while working — a tool call that missed and had to
be retried, a confusing or undocumented setup step, a flaky command, a stale
cache, a misleading error, a non-obvious gotcha — file it in the moment:

    papercut "<what you were doing → what got in the way>" --tag <area>

One or two sentences: what you were doing → what got in the way (a guess at the
cause or fix is a bonus). Do this proactively, even though none of it is
blocking — logged together it shows where the repo needs sanding down. Severity:
minor (default) for annoyances, major for time sinks, blocker for hard walls.

This is distinct from your task log (what you accomplished) and from real bug
tracking (reproducible bugs / tracked work).
```

Then periodically run `papercut list --format md` and fix what keeps coming up.

## Details

- **Config** — `PAPERCUTS_FILE` overrides the journal path; `PAPERCUTS_MODEL`,
  `PAPERCUTS_HARNESS`, `PAPERCUTS_USER` override detected identity; `PAPERCUTS_NOW`
  pins the timestamp (for reproducible tests).
- **Agents** — use `papercut list --format json` for a data-only JSON envelope;
  plain `list` is human-readable. Errors go to stderr with stable codes and
  documented exit codes. `papercut schema` returns the full machine contract.
- **Concurrency** — safe for multiple agents on one file (locking, atomic
  appends, self-healing torn lines, duplicate suppression).

## Development

```bash
cargo build --release
cargo test
```

## License

MIT

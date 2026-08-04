# papercut

A tiny CLI complaint box. When you hit friction — a dead-end tool call, a broken
link, a footgun config, a missing helper — file it in one line instead of
silently pushing through. Every entry lands in `.papercuts.jsonl` next to your
repo (or `~/.papercuts/log.jsonl`), local to your machine, and easy to review.

## Install

macOS, Apple silicon:

```bash
brew tap dowhilelabs/papercut https://github.com/dowhilelabs/papercut
brew install papercut
```

(The tap is one-time; the explicit URL matters because Homebrew would otherwise
look for `homebrew-papercut`.)

## Quickstart

```bash
papercut "yarn web:test finds no files — the workspace test cwd is apps/web" --tag tooling
papercut list
```

That's it. File a papercut, review the backlog.

## Usage

```bash
papercut "text"            # file a papercut (add is the default subcommand)
papercut -m gpt-5 "text"   # record which model filed it
papercut list              # review the backlog (newest first, --- separated)
papercut list --all        # include resolved
papercut resolve <id>      # mark one fixed
```

Each entry records who (git identity), what (model/harness, for agent-filed),
when (RFC3339 UTC), plus tags and severity. Everything is append-only —
`resolve` adds an event, it never rewrites history.

## Where papercuts live

`.papercuts.jsonl` at your repo root, falling back to `~/.papercuts/log.jsonl`
outside a repo. Plain, append-only JSONL, gitignored so it stays yours. No
server, no sync, no telemetry.

## For your AI agents

Paste this into `CLAUDE.md` / `AGENTS.md` / a system prompt so your agents file
friction instead of pushing through:

```markdown
## Papercuts

When you hit small friction while working — a tool call that missed and had to
be retried, a confusing or undocumented setup step, a flaky command, a stale
cache, a misleading error — file it in the moment:

    papercut "<what you were doing → what got in the way>" --tag <area>

One or two sentences. Do this proactively even when nothing is blocking; logged
together it shows where the repo needs sanding down. Severity: minor (default)
for annoyances, major for time sinks, blocker for hard walls.
```

Then run `papercut list --format md` periodically and fix what keeps coming up.

## Details

- **Agents / machines** — `papercut list --format json` emits a data-only JSON
  envelope (with id, severity, tags); plain `list` is the human digest.
- **Config** — `PAPERCUTS_FILE` overrides the journal path; `PAPERCUTS_MODEL`,
  `PAPERCUTS_HARNESS`, `PAPERCUTS_USER` override detected identity; `PAPERCUTS_NOW`
  pins the timestamp. `papercut schema` prints the full machine contract.
- **Concurrency** — safe for multiple agents on one file (locking, atomic
  appends, self-healing torn lines, duplicate suppression).

## Development

```bash
cargo build --release
cargo test
```

Source install: `cargo install --path .`

## License

MIT

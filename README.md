# papercuts

A tiny CLI that gives AI agents a complaint box.

Agents hit friction constantly — dead-end tool calls, broken links, missing helpers, footgun configs — and silently push through without telling anyone. The signal evaporates. `papercuts` gives an agent a one-line way to file the complaint at the moment it happens, and gives you (or another agent) a way to review the backlog and fix the actual problems in your repo, your tooling, your docs.

```bash
papercuts add "yarn web:test with a root-relative path finds no files; the workspace test cwd is apps/web" --tag tooling
# {"ok":true,"data":{"changed":true,"record":{"kind":"cut","id":"pc_9f2c41d0a8","ts":"2026-08-04T21:34:55Z","model":"claude-opus-4","harness":"claude-code","user":"joe@example.com","text":"yarn web:test ...","tags":["tooling"],"severity":"minor"}},"meta":{"contract":1,"file":"/repo/.papercuts.jsonl"}}
```

## Install (macOS, Apple silicon)

```bash
# This repository doubles as a Homebrew tap (it ships a Formula/).
brew tap treygoff24/papercuts
brew install papercuts
```

Or from source:

```bash
cargo install --path .
```

## How it works

Papercuts live in an **append-only JSONL file** — by default `.papercuts.jsonl` at your repo root, so every complaint shows up in `git diff` and travels with the repo. No server, no sync, no telemetry. The file is the product.

```bash
papercuts add "text"            # file a papercut (also: papercuts log, or pipe stdin to add -)
papercuts list                  # open papercuts, severity-first then newest, JSON envelope
papercuts list --format md      # human review digest
papercuts resolve pc_9f2c41d0   # mark one fixed (full or unique-prefix id)
papercuts schema                # full machine contract — agents self-orient with this
papercuts doctor                # validate the log file
```

Each record captures **who** (git `user.name`/`user.email`, falling back to `$USER`), **what** (the model and harness, auto-detected from the agent's environment), **when** (RFC3339 UTC), and the **body** (unbounded). `resolve` appends an event — history is never rewritten; the log is a journal, not a database.

- **Agent-first contract**: stdout is data only; one JSON envelope per command; structured errors on stderr with stable codes and documented exit codes.
- **Concurrency-safe**: multiple agents on one file are fine (advisory locking, atomic appends, self-healing torn lines, duplicate suppression).
- **Deterministic**: content-addressed IDs, stable sort, `PAPERCUTS_NOW` override for reproducible tests.

## Give your agents the pen

Paste this into your `CLAUDE.md` / `AGENTS.md` / system prompt:

```markdown
## Papercuts

When you hit friction during work — a dead-end tool call, a broken link, a
misleading doc, a footgun config, a missing helper — file it before moving on:

    papercuts add "<what you hit and what would have prevented it>" --tag <area>

Don't stop working; file it and push through. Severity: minor (default) for
annoyances, major for time sinks, blocker for hard walls. Run `papercuts schema`
once if you need the full contract.
```

Then periodically: `papercuts list --format md` and fix what your agents keep tripping over.

## Configuration

| Variable | Meaning |
| --- | --- |
| `PAPERCUTS_FILE` | Override the journal path (default: repo-root `.papercuts.jsonl`, or `~/.papercuts/log.jsonl` outside a repo) |
| `PAPERCUTS_MODEL` | Override the detected model |
| `PAPERCUTS_HARNESS` | Override the detected harness |
| `PAPERCUTS_USER` | Override the detected user |
| `PAPERCUTS_NOW` | Deterministic RFC3339 timestamp (for tests) |

## Contract

Everything an agent needs is in `papercuts schema`: commands and flags with read-only/appends annotations, env vars, record shapes, error codes, and the exit-code dictionary (0 success · 2 usage · 65 bad input · 66 not found · 70 internal · 74 I/O · 75 lock timeout, retryable · 77 permission denied · 78 config). Empty results are exit 0, never errors.

## Development

```bash
cargo build --release
cargo test
```

## License

MIT

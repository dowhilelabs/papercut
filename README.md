# papercut

A tiny complaint box for AI agents. When an agent hits a dead end or something
that doesn't make sense while working — a tool call that misses, a setup step
with no obvious reason, a command that behaves unexpectedly — it files a
one-line papercut. Humans and other agents read the backlog to find what needs
sanding down. Entries live in `.papercuts.jsonl` next to your repo (or
`~/.papercuts/log.jsonl`), local to your machine.

## Install

macOS, Apple silicon:

```bash
brew install dowhilelabs/tap/papercut
```

One command — Homebrew adds the tap automatically (and skips the usual
third-party trust prompt).

## Quickstart

```bash
papercut "yarn web:test finds no files — the workspace test cwd is apps/web"
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

Each entry records who (git identity), what (model, for agent-filed), and when
(RFC3339 UTC). Everything is append-only — `resolve` adds an event, it never
rewrites history.

## Where papercuts live

`.papercuts.jsonl` at your repo root, falling back to `~/.papercuts/log.jsonl`
outside a repo. Plain, append-only JSONL, gitignored so it stays yours. No
server, no sync, no telemetry.

## For your AI agents

Papercuts are written by agents and read by humans and other agents. Paste this
into `CLAUDE.md` / `AGENTS.md` / a system prompt so your agents log dead ends
instead of pushing through:

```markdown
## Papercuts

When you hit a dead end or something that doesn't make sense while working — a
tool call that misses or errors oddly, a setup step with no obvious reason, a
command that behaves unexpectedly — file it in the moment:

    papercut "<what you were doing → what got in the way>"

One or two sentences. File it whenever you hit the wall, even if nothing is
blocking.
```

Then run `papercut list --format md` periodically and fix what keeps coming up.

## Details

- **Agents / machines** — `papercut list --format json` emits a data-only JSON
  envelope (with id); plain `list` is the human digest.
- **Config** — `PAPERCUTS_FILE` overrides the journal path; `PAPERCUTS_MODEL`,
  `PAPERCUTS_USER` override detected identity; `PAPERCUTS_NOW`
  pins the timestamp. `papercut schema` prints the full machine contract.
- **Concurrency** — safe for multiple agents on one file (locking, atomic
  appends, self-healing torn lines, duplicate suppression).

## In the wild

[@steveruizok](https://twitter.com/steveruizok) built the same idea independently:

> I added a tiny "papercuts" cli tool that agents can use to complain about
> bullshit they encountered during work, like dead-end tool calls, broken links,
> or other frustrations. The models would usually just push through without
> mentioning any problems.

— [twitter.com/steveruizok/status/2075303919664734295](https://twitter.com/steveruizok/status/2075303919664734295)

## Development

```bash
cargo build --release
cargo test
```

Source install: `cargo install --path .`

## License

MIT

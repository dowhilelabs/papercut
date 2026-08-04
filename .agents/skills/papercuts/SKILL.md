---
name: papercuts
description: "File a papercut when you hit a dead end or something that doesn't make sense while working — a tool call that misses or errors oddly, a setup step with no obvious reason, a command that behaves unexpectedly. Log it so a future reader doesn't trip on it again. Also use to review, deduplicate, and resolve existing entries."
---

# Papercuts

Capture dead ends and confusing moments in the moment, without derailing the
current task. These are distinct from your task log (what you accomplished) and
from real bug tracking (reproducible bugs / tracked work). The `papercut` CLI
appends to `.papercuts.jsonl` at the repo root, so dead ends accumulate into a
local backlog that humans and future agents can read.

## Is it worth a papercut?

Log it when you hit a real dead end or something that didn't make sense — the
kind of thing a future reader (human or agent) would benefit from knowing
before they hit the same wall:

- **A tool call that missed or failed in a confusing way.**
- **A setup step or config with no obvious reason.**
- **A command or behavior that was surprising — didn't do what you expected.**

Skip it if it's just your environment or a one-off — your sandbox's failures,
your own shell mistakes, or transient flakiness (details below).

## Do NOT log

- **Your environment's failures.** Sandbox `EPERM` / `listen` / IPC-socket
  errors, blocked network or `fetch failed`, permission denials, missing system
  tools. That is the runner, not the repo.
- **Your own shell mistakes.** Reserved/special variable names, unquoted globs,
  a broken login-shell hook. Fix the command — nothing in the repo to sand down.
- **Transient flakiness.** A command that succeeded on retry with no repo-side
  cause (network blip, hung push, slow mirror).
- **Local state you corrupted.** A partial `node_modules` after branch-switching,
  a stale dev-server port, a dirty cache.
- **Product/correctness bugs** (fix now or track as real work), and **what you
  accomplished** (that belongs in the task summary).
- **Secrets, credentials, personal data, or sensitive paths.**

When something fails, first ask "is this the repo, or is this me/my environment?"
Only the former is a papercut.

## How to file

1. Check the backlog for an equivalent entry first: `papercut list`.
2. Write a one- or two-sentence body: **what you were doing → what got in the way**,
   with a guess at the cause or fix as a bonus. Lead with the friction.
3. Run the CLI. The tool fills in who/what/when automatically.

```bash
# add is the default subcommand, so the message is the first positional
papercut "yarn web:test with a root-relative path finds no files; the workspace test cwd is apps/web" --tag tooling --severity minor
```

Flags:

- `--tag <area>` (repeatable) — e.g. `tooling`, `docs`, `build`, `scripts`.
- `--severity` — `minor` (annoyance, default) · `major` (time sink) · `blocker` (hard wall).
- `-m` / `--model`, `--harness`, `--user` — only if auto-detection got it wrong.
- Long bodies: pipe to stdin — `printf '%s' "<text>" | papercut add -`.

## Review or resolve

Only mine a whole session or do a broad review when the user explicitly asks.

When asked to review:

1. Re-run the worth-it test on every open entry; delete any that fail it
   (environment/shell/flake noise that slipped in).
2. Deduplicate and group related entries.
3. Verify each surviving papercut still reproduces.
4. Fix the smallest safe, high-leverage entries first.
5. Mark fixed items resolved: `papercut resolve <id>` (appends a resolved event).
   Route real bugs to normal issue/fix work.

## The CLI contract

- `papercut list` is **human-readable** by default (a `ts - model - user` header,
  then the body — no ids/severity/tags in the digest); use `papercut list --format json`
  for a data-only JSON envelope (which *does* include id, severity, and tags).
- Errors go to **stderr** with stable codes and documented exit codes; exit `0`
  on success including empty results. Run `papercut schema` for the full contract.

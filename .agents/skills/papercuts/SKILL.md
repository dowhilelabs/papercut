---
name: papercuts
description: "File a papercut when you hit genuine, recurring repo friction during work — a dead-end tool call, a broken link, a misleading doc, a footgun config, a missing helper. Also use to review, deduplicate, and resolve existing entries. Use the `papercut` CLI to log to the repo journal so the friction becomes a fixable backlog instead of silently pushing through."
---

# Papercuts

Capture small friction in the moment without derailing the current task. The
`papercut` CLI appends to `.papercuts.jsonl` at the repo root, so complaints
show up in git diffs and accumulate into a backlog that gets fixed periodically.

## The two-question test

File a papercut only if **both** are true:

1. **Reproducible for anyone.** A different person, on a fresh checkout, working
   in this repo would hit the same friction. It is not specific to your sandbox,
   shell config, machine, network, or a one-time hiccup.
2. **Fixable in the repo.** A change to the repo's code, config, scripts, or
   docs would prevent or reduce it.

If either answer is "no," push through and move on — do not log it.

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
2. Write a one- or two-sentence body: **what got in the way**, and the **smallest
   useful fix or workaround**. Lead with the friction, not with what you were doing.
3. Run the CLI. The tool fills in who/what/when automatically.

```bash
papercut add "yarn web:test with a root-relative path finds no files; the workspace test cwd is apps/web" --tag tooling --severity minor
```

Flags:

- `--tag <area>` (repeatable) — e.g. `tooling`, `docs`, `build`, `scripts`.
- `--severity` — `minor` (annoyance, default) · `major` (time sink) · `blocker` (hard wall).
- `--model` / `--harness` / `--user` — only if auto-detection got it wrong.
- Long bodies: pipe to stdin — `printf '%s' "<text>" | papercut add -`.

## Review or resolve

Only mine a whole session or do a broad review when the user explicitly asks.

When asked to review:

1. Re-run the two-question test on every open entry; delete any that fail it
   (environment/shell/flake noise that slipped in).
2. Deduplicate and group related entries.
3. Verify each surviving papercut still reproduces.
4. Fix the smallest safe, high-leverage entries first.
5. Mark fixed items resolved: `papercut resolve <id>` (appends a resolved event).
   Route real bugs to normal issue/fix work.

## The CLI contract

- `papercut list` is **human-readable** by default; use `papercut list --format json`
  for a data-only JSON envelope to parse programmatically.
- Errors go to **stderr** with stable codes and documented exit codes; exit `0`
  on success including empty results. Run `papercut schema` for the full contract.

# Hum Work Order 26: wordfreq — the first real program

Date: 2026-09-22
<!-- hum-active-workorder:v1 -->
Status: ACTIVE. Supersedes WO25 as the active Work Order; WO25's formal closure
record remains pending in the CI lane and is not part of this Work Order.

## Mission and present authorization

The mission is to write the first real Hum program: `examples/tools/wordfreq.hum`,
a word-frequency tool a human can actually run, composed exclusively from
already-implemented language features. No new language surface. The program
exercises capability-root file reads, bounded stdout, a fallible app entry,
and the recognized contract predicates (`list_count`, `list_len`, `old(...)`),
and it records every rough edge in a friction ledger.

This Work Order authorizes exactly one bounded session: write the program, its
test cases, and the friction ledger. It does not authorize checker or parser
changes, new syntax, new builtins, or any other language work. If the program
cannot be written honestly from the existing surface, the session stops and
reports; inventing surface is not an authorized repair.

## Scope

- New: `examples/tools/wordfreq.hum` — reads a text file through the
  capability root (`files.read` / `files_read_text`), splits text into words,
  counts frequencies, writes a bounded summary to stdout (`stdout.write` /
  `stdout_write`).
- The app entry follows the `app ... starts with: run_tool` shape and returns
  `Result` with typed errors (the `fallible_app_entry` pattern).
- Contracts use only recognized predicates: `list_count`, `list_len`,
  `old(...)`. Tasks carry `why:` / `needs:` / `ensures:` honesty locks.
- `test` blocks cover positive, boundary, and misuse cases; misuse must fail
  closed with typed errors.
- Friction ledger: `docs/research/wordfreq-friction-ledger.md`, every rough
  edge classified as program / tooling / documentation / open-language-question.
  `docs/research/` snapshots are always allowed.

## Acceptance criteria

1. `hum check` is clean on the file with honesty locks intact.
2. Positive, boundary, and misuse cases pass through `hum run`; misuse fails
   closed with typed errors (no panics, no stack traces).
3. `hum evidence` shows every `needs:` / `ensures:` obligation linked.
4. An explicit statement of what the evidence does NOT prove, recorded in the
   PR body and the completion record.
5. A friction ledger with every rough edge classified.
6. An explicit record that no language extension was required — or, if one is,
   the session stops and reports instead of inventing surface.

## Bans

- No checker changes. No parser changes. No new syntax.
- Interpreter fixes in `src/run.rs`, if any rough edge forces them, land as
  separate atomic commits with their own tests — never bundled with the
  program commit.
- If a language extension proves necessary to write wordfreq honestly, STOP
  and report to the BDFL. Do not invent surface.

## Deliverables

1. `examples/tools/wordfreq.hum` — the program.
2. Its `test` blocks — positive, boundary, misuse cases.
3. `docs/research/wordfreq-friction-ledger.md` — the classified friction ledger.

## Current authorization gate

BDFL-issued 2026-09-22: execute the single wordfreq session above, top to
bottom. Commit the Work Order issuance first, as its own atomic commit, before
any program work. Program and ledger follow as separate atomic commits.
Draft PR against `main` is the review gate; the BDFL merges. CI on the PR runs
the Full route (the `hum-full-validation` label) until the first scheduled
main validation exists. No other work is authorized under this Work Order.

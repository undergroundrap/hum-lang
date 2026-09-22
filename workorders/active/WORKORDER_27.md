# Hum Work Order 27: text_split primitive and wordfreq — the first real program

Date: 2026-09-22
<!-- hum-active-workorder:v1 -->
Status: ACTIVE. Supersedes WO26 as the active Work Order. WO26 closed with a
STOP record: word tokenization proved inexpressible from the Milestone 0
surface (see closure record in `workorders/closed/WORKORDER_26.md`).

## Mission and present authorization

Two-part mission, in order:

**Part 1 — the language increment.** Implement `text_split(text: Text, sep: Text)
-> List Text` per decision record 0021, with probes and positive, boundary,
and misuse fixtures. This is the minimal new surface that unblocks wordfreq:
one builtin, owned copies (not views), empty separator as a typed error,
explicit edge-case semantics (leading/trailing/repeated separators, absent
separator, empty input), honest `cost:` and `allocates:` declarations.

**Part 2 — the program.** Write `examples/tools/wordfreq.hum` on top of
`text_split`, meeting WO26's original acceptance criteria: `hum check` clean
with honesty locks intact; positive, boundary, and misuse cases through
`hum run`; misuse fails closed with typed errors; `hum evidence` links every
obligation; explicit statement of what the evidence does not prove; friction
ledger classifying every rough edge.

This Work Order authorizes the checker/interpreter changes for `text_split`
only. No other new builtins, no syntax changes, no parser changes beyond what
`text_split` requires (none expected).

## Scope

- New builtin `text_split` in the interpreter (`src/run.rs`) and its checker
  recognition, per decision 0021's normative semantics.
- Probes for `text_split` covering: basic split, leading separator
  (`["", "a", "b"]`), trailing separator (`["a", "b", ""]`), repeated
  separators (`["a", "", "b"]`), absent separator (`["abc"]`), empty input
  (`[""]`), empty separator (typed error `SepEmpty`).
- Misuse fixtures: empty separator fails closed with the typed error; no
  panics, no stack traces.
- `examples/tools/wordfreq.hum` — reads a text file through the capability
  root, splits into words with `text_split`, counts frequencies, writes a
  bounded summary to stdout. App entry follows the `app ... starts with:
  run_tool` shape, returns `Result` with typed errors.
- Contracts use only recognized predicates: `list_count`, `list_len`,
  `old(...)`. Tasks carry `why:` / `needs:` / `ensures:` honesty locks.
- Friction ledger: `docs/research/wordfreq-friction-ledger.md`, every rough
  edge classified as program / tooling / documentation / open-language-question.
  The single pre-existing entry (text tokenization inexpressible; research-note
  assumption falsified) is recorded as the session's first entry.

## Acceptance criteria

1. `text_split` probes pass: all six edge cases plus the misuse fixture.
2. `hum check` is clean on `wordfreq.hum` with honesty locks intact.
3. Positive, boundary, and misuse cases pass through `hum run`; misuse fails
   closed with typed errors (no panics, no stack traces).
4. `hum evidence` shows every `needs:` / `ensures:` obligation linked.
5. An explicit statement of what the evidence does NOT prove, recorded in the
   PR body and the completion record.
6. A friction ledger with every rough edge classified.
7. An explicit record that `text_split` was the only language extension
   required, with a pointer to decision 0021.

## Bans

- No views into the source text: `text_split` returns owned copies only.
  (Views would hit the internal-references ownership debt.)
- No general string library: no join, trim, case mapping, replacement, or
  other text builtins. `text_split` is the single tokenization primitive.
- No syntax changes. No parser changes.
- No weakening of `ensures:` to make dispatch pass.
- Interpreter changes land as separate atomic commits from the program
  commit, each with their own tests.

## Deliverables

1. `text_split` implementation + probes + fixtures.
2. `examples/tools/wordfreq.hum` — the program.
3. Its `test` blocks — positive, boundary, misuse cases.
4. `docs/research/wordfreq-friction-ledger.md` — the classified friction ledger.
5. `docs/LANGUAGE_REFERENCE.md` entry for `text_split` (signature, matching
   rule, edge cases, ownership and cost).
6. `docs/DIAGNOSTICS.md` entry for the `SepEmpty` error code.
7. Falsified-assumption record in `docs/research/hum-improvement-backlog-2026-09-22.md`
   (the `three_program_sequence` note lives in the research package, not the
   repo; the repo-side record goes in the backlog snapshot).

## Current authorization gate

Issued when the BDFL merges the PR carrying this Work Order. (Claude holds no
issuing authority; the decision and the Work Order take effect on merge.)

Execute the two-part mission above, in order:

- Gate 0: decision record 0021 is accepted — the BDFL's ruling recorded in the
  decision file — before any Part 1 code is written. Part 1 implements a
  proposed decision until that ruling lands.
- Part 1 (the builtin with its probes and fixtures) is reviewed before Part 2
  (the program) begins.
- The newline-separator dependency recorded in 0021 is resolved before Part 2
  begins.

Commit the Work Order issuance first, as its own atomic commit, before any
implementation work. Draft PR against `main` is the review gate; the BDFL
merges. CI on the PR runs the Full route (the `hum-full-validation` label)
until the first scheduled main validation exists. No other work is authorized
under this Work Order.

# Hum Work Order 27: text_split primitive and wordfreq — the first real program

Date: 2026-09-22
Status: CLOSED 2026-09-23. Superseded by WO28 (for-each loop-variable types
and wordfreq completion).

## Closure record

WO27's three-part mission is complete: `text_split` shipped per decision
0021 (Part 1a), text-literal escapes shipped per decision 0022 (Part 1b,
including the runner decode fix), and the program is written —
`examples/tools/wordfreq.hum` with positive, boundary, and misuse test
blocks, `hum check` clean at 0 errors, and the classified friction ledger
(`docs/research/wordfreq-friction-ledger.md`, 15 entries).

The program is written; app execution is gated on ledger #13: the `for each`
loop variable carries no type (`iterator_type_checking_not_implemented`), so
the full type-check gate refuses the APP entry
(`blocked_by_unchecked_body_types_v0`, `execution_ready=0`). wordfreq has
never run end-to-end as an app on any platform. The `--entry` paths
(`wordfreq_words`, `wordfreq_count`) run and pass; the Session AG corpus
assertion is pinned to the fail-closed refusal until #13 lands, then flips
back to the byte-exact stdout success check.

The frequency summary is blocked on ledger #5 / decision 0028: wordfreq
cannot emit conventional `word: count` lines — no concatenation and no
integer-to-text conversion exist in the language. Decision 0028 (accepted
2026-09-23, Option E) adopts `uint_to_text` / `int_to_text`, one builtin per
integer type; concatenation stays deferred until a friction entry
demonstrates the need. The honest output with today's surface is one word
per line.

Evidence: PR #24 green on the Full profile (run 35933087903 — Ubuntu
25.4 min, Windows 25.3 min), final review passed, merged 2026-09-23.

## Mission and present authorization

Three-part mission, in order:

**Part 1a — `text_split`.** Implement `text_split(text: Text, sep: Text)
-> List Text` per decision record 0021, with probes and positive, boundary,
and misuse fixtures. This is the minimal new surface that unblocks wordfreq:
one builtin, owned copies (not views), empty separator as a typed error,
explicit edge-case semantics (leading/trailing/repeated separators, absent
separator, empty input), honest `cost:` and `allocates:` declarations.

**Part 1b — text-literal escapes.** Implement escape sequences in text
literals per decision record 0022 (`\n`, `\t`, `\\`, `\"`, unknown escapes
a checker error). The newline probe for 0021 proved Hum literals cannot name
a newline, which blocks both the input side (`text_split` on lines) and the
output side (`stdout_write` of a line break). This is a language gap every
real program will hit, not a wordfreq quirk. Part 1b is reviewed before Part 2
begins.

**Part 2 — the program.** Write `examples/tools/wordfreq.hum` on top of
`text_split` and the 0022 escapes, meeting WO26's original acceptance criteria:
`hum check` clean with honesty locks intact; positive, boundary, and misuse
cases through `hum run`; misuse fails closed with typed errors; `hum evidence`
links every obligation; explicit statement of what the evidence does not prove;
friction ledger classifying every rough edge.

This Work Order authorizes the checker/interpreter changes for `text_split`
and the parser/canonical-AST change for the 0022 escapes, and nothing else.
No other new builtins, no other syntax changes.

## Scope

- New builtin `text_split` in the interpreter (`src/run.rs`) and its checker
  recognition, per decision 0021's normative semantics.
- Escape decoding in text literals at canonical-AST build time
  (`canonical_expression_build` in `src/parser.rs`), per decision 0022's
  normative semantics: the four escapes decode, unknown escapes and trailing
  backslash are checker errors. The canonical-seal change triggers the
  Exhaustive evidence route in CI.
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
- No syntax changes and no parser changes beyond the 0022 escape decoding,
  which is the single authorized exception to this ban.
- No weakening of `ensures:` to make dispatch pass.
- Interpreter changes land as separate atomic commits from the program
  commit, each with their own tests.

## Deliverables

1. `text_split` implementation + probes + fixtures.
2. Part 1b deliverable: the 0022 escape implementation + fixtures (each
   escape decodes; unknown escapes and trailing backslash fail at check
   time; no-escape literals unchanged; quote-scanner agreement fixture;
   emitter-escaping fixture through `hum evidence` or `hum graph`).
3. `examples/tools/wordfreq.hum` — the program.
4. Its `test` blocks — positive, boundary, misuse cases.
5. `docs/research/wordfreq-friction-ledger.md` — the classified friction ledger.
6. `docs/LANGUAGE_REFERENCE.md` entries for `text_split` (signature, matching
   rule, edge cases, ownership and cost) and for the 0022 escape table.
7. `docs/DIAGNOSTICS.md` entries for the `SepEmpty` error code and the
   unknown-escape error code.
8. Falsified-assumption record in `docs/research/hum-improvement-backlog-2026-09-22.md`
   (the `three_program_sequence` note lives in the research package, not the
   repo; the repo-side record goes in the backlog snapshot).

## Current authorization gate

Issued when the BDFL merges the PR carrying this Work Order. (Claude holds no
issuing authority; the decision and the Work Order take effect on merge.)

Execute the three-part mission above, in order:

- Gate 0: decision records 0021 and 0022 are both accepted — the BDFL's ruling
  recorded in each decision file — before any Part 1 code is written. Part 1
  implements proposed decisions until those rulings land. 0021 is accepted
  (BDFL ruling 2026-09-22 on the review at 5c44b91); 0022 awaits the BDFL's
  ruling.
- Part 1 (the builtin with its probes and fixtures) is reviewed before Part 2
  (the program) begins.
- The newline-separator dependency recorded in 0021 is resolved before Part 2
  begins.

Commit the Work Order issuance first, as its own atomic commit, before any
implementation work. Draft PR against `main` is the review gate; the BDFL
merges. CI on the PR runs the Full route (the `hum-full-validation` label)
until the first scheduled main validation exists. No other work is authorized
under this Work Order.

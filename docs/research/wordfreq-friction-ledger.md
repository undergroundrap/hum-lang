# wordfreq friction ledger

Session: WO27 Part 2 — `examples/tools/wordfreq.hum` (2026-09-23).
Rule: decision 0027 — real programs drive language design; missing surface is
a STOP, never a workaround; ask whether a gap is the GENERAL fix or
wordfreq-specific before proposing anything.

Every rough edge hit while writing, testing, or evidencing wordfreq,
classified as program / tooling / documentation / open-language-question.

## 1. Text tokenization inexpressible in Milestone 0 (carried entry)

- **Class:** open-language-question → resolved by decision 0021.
- **Friction:** WO26's STOP: no way to split text into words without a
  tokenization primitive.
- **Resolution:** decision 0021 added `text_split` (exact-substring splitter,
  owned copies). wordfreq is built on it. Recorded here as the session's
  first entry per WO27.

## 2. Newline inexpressible in text literals (carried entry)

- **Class:** open-language-question → resolved by decision 0022.
- **Friction:** wordfreq must write `"\n"` after each word; without escapes
  there is no newline literal.
- **Resolution:** decision 0022 decodes `\n` (and `\t`, `\\`, `\"`) at
  canonical-AST build time; unknown escapes are H0638 checker errors.

## 3. Runner did not decode text-literal escapes (found by wordfreq, fixed)

- **Class:** tooling (interpreter) — implementation gap in accepted decision
  0022, not new surface.
- **Friction:** `text_split(text, "\n")` did not split on real newlines and
  `stdout_write("\n")` would have written backslash-n: the tree-walking
  runner's `eval_expr` took the literal's inner source text verbatim instead
  of decoding. The canonical AST decoded correctly; the runner re-parses
  source text and skipped the decode.
- **Fix (general, not wordfreq-specific):** `run.rs` now decodes every text
  literal through `parser::decode_text_escapes`. A decode failure in the
  runner fails closed as an internal error (unreachable: `hum run` refuses
  to execute when H0638 reports errors).
- **Tests:** `run::tests::runner_text_literal_newline_escape_decodes`;
  `examples/probes/decoded_newline_probe.hum` with byte-level stdout
  assertions in `tools/check_all.ps1` on both platforms.

## 4. Contract text literals: no H0638 coverage, no decode (found by wordfreq, open)

- **Class:** open-language-question (checker + interpreter).
- **Friction:** while fixing #3, probing showed `ensures: result == "a\qb"`
  (invalid escape) passes `hum check` with no H0638, and the contract
  predicate parser (`predicate.rs`) takes literal text verbatim. So contract
  literals neither reject bad escapes nor decode good ones.
- **Why not fixed here:** fixing it means extending H0638 to contract
  sections and adding a decode-failure diagnostic to the predicate parser —
  checker work beyond wordfreq's authorization. Recorded for a follow-up;
  wordfreq uses no text literals in contracts.

## 5. No way to render `word: count` lines (open)

- **Class:** open-language-question.
- **Friction:** wordfreq cannot emit conventional `hum 3` summary lines:
  there is no text concatenation, no formatting/interpolation, and no
  UInt-to-Text conversion in the language. The honest output with today's
  surface is one word per line, one line per occurrence (a histogram); the
  count is exposed as a pure `wordfreq_count` task instead.
- **0027 question:** is concatenation/interpolation the GENERAL fix, or is
  the histogram an acceptable program shape? Not decided here — no surface
  invented.

## 6. Single-separator `text_split` forces N passes (open)

- **Class:** open-language-question (program shape).
- **Friction:** tokenizing on newlines then spaces needs two passes; tabs and
  carriage returns are not split (a `\t` inside a word stays glued). A
  realistic tokenizer wants a separator set or character class.
- **0027 question:** is a multi-separator form the GENERAL fix, or is
  single-separator exact-substring the right primitive and multi-pass the
  right program shape? Not decided here.

## 7. `files_read_text` is Windows-only (open)

- **Class:** tooling / platform.
- **Friction:** the positive file-read path (`hum run wordfreq.hum --args
  <file>`) can only execute on Windows. On Linux/macOS the grant itself is
  rejected (`native_path_input_unavailable_on_non_windows_v0`) and
  `files_read_text` would raise typed `FileReadError.unavailable`. CI's
  Ubuntu leg and local Linux runs can only assert the typed fail-closed
  behavior, never the success path.
- **Consequence for evidence:** the byte-level stdout proof for wordfreq's
  own `"\n"` writes runs on Windows only; other platforms prove the same
  decoder byte through `decoded_newline_probe` (no file capability needed).

## 8. No JSON output carries the decoded `TextLiteral` value (open)

- **Class:** tooling.
- **Friction:** evidencing "the decoded newline survives JSON emission"
  could not use any CLI output: `hum graph` / `core-preview` show the
  SOURCE spelling (`a\nb` with backslash-n), and the canonical payload's
  `TextDecodedValue` is in-memory only (it feeds the seal, no emitter).
  The precise test had to be a Rust unit test composing the production
  decoder with the production JSON escaper
  (`parser::tests::decoded_text_literal_value_json_escapes_newline`).
- **0027 question:** should a JSON surface expose canonical payload values,
  or is the seal + unit test the right place for this invariant? Not
  decided here.

## 9. App start tasks must return `Unit` (by design, noted)

- **Class:** program (design constraint, not friction).
- **Friction:** none — H0616 requires `app ... starts with: run_tool` to
  return `Unit` / `Result Unit, E`. wordfreq's word count therefore lives in
  the pure `wordfreq_count` task (probed via `--entry`) rather than as the
  app's return value. Recorded so the shape choice is explicit.

## 10. `stdout_write` writes exact bytes, no newline mode (by design, noted)

- **Class:** program (design constraint, not friction).
- **Note:** `stdout.write` "writes exact UTF-8 bytes immediately with no
  newline" (0022 closed the gap by making `"\n"` expressible). wordfreq
  writes each word then an explicit `"\n"` — two bounded writes per word.
  The byte-level evidence (item 3 of the required Part 2 evidence) asserts
  on these exact bytes.

## 11. Honesty-lock warnings H0107/H0109 on the app tasks (noted)

- **Class:** documentation (checker guidance).
- **Note:** `hum check` on `wordfreq.hum` reports 0 errors and 6 warnings
  (H0107 missing `needs:`, H0109 missing `ensures:` on `run_tool`,
  `wordfreq_words`, and `write_word`) — the same warnings the model probes
  (`exact_file_read.hum`, `text_split.hum`) carry. "Clean" per WO27 AC #2
  means zero errors; the warnings are the checker's standing nudge, not
  filler to silence.

## Record required by WO27 AC #7

`text_split` was the only language extension required for wordfreq, per
decision 0021 (`docs/decisions/0021-text-split.md`). Decision 0022's escape
decoding was the other authorized change; the runner fix in #3 completes
0022's implementation rather than extending the language. No new builtins,
no new syntax, no general string library.

## 12. Resolver did not recognize list builtins in app scope (found by wordfreq, fixed)

- **Class:** tooling (interpreter) — implementation gap, not new surface.
- **Friction:** `list_append` / `list_len` / `list_count` resolved at file
  level but raised H0601 inside `app` tasks: the resolver's app-visible
  builtin list (`resolve.rs` `builtin_callee`) named only `stdout_write`,
  `clock_replay_tick`, `files_read_text`, `text_split`. An app — the
  structural shape for real programs — could not build or measure a list.
- **Fix (general, not wordfreq-specific):** added the three list builtins to
  `builtin_callee` with `session_z_list_builtin_v0` reasons. No new
  builtins; the runner already implemented them.
- **Tests:** `hum resolve` / `full-type-check` on `wordfreq.hum` (0 resolver
  errors); `resolve::tests` for the new builtin targets.

## 13. `for each` loop variables carry no type; builtins need `Text` (workaround, open)

- **Class:** open-language-question (type checker).
- **Friction:** `for_each_header` is `unchecked_statement_type_v0`
  (`iterator_type_checking_not_implemented`), so a loop variable over
  `List Text` has unknown type. `stdout_write(word)` and
  `text_split(line, " ")` are rejected (H0622, H0636) because the builtins
  require a checked `Text` argument.
- **Workaround used:** route loop variables through helper tasks with typed
  parameters (`write_word(word: Text)`, `split_spaces(line: Text)`). The
  parameter type carries the proof the loop header cannot supply. This is a
  program-shape workaround, not new surface.
- **0027 question:** is loop-variable type inference the GENERAL fix? Not
  decided here.

## 14. `try` requires unannotated `let` bindings (by design, noted)

- **Class:** program (design constraint, not friction).
- **Note:** `let wrote: Unit = try ...` is rejected (H0906
  `try_requires_unannotated_let_binding_v0`); Session W's exact form is
  `let wrote = try ...`. wordfreq uses unannotated bindings for all `try`
  lets. Recorded because the annotated form is the natural first attempt.

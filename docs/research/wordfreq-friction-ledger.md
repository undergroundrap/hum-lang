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

## 13. `for each` loop variables carry no type; the APP entry cannot execute (RESOLVED, WO28 #13)

- **Class:** open-language-question (type checker).
- **Friction:** `for_each_header` is `unchecked_statement_type_v0`
  (`iterator_type_checking_not_implemented`), so a loop variable over
  `List Text` has unknown type. `stdout_write(word)` and
  `text_split(line, " ")` are rejected (H0622, H0636) because the builtins
  require a checked `Text` argument.
- **Workaround used (partial):** route loop variables through helper tasks
  with typed parameters (`write_word(word: Text)`, `split_spaces(line: Text)`).
  The parameter type carries the proof the loop header cannot supply. This
  unblocked the `--entry` paths (`wordfreq_count`, `wordfreq_words` run and
  pass), but it does NOT unblock the APP entry.
- **BLOCKING (was):** `hum run` of the APP entry (`run_tool`) is refused by the
  full type-check gate: `recognized_core_body_type_gate_v0`,
  `status: blocked_by_unchecked_body_types_v0`, `unchecked_statements=10`,
  `execution_ready=0` (validation run 35918468834, Windows; reproduced on
  Linux — the gate is platform-independent). The app path runs nowhere
  today: Windows refuses at the gate, non-Windows never reaches it
  (`files_read` grant unavailable, ledger #7). Consequence: wordfreq has
  never run end-to-end as an app on any platform. The program is written;
  app execution is gated on this entry.
- **RESOLVED (WO28 #13):** `for_each_binding` infers the loop variable's
  type from the iterated expression — `for each word in words` where
  `words: List Text` binds `word: Text`. Non-list and unprovable shapes
  stay `unchecked_statement_type_v0`
  (`iterator_type_checking_not_implemented`), fail-closed. Two adjacent
  completions the acceptance command required: `list_len(...)` call
  expressions type as `UInt` in return position, and `test_expectation`
  accepts when the callee's declared return type matches the `returns`
  literal shape (mismatches stay unchecked). Collateral fix:
  `place_type_fact` no longer snake-normalizes a whole condition expression
  to a bound name — `if piece != ""` was rejected; it is now accepted via
  the inferred `Bool`. Local evidence (Linux): `hum full-type-check` on
  `wordfreq.hum` is `recognized_core_body_types_checked_v0` with zero
  blocking issues; `hum run` of the APP entry reaches the platform
  boundary (`native_path_input_unavailable_on_non_windows_v0`, exit 2) —
  the type gate is open. The Session AG assertion is flipped back to the
  byte-exact stdout success check (digest pin recomputed). Final proof of
  the done-condition (exit 0, 13 bytes, no CR) rests with Windows CI.
- **Disproven hypothesis:** the run-35914248940 Windows Session AG failure
  was first attributed to a mixed-separator fixture path
  (`Join-Path $RepoRoot 'fixtures/wordfreq/sample.txt'` keeping forward
  slashes). Run 35918468834's enriched stderr showed the gate refusal, never
  a path error. The path was never the cause; per-segment joins remain only
  as canonical form.
- **0027 question:** answered — loop-variable type inference is the GENERAL
  fix. WO28 orders it first with done-condition "wordfreq runs end-to-end
  on Windows", flipping the Session AG assertion back to the byte-exact
  stdout success check.

## 14. `try` requires unannotated `let` bindings (by design, noted)

- **Class:** program (design constraint, not friction).
- **Note:** `let wrote: Unit = try ...` is rejected (H0906
  `try_requires_unannotated_let_binding_v0`); Session W's exact form is
  `let wrote = try ...`. wordfreq uses unannotated bindings for all `try`
  lets. Recorded because the annotated form is the natural first attempt.

## 15. `hum check` accepts contract-only `list_count` in task bodies (found by probe, open)

- **Class:** tooling (checker) — fail-closed gap.
- **Friction:** `list_count` is contract-only Predicate v2 vocabulary
  (LANGUAGE_REFERENCE; `run.rs` refuses it at runtime with "list_count is
  contract-only Predicate v2 vocabulary"). But `hum check` accepts a task
  body that calls it — at module scope and at app scope. Probed 2026-09-23
  with the merged main: both `check` runs exit 0 with 0 errors, while
  `hum run` of the same file traps at runtime (exit 2). The resolver admits
  it as `builtin_reference_v0` (`session_z_list_builtin_v0`) since the WO27
  Part 2 app-scope fix (#12); the checker has no rule rejecting
  contract-only builtins in executable bodies, so check-time and run-time
  disagree.
- **Fix (general, not list_count-specific):** the checker must reject any
  contract-only builtin in a task body with a typed diagnostic. WO28 orders
  this as #15, after #4.

## 16. Block-scoped bindings leak in the checker and the runtime (found by probe, open)

- **Class:** language (checker + runtime) — soundness gap, pre-existing (not
  introduced by WO28 #13).
- **Friction:** the resolver treats `if` blocks and `for each` headers as
  scopes — a name bound inside one is `H0601` not-visible after the block
  closes. But the full-type-check statement environment is flat per task: a
  `let` inside an `if` block, and a `for each` loop variable, both persist
  after the block and clobber outer same-name facts. Probed 2026-09-23
  (WO28 #13 review): `let x = 3` then `if flag { let x = "s" }` then
  `return x` in a `-> Text` task is ACCEPTED with `actual=Text` — a wrong
  acceptance, since the resolver says `x` there is the outer `UInt` 3 — and
  `hum run` returns `"s"`, a wrong-value execution from a dead scope.
  Checker and runtime agree with each other and disagree with the
  language's scoping rule. The `for each` binder behaves identically to the
  existing block-scoped `let`s (probe: `let word = 3` + `for each word in
  words` reports the leaked binder type after the loop), so #13 stands as
  published; the gap predates it.
- **Fix (general):** block scoping must match the resolver in both stages —
  per-block environments in the checker (restore shadowed facts at block
  close) and properly scoped `if`-block lets in the runtime, as the runtime
  already scopes `for each` binders. Tests: leak and shadow probes across
  `if` and `for each`; the probe program above must be rejected at check
  time. WO28 orders this as #16, after #15, before #7.

## 17. Hosted-runner disks are refused by the drive-locality policy (found by probe, open)

- **Class:** platform (hosted runners) — known limitation, pending decision 0029.
- **Finding:** WO28 #13 worked: the wordfreq APP entry now runs past the
  full type-check gate on Windows (before #13 the run was refused at the
  gate, `blocked_by_unchecked_body_types_v0`, exit 2). The app entry
  executes and reaches the file read — then the read is refused with
  `FileReadError.unavailable` (exit 1, typed `WordfreqError.read` chain on
  stderr). Cause: `crates/windows-drive-locality/src/lib.rs:204` classifies
  a drive as fixed-local only for ATA/SATA/NVMe bus types; GitHub's Azure
  runner disks are virtual/SCSI, so the drive classifies
  `DriveLocality::Unknown` → `NativePathLocality::Unclassified` and the
  fixed-local-not-proven branch fires
  (`fixed_local_v0_not_proven_before_candidate_access_v0` in `run.rs`).
- **Honest #13 done-condition:** "wordfreq's app entry executes end-to-end
  through the type gate; the byte-exact success read is blocked on hosted
  runners by the locality policy (decision 0029 pending)". The Session AG
  assertion pins this refusal. The locality crate is a security policy and
  changes only by decision — not touched.

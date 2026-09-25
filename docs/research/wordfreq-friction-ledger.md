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

## 4. Contract text literals: no H0638 coverage, no decode (found by wordfreq, resolved)

- **Class:** open-language-question (checker + interpreter) → resolved by WO28 #4.
- **Friction:** while fixing #3, probing showed `ensures: result == "a\qb"`
  (invalid escape) passes `hum check` with no H0638, and the contract
  predicate parser (`predicate.rs`) takes literal text verbatim. So contract
  literals neither reject bad escapes nor decode good ones.
- **Why not fixed here:** fixing it means extending H0638 to contract
  sections and adding a decode-failure diagnostic to the predicate parser —
  checker work beyond wordfreq's authorization. Recorded for a follow-up;
  wordfreq uses no text literals in contracts.
- **Resolution (WO28 #4):** `predicate.rs` now decodes contract text literals
  through the production decoder (`parser::decode_text_escapes`), so contract
  text means the same thing as body text. Invalid escapes are H0638 errors
  (new predicate cause key 186, `invalid_text_escape_in_contract_v2`) with
  the span on the bad escape; a trailing backslash in a complete-looking
  literal reports H0638 rather than an unterminated-literal error, mirroring
  the body literal precedence (decision 0022). Note: `hum check` still
  reports nothing for contract predicates — it never ran predicate analysis
  (pre-existing stage boundary, unchanged by #4). The end-to-end evidence is
  `hum run`'s preflight, which rejects `ensures: result == "a\qb"` with
  H0638 (exit 2); runtime probes show a contract asserting `"a\tb"` holds
  against a body returning `"a\tb"` and a `"a\\tb"` contract holds against
  a `"a\\tb"` body (decode parity — under the old verbatim behavior the
  first would have failed), while a mismatched contract still fails with
  H0703.

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

## 16. Block-scoped bindings leak in the checker and the runtime (found by probe, resolved)

- **Class:** language (checker + runtime) — soundness gap, pre-existing (not
  introduced by WO28 #13) → resolved by WO28 #16.
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
- **Resolution (WO28 #16):** the full-type-check statement environment
  (`src/full_type_check.rs`) is now a scope stack shared with the return
  checker (`src/type_check.rs`) via `src/type_scopes.rs` — block headers
  push a scope, `block_close` pops it, and the for-each binder lands in the
  pushed scope. The runtime (`src/run.rs`) scopes `if`-block and for-each
  body `let`/`change` bindings, restoring outer values on every path.
  Probe-3 is now rejected with `type_errors=1`; the per-statement fact for
  `return x` shows `actual=integer_literal` (outer UInt), not Text. The
  binder probe (`let word = 3` + `for each word in words`) is rejected with
  the return fact showing the outer type. Session AB asserts all of these
  plus runtime probes returning the outer values.

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

## 18. `hum check` never runs predicate analysis — contract errors are invisible to the agent loop (pre-existing, open)

- **Class:** tooling / stage boundary, pre-existing (not introduced by WO28 #4).
- **Friction:** `hum check` reports nothing for contract predicates — it never
  runs predicate analysis. All contract errors (the H0704 family and the new
  H0638 contract-escape case from #4) surface only in `hum full-type-check`
  (human and JSON now render the predicate diagnostic's own code, #4) and in
  `hum run`'s preflight (exit 2 with the accepted-escapes repair). Entry #4
  noted the fact while resolving the escape gap; this entry records the
  consequence.
- **Why it matters:** the primary agent loop is `hum check --format json` — a
  fast, cheap, structured signal agents run repeatedly. Anything only
  full-type-check/run can see is effectively invisible during iteration: an
  agent writing a contract with an invalid escape sees green checks until it
  runs the much slower, capability-demanding full pipeline. #4 changed this
  boundary nothing.
- **0027 question:** is predicate analysis in `check` the GENERAL fix, or is
  the cheap-check / full-pipeline boundary the right shape and the docs the
  right fix? Not decided here — no surface invented.

## 19. A user task named `list_count` still gets builtin treatment (found by probe, open)

- **Class:** open-language-question.
- **Finding:** WO28 #15 rejects contract-only builtin calls (e.g.
  `list_count`) in task bodies with H0639, driven by the shared inventory
  in `resolve.rs`. A probe confirms that a user-defined `task list_count`
  does NOT shadow the builtin: calls to it in task bodies are still
  rejected with H0639, because builtin recognition precedes user-definition
  resolution. The task can be declared, but it cannot be called from a body.
- **Unmade choice:** whether to reserve `list_count` (and future
  contract-only builtins) like H0637 reserves `text_split`, `stdout_write`,
  etc. — i.e., reject the declaration itself with a reserved-name
  diagnostic — or to allow user shadowing. Nothing was changed; the probe
  only records the current behavior. This needs a semantic decision before
  any reservation surface is built.

## 20. The return checker has no for-each binder typing; the two checkers disagree on types inside loops (found during #16 review, open)

- **Class:** checker / stage disagreement, pre-existing (not introduced by
  WO28 #16).
- **Finding:** WO28 #16's review found that the binder-shadow fail fixture
  does not prove the binder fix: `src/type_check.rs` (the return checker)
  contains no for-each binder handling at all (no `for_each`/binder
  anywhere in the file), so the return checker never sees the binder and
  `type_errors=1` on that fixture happens with or without #16. The actual
  leak lived in `full_type_check.rs`'s statement environment, which a
  type-error fixture cannot observe under the restored stage precedence.
- **Why it matters:** `full_type_check.rs` types the for-each binder (from
  WO28 #13), while `type_check.rs` does not — the two checkers still
  disagree on types inside loops. The disagreement is unobservable in the
  current fixtures only because the shared `TypeScopeStack` in #16 keeps
  the statement env honest; nothing in `type_check.rs` was changed.
- **Recorded only; not fixed.** Any fix needs a semantic decision first
  (decision 0027: missing surface is a STOP, gaps become decision records
  before implementation).

## 21. User-task calls have no static diagnostics for arity/type/sign misuse (found by probe, open)

- **Class:** checker gap, pre-existing (not introduced by decision 0028).
- **Finding:** probing the three misuse shapes against a user task
  `task f(n: UInt) -> Text` shows `hum full-type-check` accepts all three
  with zero diagnostics (`rejected_statements=0`, `blocking_issues=0`):
  `f(-5)`, `f(1, 2)`, and `f("3")`. At runtime, `f(1, 2)` traps
  (`task 'f' expects 1 argument(s), got 2`) while `f("3")` and `f(-5)`
  silently complete. So the checker enforces the signature nowhere; arity
  is fail-closed only at runtime, and type/sign mismatches are not caught
  at all.
- **Why it matters for decision 0028:** the BDFL ruling is that the new
  conversion builtins must behave exactly like a user task with the same
  signature. With no existing diagnostic to map to, no new H-code may be
  allocated (STOP); `uint_to_text`/`int_to_text` misuse is checker-accepted
  and runtime-trapped, exactly like the user-task shapes above. The
  previously allocated H0640-H0643 were removed for this reason.
- **Recorded only; not fixed.** Any static checking of call shapes needs a
  semantic decision first (decision 0027: missing surface is a STOP).

## 22. Negative values reach `UInt` positions through the runtime representation gap (found by probe, open)

- **Class:** runtime representation gap, pre-existing (not introduced by
  decision 0028).
- **Finding:** integers are `Value::Int` at runtime, and negatives flow
  into `UInt` positions unchecked: `let x: UInt = -5` is left unchecked by
  the checker (`unchecked_statement_type_v0`,
  `expression_type_unknown_v0`, blocking but not rejected), and
  `task f(n: UInt)` called with `f(-5)` binds `-5` to `n`. Both reach
  `uint_to_text(x)` / `uint_to_text(n)` as a negative `Value::Int`.
- **Minimal repro** (`uint_to_text_negative.hum`):
  ```hum
  task main() -> Text {
    does:
      let x: UInt = -5
      return uint_to_text(x)
  }
  ```
  Before the decision-0028 rework this rendered the silent lie `"-5"`.
  Per the BDFL ruling the builtin now traps with a clear invariant
  violation instead:
  `uint_to_text invariant violation: received negative value -5, but UInt cannot be negative`.
  The checker-level infallibility of decision 0028 depends on `UInt`
  never being negative; the trap is the runtime's fail-closed backstop
  while the gap itself stays open.
- **Recorded only; not fixed.** Closing the representation gap (stopping
  negatives at `UInt` positions) needs a semantic decision first.

## 23. `return -9223372036854775808` panics the parser (found by probe, open)

- **Class:** parser panic, pre-existing on main.
- **Repro:** checking or running
  ```hum
  task main() -> Int {
    does:
      return -9223372036854775808
  }
  ```
  panics the compiler process:
  `thread 'main' panicked at src/parser.rs:2272:14: parser H0010 visitor
  requires a valid sealed canonical occurrence:
  "canonical_occurrence_authority_mismatch_v0"`.
- **Why it matters:** a panic violates fail-closed. Whatever is wrong with
  the minimum-`Int` literal, the correct behavior is a diagnostic, never a
  process panic. `hum run` on unchecked code must still fail closed; a
  panic is the one failure mode the honesty locks forbid.
- **Recorded only; not fixed.** The parser fix needs its own session; no
  surface invented here.

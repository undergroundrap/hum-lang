# Hum Work Order 28: for-each loop-variable types and wordfreq completion

Date: 2026-09-23
<!-- hum-active-workorder:v1 -->
Status: ACTIVE. Supersedes WO27 as the active Work Order. WO27 closed with
the program written; app execution gated on ledger #13 (see closure record
in `workorders/closed/WORKORDER_27.md`).

## Mission and present authorization

Finish what wordfreq exposed, in dependency order. Each item is a
review-sized unit; a later item does not begin until the earlier one's
acceptance criteria are met and reviewed.

**#13 — `for each` loop-variable type inference.** Infer the loop variable's
type from the iterated expression: `for each word in words` where
`words: List Text` binds `word: Text`. Existing semantics are sufficient —
a `for each` over `List T` binds `T` — so no decision record is required.
If new language surface or an unmade semantic choice proves necessary, STOP
and report to the BDFL instead of inventing it.
Done-condition: wordfreq's APP entry runs end-to-end on Windows —
`hum run examples/tools/wordfreq.hum --allow stdout.write
--allow=files.read=fixtures/wordfreq/sample.txt --args
fixtures/wordfreq/sample.txt` exits 0 with the byte-exact expected stdout
(13 bytes, no CR). Then flip the temporary fail-closed Session AG assertion
in `tools/check_all.ps1` back to the byte-exact stdout success check (pin
update with recomputed digest, reviewed at final review).

**#4 — contract text-literal decode and H0638 coverage.** Ledger #4:
contract literals (`needs:` / `ensures:`) neither reject invalid escapes
with H0638 nor decode valid ones — the predicate parser takes literal text
verbatim while body literals decode. Extend H0638 to contract sections and
decode contract text literals through the production decoder, so contract
text means the same thing as body text.

**#15 — checker rejects contract-only builtins in bodies.** Probe
2026-09-23: `hum check` accepts `list_count` in task bodies at module scope
and at app scope (0 errors in both), while `hum run` traps at runtime
("list_count is contract-only Predicate v2 vocabulary"). The resolver admits
it as a builtin (`session_z_list_builtin_v0`); the checker has no rule
against contract-only vocabulary in executable bodies. The general fix: the
checker rejects any contract-only builtin in a task body with a typed
diagnostic, so check-time and run-time agree. Not list_count-specific.

**#16 — block scoping must match the resolver.** Ledger #16: the resolver
treats `if` blocks and `for each` headers as scopes, but the full-type-check
statement environment is flat per task (block bindings persist and clobber
outer same-name facts) and the runtime leaks `if`-block `let`s (the runtime
already scopes `for each` binders correctly). Observed wrong acceptance:
`let x = 3`, `if flag { let x = "s" }`, `return x` in a `-> Text` task is
accepted with `actual=Text` and runs to `"s"`. The general fix: per-block
environments in the checker (restore shadowed facts at block close) and
scoped `if`-block lets in the runtime. Tests for leak and shadow across `if`
and `for each`; the probe program above must be rejected at check time. This
is a correctness item, not new language surface — the resolver already
defines the rule; the checker and runtime must follow it. Executes after
#15's review, before #7.

**#7 — portable non-Windows file read.** Ledger #7: the `files_read_text`
positive path is Windows-only; other platforms reject the grant
(`native_path_input_unavailable_on_non_windows_v0`), so the wordfreq
success path is provable nowhere but Windows. Define and implement the
portable non-Windows file read so the success path is provable on all
platforms. If implementing portability requires new language surface or an
unmade semantic choice, STOP and report to the BDFL instead of inventing
it.

**Optional — wordfreq prints the frequency summary.** Once #13 and
decision 0028 land, wordfreq prints a `word: count` summary using nested
loops, with the quadratic cost declared honestly in `cost:` / `allocates:`
and a perf-debt note. If the loops prove awkward to express, record that
as a friction ledger entry instead of contorting the program.

**Decision 0028 — integer-to-Text conversion.** Implement `uint_to_text` /
`int_to_text`, one builtin per integer type, per decision 0028 Option E
(accepted 2026-09-23). Concatenation stays deferred until a friction entry
demonstrates the need. Per 0028's correction ("C alone solves wordfreq's
output"), output composes by sequential writes — `stdout_write(word);`
`stdout_write(": ");` `stdout_write(uint_to_text(n));` `stdout_write("\n")`
— so the conversion builtins DO enable printing conventional `word: count`
lines; concatenation is needed only for text as a VALUE. What still blocks
a real frequency summary is computing per-word counts without a
map/dictionary type: quadratic nested loops (per 0028, out of scope — a
future map/dictionary design question), which also need #13.

## Scope

- Checker work for #13 (loop-variable inference), #4 (contract literal
  decode / H0638), #15 (contract-only builtin rejection in bodies), and
  #16 (block scoping in the checker).
- Runtime work for #16 (block-scoped `if` lets), and platform work for #7
  as that item defines it.
- Two new builtins for decision 0028 (`uint_to_text`, `int_to_text`) with
  probes and fixtures.
- The Session AG assertion flip in `tools/check_all.ps1` when #13 lands.

## Bans

- No other new builtins, no other syntax changes. Decision 0028's two
  conversion builtins are the only authorized additions.
- No weakening of `ensures:` to make dispatch pass.
- If any item needs new language surface or an unmade semantic choice,
  STOP and report to the BDFL. Do not invent surface.

## Acceptance criteria

1. #13: wordfreq APP entry runs end-to-end on Windows, byte-exact stdout;
   Session AG assertion flipped back to the success check; both platforms
   green.
2. #4: invalid escapes in contracts are H0638 errors; valid escapes decode
   identically in contracts and bodies (probe).
3. #15: `hum check` rejects `list_count` — and any contract-only builtin —
   in task bodies at both scopes with a typed diagnostic; `hum run`
   behavior stays fail-closed (now unreachable through checked code).
4. #16: block-scoped bindings match the resolver in the checker and the
   runtime — per-block checker environments restoring shadowed facts at
   block close, `if`-block lets scoped in the runtime; leak and shadow
   probes across `if` and `for each` pass, and the probe-3 program is
   rejected at check time; both platforms green.
5. #7: the wordfreq success path is provable on non-Windows.
6. Decision 0028: `uint_to_text` / `int_to_text` probes pass; wordfreq
   prints `word: count` lines via sequential writes. The full frequency
   summary (unique words and counts) remains blocked on quadratic nested
   loops without a map type — 0028 out of scope, a separate ledger item.
7. (Optional) Frequency summary: wordfreq prints `word: count` lines from
   nested loops; quadratic cost declared honestly in `cost:` / `allocates:`
   with a perf-debt note, or a ledger entry if it proves awkward.

## Deliverables

1. The six ordered items, each as review-sized atomic commits with tests;
   the optional frequency-summary item, if taken, gets its own commit too.
2. Friction ledger entries as items resolve or expose more.
3. `docs/LANGUAGE_REFERENCE.md` and `docs/DIAGNOSTICS.md` entries for every
   new diagnostic and builtin.

## Current authorization gate

Issued when the BDFL merges the PR carrying this Work Order.

Execute the items in order, #13 first: each item is reviewed before the next
begins. Draft PR against `main` is the review gate; the BDFL merges. No other
work is authorized under this Work Order.

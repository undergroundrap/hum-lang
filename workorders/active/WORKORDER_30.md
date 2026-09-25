# Hum Work Order 30: the checker enforces declared task signatures

Date: 2026-09-25
Status: DRAFT — pre-issuance review. Not active. WO28 remains the active Work
Order. (The active-workorder marker comment is intentionally absent from this
file; it is added only when this Work Order is activated.)

## Authorization

No new decision record is required. This Work Order is a correctness item,
not new surface: a task's declared parameter list already defines the rule
for every call to it — how many arguments, of which types. The checker
enforcing that rule is the same reasoning as WO28 #16 (the resolver already
defined block scoping; the checker had to match it). Decision 0027's
general-vs-specific test resolves to the GENERAL fix: one call-shape check
driven by signatures for every callable, never per-builtin codes.

## Mission and queue position

Teach the checker to reject, at check time, call shapes the declared
signature already forbids: wrong arity, wrong argument type, and statically
known negative values in `UInt` positions. Make the builtins
(`uint_to_text`, `int_to_text`, `text_split`, `list_len`) subject to the same
single check via a builtin signature table. Turn the parser panic on the
exact `i64::MIN` literal spelling into a diagnostic.

Queue: this Work Order executes **after WO28 closes, ahead of WO29** —
Ocean's call. A draft Work Order changes nothing until it is activated.

## Evidence base

Ledger #21, #22, #23 (wordfreq friction ledger; landing on main via PR #44,
"WO28 decision 0028: uint_to_text / int_to_text builtins", currently open).
Pre-issuance probes below were verified on main at `a9ad460` on 2026-09-25
unless noted.

- **#21:** `task f(n: UInt) -> Int` called as `f(1, 2)`, `f("3")`, `f(-5)` —
  `hum full-type-check` accepts all three with zero diagnostics
  (`rejected_statements=0`, `blocking_issues=0`), at module scope and at app
  scope (tasks nested in `app`). At runtime, `f(1, 2)` traps
  (``task `f` expects 1 argument(s), got 2``) while `f("3")` and `f(-5)`
  silently complete.
- **#22:** `let x: UInt = -5` is left unchecked by the checker
  (`blocked_by_unchecked_body_types_v0`, `blocking_issues=1`,
  `rejected_statements=0`, reason `expression_type_unknown_v0`) and `hum run`
  executes it silently. Integers are `Value::Int` at runtime; negatives flow
  into `UInt` positions unchecked.
- **#23:** `return -9223372036854775808` panics the compiler process (see
  Item 5 for the exact shape).

---

## Item 1 — call-site arity is a checker error

For a call expression whose callee resolves to a declared signature with
`n` parameters, passing any other number of arguments is a checker error.
`f(1, 2)` against `task f(n: UInt)` is rejected; today it is accepted and
traps at runtime.

- The check counts the arguments of the canonical `Call` node. It sees
  through `Try` (`try f(1, 2)`) and `Group` (parenthesized calls) wrappers
  to the call.
- It applies only when the callee resolves: to a user task (declared
  parameters from the task signature) or to a builtin with a registered
  signature (Item 4). An unresolved callee keeps the existing H0601
  (unresolved name); the arity check does not fire on top of it.
- Name-resolution precedence (user task vs builtin, ledger #19) is unchanged
  and out of scope: the check uses whatever the resolver resolves to.
- Violation code: **H0640** (new; see Item 6).

## Item 2 — argument type is a checker error

For a call expression whose callee resolves to a declared signature, each
argument whose type is known must match the declared parameter type.
`f("3")` against `task f(n: UInt)` is rejected; today it is accepted and
runs.

Type-compatibility rule (exact, no inference beyond this):

- Argument types come from the canonical expression: `TextLiteral` →
  `Text`, `BoolLiteral` → `Bool`, `UIntLiteral(v)` → a non-negative integer
  literal, `IntLiteral(v)` → an integer literal with known sign, identifiers
  and places → their scope type facts, calls → the callee's return-type
  fact. The check is AST-driven, not text-driven.
- A non-negative integer literal is compatible with both `Int` and `UInt`
  parameters (the parser files every non-negative digit run as
  `UIntLiteral`; `uint_to_text(42)` and `int_to_text(42)` are both valid —
  pinned by the PR #44 valid-call tests).
- A negative integer literal is compatible with `Int` parameters and is
  rejected for `UInt` parameters — that rejection is Item 3 (H0642), not
  this item.
- Otherwise the parameter type name must equal the argument type name,
  except: a bare `List` parameter accepts any list-typed argument (`List`,
  `List Text`, list literals). This is the only width rule; it exists so
  `list_len` (Item 4) accepts every list the runtime accepts today.
- An argument whose type cannot be determined gets **no** diagnostic from
  this check. The honesty locks (decision 0014) forbid claiming a mismatch
  the implementation cannot prove; the runtime trap stays the backstop.
  (Consequence, accepted: `text_split`'s migrated type clause becomes
  silent on uninferable arguments where H0636 fires today. Fail-closed at
  runtime is unchanged.)
- Violation code: **H0641** (new; see Item 6).

H0606 (return type mismatch) is unchanged: it covers `return` expressions
only. H1402 (callable signature mismatch) is a different domain — task
*values* passed as callables under Session AL — and is not used here.

## Item 3 — statically known negatives never reach `UInt`

A negative integer literal reaching a `UInt` parameter or a `UInt`-annotated
binding is a checker error. `f(-5)` and `let x: UInt = -5` are rejected;
today both are accepted and run.

**Exactly what is statically decidable** (closed list — the checker decides
nothing else about sign):

1. A call argument against a `UInt` parameter (user task or builtin
   signature), where the argument expression, seen through `Group` nodes
   only, is `IntLiteral(v)` with `v < 0`.
2. A `let` or `change` binding whose declared annotation is `UInt`, where
   the value expression, seen through `Group` nodes only, is
   `IntLiteral(v)` with `v < 0`.

Everything else is **not** decidable and the runtime trap stays as the
backstop: variables of type `Int`, arithmetic (`0 - 5` — the shape PR #44's
trap test uses precisely because the checker cannot see through it), calls
returning `Int`, and any value whose negativity requires evaluation or
dataflow. No constant folding, no definite-value tracking.

Out of scope but noted: `set` targets with `UInt` type are not covered by
this item (same decidability would apply; left for a follow-up — flagged,
not built).

- The `uint_to_text` / `int_to_text` invariant-violation traps from PR #44
  and the runtime arity trap stay exactly as they are: they are the
  fail-closed backstop for every shape above the decidable list.
- Violation code: **H0642** (new; see Item 6).

## Item 4 — builtins expose signatures to the one general check

One general call-shape check, driven by signatures, covers user tasks and
builtins through a single code path. Never per-builtin codes for arity or
argument types.

**How builtins expose their signatures:** a single builtin signature table
(name → ordered parameter types → return type), consulted by the general
check exactly as a user task's declared parameters are. Required entries:

| builtin | signature |
|---|---|
| `uint_to_text` | `(UInt) -> Text` |
| `int_to_text` | `(Int) -> Text` |
| `text_split` | `(Text, Text) -> List Text` |
| `list_len` | `(List) -> UInt` |

(The table extends the existing return-type-only registry,
`session_z_builtin_return_types` in `src/full_type_check.rs`, which today
carries no parameter information — the parameter knowledge currently lives
hardcoded in the per-builtin `*_type_issue` functions and in the runtime
`eval_*` traps.)

**Disposition of the existing per-builtin codes** (their signature-shape
reasons migrate to H0640/H0641; value-level reasons stay):

- H0636 (`text_split`): the arity reason
  (`text_split_requires_exactly_two_arguments_v0`) and the type reason
  (`text_split_arguments_must_be_text_v0`) migrate to the general check.
  H0636 keeps its value-level reasons — stray empty argument
  (`text_split_rejects_stray_empty_argument_v0`) and directly-written empty
  separator literal
  (`text_split_separator_must_not_be_empty_literal_v0`) — and its catalog
  row is updated to the narrowed meaning ("invalid text_split call" on
  value shapes). Existing arity/type tests migrate to H0640/H0641.
- H0622 (`stdout_write`), H0626 (`clock_replay_tick`), H0632
  (`files_read_text`): pure signature-shape codes. Their reasons migrate to
  the general check; the codes become unemitted. Per the catalog stability
  rules they are retained and never reused for a different meaning.
  Existing tests migrate to H0640/H0641.
- H0633/H0637 (reserved builtin names), H0638, H0639: untouched.

## Item 5 — the parser panic becomes a diagnostic, never a panic

**Verified shape (main at `a9ad460`, 2026-09-25):** the source text
`-9223372036854775808` — exactly the `i64::MIN` spelling, `-` glued to the
digit run with no space — in any expression position (`return`, `let`
binding, parenthesized, inside a binary expression) panics `hum check`:

``thread 'main' panicked at src/parser.rs:2272:14: parser H0010 visitor
requires a valid sealed canonical occurrence:
"canonical_occurrence_authority_mismatch_v0"``

The H0010 retained-occurrence visitor (`.expect()` in
`retain_validated_occurrence`) assumes every retained occurrence carries a
valid seal; this literal corrupts the seal and the assumption becomes a
process panic. Boundary probes: `-9223372036854775807` is fine;
`- 9223372036854775808` (space) is fine; `-9223372036854775809` (out of
range) and positive `9223372036854775808` are silently accepted with 0
errors — separate gaps, noted here, not silently fixed.

**Requirement:** the panic becomes a diagnostic. Never a panic — a panic is
the one failure mode the honesty locks forbid.

- The seal-building path for signed integer literals must fail closed: the
  existing `IntegerLiteralOutOfRange` malformed-completion machinery
  (`projected_out_of_range_integer` / `retained_out_of_range_integer`)
  already understands `-`-glued digit runs, but its
  `spelling.parse::<i64>()` check is blind to exactly this case (the signed
  text parses as `i64::MIN` while the digit run `9223372036854775808` does
  not fit `i64` — the sign and the digits are validated asymmetrically).
  Fix the asymmetry so the literal takes the malformed-completion path
  instead of corrupting the seal.
- Defense in depth: `retain_validated_occurrence`'s `.expect()` becomes a
  diagnostic emission. A corrupt seal is evidence of a bug; the user-facing
  behavior is still a diagnostic, never a panic.
- Violation code: **H0011** (new; see Item 6). The seal-internal
  `integer_literal_out_of_range_v2` cause exists only in the predicate path
  (mapped to H0704, contract predicates); no user-facing source-shape code
  covers it.

## Item 6 — diagnostics: mapping and authorized allocations

Each shape was mapped to existing codes first. None fits; the Work Order
authorizes four new allocations (decision 0027's test: these are the
general fix, one code per violation shape, not per builtin):

| code | family | title | meaning |
|---|---|---|---|
| `H0640` | `front_end_semantics` (H0600–H0699) | call argument count mismatch | A call passes a different number of arguments than the callee's declared signature (user task parameters or builtin signature-table entry). |
| `H0641` | `front_end_semantics` (H0600–H0699) | call argument type mismatch | A call argument of known type does not match the declared parameter type. |
| `H0642` | `front_end_semantics` (H0600–H0699) | negative integer literal in UInt position | A statically known negative integer literal reaches a `UInt` parameter or `UInt`-annotated binding. |
| `H0011` | `source_shape` (H0000–H0099) | integer literal out of range | An integer literal (including a signed literal whose digit run overflows) does not fit the 64-bit range; the compiler emits this instead of panicking. |

Considered and rejected: H0606 (return expressions only), H0622/H0626/
H0632/H0636 (per-builtin — Item 4 forbids extending that pattern), H1402
(callable *values*, wrong domain), H0704 (contract predicates, wrong
domain).

The allocation follows the project's H-code checklist: `diagnostic_causes!`
entries, `diagnostic_code_allocations!` entries with the next free key
indexes, `historical_public_ordinal` match arms, `DIAGNOSTICS` detail
entries, hardcoded-count updates, the `docs/DIAGNOSTICS.md` mirror rows, and
the `check_all.ps1` pinned count plus `test_ci_policy.ps1` `$CompilerBodies`
digest (per the standing AGENTS.md checklist — the fourth hardcoded-pin
break was a digest pin on exactly this kind of change).

---

## Acceptance criteria

1. **Fixtures per shape at module and app scope.** For each of Items 1–4:
   fixtures with the offending call/binding at module scope (top-level
   tasks) and at app scope (tasks nested in `app`) are rejected by
   `hum full-type-check` with the specified new code (H0640 / H0641 /
   H0642). Valid shapes — including `uint_to_text(42)`, `int_to_text(-7)`,
   `text_split` valid calls, and `list_len` on list-typed arguments — stay
   accepted.
2. **`hum check` AND `full-type-check` agreement (ledger #18 boundary
   noted).** The call-shape check lives in `full-type-check`, the stage
   that owns expression checking. `hum check` (`src/type_check.rs`) is
   unchanged: its non-claims already state "no call, overload, field, or
   operator type checking" and "no generic arity validation". Agreement
   means: `full-type-check` rejects each fixture with the new code, while
   `hum check` emits no call-shape diagnostic for the same fixtures —
   silence by declared design, asserted in the fixture expectations — so
   the two stages never contradict (the ledger #18 boundary: `hum check`
   claims nothing about calls, and continues to claim nothing). `hum run`'s
   preflight behavior is unchanged; the runtime traps stay the backstop.
3. **The PR #44 characterization tests flip from "accepted" to "rejected".**
   `uint_to_text_misuse_shapes_are_checker_accepted_like_user_tasks` and
   `int_to_text_misuse_shapes_are_checker_accepted_like_user_tasks`
   (in `src/full_type_check.rs`) are reworked to assert rejection:
   wrong-arity shapes → H0640, wrong-type shapes (including the cross-type
   variable shapes `uint_to_text(n)` with `n: Int` and `int_to_text(n)`
   with `n: UInt`) → H0641, negative-literal shapes → H0642. The valid-call
   tests stay green; the runtime trap tests (`uint_to_text_negative_value_traps`,
   normative rendering) stay green — the backstop is not removed.
4. **Item 5:** `return -9223372036854775808` (and the `let`, parenthesized,
   and binary-expression variants) produce H0011 and a check failure —
   never a panic. A regression test feeds every previously panicking shape
   through the full parse and asserts no panic.
5. **Catalog integrity:** `docs/DIAGNOSTICS.md` mirrors the four new codes;
   the H-code checklist (Item 6) is complete; H0622/H0626/H0632 are marked
   unemitted-but-reserved; H0636's row reflects the narrowed meaning. The
   full test suite passes with no new false positives on existing fixtures.

## Lane assignments and STOP conditions

- **Research lane (this draft):** the Work Order text, the evidence
  characterization (ledger #21/#22/#23 plus the pre-issuance probes), the
  diagnostics mapping, and the migration dispositions.
- **Builder lane:** the general call-shape check in `full-type-check`, the
  builtin signature table, the H0640/H0641/H0642 diagnostics, the parser
  panic fix with H0011, the catalog updates, and the tests. The diagnostic
  catalog changes only by decision or Work Order; this Work Order is that
  authorization for the four allocations above.
- **STOP:** if implementing any Item requires new language surface or an
  unmade semantic choice (e.g. the `set`-target sign shape, the
  silently-accepted positive-overflow gap), STOP and report to the BDFL
  instead of inventing it. If a per-builtin migration cannot be done
  without changing the code's user-facing meaning beyond the narrowing
  specified in Item 4, report it — do not silently widen it.

## Review and evidence requirements

- Independent pre-issuance review of this draft (Claude) before any PR.
  No PR is opened until that review lands.
- Honesty locks (decision 0014): no diagnostic may claim more than the
  implementation proves — hence unknown argument types stay silent (Item
  2), and the decidable sign set is closed (Item 3).
- Every Item's acceptance criteria are asserted by tests, not by prose.
  Items are implemented in dependency order (4 → 1 → 2 → 3 → 5 → 6; the
  signature table comes first, the catalog updates last).

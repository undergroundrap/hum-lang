# Duplication audit: `src/type_check.rs` vs `src/full_type_check.rs` (re-run)

**Date:** 2026-09-25 (re-run). **Repo state:** `origin/main` at `18a31cd`
(PR #39 merged: WO28 #16, shared `TypeScopeStack`). **Scope:** read-only
inspection; no code changed, no behavior measured. Line numbers below refer
to this repo state. This note supersedes the pre-#39 audit (2026-09-25,
`699122e`); its caveat 7 is answered in §1.

**Verdict:** the shared `TypeScopeStack` landed truly shared — one
definition in `src/type_scopes.rs`, imported by both files — so the scoping
mechanism converged and duplication shrank. The ~21 byte-identical helpers
are unchanged (re-verified by diff). The two files still maintain parallel
implementations of expression-type inference and return checking, now
threaded over the same stack type with different borrow disciplines
(immutable in `type_check`, mutable in `full_type_check`).

Scale: `type_check.rs` 2,954 lines; `full_type_check.rs` 3,713 lines
(pre-#39: 2,865 / 3,711). Test modules excluded (independent fixtures are
normal isolation, not duplication).

## 1. Caveat 7 answered: the stack is truly shared

PR #39's shared `TypeScopeStack` is not duplicated — it is defined once:

- `src/type_scopes.rs:13`: `pub(crate) struct TypeScopeStack<T>` — the
  single definition.
- `src/type_check.rs:20` and `src/full_type_check.rs:16`:
  `use crate::type_scopes::TypeScopeStack;`
- Both files' statement walkers call `handle_block_boundary` (WO28 #16:
  block scoping must match the resolver — block openers push, block close
  pops, so `let` bindings inside blocks don't leak out and shadowed outer
  facts are restored).

How each file holds it still differs:

- `type_check.rs:1542`: `let mut scopes =
  TypeScopeStack::new(initial_task_type_environment(task));` — the walker
  owns the mutable stack and passes it as `&TypeScopeStack<TypeFact>` to
  `checked_return` (:1579), `binding_type_fact` (:1659), and
  `infer_expression_type` (:1699); bindings are inserted by the owner
  (`scopes.insert(&name, fact)`).
- `full_type_check.rs:817`: `let mut scopes =
  TypeScopeStack::new(initial_environment(item_params(item)));` — threaded
  as `&mut TypeScopeStack` through `type_statement` (:868), `&` elsewhere
  (`typed_statement` :1547, `collect_task_return_types` :1686, and the
  inference/binding helpers).

Net: one mechanism, two threading disciplines. The pre-#39 audit's "flat
vs threaded env" row is now "immutable-borrow + owner-inserts vs mutable
threading" — narrower, but still two models of who may mutate the scope
stack during the walk.

## 2. Byte-identical helpers (re-verified by diff, 2026-09-25)

Re-verified identical by direct diff of function bodies at `18a31cd`:

- `initial_task_type_environment` (tc:1562) / `initial_environment`
  (ftc:1703) — bodies identical; only the signature differs (`&task.params`
  vs `params`, signature-driven). Different names hide the identity.
- `strip_keyword`, `name_key`, `snake_identifier`, `prefixed_id`,
  `record_literal_type_name`, `path_root_type_name`, `is_type_like_name`,
  `expected_return_value_type`, `type_tokens` — identical in both files.
- `diagnostic_occurrence_set` — identical 5-line delegating wrapper.
- `sole` (tc:818) / `unique` (ftc:708) — bodies byte-identical 3-line
  helper, **different names**.
- Hand-rolled JSON emitters, all 8 identical: `push_string_array`,
  `push_span_field`, `push_optional_string_field`, `push_string_field`,
  `push_usize_field`, `push_json_string`, `push_indent`, `push_comma_newline`.
- `struct TypeFact` — structurally identical
  (`type_text: String, source: &'static str`), but still two separate
  types, so facts cannot cross the boundary without conversion.

Unchanged from the pre-#39 audit: ~21 items, none deduplicated by #39
(#39 shared the scope stack, not the helpers).

## 3. Near-duplicates with a semantic reason (unchanged)

- **`infer_expression_type`** (tc:1699 / ftc:1814). Same name; the full
  version is a strict superset. Every `type_check` arm (empty → Unit,
  bool, text, closed-view-derivation, digits → `integer_literal`, record
  literal, path root, env lookup) appears in `full_type_check` with
  identical `(type_text, source)` outputs. The full version adds:
  permission-expression stripping, list literals, `place_type_fact`
  (field types), condition expressions, additive/multiplicative
  inference, call resolution via `task_returns`. Ordering note: the full
  version checks `place_type_fact` *before* `record_literal_type_name`;
  `type_check` has no place facts at all. Inherent to the full stage's
  job, but a future arm reorder could silently change which rule wins.
- **`binding_type_fact`** (tc:1659 / ftc:1720). Same (name, TypeFact)
  output contract; both now take `&TypeScopeStack<TypeFact>`. Decomposition
  still differs — `type_check` parses `statement.kind` inline and infers
  from the value against the scopes; `full_type_check` delegates to
  `binding_annotation`/`binding_name`/`binding_left` helpers and takes a
  pre-computed `actual`.
- **`return_types_compatible`** (tc:1791) / **`types_compatible`**
  (ftc:1991). Same core (name_key comparison, `integer_literal` →
  int/uint/float); the full version drops the `expected_value_type` param
  and adds a `list_literal` arm.

## 4. Diverged: same name/purpose, different behavior (unchanged)

- **`diagnostic_occurrence_set_from_source`** (tc:1025 / ftc:746). Same
  name, different construction: `type_check` builds from the type-env
  report + type diagnostics and returns the set directly;
  `full_type_check` builds from the `core_verify` projection +
  `typed_failure` analysis and returns
  `Result<_, DiagnosticInvariantError>`. The stage difference is
  legitimate, but the shared name hides different semantics — a reader
  (or a future caller) will assume they are the same function.
- **Return checking** (tc `checked_return` :1579 / `collect_checked_returns`
  :1501 vs ftc `typed_statement` :1547 / `type_statement` :863 /
  `collect_task_return_types` :1686). Reimplemented, not shared:
  `type_check` checks only `return` statements (immutable stack borrow)
  producing `CheckedReturn`; `full_type_check` types *every* statement
  (mutable threading) producing `TypedStatement`. The report shapes
  differ (structural), but the *checking logic* — what counts as a
  return type error — now lives in two places over one shared stack type.

## 5. Risk ranking (where a fix in one file silently needs the same fix in the other)

1. **`infer_expression_type`.** Unchanged from pre-#39: the shared
   literal-classification arms encode the same type facts twice. A fix to
   any arm (a new literal form, a changed `source` string that feeds
   occurrence routing) must land in both files or the two stages disagree
   on expression types — exactly the check / full-type-check disagreement
   class the project keeps repairing (H0639, WO28 #15). The superset
   structure means the fix locations don't even line up.
2. **Return-statement checking.** Narrowed by #39 but not closed: the
   env-model divergence is now one shared type with different threading,
   so a scope-semantics change in `type_scopes.rs` propagates to both
   (good) — but return-compatibility logic is still implemented twice,
   and the two still differ in coverage (returns only vs all statements).
3. **`diagnostic_occurrence_set_from_source` name collision.** Low
   probability, high confusion: same name, different signatures and
   semantics, both resolving locally with no compiler complaint.

## 6. Connection to ledger #18 (check vs predicate analysis)

Re-verified at `type_check.rs:1284`: the type-check report still renders
`predicate::analyze_program(program).place_facts_text()` — the predicate
data is in hand at the type-check stage; only diagnostic *surfacing* is
missing. Any Option-A implementation (running predicate analysis in `hum
check`) starts from an already-imported, already-computed value, not a new
dependency.

## 7. Caveats

- This re-run answers the pre-#39 audit's caveat 7: the stack is truly
  shared (`src/type_scopes.rs`), not per-file. That audit is superseded.
- `push_items`/`push_statements` (ftc) vs `push_checked_returns` (tc) are
  structurally similar hand-rolled JSON loops — boilerplate, not logic
  duplication; not counted.
- Both files' public APIs remain load-bearing shared surface
  (`type_check::` referenced from ~20 files, `full_type_check::` from ~8),
  so any dedup must go through them, not around them.
- This audit classifies structure, not intent. Whether dedup is worth the
  churn (vs. the two files' different report shapes and stage
  responsibilities) is a Builder-lane / BDFL call; nothing here is a
  decision.

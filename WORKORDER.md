# Hum Work Order 4: Corpus Burn-Down One

Date: 2026-07-09
Status: active, issued under delegated authority (GOVERNANCE.md), BDFL veto open
Owner: BDFL (Ocean). Reviewer/ruler: architect-reviewer. Implementer: agent sessions.
Predecessor: Work Order 3 closed at commit `988e7c8` with the Session M
retrospective: 1 of 12 corpus programs runs; ownership hit three strikes;
the mandated repair is checked sub-view provenance, then disjoint-field
projection.

## Why this document exists

Decision 0014 bought candidate A's ergonomics with a maturity debt. This
work order pays the first installments, in the order Session M's friction
evidence mandated — not preference order. Target: the corpus goes from
1/12 running to roughly 6/12, with every new acceptance earned by a real
fixture and every gap recorded honestly.

The 0014 honesty locks continue to bind all output text: no full
ownership-safety, borrow-soundness, memory-safety, or safety-critical
readiness claims. Session M's "Honesty locks after Session M" list is the
current truth; sessions here may narrow it only by shipping the checked
feature that retires a lock, never by rewording.

## Global bans (all sessions)

- No closures or tasks-as-values (effect-polymorphism gate; program 3's
  retain-with-predicate formulation stays blocked and recorded).
- No internal references: `from parser.buffer` stays rejected by H0805.
  That is the repair AFTER this work order, pending Session Q evidence.
- No general provenance inference. View provenance flows only through a
  small closed set of recognized view-deriving operations, listed in the
  docs, extended only by session mandate.
- No concurrency, FFI, backends, generics, new schemas, or new report
  subcommands.
- Every new rejection rule ships with a misuse fixture and a stable
  diagnostic with blame-style help naming the site and the fix.
- Grammar or surface changes update MILESTONE_0_GRAMMAR.md,
  LANGUAGE_REFERENCE.md, syntax surface, TextMate grammar, and the README
  showcase in the same session.

## Session N: sub-view provenance (make program 9 real)

Scope:

1. Add a minimal recognized text sub-view operation (e.g.
   `slice_until(text, separator)`) to the executable subset — the first
   member of the closed view-deriving set. Its result carries the
   provenance of its input.
2. Extend returned-view checking: a view derived from parameter `text`
   through closed-set operations satisfies `-> Slice Text from text`.
   Deriving from a local still rejects with H0805. The bare-parameter
   `echo_view` case remains valid.
3. Port corpus program 9 for real:
   `first_word("hum language")` returns `hum` under `hum run`, with the
   result-to-parameter dependency still visible in `hum graph` and
   ownership JSON for the derived case.
4. Misuse fixtures: sub-view of a local returned (H0805), and a
   derivation chain that loses provenance (e.g. via a non-closed-set
   operation) rejected honestly rather than guessed.

Acceptance criteria:

- `hum run ... --entry first_word --args "hum language"` prints `hum`.
- H0805 still fires on local and internal-reference sources.
- Graph/ownership JSON show the dependency for derived views.
- The Session L `blocked` friction record for program 9 is marked
  resolved in the ledger (do not delete it; annotate it).
- `.\tools\check_all.ps1` passes. Stop for review.

## Session O: field places and disjoint fields (programs 8 and 11)

Scope:

1. Field-place assignment in the executable subset: `set point.x = ...`
   through a `change` parameter or `change` local, statically checked and
   runtime-enforced.
2. Disjoint-field v0: two writes to definitely distinct fields of one
   record are accepted (the swap); writes through a `borrow` parameter
   still reject (H0802 family).
3. Port corpus programs 8 and 11 for real: `swap_xy(Point{x:1,y:2})`
   yields `{x:2,y:1}`; `complete_item` sets `done` in place and
   provably preserves `title` (assert via ensures or test expectation).
4. The corpus misuses that require field *views* (stale field view after
   update) are NOT expressible yet: record that as a friction entry
   honestly instead of faking a weaker misuse under the same name.

Acceptance criteria:

- Both programs run with correct outputs and at least one firing
  contract each.
- Field write through borrow rejects with blame help.
- Friction records filed for the view-dependent misuses.
- `.\tools\check_all.ps1` passes. Stop for review.

## Session P: minimal list growth (programs 12 and 3's misuse)

Scope:

1. The smallest honest list-growth surface: `list_append(change list,
   item)` in the executable subset (this consumes part of backlog item
   10; full stdlib list design remains future work).
2. Iteration-conflict rule: structural mutation of a collection that is
   being iterated by an active `for each` rejects with a new diagnostic
   naming both the loop and the mutation (corpus program 3's misuse —
   expressible today without closures).
3. Port corpus program 12 for real: a builder task appends three items
   and `consume`-finishes; `builder_demo()` returns the completed list;
   add-after-finish rejects via the existing consume machinery.
4. Stale-element-view-across-growth misuse is view-dependent and NOT
   expressible yet: friction record, not fake.

Acceptance criteria:

- Builder probe runs with correct output and a firing contract.
- Iteration-conflict misuse fixture fires the new diagnostic.
- Add-after-finish rejects.
- `.\tools\check_all.ps1` passes. Stop for review.

## Session Q: corpus retrospective two

Scope:

1. Update SCORECARD.md "Implementation status" for all twelve programs;
   record the burn-down (expected: programs 9, 8, 11, 12 move to Runs,
   3 moves to partially-runs-with-misuse-rejection; verify honestly).
2. Consolidate new friction records; re-apply the three-strike rule.
3. Honest summary: which honesty locks (if any) can narrow, which
   remain; recommendation for the next work order between internal
   references (parser state, program 5) and the remaining view machinery
   (stale field/element views), justified from this work order's
   friction data.

Acceptance criteria:

- Implementation status covers all twelve, no TBD, degenerate cases not
  counted.
- Three-strike rule applied to the updated ledger.
- Recommendation argued from friction records, not preference.
- `.\tools\check_all.ps1` passes. Stop: Work Order 4 ends here; the next
  work order is written by the architect-reviewer from Session Q.

## Design probe system (standing)

Probe sources: regret-ledger probes, construct-pair probes, misuse
probes, domain-slice probes. Friction record format:

```text
friction:
  program: <file and line>
  wanted: <what the author tried to write>
  forced: <what the language required instead>
  severity: blocked | wrong-by-default | awkward | verbose
  indicts: <contracts | ownership | loops | types | diagnostics | stdlib | checker | core-body-grammar>
  proposal: <optional one-line fix direction>
```

Rules: three or more records indicting one area triggers a decision
record or work-order item; `blocked`/`wrong-by-default` triaged before
the next session; prose `needs:`/`ensures:` lines feed the contract
wishlist. Current strikes: ownership 3 (triggered; this work order is
the response), contracts 2, stdlib 1 (Session P consumes part), types 1,
core-body-grammar 1.

## Showcase discipline (standing)

README/SPEC examples over five lines are extracted from checked fixtures
(preflight-enforced); README shows minimal and full-contract forms plus
the Magic Comment Problem section; surface-changing sessions update the
showcase in the same session.

## Backlog: accepted taste, not scheduled

1. Deterministic run mode (virtual clock, seeded random, fixed schedule;
   virtualize time before stdlib clock APIs; bit-for-bit replay; stable
   test-mode iteration order).
2. Semantic diff (`hum diff`): effect/contract/capability deltas.
3. Machine-applicable fixes (`hum fix --apply`); gates the public
   agent-native claim per risk R016.
4. Sandboxed execution flags (`--allow`/`--deny`) when IO capabilities
   arrive.
5. Fault containment: direction settled; design work order before any
   concurrency syntax.
6. Units of measure: direction settled; waits for type-system maturity.
7. Language editions before public alpha stability promises.
8. Contract check policy ADR (proved | boundary | unproved |
   external-trust exported as evidence).
9. Predicate grammar v1, grown only from the contract wishlist
   (collection-count predicates: one recorded demand).
10. List operation surface beyond Session P's minimal append: retain
    (needs effect polymorphism), element views, capacity and profile
    behavior.
11. Numerics policy ADR (checked-by-default all modes; explicit
    families; benchmark gate; two FP regimes; decimal library-first).
12. Text model tiers (Bytes / Text / OsText) with early stdlib design.
13. Source policy hardening: bidi-control rejection in text hygiene;
    ASCII-identifier policy recorded as deliberate.
14. Ownership repairs after this work order, order pending Session Q
    evidence: internal references (program 5) versus stale-view
    machinery (field and element views), then broader flow-sensitive
    borrowing.
15. Effect polymorphism decision record: required before closures,
    tasks-as-values, retain-with-predicate, or any higher-order stdlib
    API. Research direction settled (cluster 15); the ADR itself remains
    unwritten and blocks program 3 and 4 completion.

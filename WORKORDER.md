# Hum Work Order 3: Ownership Checker Foundations

Date: 2026-07-08
Status: active, issued under delegated authority (GOVERNANCE.md), BDFL veto open
Owner: BDFL (Ocean). Reviewer/ruler: architect-reviewer. Implementer: agent sessions.
Predecessor: Work Order 2 (ownership bake-off) completed; decision 0014
accepted at commit `a396965`.

## Why this document exists

Decision 0014 adopted ownership and borrowing as Hum's core model. This
work order builds the first three rungs of the ADR's checker roadmap:
ordinary owned values and moves, linear resource path checking, and
returned-view dependencies from parameters. The repairs (disjoint-field
projection, internal references, flow-sensitive borrowing) are explicitly
OUT of scope: they are future work orders that must be earned by evidence
from this one.

Honesty locks from 0014 apply to every session: no tool output, doc, or
README text may claim full ownership safety, borrow soundness, memory
safety, or safety-critical readiness. Reports name exactly what is
checked and what is not.

## Global bans (all sessions)

- No disjoint-field projection, internal references, or flow-sensitive
  borrowing. If a fixture seems to need one, record a friction entry and
  route around it.
- No closures, tasks-as-values, generics, concurrency, FFI, backends.
- No new schemas or report subcommands; extend `hum.ownership_check.v0`
  and existing surfaces in place, bumping only if shape must break.
- Every new rejection rule ships with at least one deliberately wrong
  fixture that triggers it (misuse-probe rule) and a stable diagnostic
  code with blame-style help text.
- Grammar changes update MILESTONE_0_GRAMMAR.md, LANGUAGE_REFERENCE.md,
  the syntax surface, and the TextMate grammar in the same session
  (showcase discipline applies).

## Session J: parameter permissions and moves

Scope:

1. Add parameter permission modes to the grammar: `borrow` (default when
   unmarked), `change`, and `consume`, e.g.
   `task rename(change item: WorkItem, new_title: Text)`. Record the
   default-borrow rule in LANGUAGE_REFERENCE.md with a short rationale
   note referencing 0014.
2. Extend the ownership checker to track owned locals and moves in the
   recognized executable subset: a local passed by `consume` or returned
   is moved; use after move is rejected with a new diagnostic whose help
   names the move site.
3. `hum run` enforces the same rules dynamically where static coverage
   is partial, trapping with the same diagnostic identity.
4. Fixtures: at least one passing program per permission mode plus
   misuse fixtures for use-after-move, writing through `borrow`, and
   consuming the same value twice.

Acceptance criteria:

- All new fixtures pass/fail as designed under both `hum check`-family
  gates and `hum run`.
- Diagnostics carry blame-style help (who must fix it, at which line).
- `.\tools\check_all.ps1` passes. Stop for review.

## Session K: linear resources

Scope:

1. A `consume`-obligated value must be consumed exactly once on every
   control-flow path of the recognized subset: missing-consume on any
   path and double-consume are both rejected, with path-naming
   diagnostics per the 0014 blame style.
2. Port corpus program 10 (transaction commit-or-rollback) from the
   candidate A sketch into a runnable fixture under `examples/probes/`,
   with begin/debit/credit/commit/rollback stubbed in the interpreter as
   needed (smallest possible stubs).
3. Misuse fixtures: early return skipping consume, double consume after
   commit, consume inside only one branch of an if.

Acceptance criteria:

- The transaction probe runs green under `hum run`; each misuse fixture
  produces its diagnostic with the offending path named.
- Friction records for anything the linear rules made awkward.
- `.\tools\check_all.ps1` passes. Stop for review.

## Session L: returned-view dependencies

Scope:

1. Add the checked `from` relationship for return types limited to
   parameters: `task first_word(text: borrow Text) -> Slice Text from
   text`. Returning a view into a local is rejected (the program 9
   misuse); returning a view derived from the named parameter is
   accepted.
2. The semantic graph and ownership report expose the dependency as a
   fact (which result depends on which parameter) — this is the
   evidence-native payoff of `from` relationships and must be visible in
   JSON output.
3. Port corpus program 9 as passing and misuse fixtures. Internal
   references (`from parser.buffer` on stored fields) remain banned; a
   friction record marks where they were wanted.

Acceptance criteria:

- Program 9 fixture pair behaves as designed under check and run.
- `hum graph`/ownership JSON shows the result-to-parameter dependency.
- `.\tools\check_all.ps1` passes. Stop for review.

## Session M: corpus retrospective

Scope:

1. Attempt every corpus program that the current subset can express as a
   real runnable fixture; record per-program status (runs, blocked by
   ban, blocked by missing feature) in a new section of
   `docs/bakeoff/SCORECARD.md` titled "Implementation status".
2. Append all accumulated friction records to CORE_LANGUAGE_SHAPE.md's
   friction ledger. Apply the three-strike rule: any area with three or
   more records gets a proposed work-order item or decision record.
3. Write an honest summary: which 0014 honesty locks remain (expected:
   all), and what the next work order should build, with a
   recommendation between disjoint-field projection and flow-sensitive
   borrowing as the first repair (justified from friction data, not
   preference).

Acceptance criteria:

- Implementation-status section covers all twelve programs, no TBD.
- Friction ledger updated; three-strike rule applied.
- `.\tools\check_all.ps1` passes. Stop: Work Order 3 ends here. The next
  work order is written by the architect-reviewer from Session M's
  evidence.

## Design probe system (standing)

Probe sources: regret-ledger probes, construct-pair probes, misuse
probes, domain-slice probes. Friction record format:

```text
friction:
  program: <file and line>
  wanted: <what the author tried to write>
  forced: <what the language required instead>
  severity: blocked | wrong-by-default | awkward | verbose
  indicts: <contracts | ownership | loops | types | diagnostics | stdlib | checker>
  proposal: <optional one-line fix direction>
```

Three or more records indicting one area triggers a decision record or
work-order item. Contract wishlist state: collection-count predicates
have one recorded demand; `contracts` holds two of three strikes.

## Showcase discipline (standing)

README/SPEC examples over five lines are extracted from checked fixtures
(preflight-enforced); README shows minimal and full-contract forms;
surface-changing sessions update the showcase in the same session.

## Backlog: accepted taste, not scheduled

1. Deterministic run mode (virtual clock, seeded random, fixed schedule;
   virtualize time before stdlib clock APIs; bit-for-bit replay; stable
   test-mode iteration order).
2. Semantic diff (`hum diff`): effect/contract/capability deltas.
3. Machine-applicable fixes (`hum fix --apply`).
4. Sandboxed execution flags (`--allow`/`--deny`) when IO capabilities
   arrive.
5. Fault containment: direction settled (fault domains, supervisors,
   restart budgets, no general unwinding); design work order before any
   concurrency syntax.
6. Units of measure: direction settled (core-language F#-style,
   first-order, erasure, boundary reification); waits for type-system
   maturity.
7. Language editions (RELEASE_AND_VERSIONING.md) before public alpha
   stability promises.
8. Contract check policy ADR (proved | boundary | unproved |
   external-trust classification exported as evidence).
9. Predicate grammar v1, grown only from the contract wishlist.
10. List operation surface: unblocked by 0014; smallest growable-list
    API design lands alongside Session M evidence.
11. Numerics policy ADR (checked-by-default all modes; explicit
    families; benchmark gate; two FP regimes; decimal library-first).
12. Text model tiers (Bytes / Text / OsText) with early stdlib design.
13. Source policy hardening: ASCII-identifier policy recorded as
    deliberate; extend check_text_hygiene to reject bidi controls (small
    standalone task any maintenance session may take).
14. Ownership repairs, in evidence-driven order after Session M:
    disjoint-field projection, flow-sensitive borrowing, internal
    references. Each is its own work order with corpus programs as
    acceptance criteria.

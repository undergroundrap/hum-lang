# Hum Work Order 2: Ownership Bake-Off

Date: 2026-07-08
Status: active, BDFL-approved
Owner: BDFL (Ocean). Reviewer: external architect pass. Implementer: agent sessions.
Predecessor: Work Order 1 (Milestone 1 vertical slice) completed in full;
see git history through commit `e3b2713`.

## Why this document exists

Hum now runs programs and enforces contracts with blame. The next decision
is the most consequential one the language will ever make: the ownership
model. This work order decides it as a paper exercise against a fixed
program corpus, producing decision record 0014. No checker gets built until
the model is chosen; retrofitting an ownership model after implementation
is how languages die.

Research inputs, both already absorbed:

- [docs/research/2026-07-08-ownership-contracts-effects-determinism.md](docs/research/2026-07-08-ownership-contracts-effects-determinism.md)
  Section 1, which sets the null hypothesis below.
- The Session D friction ledger in
  [docs/CORE_LANGUAGE_SHAPE.md](docs/CORE_LANGUAGE_SHAPE.md), which
  contributes programs 11 and 12.

## Rules for this work order

1. Paper only. No changes to `src/`, no interpreter work, no checker
   prototypes, no new diagnostics. The outputs are documents and `.hum`-
   shaped program sketches under `docs/bakeoff/`. That directory is the
   sanctioned new-doc surface for Sessions E-I; nothing else.
2. Candidates share Hum's existing permission vocabulary where possible
   (`let`, `change`, `owned`, `borrow`, `shared`, `consume` from
   MEMORY_SAFETY_MODEL.md and STATE_MODEL.md). What differs between
   candidates is the rules those words obey, not cosmetic renaming.
   Sketches honor decisions 0009 (formal readability) and 0012
   (snake_case).
3. Advocate rule. Each candidate session is written by that candidate's
   best advocate: make the strongest honest case, including its escape
   hatches. Cross-candidate criticism is deferred entirely to Session I.
   This exists because the research recommends candidate A, and the
   bake-off is worthless if the other two are written as strawmen.
4. Friction records still apply. Writing a corpus program that fights the
   candidate's rules is data: record it (`indicts: ownership`) in the
   candidate document, not in CORE_LANGUAGE_SHAPE.md.
5. Session boundaries are hard: one session, one deliverable, stop for
   review. The BDFL reads the corpus before candidates are written and
   reads all three candidates before Session I.

## The candidates

- Candidate A (null hypothesis): Rust-like ownership and borrowing,
  designed from day one against the known repair list (flow-sensitive
  conditional returns, disjoint-field projections, internal references),
  plus linear resources for exactly-once protocols, plus explicit
  arenas/regions as source-visible opt-in.
- Candidate B: mutable value semantics with second-class references
  (Hylo direction): whole-value semantics, `inout`-style exclusive
  parameter access, no first-class references stored in values.
- Candidate C: region/arena-first ownership: allocation and lifetime
  belong to named regions; values are region-tagged; escape analysis and
  explicit region arguments replace per-value borrows.

## Session E: pin the corpus

Scope:

1. Write `docs/bakeoff/CORPUS.md` containing the twelve programs listed in
   the appendix. For each program: the behavior specification (what it
   does, concretely, with example inputs/outputs), why it is hard (which
   ownership question it isolates), the misuse that must be rejected (the
   bug a checker must catch), and the success criteria for a candidate.
2. Specifications are model-neutral: they describe behavior and required
   rejections, never mechanism. A corpus entry that presupposes borrows,
   regions, or value semantics is a defect.
3. Keep each program small enough to sketch in under ~60 lines of Hum.

Acceptance criteria:

- All twelve programs have all four fields.
- No specification names a candidate mechanism.
- `.\tools\check_text_hygiene.ps1` passes.
- Stop for BDFL corpus review before Session F.

## Session F: candidate A, written by its advocate

Scope:

1. Write `docs/bakeoff/candidate_a_borrows.md`: all twelve corpus programs
   sketched in Hum surface syntax under candidate A rules.
2. Per program: the code sketch; the rule that rejects the corpus misuse;
   a one-line diagnostic sketch in Hum's blame style; a beginner
   explanation of why the rule exists, two sentences maximum.
3. A rules section: the candidate's core model stated in under a page,
   including its planned repairs and its two escape valves (linear
   resources, explicit arenas), with each escape hatch naming its cost
   (allocation, runtime check, proof obligation, or unsafe
   responsibility).
4. Honest self-score against the rubric in the appendix, including which
   programs needed an escape hatch.

Acceptance criteria:

- Twelve sketches, each with all four per-program artifacts.
- Self-score table present; escape-hatch usage explicitly counted.
- Friction records included for any program that fought the model.
- Hygiene passes. Stop for review.

## Session G: candidate B, written by its advocate

Same scope, template, and acceptance criteria as Session F, for mutable
value semantics with second-class references, in
`docs/bakeoff/candidate_b_values.md`. The advocate must engage the known
hard cases for this model (cyclic graphs, stored callbacks,
self-referential parsing) rather than skipping them: if the answer is
"this program is restructured as indices/arena," show that restructuring
as the sketch and price it honestly.

## Session H: candidate C, written by its advocate

Same scope, template, and acceptance criteria as Session F, for
region/arena-first ownership, in `docs/bakeoff/candidate_c_regions.md`.
The advocate must engage the Cyclone/MLKit postmortem directly: state
which escape hatches (dynamic regions, unique pointers, RC) the candidate
imports, and price each.

## Session I: scorecard and decision record

Scope:

1. Write `docs/bakeoff/SCORECARD.md`: the 12-by-3 matrix scored on the
   rubric, with the quantified gate applied. This session is the first
   place cross-candidate criticism is allowed, and it must criticize all
   three.
2. Draft decision record `docs/decisions/0014-adopt-ownership-model.md`
   with a recommendation, an alternatives-rejected section naming what
   dies with each loser, and a salvage section naming which losing ideas
   survive as escape hatches or profile features.
3. Identify what the winner does NOT settle (concurrency sharing rules,
   record-update syntax sugar, list growth API) and route each to the
   backlog or a named future decision.

Acceptance criteria:

- Scorecard covers all 36 cells; no "TBD" cells.
- The quantified gate is applied: the recommended model clears at least
  eight of twelve programs without escape hatches, or the ADR explains
  why the gate should move and what that concedes.
- ADR drafted with status `proposed`. The BDFL, not the session, flips it
  to `accepted`.
- Hygiene passes. Stop. No implementation work follows until the BDFL
  accepts 0014.

## Design probe system (standing)

Probe programs are to language design what property tests are to code.
Sources: regret-ledger probes, construct-pair probes, misuse probes,
domain-slice probes. Every probe session appends friction records:

```text
friction:
  program: <file and line>
  wanted: <what the author tried to write>
  forced: <what the language required instead>
  severity: blocked | wrong-by-default | awkward | verbose
  indicts: <contracts | ownership | loops | types | diagnostics | stdlib | checker>
  proposal: <optional one-line fix direction>
```

Rules: three or more records indicting one area triggers a decision record
or work-order item; `blocked`/`wrong-by-default` records are triaged before
the next session; prose `needs:`/`ensures:` lines are contract-wishlist
entries, frequency-ranked to drive predicate grammar v1. Current wishlist
state: collection-count predicates have one recorded demand
(word_count.hum); `contracts` has two of three strikes.

## Showcase discipline (standing)

README/SPEC examples over five lines are extracted from checked fixtures
(preflight-enforced); the README shows minimal and full-contract forms;
surface-changing decisions update the showcase in the same session.

## Backlog: accepted taste, not scheduled

1. Deterministic run mode: virtual clock, seeded random, fixed schedule as
   `hum run --deterministic`. Gates: virtualize time/scheduling before the
   stdlib grows ambient clock APIs; bit-for-bit replay from one artifact;
   stable test-mode collection iteration order.
2. Semantic diff (`hum diff`): changes reported as effect/contract/
   capability deltas. The code-review killer demo.
3. Machine-applicable fixes: diagnostics carry structured edits;
   `hum fix --apply`.
4. Sandboxed execution flags: capability policy at the `hum run` boundary
   (`--allow`/`--deny`) when IO capabilities arrive.
5. Fault containment doctrine: crash isolation, supervision, restart
   budgets. Research in flight (2026-07-08); required before concurrency
   design.
6. Units of measure: research in flight (2026-07-08); library-plus-checker
   candidate before core syntax.
7. Language editions: source longevity mechanism in
   RELEASE_AND_VERSIONING.md before any public alpha stability promise.
8. Contract check policy ADR: debug runs all contracts; release runs
   boundary and unproved contracts, elides mechanically proved internal
   ones; compiler exports proved | boundary | unproved | external-trust
   classification as build evidence. Needed before profiles or a release
   mode; higher-order blame wrappers gate closures alongside effect
   polymorphism.
9. Predicate grammar v1: grown only from the contract wishlist above.
10. List operation surface: smallest growable-list API (Session D friction);
    design lands after 0014 because growth and aliasing are ownership
    questions.

## Appendix: the twelve-program corpus

Programs 1-10 from the original suite; 11-12 added from the Session D
friction ledger per the probes rule.

1. Doubly linked list with back-pointers.
2. Arena-allocated graph with cycles, freed as a unit.
3. Mutating a collection while iterating it (must be rejected; diagnostic
   text is part of the score).
4. Callback registry that stores references to caller-owned state.
5. Parser holding a slice into the buffer it owns.
6. Producer/consumer ownership handoff between two workers.
7. Memoizing cache read through a shared path.
8. Swapping two fields of one record.
9. Task returning a reference derived from one of its parameters.
10. Transaction that must commit or roll back exactly once (linear
    resource; ties to STATE_MODEL.md).
11. Update one field of a record while preserving the rest (Session D
    friction: functional update vs in-place mutation is an ownership
    question, not syntax sugar).
12. Builder that accumulates items into a growing list and then hands the
    finished list away (append, amortized growth, and end-of-build
    ownership transfer; ties to friction "no list append").

Rubric per program per candidate: can it express the program safely; what
the user actually writes; what diagnostic appears on misuse; can a beginner
explain why the rule exists. Quantified gate: the recommended model clears
at least eight of twelve without escape hatches or incidental allocation,
and every escape hatch names its cost.

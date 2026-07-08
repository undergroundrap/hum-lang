# Ownership, Contract Policy, Effect Polymorphism, And Determinism Research

Date: 2026-07-08
Status: distilled snapshot, triaged by external reviewer

## Provenance And Confidence

Source: agent-run deep research commissioned against the four open design
decisions after WORKORDER Sessions A-C. Citation markers in the raw report
were tool-mangled and are not independently verifiable links; the substance
below was cross-checked against reviewer knowledge and kept only where it
matches known primary sources or is explicitly flagged. Claims the report
itself marked "unknown" or "thin" are preserved as such — that honesty is
part of the value.

## 1. Ownership Model

Findings accepted:

- Rust's remaining ergonomic pain points are increasingly well-scoped and
  under active repair inside the model: Polonius-class flow-sensitive
  borrowing (conditional returns, lending iterators), view types and field
  projections (disjoint borrows through methods), and internal-reference
  proposals (self-referential values). These target the exact cases in the
  WORKORDER bake-off suite.
- Cyclone's own retrospectives concede pure lexical regions were awkward
  enough to require dynamic regions, unique pointers, and RC escape hatches.
  MLKit sustained region inference for decades but needed GC hybrids and had
  a published correction to prior GC-safety claims. Verona remains research,
  not ready. Lesson: regions survive as a substrate or explicit opt-in, not
  as a mainstream language's whole ergonomic story.
- Hylo/mutable value semantics remains a design-research signal, not a
  deployment evidence base. No 2024-2026 production case studies surfaced.
- Austral's linear types are the cleanest fit for exactly-once protocols
  (commit-or-rollback); Lobster's compile-time RC usability claim is
  attractive but self-reported; Vale's generational references trade static
  discipline for runtime checks.
- Teachability evidence for any model is thin. "Borrow checking is
  learnable" is established; "which model is easiest at equal safety" is
  not. Claims here are mostly anecdote, including from Rust's own designers.

Working recommendation entering the bake-off (null hypothesis, not verdict):

```text
core: Rust-like ownership and borrowing, designed against the known 2024-2026
      repair list from day one (flow-sensitive borrows, place-based lifetime
      story, disjoint-field projections)
plus: linear resources for exactly-once protocols (ties to STATE_MODEL.md)
plus: arenas/regions as explicit, source-visible opt-in
```

The bake-off still runs: Hum's surface is not Rust's, and the ADR must kill
alternatives explicitly. Mutable value semantics wins only if it beats this
null hypothesis on the ten-program corpus.

## 2. Runtime Contract Check Policy

Findings accepted:

- No mature contract ecosystem is "always on everywhere" or "debug only."
  Eiffel: per-project assertion settings. Ada 2022: `Assertion_Policy`
  (Check/Ignore). D: contracts stripped in release, explicitly not for
  input validation. Racket: always-monitored module boundaries with blame.
  Dafny: static proof culture plus `expect` for runtime evidence.
- Measured overhead is workload-sensitive: Racket literature spans low
  single digits to ~83% on realistic higher-order benchmarks. A single
  global policy cannot be right.
- SPARK's operational ladder (Bronze: data flow; Silver: absence of runtime
  errors; Gold: key integrity; Platinum: full functional proof) is the
  concrete model for per-unit assurance levels. AdaCore practice: prove
  checks cannot fail, then let the compiler remove them; documented legacy
  case studies delete in-body guards made unreachable by trusted contracts.
- Findler-Felleisen: once functions are values, caller/task blame at the
  failure site is insufficient; argument positions reverse blame, result
  positions preserve it. Correct higher-order blame needs wrappers that
  carry contract provenance.

Direction for the future decision record (not yet accepted):

```text
debug/test: run all contracts
release:    run contracts at trust boundaries and all unproved contracts;
            mechanically proved internal contracts may be elided
always:     compiler classifies every contract as proved | boundary |
            unproved | external-trust and exports the classification as
            build evidence (fits evidence-native directly)
```

This resolves the Session C observation that `divide`'s body guard is
unreachable under checked execution: keep both only while the boundary is
untrusted; elide or delete once proved or boundary-enforced.

## 3. Effect Polymorphism

Findings accepted, including one correction to RESEARCH_MAP cluster 15:

- Koka (row polymorphism), Flix (Boolean effect formulas), and Effekt
  (capability passing) all converged on effect polymorphism as the thing
  that keeps higher-order code out of annotation tar. Flix states directly
  that `map` must be effect-polymorphic in its callback.
- Correction: cluster 15 previously said monomorphic closure effects were
  an acceptable first answer. The evidence says that is acceptable only
  while the standard library stays first-order. The moment `map`, `fold`,
  `retry`, `with_timeout`, or `parallel_map` exist, monomorphic effects
  force API duplication or worst-case over-approximation. The minimum
  viable design is one mechanism for "this function's effect includes its
  callback's effect."
- OCaml 5 is live proof that shipping runtime effects without effect typing
  is viable and that the debt stays visible years later.
- Scala capture checking (2024-2026) is the strongest signal that
  capability-capture tracking may be a lighter adjunct for resource safety,
  with reported low-syntax migrations of collections and an async library —
  but it remains experimental. A research bet, not a foundation.
- Async: Go and Java Loom avoid function coloring by paying in the runtime;
  Rust and Swift pay in infectious signatures. If effects are already in
  signatures, async-as-effect is the coherent choice for Hum, but only with
  a runtime that keeps direct-style code pleasant and effect-polymorphic
  combinators that prevent sync/async API bifurcation.

## 4. Deterministic Execution

Findings accepted:

- FoundationDB, TigerBeetle (VOPR: whole cluster on one thread, virtual
  time, continuous fault injection), Antithesis, and rr/Pernosco all pay a
  large retrofit cost to reclaim determinism: wrapping every source of
  nondeterminism. Hum's capability discipline makes those wrappers
  structural rather than retrofitted — the strongest positive interaction
  between Hum's existing design and any researched domain.
- No mainstream language ships a comprehensive built-in deterministic mode
  (seeded random, virtual clock, deterministic scheduling) as a first-class
  feature. Hum would be productizing an infrastructure pattern, not copying
  a precedent: risk and differentiator at once.
- Golden-output tests are only stable if collection iteration order,
  serialization order, scheduling, clock, and randomness are deterministic
  in test mode or recorded as capability inputs. Otherwise they are flaky
  snapshots with a confident name.

Design gates adopted into WORKORDER backlog item 1:

- Virtualize time and scheduling before the standard library grows ambient
  clock APIs (ordering constraint, cheap now, expensive later).
- A failing deterministic-mode test must replay bit-for-bit from a single
  artifact.
- Core collections need stable test-mode iteration/serialization semantics.

## Open Verification Debt

- IEC 62304 normative text is paywalled; the defensive-code-vs-proof stance
  for medical software could not be confirmed from public sources. Buy the
  standard before any medical-wedge claim.
- Racket overhead figures, TigerBeetle VOPR specifics, and Hylo project
  status were consistent with reviewer knowledge but not re-verified against
  live primary sources in this pass.

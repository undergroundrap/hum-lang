# Hum Governance And Evolution

Date: 2026-07-06

## Purpose

Hum needs a way to accept ideas without becoming a pile of good intentions.

If hundreds of people eventually argue about the language, the process must keep
the language small, teachable, safe, fast, and agent-readable.

Hum should learn from C++ without becoming C++:

- welcome deep expertise
- require written rationale
- preserve compatibility deliberately
- reject feature sprawl
- avoid committee compromise syntax
- make evidence stronger than preference

## Governance Model

Hum uses a BDFL model with evidence-first review.

BDFL authority is constrained by the BDFR safety directive:

- protect the developer machine
- protect users and downstream systems that Hum programs touch
- keep Hum tooling offline-first until trust gates exist
- prioritize parser, diagnostics, semantic graph, and local proof before risky features
- adopt existing ecosystems through safe interop before trying to rewrite them
- preserve portability as a product feature

See [BDFR_SCOPE_AND_SAFETY_DIRECTIVE.md](BDFR_SCOPE_AND_SAFETY_DIRECTIVE.md).
See [SAFETY_OF_MAKER_AND_USER.md](SAFETY_OF_MAKER_AND_USER.md) for the
two-sided safety philosophy.

The project founder is the Benevolent Dictator For Life for the language vision
and final direction. That role exists to preserve taste, coherence, and courage.

But final decisions should still leave a public reasoning trail:

- what problem was solved
- what alternatives were rejected
- what evidence was considered
- what risks remain
- what migration path exists

The BDFL can say no because a feature feels wrong for Hum. The decision record
should still explain the taste, because future contributors need to learn the
language's judgment.

## Decision Principle

```text
Taste chooses the direction.
Evidence earns stability.
Compatibility earns trust.
Pedagogy earns adoption.
```

## Review Bodies

As Hum grows, create small domain groups. They advise; they do not own the soul
of the language.

Suggested groups:

- language design
- compiler and IR
- safety and security
- performance and benchmarks
- standard library
- tooling and diagnostics
- pedagogy and beginner experience
- agent semantics and semantic graph

Each group should be responsible for evidence, not politics.

## Change Types

### Design Note

Explores an idea. No commitment.

### Experiment

Implemented behind an experimental flag or in examples. No stability promise.

### Candidate

Feature is coherent enough for serious use, but may still change.

### Stable

Feature has compatibility guarantees.

### Deprecated

Feature remains available but has a planned replacement.

### Removed

Only for experimental features or edition boundaries with a migration path.

## Feature Admission Gates

A feature cannot become stable unless it has:

1. Problem statement.
2. Beginner explanation.
3. Senior-engineer explanation.
4. Syntax examples.
5. Semantic graph representation.
6. Diagnostics design.
7. Formatter behavior.
8. Ergonomics and operator impact.
9. Tests and generated-test story.
10. Runtime performance and allocation impact.
11. Compile-time impact.
12. Optimization and DSA evidence when relevant.
13. Toolchain impact: syntax highlighting, LSP, debugger, profiler, and source maps.
14. Runtime profile impact: normal, realtime, engine, safety-critical, or certified-toolchain behavior.
15. Safety and security analysis.
16. Maker/user safety impact.
17. Agent documentation impact.
18. Cross-language regret ledger check.
19. Language project risk register check.
20. Migration and compatibility story.
21. Rejected alternatives.
22. BDFL decision note.

If a feature cannot pass these gates, it can remain experimental.

## Stability Levels

```text
sketch        docs only, may disappear
experimental  available behind a flag, no compatibility promise
candidate     expected shape, still allowed to change
stable        compatibility promise
legacy        supported only for migration
```

Hum should make stability visible in docs and semantic graph output.

## Compatibility And Editions

Hum should not break stable code casually.

Use editions for rare language-level shifts:

```text
edition 2026
edition 2027
```

Edition changes must include:

- formatter migration
- semantic graph migration
- diagnostic migration
- package metadata migration
- compatibility report
- old/new examples

Editions are a release valve, not a broom.

## Experimental Features

Experimental features must be visibly marked:

```text
use experimental feature checked cost prove
```

or by package/build config.

Experimental features may be removed. Stable features should not.

## BDFL Vetoes

The BDFL should veto features that:

- add multiple ways to express the same core idea
- make source harder for humans to scan
- make semantic graphs weaker
- make agents guess instead of consume facts
- hide effects, allocation, mutation, or failure
- weaken safety defaults
- make the compiler much harder without major user benefit
- only exist because another language has them

A veto should be short, direct, and recorded.

## Standard Library Governance

The standard library should be harder to grow than a package ecosystem.

No API enters `std` unless it has:

- contract blocks
- examples
- tests
- fuzzing when input-facing
- benchmarks when performance-facing
- optimization and DSA evidence when structure-facing
- misuse guidance
- semantic graph docs
- stability promise

The standard library should prefer small, sharp, composable APIs over a huge
surface area.

See [STDLIB_CONSTITUTION.md](STDLIB_CONSTITUTION.md) for the full admission packet and stability rules.

## Decision Records

Every accepted or rejected major decision should get a short decision record:

```text
docs/decisions/0001-use-task-not-fn.md
docs/decisions/0002-tests-are-top-level-test-kind.md
```

A decision record should include:

- status
- context
- decision
- consequences
- alternatives rejected
- BDFL note

## Versioning And Tags

Hum follows the local release policy in [RELEASE_AND_VERSIONING.md](RELEASE_AND_VERSIONING.md).

Rules:

- keep `VERSION` and `Cargo.toml` in sync
- use SemVer-shaped versions from the beginning
- treat `0.0.x` as pre-alpha snapshots, not compatibility promises
- use annotated `vX.Y.Z` Git tags only after release-readiness checks pass
- do not push tags or remotes without explicit human approval

## Release Rhythm

Early Hum should move in milestones, not calendar promises.

Suggested rhythm:

- Milestone 0: semantic graph and diagnostics
- Milestone 1: executable core and tests
- Milestone 2: ownership and effects
- Milestone 3: stores and stdlib lab
- Milestone 4: unsafe and FFI
- Milestone 5: compile-time discipline, Nectar, and editor spine
- Milestone 6: native backend
- Milestone 7: self-hosting preparation
- Milestone 8: agent-native tooling and 2050 developer experience

After real users exist, add regular preview releases and rare stable releases.

## Community Culture

Hum should welcome strong opinions but require strong artifacts.

Good contribution:

- shows the problem
- gives before/after Hum examples
- explains safety/performance impact
- adds diagnostics
- updates semantic graph schema
- adds tests or benchmarks

Weak contribution:

- asks for a feature because another language has it
- argues from taste without examples
- adds syntax without diagnostics
- adds magic agents must infer
- grows the language without reducing confusion

## Brutal Warning

A language can die from bad governance before it dies from bad code.

If Hum becomes a place where every smart person adds their favorite abstraction,
it will fail.

Hum should be opinionated enough to say no, transparent enough to earn trust,
and empirical enough to improve when evidence beats taste.

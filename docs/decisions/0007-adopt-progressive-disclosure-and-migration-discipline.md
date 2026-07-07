# 0007: Adopt Progressive Disclosure And Migration Discipline

Date: 2026-07-07
Status: accepted

## Context

A user-provided transcript of a Chris Lattner interview covered lessons from
LLVM, Clang, Swift, MLIR, and Mojo. The lessons map directly onto Hum's biggest
risks: overgrown syntax, premature stability promises, rewrite-cliff adoption,
backend marketing before evidence, and feature accretion under pressure.

Hum already has evidence-native and resource-explicit doctrine. This decision
adds the adoption and complexity discipline needed to keep that doctrine usable.

## Decision

Hum will treat progressive disclosure, staged compatibility, migration tooling,
and evidence-backed backend claims as language design requirements.

Accepted rules:

1. Ordinary Hum must stay small at the point of use. Advanced concepts such as
   ownership, profiles, unsafe, FFI, compile-time execution, generics, and native
   backend details should appear only when the program needs them.
2. No major feature is stable until it has surface syntax, core semantics,
   diagnostics, semantic graph facts, tooling impact, profile impact, tests or
   evidence, and teaching material.
3. Pre-alpha and alpha Hum may intentionally break source and schemas, but only
   with explicit versioning, release notes, and a migration story. Stable Hum
   requires a higher compatibility bar.
4. Adoption must be incremental. Hum should support one-module, one-boundary,
   one-tool, or one-hot-path adoption rather than all-or-nothing rewrites.
5. Migrator tools are part of the language plan. Formatting, syntax upgrades,
   edition changes, schema changes, and interop wrappers should become
   mechanical where possible.
6. Hum rejects special-case syntax added for one demo, framework, or slide. Prefer
   one general, checkable mechanism over many local conveniences.
7. Backend and performance claims require hard evidence: reproducible fixtures,
   target details, benchmark harnesses, resource reports, and comparison
   baselines.
8. Major design work should begin with a research sweep or design note that
   studies prior systems and rejected alternatives.

## Consequences

Hum's beginner path should remain readable and compact even as the language gains
systems power.

The project can keep changing quickly before public stability, but it must be
honest about that instability and build the tools that help users move.

The backend plan stays staged: semantic graph and Hum IR first, interpreter for
executable semantics, Cranelift for first native proof, LLVM for mature optimized
AOT, and MLIR only when layout, vector, tensor, sparse, GPU, or accelerator work
justifies it. See [0008](0008-adopt-swappable-backend-ladder.md).

The standard library and language surface should resist local sugar unless the
underlying mechanism is general enough to preserve diagnostics, formatting,
profile policy, graph facts, and compile-time budgets.

## Alternatives Rejected

- Promise source stability before the language has enough mileage.
- Require users to rewrite a whole system before Hum is useful.
- Add syntax because it makes one example prettier.
- Treat LLVM, MLIR, or any backend as Hum's semantic center.
- Claim performance leadership before benchmark and resource evidence exists.
- Let chat memory or personal taste replace written design records.

## BDFL Note

Hum can be ambitious only if ordinary code stays approachable. Power should be
there when needed, but the language should not make every user pay the cognitive
cost for every advanced feature on day one.
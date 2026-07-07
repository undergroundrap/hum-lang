# Chris Lattner Compiler Lessons

Date: 2026-07-07
Source status: user-provided transcript of a Chris Lattner interview; raw transcript is not committed.

## Purpose

This note distills compiler, language, and adoption lessons from a long-form
interview with Chris Lattner, creator of LLVM, Clang, Swift, MLIR, and Mojo.

This is not a transcript archive. It is a Hum design input. The goal is to turn
experience from LLVM, Swift, MLIR, and Mojo into constraints on Hum's language,
compiler, tooling, adoption path, and claim discipline.

## Executive Read

The strongest lesson is that successful languages are not won by syntax alone.
They win by combining a coherent technical core with adoption mechanics:
interop, migration, tooling, docs, diagnostics, staged compatibility, and a way
for users to adopt the new thing without rewriting the old world all at once.

The second strongest lesson is that complexity must be staged. A language can
have serious power, but it should not force beginners, normal users, or even
experienced engineers to hold all the machinery in their heads for ordinary
programs.

The third lesson is that big backend claims require hard validation. It is not
enough to say a new compiler stack is elegant. A project must prove the hard
hypothesis with targeted prototypes, benchmarks, and evidence before building a
marketing story around it.

## Lessons For Hum

### 1. Build To Understand Before Scaling Consensus

Lattner repeatedly describes learning by building. Swift began as a small,
private prototype before it became an organization-wide commitment. The lesson
for Hum is not secrecy for its own sake. The lesson is that unclear ideas need a
small space where they can become coherent before a large group turns every open
question into process.

Hum rule: prototype the core, reports, diagnostics, and examples before asking a
large audience to evaluate the language. Scale the conversation through docs and
machine-readable artifacts once there is something concrete to inspect.

### 2. Adoption Is Migration, Not Conversion

Swift succeeded partly because Objective-C users could move one class or feature
at a time. Mojo's Python strategy follows the same adoption logic: use the old
ecosystem while gradually moving hot or important code into the new language.

Hum rule: Hum should never require an all-or-nothing rewrite to be useful.
Interop, process boundaries, C/Rust/Python/Wasm bridges, and future migrator
tools are adoption infrastructure, not optional extras.

### 3. Honest Instability Beats Fake Stability

Swift could launch before its design was fully settled because the team was
explicit that source compatibility would break while the language gained real
mileage. The relief valve mattered: users could opt in without believing they
were entering a permanent compatibility contract.

Hum rule: pre-alpha and alpha Hum must be explicit about what may break. Stable
promises require versioned schemas, migration tooling, examples, tests, docs,
and enough real usage to justify the promise.

### 4. Progressive Disclosure Is A Language Feature

A small program should not force users to learn the whole advanced model. Power
should be reachable as users need it, not pushed into every hello-world path.
This is not only a beginner issue. Experienced users also benefit when ordinary
code has less mental overhead.

Hum rule: the first screen of Hum should teach `task`, `why:`, `uses:`,
`changes:`, `needs:`, `ensures:`, and `does:` before exposing ownership,
profiles, unsafe, FFI, compile-time execution, generics, or backend details.

### 5. Avoid Special-Case Accretion

The transcript is blunt about a Swift failure mode: rapid adoption and product
pressure can lead to feature-specific syntax and special cases that hurt
compile-time performance and language simplicity.

Hum rule: do not add syntax just to make one demo, framework, or screenshot look
better. Prefer one general, checkable mechanism over many local conveniences.
Macros, compile-time execution, builders, decorators, and generated code must
wait until the compiler can explain them through diagnostics, graph facts,
formatting, profile policy, and timing budgets.

### 6. Delay Big Features Until The Core Is Excellent

The Go generics story and Mojo's decision to defer classes both support the same
lesson: a missing feature can be healthier than a rushed feature. Experience can
make the eventual design better.

Hum rule: no generics, async, macro system, user-defined operators, inheritance,
large FFI surface, or native backend prestige until the formal core, effects,
ownership, diagnostics, semantic graph, and docs are strong enough to carry it.

### 7. Value Semantics And Ownership Fit Modern Hardware

The transcript argues against treating strict pure-functional data copying as the
best general model for modern systems. Cache locality, in-place updates under
unique ownership, and predictable layout matter on current hardware.

Hum rule: Hum should favor value semantics plus explicit ownership and mutation.
Persistent data structures may exist, but hot paths and systems profiles need
layout, allocation, and mutation to be visible and optimizable.

### 8. Survey The Whole Field Before Inventing

Lattner describes studying many old and new languages and taking specific good
ideas where they fit. The lesson is not to copy any one language. It is to build
a map before building a castle.

Hum rule: significant language features require a research sweep or design note
that studies old systems, modern systems, failure modes, and rejected
alternatives before the feature hardens.

### 9. General Compiler Infrastructure Can Beat Point Solutions

The Modular/Mojo discussion argues that a sufficiently general compiler stack can
beat hand-specialized kernels because humans cannot special-case every hardware
shape forever. The caveat is that the compiler must preserve the right
abstractions long enough to optimize them.

Hum rule: resource and layout facts must survive into Hum IR. `hum graph`,
`hum resource-report`, profile reports, and future optimization evidence are the
foundation for backend intelligence.

### 10. Use LLVM's Good Parts, Not LLVM As A Soul

The transcript reinforces a point Hum already documents: LLVM is useful, mature,
and important, but it is not a full language architecture. MLIR exists partly
because modern hardware and domain-specific lowering need multiple levels of
abstraction.

Hum rule: keep LLVM as a serious optimized AOT backend target. Keep Cranelift as
a likely first native experiment. Keep MLIR as the future accelerator and data
layout path. Hum's semantic graph and Hum IR must come first.

### 11. Validate Hard Hypotheses Before Marketing

The transcript's Modular example is a useful discipline: build the hard compiler
prototype, compare against a serious baseline, and only continue the claim if the
prototype proves the hypothesis.

Hum rule: do not claim Hum is faster than Rust, C++, CUDA, or anything else
without a benchmark harness, source fixtures, target details, reproducibility,
and counterexamples. A claim that cannot survive `hum resource-report`, profile
policy, tests, and benchmarks should stay private.

### 12. Clear Ownership And Scaled Docs Matter

Large technical changes need a decision owner, but they also need written
arguments that scale beyond hallway conversations. Hum's BDFL model only works
if the reasoning is visible and evidence-backed.

Hum rule: major features need a decision record or RFC, not just chat memory.
The BDFL can decide taste, but the decision should leave evidence, tradeoffs,
and rejected alternatives behind.

## Direct Hum Changes From This Note

Accepted design direction:

- progressive disclosure is a language and tooling requirement
- migration tooling is part of the adoption story
- early instability must be explicit and versioned
- special-case syntax is rejected unless it collapses into a general mechanism
- backend claims require prototypes and benchmarks
- Hum IR must preserve resource, layout, ownership, effect, and evidence facts

Related docs:

- [../decisions/0007-adopt-progressive-disclosure-and-migration-discipline.md](../decisions/0007-adopt-progressive-disclosure-and-migration-discipline.md)
- [../ARCHITECTURE.md](../ARCHITECTURE.md)
- [../INTEROP_AND_PORTABILITY.md](../INTEROP_AND_PORTABILITY.md)
- [../BACKEND_STRATEGY.md](../BACKEND_STRATEGY.md)
- [../RESEARCH_MAP_2026.md](../RESEARCH_MAP_2026.md)

## Do Not Overlearn

This transcript does not mean Hum should copy Swift, Mojo, Python, MLIR, or
Apple's secrecy model. It means Hum should copy the durable engineering
patterns:

- prototype before broad commitment
- write the reasoning down
- make migration incremental
- be honest about compatibility
- keep simple code simple
- delay features until their core is ready
- prove performance claims with evidence
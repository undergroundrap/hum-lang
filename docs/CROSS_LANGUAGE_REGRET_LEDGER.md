# Hum Cross-Language Regret Ledger

Date: 2026-07-06

## Thesis

Hum should be built like we have already lived through C++, Go, Rust, Zig,
Python, JavaScript, Swift, Java, and every major build tool ecosystem.

The goal is not to copy one language. The goal is to refuse the regrets that
other communities had to discover in production.

This document is a design checklist. A Hum feature should be viewed suspiciously
if it recreates one of these known pain patterns.

## Meta Rule

Every feature must answer:

```text
Which old language regret does this avoid?
Which old language regret could this recreate?
What tool will catch misuse before it becomes culture?
```

If the answer is vague, the feature is not ready.

## 2026 Pain Sweep Update

See [LANGUAGE_PAIN_SWEEP_2026.md](LANGUAGE_PAIN_SWEEP_2026.md).

The sweep sharpens this ledger with current developer pain around Rust, Zig,
C++, Python, Go, JavaScript/TypeScript, Java/C#, Swift/Kotlin, and Epic Games'
Verse.

New rule:

```text
A Hum feature is not good because another language has it.
A Hum feature is good only if it removes a known tax without recreating a worse one.
```

The most important cross-language taxes are feedback latency, package/build
friction, unsafe escape hatches, hidden runtime cost, clever abstraction,
concurrency ambiguity, and migration pain.

## COBOL Lessons

### Regrets To Avoid

- natural-language mimicry creating a false sense of simplicity
- verbosity burying the algorithm under filler words
- multiple English spellings for one operation
- prose that looks understandable but has narrow hidden syntax rules
- business-readable text replacing precise compiler-owned semantics

### Hum Response

- formal readability over English mimicry
- one canonical spelling per core concept
- readable intent blocks that become checked facts before they become authority
- precise Core Hum lowering before stable executable syntax
- diagnostics that explain the formal model behind friendly words

### Design Rule

Readable code should be easy to scan because the structure is formal and small,
not because it reads like a sentence.

See [decisions/0009-adopt-formal-readability-not-english-mimicry.md](decisions/0009-adopt-formal-readability-not-english-mimicry.md).

## C++ Lessons

### Regrets To Avoid

- too many ways to express the same idea
- unsafe defaults
- undefined behavior as normal user risk
- header/build complexity
- template and generic error complexity
- fragmented package/build ecosystem
- style fragmentation
- backwards compatibility freezing bad early choices forever

### Hum Response

- one obvious syntax for each common operation
- memory safe by default
- unsafe must carry `why:`, `needs:`, `protects:`, `trusts:`, and `watch for:`
- Nectar is first-party from the beginning
- `humfmt` is first-party from the beginning
- `chirp` catches unclear, risky, and overly clever patterns
- editions are rare but planned
- feature admission requires migration and rejected alternatives

### Design Rule

Do not add both options because both camps are loud.

Pick the Hum-shaped answer and document the rejected path.

## Rust Lessons

### Regrets To Avoid

- compile-time pain becoming a normal tax
- macro expansion hiding too much generated code
- generic/trait solving becoming hard for humans and agents
- unsafe requirements living in prose instead of checked structure
- ownership learning cliff
- async/concurrency complexity stabilizing before the model feels obvious

### Hum Response

- `hum check` and `nectar check` are first-class fast paths
- feature proposals need compile-time impact
- macros are delayed and must be visible in the semantic graph
- unsafe review packets are syntax
- ownership diagnostics explain responsibility and permission
- async waits until cancellation, allocation, effects, and executor boundaries are clear

### Design Rule

Keep Rust's safety soul. Do not copy every ergonomic cliff.

## Go Lessons

### Regrets To Avoid

- omitting core abstraction tools so users simulate them poorly
- error handling becoming repetitive boilerplate
- lack of sum types / variant types forcing awkward interface patterns
- garbage collection making the language unsuitable for some low-level systems work
- simplicity becoming a reason to reject useful precision

### Hum Response

- include enums / tagged unions / algebraic data types early
- use typed `Result` and `fails when:` instead of exceptions or unchecked error values
- no ordinary null
- no hidden GC dependency for systems code
- keep explicit errors, but make generated tests and diagnostics reduce repetition
- require compile-time and beginner-readability budgets for abstractions

### Design Rule

Simplicity is not the same as omission.

A missing feature is only elegant if users do not rebuild a worse version in
libraries and conventions.

## Zig Lessons

### Lessons To Keep

- no hidden allocation
- no hidden control-flow magic
- comptime can be powerful when transparent
- build modes should make safety/performance tradeoffs explicit
- allocators should be visible at important API boundaries
- docs and tests can be integrated tightly

### Regrets To Avoid

- manual memory becoming too much burden for ordinary safe code
- compile-time execution becoming an unbounded second language
- instability making production users wait too long
- build system power becoming its own learning cliff

### Hum Response

- allocation claims are visible through `allocates:` and `cost:`
- safe ownership model should remove ordinary manual lifetime burden
- compile-time execution is delayed and budgeted
- Nectar uses declarative project metadata first
- editions and stability levels are planned early

### Design Rule

Expose low-level power without making every programmer be the allocator and
lifetime expert all day.

## Python Lessons

### Regrets To Avoid

- packaging and environment fragmentation
- runtime type failures that could have been caught earlier
- dependency confusion between system packages, virtual environments, and project packages
- performance cliffs hidden behind friendly syntax

### Hum Response

- Nectar is first-party
- lockfiles by default
- static types by default
- performance contracts visible in source
- no hidden dynamic type fallback in systems code

### Design Rule

Beginner-friendly syntax must not mean runtime surprises.

## JavaScript And TypeScript Lessons

### Regrets To Avoid

- package ecosystem chaos
- too many transpilation/build layers
- semantic gaps between source language and runtime language
- configuration sprawl
- type system bolted onto a dynamic runtime with escape hatches everywhere
- intentional unsoundness becoming invisible to ordinary users
- static typing moving bugs from logic into tooling and configuration failures

### Hum Response

- Nectar owns package/build conventions
- Hum compiler owns source truth
- semantic graph is emitted by the compiler, not inferred by fragile tools
- configuration starts minimal
- escape hatches are explicit unsafe or foreign boundaries
- soundness holes must be explicit, named, and visible in diagnostics
- async, dependency, and configuration complexity are part of the language/tooling design

### Design Rule

Do not let the toolchain become a stack of half-authoritative translators.

## Swift Lessons

### Regrets To Avoid

- type inference and generics causing surprising compile-time costs
- frequent source churn hurting long-lived codebases
- excellent language ideas trapped by platform/ecosystem coupling

### Hum Response

- type inference is bounded and explainable
- compile-time budgets are feature gates
- editions exist for controlled evolution
- Hum should be platform-portable from the beginning

### Design Rule

Inference should save typing, not hide the program from the compiler team.

## Java And C# Lessons

### Regrets To Avoid

- null becoming ambient language debt
- annotation-heavy programming replacing real effects
- GC and runtime assumptions limiting low-level control
- enterprise framework style overwhelming the core language
- type erasure limiting optimization and runtime understanding
- object identity and boxing becoming default layout tax
- checked exceptions proving that visible failure can still become paperwork

### Hum Response

- no ordinary null
- effects are language-level blocks, not only annotations
- no required GC for systems code
- standard library stays smaller and more intentional than the ecosystem
- generics preserve enough layout/type information for optimization
- value-like records and explicit identity are core design, not later retrofits
- typed `Result` and `fails when:` make failure visible without exception-signature sprawl

### Design Rule

Do not turn source code into paperwork. Checked intent blocks must remain readable.

## Haskell And Scala Lessons

### Regrets To Avoid

- abstraction power outrunning team readability
- error messages becoming type-theory puzzles
- too many symbolic operators and clever encodings
- library authors designing languages inside the language

### Hum Response

- abstraction admission requires beginner and senior explanations
- symbolic density is limited
- `chirp` flags overly clever public APIs
- semantic graph must represent abstractions clearly enough for tools and agents

### Design Rule

If only the author can explain it, it is not Hum-shaped.

## Tooling Lessons

Even beautiful syntax needs tools.

Hum still needs:

```text
humfmt   because style drifts
chirp    because readable code can still be misleading
nectar   because dependencies and builds are part of the program
hum graph because agents and IDEs need compiler facts
```

The dream is not "no formatter needed." The dream is:

```text
The formatter almost never surprises you.
The linter teaches instead of nags.
The package manager makes trust visible.
The compiler explains the same way a senior engineer would.
```

## Optimization And DSA Lessons

Hum should learn from modern data-structure research without turning every paper
into a standard feature.

Regrets to avoid:

- clever structures entering `std` before they survive hostile inputs
- micro-optimizations distracting from algorithm and layout wins
- benchmark wins that hide compile-time, memory, binary-size, or security losses
- hardware-specific tricks becoming the only fast path
- research features that agents and beginners cannot understand from the source

Hum response:

- every optimized structure starts in a lab tier
- benchmark against existing language and domain baselines
- require adversarial tests for maps, parsers, allocators, and concurrency
- expose layout, allocation, and hardware assumptions in semantic graphs
- keep the default path safe, boring, and explainable

See [OPTIMIZATION_AND_DSA_STRATEGY.md](OPTIMIZATION_AND_DSA_STRATEGY.md).

## Regret Gates For Every Feature

Before stabilizing a feature, answer:

1. Does it create another way to do something Hum already does?
2. Does it hide allocation, mutation, IO, failure, control flow, or unsafe behavior?
3. Does it make common code harder for beginners to read?
4. Does it make large projects slower to check?
5. Does it make semantic graph output less clear?
6. Does it make agent repair harder?
7. Does it require style conventions the formatter cannot enforce?
8. Does it require lints to prevent common misuse?
9. Does it interact cleanly with Nectar packages and lockfiles?
10. Does it preserve optimization and DSA evidence where relevant?
11. Can we explain the feature in one beginner example and one systems example?

If any answer is bad, the feature stays experimental.

## Brutal Rule

Hum should not become a museum of everyone else's good ideas.

Hum should become a small number of deeply integrated ideas that solve known
problems without recreating known regrets.

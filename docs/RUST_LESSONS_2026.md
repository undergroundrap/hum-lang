# Hum Lessons From Rust In 2026

Date: 2026-07-06

## Thesis

Rust is the language Hum should respect most.

Hum should not be a Rust rejection. Hum should be a serious attempt to ask:

```text
What would Rust-like systems safety look like if we could design the language,
compiler, package manager, diagnostics, pedagogy, and agent surfaces together
from day one in 2026?
```

Rust got many hard calls right. Hum should keep those lessons and design around
what Rust had to retrofit later.

## 1. Incremental Compilation From Day One

Rustc has a query system for incremental compilation, but the compiler guide
notes that not every compiler phase was originally designed that way.

Hum should make every major compiler phase cache-aware from the beginning:

- lexing
- parsing
- semantic graph building
- name resolution
- type checking
- effect checking
- ownership checking
- generated tests
- codegen

Hum should not treat incremental compilation as an optimization pass. It should
be part of the architecture.

## 2. Check Mode Before Codegen

Rust has `cargo check`, and developers rely on it heavily.

Hum should design this split into the language identity:

```text
hum check      source truth, no native codegen
nectar check   package truth, cached project graph
nectar build   dev binary
nectar build --release optimized binary
```

A fast correctness loop is how Hum earns daily trust.

## 3. Compile-Time Budgets For Features

Rust's performance guidance calls out macro expansion, LLVM IR bloat, and generic
monomorphization as compile-time pressure points.

Hum should require every major feature proposal to include compile-time impact:

- expected cache key impact
- worst-case examples
- code size growth
- graph size growth
- whether it blocks parallelism
- whether it creates hidden code

A feature can be elegant and still too expensive.

## 4. Macro Discipline

Rust macros are powerful, but macro-generated code can be hard to read, hard to
cache, hard to diagnose, and slow to compile.

Hum should delay general macros.

Prefer:

- compiler-known derives
- generated tests from `needs:`, `ensures:`, and `watch for:`
- declarative schemas
- explicit code-generation commands
- semantic graph transforms that tools can inspect

If Hum adds macros, they must be visible in the semantic graph.

## 5. Unsafe Requirements As Syntax

Research on unsafe Rust has found that safety requirements across unsafe APIs can
be inconsistent or insufficiently documented.

Hum should make unsafe review obligations part of the language:

```hum
unsafe task read register(address: Address) -> UInt32 {
  why:
    read mapped device memory

  needs:
    address is aligned for UInt32
    address belongs to mapped device memory

  protects:
    safe code cannot read arbitrary process memory

  trusts:
    operating system mapped this range correctly

  watch for:
    volatile read must not be optimized away
}
```

Unsafe without its review packet should not compile.

## 6. Ownership Pedagogy As A Language Feature

Rust's ownership model is powerful, but learning it is a major ramp.

Hum should teach ownership in source terms a beginner can explain:

- `owned`: responsible for the value
- `borrow`: look without owning
- `change`: exclusive permission to mutate
- `shared`: shared only through checked safe forms

The compiler should explain ownership errors in terms of responsibility and
permission before it uses formal terms.

## 7. Agent-Readable Structure From The Beginning

By 2026, coding agents are useful but still struggle with repository-wide code
structure and strict type/trait semantics in Rust-like ecosystems.

Hum should make the compiler emit the map agents wish they had:

- semantic graph
- effects graph
- test obligations
- trust boundaries
- unsafe packets
- dependency graph
- diagnostic codes
- repair hints

Agents should consume compiler facts, not scrape terminal prose.

## 8. Package Manager Trust Model Early

Cargo is one of Rust's greatest successes.

Hum should learn from it, but Nectar should make package trust more visible:

- lockfiles by default
- native dependencies explicit
- build scripts restricted
- unsafe usage summarized
- network/filesystem access declared
- package semantic graph available
- advisories integrated into diagnostics

Dependency management is security infrastructure.

## 9. Formatter And Linter From The Start

Rustfmt and Clippy helped Rust feel coherent.

Hum should ship first-party tools early:

```text
humfmt   formatter
chirp    linter and mentor tool
```

A language that cares about readability cannot outsource formatting culture.

## 10. Async And Concurrency Design Before Stabilization

Rust's async ecosystem is powerful but complex.

Hum should not stabilize async until it has clear answers for:

- cancellation
- structured concurrency
- task lifetimes
- effect propagation
- allocation behavior
- executor boundaries
- diagnostics beginners can understand

Concurrency should be visible in intent blocks, not hidden behind magic runtimes.

## Sources To Keep Studying

- Rust Compiler Development Guide: compiler stages, queries, incremental compilation, bootstrapping.
- Rust Performance Book: compile-time pressure from macros, LLVM IR, and generic instantiation.
- Unsafe Rust API research: unsafe safety requirements need systematic documentation.
- Rust learning research: ownership concepts are a major learning obstacle.
- Rust agent benchmark research: agents need better repository-wide structure and Rust-specific semantic support.

## Brutal Rule

Hum should not say "Rust should have done this" unless Hum actually does it.

Every lesson here needs a compiler feature, diagnostic, doc page, test, or tool.
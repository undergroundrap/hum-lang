# Hum Self-Hosting Plan

Date: 2026-07-06

## Thesis

Hum should compile Hum one day, but not soon.

Self-hosting is not the goal. Correct self-hosting is the proof that Hum has
become a real systems language.

The Rust bootstrap remains the reference compiler until Hum can match it on
parsing, checking, diagnostics, semantic graph output, tests, and reproducible
build behavior.

## What Self-Hosting Means

A language is self-hosting when major parts of its compiler are written in the
language itself.

For Hum, self-hosting should mean:

```text
Hum compiler source written in Hum
  compiled by the old trusted compiler
  produces a new Hum compiler
  which can compile the same Hum source again
  and produce equivalent compiler behavior
```

This is the classic stage idea:

```text
stage0: trusted Rust bootstrap compiler
stage1: Hum compiler built by stage0
stage2: Hum compiler built by stage1
```

A serious fixed-point gate should extend the idea when Hum is mature enough:

```text
stage0 builds stage1
stage1 builds stage2
stage2 builds stage3
stage2 and stage3 match byte-for-byte, or behavior-for-behavior with every nondeterministic field documented
```

Hum should not call itself self-hosted until stage1 and stage2 agree on behavior for a serious test suite. It should not call self-hosting stable until later stages prove a fixed point across diagnostics, semantic graph output, generated artifacts, and executable behavior. Build caches may shorten these runs, but no cache hit may hide a stage mismatch.

## Why Rust Stays For Now

Rust stays because it gives the bootstrap:

- memory safety without a garbage collector
- explicit errors through `Result`
- strong enums, pattern matching, and ownership
- mature tests, formatting, linting, and package tooling
- a direct path to serious systems implementation

Hum should keep many Rust ideas:

- ownership as the default memory model
- borrowing instead of hidden copying
- explicit mutation
- null-free ordinary values
- typed errors instead of exceptions as ambient control flow
- algebraic data types
- exhaustive `match`
- unsafe as a small, named boundary

Hum should not copy Rust's hardest learning cliffs unless they earn their keep.

## What Hum Must Prove Before Self-Hosting

Hum needs all of this before any compiler component is rewritten in Hum:

1. Stable grammar for the executable core.
2. Lossless parser or reliable concrete syntax tree.
3. AST and semantic graph with stable schemas.
4. Modules, packages, imports, and visibility.
5. `type`, records, enums, and generics subset.
6. `Result`, typed failures, and propagation.
7. Ownership, moves, borrows, and exclusive mutation.
8. Effects: `uses`, `changes`, allocation, IO, unsafe, concurrency.
9. Deterministic text, bytes, lists, maps, and arenas.
10. File IO and path APIs.
11. Test runner and golden-output test support.
12. Fuzz/property test harness for parsers and checkers.
13. Stable diagnostics with codes.
14. Build system and package lockfile.
15. Interpreter, Cranelift, or LLVM path capable of compiling the compiler.
16. A way to compare stage outputs exactly enough to trust them.

If even one of these is missing, the Rust bootstrap remains the compiler of
record.

## Phase 0: Rust Reference Front-End

Status: started.

Build in Rust:

- parser
- AST
- semantic graph
- diagnostics
- intent checks
- test skeleton generation
- formatter prototype

Exit criteria:

- all examples check cleanly
- semantic graph schema is documented
- diagnostics have stable codes
- parser has fuzz tests
- docs explain every syntax form to beginners

## Phase 1: Executable Hum Subset

Build enough Hum to run tiny programs:

- immutable `let`
- mutable local `change`
- `set`
- `if` / `else`
- `match`
- `for each`
- bounded `while`
- records
- enums
- `Result`
- simple tests

Exit criteria:

- task-list example executes
- generated tests can run
- no hidden mutation
- no hidden allocation for checked tasks

## Phase 2: Safety Core

Add Rust-grade safety foundations:

- move checking
- shared borrow
- exclusive changing borrow
- lifetime regions or an easier equivalent
- drop/destructor order
- no use after move
- no data races in safe code
- explicit unsafe boundary

Hum-specific additions:

- every unsafe block has `why:`, `needs:`, `protects:`, `trusts:`, and `watch for:`
- every external mutation appears in `changes:`
- every ambient capability appears in `uses:`
- every public task has readable beginner and senior summaries generated from source

Exit criteria:

- safe Hum can express the compiler AST and parser without unsafe
- unsafe policy is documented and checked
- borrow/effect diagnostics are understandable to non-experts

## Phase 3: Write Small Compiler Tools In Hum

Do not rewrite the compiler first.

Write supporting tools in Hum:

- example linter
- docs extractor
- semantic graph reader
- golden test runner
- formatter experiments

Exit criteria:

- Hum tools can read real Hum projects
- Hum tools pass the Rust bootstrap test suite
- Hum tools are easier to understand than their Rust equivalents

## Phase 4: Rewrite Non-Critical Compiler Pieces In Hum

Good candidates:

- semantic graph post-processing
- diagnostics catalog
- docs generation
- test skeleton generation
- beginner explanations

Bad early candidates:

- parser core
- type checker
- borrow checker
- optimizer
- codegen

Exit criteria:

- Rust and Hum implementations produce matching output on golden tests
- differential tests run both implementations
- failures explain which semantic fact diverged

## Phase 5: Hum Parser In Hum

Only after the grammar is stable.

The parser is the first serious self-hosting candidate because it is central but
can be heavily tested with golden files and fuzzing.

Exit criteria:

- Rust parser and Hum parser agree on AST and semantic graph for the full corpus
- parser fuzzing finds no disagreement for accepted syntax
- error recovery is at least as good as the Rust parser

## Phase 6: Hum Checker In Hum

Rewrite selected checks in Hum:

- section rules
- declared mutation checks
- effect checks
- cost-contract shape checks
- test obligation linking

Exit criteria:

- Rust checker and Hum checker agree on diagnostics and graph output
- diagnostic messages remain beginner-readable
- no performance cliff on large codebases

## Phase 7: Stage1 Compiler

Build a Hum compiler that can compile enough Hum to build itself.

Stage flow:

```text
Rust humc builds Hum humc-stage1
Hum humc-stage1 builds Hum humc-stage2
stage1 and stage2 run the same tests
```

Exit criteria:

- stage1 and stage2 semantic graphs match
- stage1 and stage2 diagnostics match
- stage1 and stage2 generated binaries pass the same test suite
- stage2 and stage3 reach a documented fixed point before any stable self-hosting claim
- release builds are reproducible on the same machine

## Phase 8: Rust Bootstrap Becomes Safety Net

Rust does not disappear immediately.

It remains:

- the trusted recovery compiler
- the differential test oracle
- the implementation people can audit while Hum matures
- the way to rebuild Hum if a self-hosted compiler regresses

Only after several stable releases should Hum consider making the Hum compiler
the default compiler of record.

## Brutal Rule

Hum can start compiling Hum when Hum makes compiler code clearer, safer, and
more checkable than the Rust version.

Not before.

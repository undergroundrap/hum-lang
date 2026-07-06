# Hum Bootstrap Compiler

Date: 2026-07-06

## Decision

Hum's first compiler front-end is bootstrapped in Rust.

This is not because Hum should become Rust with different words. It is because a
systems language that claims safety should start from a toolchain with memory
safety, strong tests, good diagnostics, and a mature build system.

The first Hum compiler must be boring before it is brilliant.

## Bootstrap Rules

The Rust bootstrap must follow these rules:

1. `#![forbid(unsafe_code)]` in compiler code.
2. No third-party crates in Milestone 0.
3. No build scripts, proc macros, or hidden code generation in Milestone 0.
4. `cargo test` must pass before changes are considered real.
5. `cargo clippy --all-targets -- -D warnings` must pass before changes are considered clean.
6. The compiler should emit structured facts before it emits machine code.
7. The parser should preserve spans so diagnostics and agents can point back to source.

These rules are intentionally strict. The compiler is the trust root.

## What Rust Gives Us

Rust gives the bootstrap:

- memory safety for ordinary compiler code
- explicit error handling
- strong enums and pattern matching for ASTs
- a stable package/build/test workflow
- Clippy for early style and bug checks
- a plausible path to later Miri, fuzzing, and model-checking tools

Rust does not solve Hum's language problem by itself.

## What Hum Must Add Beyond Rust

Hum should make visible what Rust often leaves implicit or scattered across code,
comments, docs, tests, and review culture:

- why a function exists
- what it uses
- what it changes
- what it needs before running
- what it promises after running
- what it protects
- what it trusts
- what edge cases matter
- what allocation and cost claims are being made
- what tests prove those claims

Hum's safety pitch is not "the borrow checker but nicer syntax." The pitch is:

```text
The compiler understands the senior engineer's checklist and refuses to let the
code hide important promises.
```

## Compiler Shape

Milestone 0 uses this shape:

```text
Hum source
  -> parser
  -> AST
  -> intent checks
  -> hum.semantic_graph.v0 JSON
```

Future milestones should grow toward:

```text
Hum source
  -> lossless syntax tree
  -> AST
  -> semantic graph
  -> Hum IR
  -> ownership/effect/cost analysis
  -> interpreter or Cranelift
  -> LLVM/MLIR when the semantics are stable enough
```

Do not rush to LLVM. LLVM cannot recover intent the front-end forgot.

## Lessons From Rustc

Rustc's public compiler guide describes a staged compiler pipeline: lexing and
parsing produce tokens and AST, AST lowers into HIR, HIR/THIR lower into MIR,
MIR supports borrow checking and other dataflow checks, and codegen eventually
lowers toward LLVM IR.

Hum should learn the staged architecture, not copy the whole complexity on day
one.

The Hum-specific addition is that the semantic graph is not a debug artifact. It
is a product surface for humans, agents, tools, docs, tests, and review.

## Current Commands

```powershell
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- check examples
cargo run -- graph examples/task_list.hum
cargo run -- test-skeletons examples
cargo run -- syntax
cargo run -- syntax --format textmate
```

Use the equivalent full path to Cargo only in a local shell when Cargo is not on `PATH`; do not commit machine-specific Cargo paths.

## Self-Hosting Policy

Hum should only compile Hum after it can make compiler code clearer, safer, and
more checkable than the Rust version.

See [SELF_HOSTING_PLAN.md](SELF_HOSTING_PLAN.md).

## Current Brutal Limit

The Milestone 0 parser is intentionally small. It is line/block-oriented and
built to validate the language shape, not to be the final grammar engine.
See [MILESTONE_0_GRAMMAR.md](MILESTONE_0_GRAMMAR.md) for the current bootstrap
parser contract.

The next parser upgrade should either become a deliberate hand-written recursive
descent parser with a lossless token stream, or adopt a proven incremental
syntax-tree strategy. Do not bolt complexity onto the prototype until the core
checks are worth preserving.
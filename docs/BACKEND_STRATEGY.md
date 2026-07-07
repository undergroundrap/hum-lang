# Hum Backend Strategy

Date: 2026-07-06

## Video Takeaway

The Fireship "LLVM in 100 Seconds" transcript is useful because it names the
standard compiler pipeline:

```text
source -> lexer -> parser -> AST -> IR -> optimizer -> backend -> machine code
```

That framing helps, but it leaves out the part that matters most for Hum:

```text
source -> Hum semantic graph -> Hum IR -> verifier/effect/ownership passes -> backend IR
```

Hum cannot lower directly from a Python-like readable surface into LLVM IR and
call the language designed. If we lower too early, we lose the information that
makes Hum different.

## Brutal Conclusion

LLVM should be a backend target, not Hum's soul.

LLVM is excellent for mature native code generation, optimization, link-time
optimization, sanitizers, debug info, and target coverage. It is not where Hum
should represent `why:`, `watch for:`, `protects:`, store intent, blame
semantics, ownership, effects, or verification obligations.

Hum needs its own semantic IR first.

The Chris Lattner compiler lessons reinforce the same rule from another angle:
modern hardware, accelerators, NUMA, SIMD, matrix units, and domain lowering
need the right abstractions to survive long enough for the compiler to optimize
them. Use LLVM's good parts; do not make LLVM the language design.

Backend ladder doctrine: interpreter first, Cranelift for the first native proof,
LLVM for mature optimized AOT, MLIR only when Hum has real multi-level lowering
facts, and custom backend work only after evidence shows existing backends cannot
use Hum's facts well. See
[decisions/0008-adopt-swappable-backend-ladder.md](decisions/0008-adopt-swappable-backend-ladder.md).

## Recommended Pipeline

```text
.hum source
  -> tokens
  -> concrete syntax tree
  -> AST
  -> Hum semantic graph
  -> Hum IR
  -> checks:
       types
       effects
       ownership
       allocation policy
       contracts
       blame semantics
       security obligations
  -> optimization planning
  -> backend adapter contract
  -> backend lowering:
       interpreter for first executable semantics
       Cranelift for first native/JIT/AOT experiments
       LLVM for mature optimized AOT builds
       MLIR for future accelerator/data-layout dialects
       C or Wasm as portability escape hatches
       custom Hum backend only if later evidence justifies it
```

The first compiler milestone should stop at Hum semantic graph plus JSON
diagnostics. The second should interpret a tiny core before native backend work.
Native code comes after we can prove the frontend understands Hum.

## Backend Adapter Contract

The machine-readable contract is emitted by `hum backend-contract --format json`
and documented in [BACKEND_CONTRACT_SCHEMA.md](BACKEND_CONTRACT_SCHEMA.md).

Backends are replaceable consumers of checked Hum IR. No backend may become the
owner of Hum source semantics.

Every backend adapter should preserve or explicitly report loss of:

- source spans and semantic graph node IDs
- task, test, type, and store identity
- typed failure behavior
- effect and capability facts
- ownership and aliasing assumptions
- allocation and resource facts
- profile restrictions
- unsafe and foreign boundaries
- debug/profiling provenance
- unsupported features or weakened guarantees

This contract is what lets Hum move from interpreter to Cranelift to LLVM to
MLIR or a future custom backend without rewriting the language. Backend work must
update `hum.backend_contract.v0` before it changes the meaning of any lowered
Hum program.

## Backend Options

### LLVM

Use LLVM when we want mature optimized native AOT code.

Strengths:

- broad target support
- strong optimizer and code generator
- debug info ecosystem
- sanitizers and compiler runtime support
- linkers, tooling, and production history

Risks:

- huge complexity
- C++ API and build weight
- undefined-behavior assumptions can fight Hum's safety model
- lowering too early destroys high-level semantic intent
- agents are weak at patching compiler-scale LLVM issues without strong harnesses

Hum should target LLVM through a small lowering layer only after Hum IR is
stable enough to preserve source spans, profile facts, ownership/effect facts,
and debug/profiling provenance. LLVM is a production backend target, not the
first proof that Hum can run.

### MLIR

Use MLIR when Hum needs multiple abstraction levels, domain-specific lowering,
accelerator work, data layout transformations, or hardware-aware optimizations.

Strengths:

- dialects can preserve higher-level meaning longer
- good fit for tensor, vector, GPU, sparse, affine, and hardware paths
- designed for reusable compiler infrastructure
- can eventually lower toward LLVM

Risks:

- more compiler-infrastructure complexity up front
- easy to over-engineer before the language exists
- not a machine-code backend by itself

Hum should not start with MLIR as the first implementation unless the first goal
is compiler research instead of a working language seed. But Hum's own IR should
be MLIR-shaped in spirit: explicit, verifiable, printable, and pass-driven.

### Cranelift

Use Cranelift when we want a smaller Rust-native codegen path. Cranelift is not
a newer LLVM; it is a different backend with a smaller integration surface and a
better fit for Hum's first native proof.

Strengths:

- written in Rust
- designed as a library
- fast compilation
- simpler than LLVM
- strong security/correctness culture
- no undefined behavior in its IR by design
- suitable for JIT and AOT experiments

Risks:

- less peak optimization than LLVM
- fewer targets
- less mature debug/tooling ecosystem
- not a full compiler infrastructure replacement

Cranelift is the best first native backend candidate for Hum because it lets us
build the compiler in Rust without swallowing LLVM's complexity on day one.

### Custom Hum Backend

A custom Hum backend or custom optimization stack is a future option, not a
near-term promise.

Use it only if Hum eventually has measured, backend-relevant facts that existing
backends cannot exploit well, such as resource proofs, layout guarantees,
profile-specific subsets, deterministic replay facts, verified allocation
freedom, or domain-specific lowering shapes.

Risks:

- enormous compiler maintenance burden
- target coverage and debug info start from zero
- easy to spend years rebuilding commodity backend machinery
- safety-critical credibility requires evidence, not custom code pride

Hum should earn a custom backend by first making the swappable backend contract
real with interpreter, Cranelift, and LLVM.

### C Backend

Use C as a temporary portability and debugging target, not as the flagship.

Strengths:

- easy to inspect generated output
- portable through existing C compilers
- useful for bootstrapping

Risks:

- C has undefined behavior traps
- hard to preserve Hum safety semantics
- poor fit for precise ownership/effects unless generated very carefully

### Interpreter

Use an interpreter first if it speeds semantic validation.

Strengths:

- simplest execution path
- easy diagnostics
- deterministic testing
- good for contract experiments

Risks:

- no performance story
- can hide backend problems

An interpreter is useful for milestone 1, but should not become the identity of
the language.

## Hum IR Requirements

Hum IR must preserve:

- task names and source spans
- `why:` rationale
- `uses:` capabilities
- `changes:` permissions
- `needs:` preconditions
- `ensures:` postconditions
- `keeps:` invariants
- `protects:` security obligations
- `trusts:` boundaries
- `watch for:` hazards
- `allocates:` policy
- ownership and aliasing facts
- effect facts
- store intent
- blame targets
- generated test/proof obligations

This IR should be:

- printable as text
- serializable as JSON
- stable enough for agents
- small enough for humans to read
- validated by an IR verifier
- fuzzed independently of the parser

## What LLVM Does Not Solve

LLVM does not design Hum's:

- syntax
- type system
- borrow or ownership model
- effect model
- security model
- contract language
- package model
- standard library API
- agent-facing diagnostics
- semantic graph

It only helps once Hum has already answered those questions well enough to lower
them.

## Brutal Risk

The fastest way to make a mediocre language is to build a parser, emit LLVM IR,
and celebrate too early.

That proves we can make code run. It does not prove Hum is readable, safe,
verifiable, optimizable, or worth existing.

## Near-Term Decision

For Hum v0:

1. Build parser and semantic graph first.
2. Define Hum IR before choosing final native codegen.
3. Use Rust for the compiler implementation.
4. Build an interpreter for the first executable semantics proof.
5. Use Cranelift as the first native backend candidate after the interpreter.
6. Keep LLVM as the serious optimized AOT backend target for mature native
   releases.
7. Keep MLIR as the future path for data layout, SIMD, GPU, tensor, sparse, and
   accelerator work.
8. Keep custom backend work future-only until Hum has measured facts existing
   backends cannot use well.
9. Validate hard backend hypotheses with targeted kernels, reproducible
   benchmarks, and resource reports before making performance claims.
10. Keep all backend experiments local-only until the BDFR safety directive
    allows generated-code execution.

## Sources

- LLVM overview: https://llvm.org/
- LLVM Kaleidoscope frontend tutorial: https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/index.html
- MLIR overview: https://mlir.llvm.org/
- Cranelift overview: https://cranelift.dev/
- LLVM-Bench, 2026: https://arxiv.org/abs/2607.00700
- IRFuzzer, 2024/2025: https://arxiv.org/abs/2402.05256
- Lattner compiler lessons: [research/2026-07-07-lattner-compiler-lessons.md](research/2026-07-07-lattner-compiler-lessons.md)

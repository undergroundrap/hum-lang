# 0008: Adopt A Swappable Backend Ladder

Date: 2026-07-07
Status: accepted

## Context

Hum needs a backend path that is ambitious enough for serious systems work but
honest about current project capacity.

LLVM is mature, widely deployed, and strong for optimized native AOT builds. It
is also large, old, complex, and not a language architecture. Cranelift is not a
newer LLVM; it is a different backend with a smaller Rust-native integration
surface, fast compilation, and a strong fit for first native execution proofs.
MLIR is powerful for multi-level lowering, vector, tensor, sparse, GPU, and
accelerator work, but it is too much machinery to put at the center before Hum's
own IR exists.

The Chris Lattner compiler lessons reinforce this: use LLVM's good parts, but do
not make LLVM the language's soul. Hum's differentiator must be the checked
front-end, semantic graph, formal core, Hum IR, profiles, diagnostics, and
evidence surfaces that survive before backend lowering.

## Decision

Hum will use a swappable backend ladder:

1. Interpreter first, to prove executable semantics and contract behavior without
   backend complexity.
2. Cranelift next, as the first native backend candidate for local Rust-native
   execution proofs and fast feedback.
3. LLVM later, as the serious optimized AOT backend target for mature native
   releases, debug info, sanitizers, target coverage, and production toolchain
   integration.
4. MLIR later, only when Hum has real layout, vector, tensor, sparse, GPU,
   accelerator, or domain-lowering facts that justify multi-level lowering.
5. Custom Hum backend or custom optimization stack only after Hum has measured
   unique optimization facts that existing backends cannot use well.

Hum IR, not any backend IR, owns Hum's semantics. Backends are adapters behind a
narrow contract.

Every backend adapter must preserve or report:

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

## Consequences

Hum can use LLVM without being defined by LLVM.

Cranelift can let Hum reach native execution sooner without swallowing LLVM's
full complexity on day one.

LLVM remains the mature optimized backend path for serious AOT builds, including
future medical, aerospace, defense, finance, embedded, and safety-critical
profiles, but those profiles must be earned above LLVM through Hum semantics,
checks, evidence, reproducibility, and release discipline.

MLIR and custom backend work remain future options, not identity claims. They
must be justified by evidence, not prestige.

The backend interface must be designed early enough that an interpreter,
Cranelift, LLVM, MLIR, Wasm, C, or future custom backend can consume checked Hum
IR without changing surface Hum.

## Alternatives Rejected

- Emit LLVM IR directly from surface Hum and call that a language.
- Treat Cranelift as a newer or better LLVM instead of a different staged tool.
- Start with MLIR before Hum has a stable semantic graph and Hum IR.
- Promise a custom backend before Hum proves unique backend-relevant facts.
- Let backend limitations decide Hum's source semantics.
- Claim safety-critical credibility because LLVM or any backend is used.

## BDFL Note

Hum should be brave at the layer where it is actually new: intent, evidence,
semantic graph facts, profiles, and source-visible resource promises. Backend
ambition comes after those facts exist. The win is not out-LLVMing LLVM; the win
is handing every backend better truth.

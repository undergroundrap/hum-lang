# Minimum Compiler-Ready Standard Library: Triage

Date: 2026-07-12
Status: triaged by the architect-reviewer; raw report in
`deep/2026-07-12-minimum-compiler-ready-stdlib.txt`

## Provenance And Confidence

Agent-run deep research commissioned to answer one milestone question:
what is the smallest standard library Hum needs before it can be written
in itself (self-host its compiler)? Citation markers were tool-mangled
and stripped on import; source identities survive in prose (Rust, Go,
Zig, OCaml, TypeScript, Roc, Gleam bootstrap histories, with dates). The
report's own "thin evidence"/"unsourced inference" flags are preserved
and are placed honestly. Verdict: STRONG. This is timing-gated input --
it governs the stdlib build that begins after the callable spine (Work
Order 8), not current work. A companion stdlib research pass is expected
and will extend this snapshot.

## Core Finding

The compiler-ready line sits far below a full user stdlib. Go 1.5's
documented build graph (toolchain first, then the rest of std), Zig
(self-hosted compiler default while std stays intentionally unstable),
and TypeScript (a self-hosting compiler on a thin `CompilerHost`
boundary) all prove the compiler comes first. OCaml is the cautionary
outlier: a fat precompiled `boot/` stdlib because the compiler entangled
with the library early. Hum should target a compiler-ready stdlib, not a
general-purpose one, and draw the self-host line Go/TypeScript/Zig-style,
not OCaml-style.

## Adopted Sharpenings To STDLIB_STRATEGY

The existing layer order (core -> alloc -> data -> io/parse) is
directionally right; two corrections adopted:

1. Text belongs earlier than usually placed. A self-hosting compiler is a
   text processor before anything else; Rust split text as its own
   allocation-dependent layer and TypeScript's minimal host cannot proceed
   without a narrow text/filesystem model. Move `std.text` up to sit right
   after `std.alloc`, before general collections.
2. Compiler-support is not public stdlib. Arena/bump allocators, string/
   symbol interning, diagnostics/line-maps, and CFG/graph helpers can ship
   as unstable internal `compiler_support` modules first and stabilize
   only after the self-hosted compiler proves them. Do not gate self-host
   on polishing these as public APIs.

## The Pre-Self-Host Checklist (from the report, condensed)

Pre-self-host, in dependency order: (1) `std.core` (Option/Result/slices/
views/arithmetic/comparison); (2) `std.alloc` (one allocator + raw
memory); (3) `std.text` (byte/UTF-8 views, owned buffer, scan/split,
format buffer); (4) `std.data.vec`; (5) one fast non-adversarial hash
map/set for symbol tables; (6) arena/bump allocator (support); (7)
interning (support); (8) `std.fs`/`std.path` over explicit capabilities,
Windows path/case aware; (9) diagnostics with spans/line-map/typed
failures (support); (10) CFG/graph helpers (support); (11) minimal CLI/
arg parsing. Post-self-host: richer parsing, networking, concurrency,
time, randomness, compression, regex, package APIs, serialization.

Compiler-stress note: compilers hammer a narrow cluster -- fast growable
vector, fast non-adversarial map (rustc's FxHashMap archetype: trusted
input, not hostile-hash resistance), interning, arena with stable
addresses, source-text indexing, Windows-aware path/file IO. Solve "the
compiler collections problem," not "the collections problem."

## Standing Gate Confirmed (Section 4)

The report independently reached the gate already proposed for Hum:
do not build the stdlib as "Rust, but renamed." Evidence: stable APIs
calcify (Rust RFC 0040 facade, Go's `math/rand/v2` admission that early
APIs were mistakes), host idioms leak in (TypeScript's Node `crypto`
hardwiring), and Windows string/path/process semantics are where "simple"
abstractions become security liabilities (Rust's 2024 `Command` batch-file
escaping advisory). Adopt as standing stdlib gates:

- Every Hum stdlib API must be justified by Hum's own model (contracts,
  capabilities, ownership moves/views, causal typed failure), not
  inherited from how Rust or C++ shaped it.
- Freeze `std.core` last, not first: stabilize names/paths only after at
  least one compiler-internal cycle exercises them.
- Stabilize capability-bearing IO (process, path, cwd, case, quoting)
  only after Windows semantics are pinned in painful detail.
- Keep compiler-support libraries unstable and separable from user std.
- Bias to dependency injection over ambient authority (Gleam's portable
  compiler is the positive example; maps onto Hum capabilities).

## Companion Pass (2026-07-12, second agent, FULL sourced report)

Provenance upgrade: the full second report was received (rigorous, with
live primary sources -- Go 1.5 notes go.dev/doc/go1.5, Zig 0.10 release
notes, Lean 4 repo + 2021 CADE paper, rustc internals for arenas/fx/index/
symbol, LLVM Programmer's Manual, OCaml Bytes/safe-string history,
TypeScript compiler source, Rust std::path and the 2024 Command batch-file
advisory, OCaml Batteries/Base/Core). It CONFIRMS the first report's shape
-- two independent full passes converging on the same compiler-ready line
is now the strongest confidence signal in the stdlib evidence. The full
report was a chat paste with canvas artifacts; its substance and source
list are captured here as the authoritative record rather than archiving a
mangled raw file.

Structural additions already adopted above (compiler.backend as an
explicit pre-self-host emission-path item; std.os as its own layer;
concrete rustc-shaped compiler.support) stand. The full report adds these
deeper findings, several of which reinforce accepted Hum decisions:

1. "Normalized capability closure," not "N modules." The right unit is
   capabilities, normalized by WHO supplies each (language primitive,
   runtime, stable std, foundation package, compiler.support, stage-0 Rust
   host, external toolchain). Go/OCaml/TypeScript/Lean look "smaller" only
   because primitives/runtime/host supply vectors, maps, strings, alloc,
   and IO. Adopt a bootstrap capability ledger: a dependency supplied only
   by the Rust interpreter is not yet a Hum bootstrap dependency until
   stage 1 has an equivalent.
2. Three distinct error channels, not one (reinforces decision 0016).
   Compilers need operational failure (missing file, OOM) as causal typed
   failure; source diagnostics (expected-invalid programs, accumulated) as
   data; and compiler-invariant failure (a compiler bug) as contract/blame.
   Conflating them into one Result/exception/Diagnostic loses exactly what
   Hum is built to preserve. This three-way split belongs in the first
   compiler architecture, not retrofitted.
3. IDs over shared ownership for compiler graphs (reinforces decision
   0014). Arena/table-owned nodes referenced by compact typed IDs, not
   Rc/Arc. Consequence: Hum does NOT need reference counting in the minimum
   library, and IDs fit checked ownership better than lifetime-heavy
   references. De-risks a whole category.
4. Capability-bearing allocator and IO (reinforces decision 0017). The
   allocator is authority, not ambient; every collection op states its
   authority and failure behavior. Cannot recover an allocator/filesystem
   capability from a global singleton.
5. Self-host is a fixed-point proof, and evidence-native. stage-0 (Rust
   interpreter) -> stage-1 (Hum compiler) -> stage-2 (compiler produced by
   stage 1) -> semantic-equivalence then reproducibility evidence. Require
   semantic equivalence first; pursue byte-identical artifacts later under
   a declared reproducible-build profile. This is the actual definition of
   "self-hosting proven" and matches the brand.
6. Stage-0 vs stage-1 benchmarking (Hum-specific). A data structure fine in
   native stage 1 can be painfully slow through the stage-0 interpreter.
   Both profiles matter; stage-0 slowness alone must not force a bad
   permanent API. Test every proposed stable API through the interpreter.
7. Three maturity states for governance: bootstrap-private (compiler may
   change freely) / foundation-experimental (official, versioned with
   migration) / stable (compatibility commitment). Promotion to stable
   requires two independent consumers (the compiler alone is not enough)
   and law suites, not just examples (Eq/Hash compatibility, view
   invalidation, move-once, allocator ownership, causal-error preservation).
   Complements STDLIB_CONSTITUTION.
8. Separate four text-shaped forms from day one: Bytes/BytesView; validated
   Text/TextView; compiler-local Symbol; opaque OsPath/OsName. Source spans
   are byte offsets; line/column is derived. Path equality/hash is lexical
   and side-effect-free; filesystem canonicalization is an explicit
   capability op. Windows: never build process invocation by concatenating
   a command string (the Rust Command advisory).

The report's 17-item ordered checklist (bootstrap profile -> core -> ...
-> compiler.support (item 14) -> backend bridge (item 15, the compiler-
ready line) -> bootstrap harness (item 16, the self-host-proven line))
supersedes the condensed checklist above in detail when the stdlib work
order opens; the stable-stdlib boundary sits around items 12-13, item 14
is official-but-versioned, item 15 may be target-specific.

## Routing

- Feeds the future stdlib work order (after Work Order 8). The pre-self-
  host checklist (now including `std.os` and a minimal `compiler.backend`
  emission path) becomes that order's session map; the Section 4 gates
  become its standing locks.
- The "text earlier," "compiler-support is not public std," "std.os as its
  own layer," and "a minimal backend/emission path is pre-self-host"
  corrections should be folded into STDLIB_STRATEGY when that work order
  opens.
- New decision seed for the self-host milestone: how Hum emits code to
  bootstrap (emit-C-and-shell-out vs bytecode vs native object). Not now.
- Not actionable now: Work Order 8 (callable spine) precedes all stdlib
  work and changes what the stdlib can express.

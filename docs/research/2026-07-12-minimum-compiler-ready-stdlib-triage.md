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

## Companion Pass (2026-07-12, second agent, recommendation excerpt only)

A second stdlib research pass returned a recommendation that strongly
CONFIRMS the report above -- same shape: compiler-ready is a narrow line
below the full user stdlib, split into a stabilize-first set (core, alloc,
data, text, os, io) and a provide-but-keep-unstable set (diagnostics,
compiler-support, backend), with the same defer list (generic parsing,
broad collections, async, networking, packages, full Unicode,
serialization, memory-mapping, file-watching, parallel compilation). Two
independent passes converging on the same line raises confidence. Only the
recommendation excerpt was provided; no full report was archived.

Three genuinely additive findings adopted:

1. compiler.backend is an explicit pre-self-host line item: "the narrowest
   viable bytecode, C, object, or linker bridge." The first report stopped
   at diagnostics/CFG; this correctly notes that self-hosting needs SOME
   code-emission path. This surfaces a real future decision (emit-C vs
   bytecode vs native object) that ties to the backend ladder and the
   eventual .exe/.dll story. Seed for a backend-selection decision at the
   self-host milestone; not settled now.
2. std.os as a distinct layer: opaque path/name handling plus target
   facts. Maps directly onto already-shipped work -- the native Path
   boundary (Session AB) and target-facts surfaces. The stdlib order
   should carry `std.os` as its own layer rather than folding paths into
   a generic fs module.
3. compiler.support enumerated concretely (the rustc-shaped toolkit):
   symbols/interning, typed IDs and IndexVec, compiler arenas, bitsets,
   sparse sets, worklists, SCC/topological utilities, fast ephemeral maps,
   and stable compiler hashing. Sharper than the first report's
   "arena/intern/diag/CFG." This becomes the concrete contents of the
   internal `compiler_support` modules.

Minor structural notes: the companion folds args/environment/buffering
into `std.io` and puts sorting/binary-search into `std.data`; it places
"contracts, blame substrate, comparison/hash laws" explicitly in
`std.core`, which fits Hum's model and the law-bearing-bounds direction
from the generics research.

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

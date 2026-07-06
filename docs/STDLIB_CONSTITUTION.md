# Hum Standard Library Constitution

Date: 2026-07-06

## Purpose

This document is the admission law for Hum's standard library.

The standard library is not a convenience drawer. It is part of the language's
contract with programmers, tools, benchmarks, agents, and future maintainers.

A `std` API should be harder to stabilize than an ordinary package API because a
bad `std` decision becomes culture. Once millions of programs depend on it, even
small mistakes become permanent taxes.

## Core Law

```text
No API enters stable std until its promises are checked, measured, documented,
and explainable.
```

Hum should prefer a small standard library with excellent contracts over a large
standard library full of clever APIs that nobody can safely remove.

The standard library follows the [Paved Road Doctrine](PAVED_ROAD_DOCTRINE.md):
one obvious default API for ordinary use, explicit side roads for specialized
constraints, and evidence before any specialized path becomes stable.

## Stability Tiers

### sketch

A design note or `.hum` fixture. No implementation promise.

### lab

A Rust prototype, external implementation wrapper, or Hum fixture used for
measurement. No compatibility promise.

### candidate

A serious API shape with examples, diagnostics, benchmarks, and known gaps.
Allowed to change with migration help.

### stable

A compatibility promise. Stable APIs require BDFL approval and a review packet.

### legacy

Kept only to avoid breaking users while a better path exists.

## Universal Admission Packet

Every candidate for `std` must include:

1. Name and module path.
2. Problem statement.
3. Beginner explanation.
4. Systems explanation.
5. Hum examples.
6. Contract blocks: `why:`, `needs:`, `ensures:`, `fails when:`, `cost:`, and
   `watch for:` where relevant.
7. Semantic graph shape.
8. Diagnostics and repair hints.
9. Formatter examples.
10. Reference implementation.
11. Unit tests.
12. Property tests where a law can be stated.
13. Fuzz tests for input-facing or parser-facing APIs.
14. Benchmarks for performance-facing APIs.
15. Adversarial tests for maps, parsers, allocators, crypto, and sync.
16. Security review for any API touching input, memory, crypto, unsafe, or FFI.
17. Allocation behavior.
18. Panic/failure behavior.
19. Compile-time impact.
20. Migration story.
21. Rejected alternatives.
22. BDFL decision note.

If this packet feels too heavy, the API belongs in a package or lab, not stable
`std`.

## Default API Rules

Stable `std` APIs should default to:

- memory safety
- no ordinary null
- no hidden IO
- no hidden heap allocation in `std.core`
- fallible allocation where low-level code needs it
- explicit mutation
- explicit failure
- explicit trust boundaries
- bounded behavior for hostile input when exposed to untrusted data
- deterministic behavior when the docs promise determinism
- readable examples that survive `humfmt`
- semantic graph facts agents can consume
- one obvious paved-road API before specialized variants

A fast API that hides risk is not a Hum default. It can be a trusted or unsafe
specialization if the source says so.

## Primitive-Specific Gates

### `Result` And `Option`

`Result` and `Option` are language primitives as much as library primitives.

Rules:

- no ambient null
- no unchecked exception path for normal failure
- `fails when:` must map to typed failure where possible
- `try` propagation must keep failure visible in the semantic graph
- panic-like exits must be reserved for contract violation or explicit abort
- examples must show recovery, propagation, and narrowing

### `Vec`, Slices, And Inline Storage

Rules:

- document length, capacity, and allocation separately
- document whether push can reallocate
- document whether element addresses are stable
- provide fallible reserve APIs
- expose initialized and spare capacity only with safe wrappers or audited unsafe
- split ordinary `Vec` from `InlineVec` / `SmallVec` instead of hiding stack
  storage behind one surprising type
- make growth policy measurable but not overpromised unless stabilized

### `Map` And `Set`

Rules:

- provide one paved-road `Map` API for ordinary use before exposing specialized variants
- separate present-key lookup, missing-key lookup, insert, delete, resize, and
  iteration costs
- document memory overhead and load-factor behavior
- document whether entries move
- document pointer stability and iteration order
- state whether keys are trusted or hostile
- default map must resist practical HashDoS-style abuse
- high-speed trusted maps require source-visible trust
- research maps enter through `map-lab`, not directly into stable `std`

### `Text` And `Bytes`

Rules:

- `Bytes` is raw data
- `Text` is valid Unicode text
- byte indexing, scalar indexing, and grapheme traversal are distinct operations
- no API should imply that user-perceived characters are fixed-width
- segmentation must name the Unicode version or profile used
- locale-sensitive behavior must be explicit
- invalid text from IO or FFI must enter as `Bytes` or `TextDecodeResult`

### Allocators

Rules:

- allocator choice is policy, not trivia
- every allocator documents fragmentation behavior, lifetime model, thread
  behavior, zeroing behavior, and failure behavior
- arena/bump/slab/pool/page/secure allocators are separate concepts
- secure allocators must preserve zeroing against optimizer removal
- allocator telemetry must be available for `hum bench` and `hum profile`
- global allocator replacement must not be the only path to performance

### Parsers

Rules:

- parser APIs consume `Bytes` or validated `Text` explicitly
- partial parsing is opt-in
- structured errors are required
- recursion depth and input size limits are visible
- every stable parser has fuzz targets
- parser examples include malformed input
- generated parsers must emit semantic graph facts for grammar, bounds, and
  recovery behavior

### Sync And Memory Reclamation

Rules:

- memory ordering names must be readable and precise
- lock order must be declarable for complex code
- cancellation and blocking behavior must be documented
- RCU, epoch reclamation, and hazard pointers are advanced APIs, not beginner
  defaults
- any reclamation API must state when a reference may be held and when memory may
  be freed
- examples must include the misuse that would cause use-after-free in C/C++

### Crypto And Secrets

Rules:

- no footgun primitives on the default path
- secret types must prevent accidental logging and equality timing leaks where
  possible
- constant-time claims need tests or a clear measurement story
- randomness APIs must distinguish secure random from deterministic test random
- keys should carry purpose, algorithm, and lifetime metadata

### SIMD, Hardware, And Accelerators

Rules:

- portable path first
- specialized path second
- runtime dispatch must be generated or standardized
- hardware assumptions appear in the semantic graph
- benchmark transfer cost, not only kernel speed
- accelerator APIs live outside core `std` until CPU semantics are mature

## Benchmark Rules

A benchmark packet must include:

- workload description
- data shape
- input distribution
- adversarial distribution if relevant
- hardware and OS
- compiler/build profile
- warmup policy
- wall time
- allocations
- memory bytes per element where relevant
- cache and branch counters where available
- tail latency where relevant
- binary size and compile-time impact where relevant
- comparison against established libraries

Benchmarks must be reproducible through Nectar before they influence stability.

## Fuzz And Adversarial Rules

Fuzzing is required for:

- parsers
- decoders
- text processing
- maps exposed to untrusted keys
- allocators
- crypto boundaries
- unsafe wrappers
- FFI adapters

Adversarial tests should include:

- empty input
- huge input
- malformed input
- repeated input
- collision-heavy input
- deeply nested input
- cancellation during operation
- allocation failure
- concurrent access when supported

## Semantic Graph Requirements

Every stable `std` API must tell tools:

- what it reads
- what it changes
- what it allocates
- what it may fail with
- what it may panic or abort on
- what cost claims it makes
- what trust assumptions it has
- what tests and benchmarks cover it
- what stability tier it belongs to

Agents should never need to reverse-engineer `std` behavior from prose alone.

## Rejection Rules

Reject or keep in lab any API that:

- duplicates an existing concept without removing a real tax
- hides allocation, mutation, IO, failure, or unsafe behavior
- needs style guidance `humfmt` cannot enforce
- needs lints to be safe in ordinary use
- cannot be explained to a beginner and a systems engineer
- makes semantic graph output weaker
- depends on one benchmark shape
- lacks a fallback when hardware, input, or workload changes
- enters `std` mainly because another language has it

## Brutal Assessment

The standard library is where Hum can become real or become fantasy.

The right ambition is not the biggest `std`. It is a standard library where every
primitive carries enough intent that the compiler, profiler, docs, and agents can
explain why it exists and when it should not be used.

## Sources

- Hum standard library strategy: STDLIB_STRATEGY.md
- Hum optimization and DSA strategy: OPTIMIZATION_AND_DSA_STRATEGY.md
- Rust `Vec` guarantees: https://doc.rust-lang.org/std/vec/struct.Vec.html
- Rust `Result`: https://doc.rust-lang.org/std/result/enum.Result.html
- Rust `String`: https://doc.rust-lang.org/std/string/struct.String.html
- Unicode Text Segmentation, UAX #29: https://www.unicode.org/reports/tr29/
- LLVM Programmer's Manual, data structure guidance: https://llvm.org/docs/ProgrammersManual.html
- Abseil container guide: https://abseil.io/docs/cpp/guides/container
- Hashbrown README: https://github.com/rust-lang/hashbrown
- Optimal Bounds for Open Addressing Without Reordering: https://arxiv.org/abs/2501.02305
- Mesh allocator paper: https://arxiv.org/abs/1902.04738
- mimalloc documentation: https://microsoft.github.io/mimalloc/
- scalloc paper: https://arxiv.org/abs/1503.09006
- Linux kernel RCU documentation: https://www.kernel.org/doc/html/latest/RCU/whatisRCU.html
- Crossbeam epoch documentation: https://docs.rs/crossbeam-epoch/latest/crossbeam_epoch/
- LLVM libFuzzer documentation: https://llvm.org/docs/LibFuzzer.html
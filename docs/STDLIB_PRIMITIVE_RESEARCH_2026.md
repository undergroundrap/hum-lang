# Hum Stdlib Primitive Research Sweep 2026

Date: 2026-07-06

## Purpose

This is the first focused research sweep for Hum standard-library primitives.

The question is not "what should we copy?" The question is:

```text
what evidence should shape each primitive before Hum stabilizes it?
```

Hum should treat `std` primitives as measured contracts, not folklore.

## Method

For each primitive family, this sweep records:

- useful prior art
- what Hum should learn
- what Hum should not blindly copy
- first lab deliverables

The goal is to decide what to prototype, benchmark, fuzz, and explain first.

## Priority 0: `Result` And `Option`

### Prior Art

Rust's `Result` and `Option` prove that explicit success/failure and presence
checking can become normal programming culture. Rust's APIs also show that
fallible chaining, lazy recovery, and infallible extraction all matter.

### Hum Lesson

Hum should make failure even more visible than Rust by connecting `Result` to:

- `fails when:` sections
- `try` propagation
- generated tests
- semantic graph failure edges
- docs that show why each failure exists

`Option` should be the only ordinary absence type. There should be no ambient
null.

### Hum Risk

Do not turn failure into paperwork. `fails when:` must stay readable and tools
must reduce boilerplate.

### First Lab Deliverables

- `.hum` fixtures for `Result`, `Option`, and `try`
- graph output for failure edges
- diagnostics for ignored failure
- examples for recovery, propagation, and impossible failure

## Priority 1: `Vec`, Slices, And Inline Storage

### Prior Art

Rust's `Vec` documentation is valuable because it states concrete guarantees:
length and capacity are separate, reallocation can move elements, capacity can be
relied on, and ordinary `Vec` does not hide small-stack optimization.

LLVM's `SmallVector` and Abseil's `InlinedVector` show that inline storage is a
real performance tool, especially for small collections, but it must be explicit
because it changes move cost and address stability.

### Hum Lesson

Hum should split the family clearly:

```text
Vec<T>        heap-backed growable array
InlineVec<T>  fixed inline capacity, spills only if allowed by type/profile
SmallVec<T>   inline-first growable array, explicit address-stability warning
Span<T>       borrowed contiguous view
Slice<T>      borrowed range of contiguous elements
```

Every growable collection should document:

- length
- capacity
- allocation behavior
- growth behavior if promised
- whether push can move elements
- whether addresses are stable
- whether allocation can fail

### Hum Risk

Do not hide inline storage behind the ordinary `Vec`. That makes the simple type
harder for unsafe code, FFI, and agents to reason about.

### First Lab Deliverables

- Rust reference `VecProfile` benchmark harness
- `.hum` fixtures for capacity promises
- diagnostics for claiming stable addresses on a moving vector
- fallible reserve API sketch

## Priority 2: `Map` And `Set`

### Prior Art

The Farach-Colton, Krapivin, and Kuszmaul open-addressing paper shows that old
probe-bound assumptions can move. SwissTable-style maps, Abseil containers, and
Rust hashbrown show the practical value of dense cache-aware maps.

### Hum Lesson

Hum needs one paved `Map` API whose internal strategy can be selected or
rejected from profile-backed evidence.

Map contracts must separate:

- present-key lookup
- missing-key lookup
- insertion
- deletion
- resize
- iteration
- memory overhead
- key trust
- iteration order
- pointer stability
- element reordering

### Hum Risk

Do not make the fastest trusted-key map the default. The default map must survive
hostile input. Trusted fast maps can exist only behind visible intent.

### First Lab Deliverables

- `map-lab` benchmark harness
- secure default baseline
- hashbrown/SwissTable baseline
- elastic/funnel-inspired lab variants
- adversarial collision tests
- graph output explaining selected strategy

## Priority 3: `Text` And `Bytes`

### Prior Art

Rust `String` treats owned text as UTF-8. Unicode UAX #29 shows why user-visible
text is not the same as bytes, scalar values, or fixed-width characters. Grapheme
clusters, word boundaries, and sentence boundaries need explicit algorithms and
versioned data.

### Hum Lesson

Hum should make text units visible:

```text
Bytes        raw bytes
Text         valid Unicode text
Scalar       Unicode scalar value
Grapheme     user-perceived text segment under a named Unicode profile
```

Indexing by byte, scalar, and grapheme should be different operations. Any API
that counts "characters" must say which unit it means.

### Hum Risk

Do not make beginner-readable text APIs lie. Text that looks simple is often
where international bugs hide.

### First Lab Deliverables

- `.hum` fixtures for `Text`, `Bytes`, and UTF-8 decode
- Unicode version metadata in semantic graph
- diagnostics for byte indexing a `Text` without saying `bytes`
- grapheme iteration benchmark and conformance tests

## Priority 4: Allocators And Regions

### Prior Art

Mesh shows fragmentation can be attacked with allocator design. mimalloc shows
free-list sharding, first-class heaps, eager page purging, secure modes, and
bounded behavior matter in production allocators. scalloc shows multicore
allocation needs scalability and fragmentation discipline.

### Hum Lesson

Allocator policy belongs in source-level intent:

```text
allocates:
  arena request lifetime
  fallible allocation
  zero secrets on release
```

The stdlib should provide distinct allocator families:

- global allocator interface
- arena allocator
- bump allocator
- slab allocator
- pool allocator
- page allocator
- secure zeroing allocator
- telemetry wrapper

### Hum Risk

Do not make every API allocator-noisy. Allocator choice should be visible at
important boundaries and inferred or defaulted where the contract is obvious.

### First Lab Deliverables

- allocator trait sketch
- fallible allocation examples
- arena-backed parser fixture
- fragmentation benchmark plan
- zeroing/secret-memory misuse tests

## Priority 5: Parsers And Input Handling

### Prior Art

LangSec argues that input handling should be treated as language recognition,
not ad hoc string manipulation. libFuzzer shows the practical shape of fuzzable
APIs: deterministic, fast, narrow targets that tolerate malformed input.

### Hum Lesson

`std.parse` should make secure parsing the normal path:

- bounded input contracts
- complete-consumption by default
- partial parsing only when explicit
- structured errors
- recursion limits
- fuzz target generation
- malformed input examples
- zero-copy and arena-backed AST options

### Hum Risk

Do not let parser combinators become a private language that hides complexity.
The semantic graph must expose grammar, bounds, and failure modes.

### First Lab Deliverables

- `parse bytes as T` fixture
- generated fuzz target shape
- structured parse error schema
- grammar-to-semantic-graph sketch

## Priority 6: Sync And Memory Reclamation

### Prior Art

Linux RCU shows the power and difficulty of splitting removal from reclamation
for read-mostly data. Crossbeam epoch shows Rust can expose epoch-based
reclamation through safer APIs. C++26 hazard pointers show that standard
libraries are now absorbing memory-reclamation primitives.

### Hum Lesson

Hum should not make lock-free primitives easy to type before they are easy to
reason about.

The sync layer needs:

- readable memory ordering names
- lock-order declarations
- cancellation behavior
- structured task ownership
- RCU, epoch, and hazard-pointer APIs in advanced tiers
- semantic graph facts about reference lifetime and reclamation

### Hum Risk

Concurrency APIs become dangerous when they look like ordinary collections. Hum
must make reclamation and lifetime boundaries visible.

### First Lab Deliverables

- `std.sync` concept fixtures
- memory-order glossary
- misuse diagnostics for holding a reclaimed reference
- benchmark plan for read-mostly, write-heavy, and mixed workloads

## Priority 7: SIMD, Hardware, And Accelerators

### Prior Art

C++26 is adding more standard hardware-facing library pieces. LLVM and MLIR show
that hardware-aware lowering should be staged, not bolted onto syntax first.

### Hum Lesson

Hum should offer portable vector APIs before target-specific tricks.

Hardware assumptions should appear in:

- source contracts
- semantic graph
- benchmark profile
- fallback selection

### Hum Risk

Do not let accelerator code infect the core language before CPU semantics are
solid.

### First Lab Deliverables

- portable SIMD API sketch
- runtime dispatch design note
- benchmark packet format for hardware features
- source-map requirement for optimized/vectorized code

## Practitioner Primitive Families

[PRACTITIONER_PAIN_SWEEP_2026.md](PRACTITIONER_PAIN_SWEEP_2026.md) adds
primitive families that should be reserved early even if they are not the first
implemented `std` APIs:

- operations: `Path`, `Command`, `Env`, `Process`, `Service`, `User`, `Group`,
  `Secret`, `Credential`, `DesiredState`, `Plan`, `Diff`, `RollbackPlan`
- network: `IpAddr`, `Cidr`, `MacAddr`, `Port`, `SocketAddr`, `VlanId`, `Asn`,
  `Route`, `Packet`, `Endian`, `Checksum`, `TelemetryPath`
- reliability: `Duration`, `Deadline`, `Timeout`, `RetryPolicy`, `Backoff`,
  `RateLimit`, `ResourceBudget`, `Trace`, `Metric`, `LogField`
- low-level: `Layout`, `Align`, `Register`, `Volatile`, `Mmio`, `DmaBuffer`,
  `TargetFeature`, `StackBudget`, `CodegenBudget`
- numeric and tensor: `Unit`, `Exact`, `Approx`, `Tolerance`, `ErrorBound`,
  `Decimal`, `BigInt`, `Rational`, `Complex`, `Interval`, `Shape`, `DType`,
  `Device`, `Tensor`, `SparseTensor`, `Seed`, `DeterminismPolicy`

These should not all enter `std` at once. The near-term job is to reserve graph
fields, diagnostics, and profile hooks so these domains do not require breaking
changes later.

## First Primitive Set

Hum should start with four, not forty:

1. `Result` / `Option`
2. `Vec` / `Slice` / `Span`
3. `Map` / `Set`
4. `Text` / `Bytes`

That set is enough to exercise failure, ownership, allocation, data-structure
intent, parsing, Unicode, diagnostics, and semantic graph output.

Allocators and parsers come immediately after because they shape real systems
code, but they need the first primitives to exist as fixtures.

## What This Changes For Hum

1. `std` needs a constitution before more API sketches.
2. Every primitive needs a contract fixture before implementation.
3. `Map` gets `map-lab`; `Vec` gets `vec-lab`; `Text` gets `text-lab`; parsers
   get `parse-lab`.
4. The semantic graph must represent stdlib promises, not only user tasks.
5. Benchmarks must become Nectar-reproducible before they influence stability.

## Brutal Assessment

The best standard library is not the one with the most features.

It is the one where the default path is safe, the fast path is honest, the weird
path is explicit, and every important cost can be found by a human, compiler,
profiler, or agent. This is the [Paved Road Doctrine](PAVED_ROAD_DOCTRINE.md)
applied to `std`.

## Sources

- Rust `Vec`: https://doc.rust-lang.org/std/vec/struct.Vec.html
- Rust `Result`: https://doc.rust-lang.org/std/result/enum.Result.html
- Rust `Option`: https://doc.rust-lang.org/std/option/enum.Option.html
- Rust `String`: https://doc.rust-lang.org/std/string/struct.String.html
- Unicode Text Segmentation, UAX #29: https://www.unicode.org/reports/tr29/
- LLVM Programmer's Manual: https://llvm.org/docs/ProgrammersManual.html
- Abseil container guide: https://abseil.io/docs/cpp/guides/container
- Hashbrown README: https://github.com/rust-lang/hashbrown
- Optimal Bounds for Open Addressing Without Reordering: https://arxiv.org/abs/2501.02305
- Mesh allocator paper: https://arxiv.org/abs/1902.04738
- mimalloc documentation: https://microsoft.github.io/mimalloc/
- scalloc paper: https://arxiv.org/abs/1503.09006
- Linux kernel RCU documentation: https://www.kernel.org/doc/html/latest/RCU/whatisRCU.html
- Crossbeam epoch documentation: https://docs.rs/crossbeam-epoch/latest/crossbeam_epoch/
- C++26 hazard pointer header reference: https://en.cppreference.com/cpp/header/hazard_pointer
- LLVM libFuzzer documentation: https://llvm.org/docs/LibFuzzer.html

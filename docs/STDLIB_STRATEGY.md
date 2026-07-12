# Hum Standard Library Strategy

Date: 2026-07-06

## Pending Evidence-Based Refinements (2026-07-12)

Two independent deep-research passes on the compiler-ready standard library
converged on refinements to the layer plan below. They are research input,
not accepted design; apply them through the stdlib work order (after the
callable-spine work order), not by rewriting this doc now. See
[research/2026-07-12-minimum-compiler-ready-stdlib-triage.md](research/2026-07-12-minimum-compiler-ready-stdlib-triage.md)
for the full triage and sources. Summary of the refinements to weigh:

- The compiler-ready dependency closure is deliberately larger than the
  permanently stable stdlib: do not freeze interning, typed compiler IDs,
  arenas, SCC/graph utilities, or backend helpers as stable `std` just
  because the first compiler needs them. Use maturity states
  bootstrap-private / foundation-experimental / stable.
- Move `std.text` (and no-alloc UTF-8) earlier: a self-hosting compiler is
  a text processor first.
- Add `std.os` as its own layer (opaque path/name + target facts), mapping
  onto the shipped native Path boundary; never treat a path as `Text`.
- Split compiler-essential literal decoding from any generic `parse`
  framework (generic parsing is post-self-host).
- Add a minimal `compiler.backend` emission path (bytecode / emit-C /
  object) to the pre-self-host line; it seeds a future backend-selection
  decision tied to the `.exe`/`.dll` story.
- Model three distinct error channels (operational failure, source
  diagnostics, compiler-invariant failure) rather than one; this aligns
  with decision 0016.
- Prefer compact typed IDs over `Rc`/`Arc` for compiler graphs; reference
  counting is not part of the minimum library (aligns with decision 0014).
- Prove self-hosting as a stage-2 fixed point: semantic equivalence first,
  byte-identical reproducibility later under a declared profile.

## Brutal Starting Point

Hum should not start by writing a giant standard library in Hum.

That would feel productive and probably be wrong. A language without a stable
parser, type checker, ownership model, effect checker, and benchmark harness
cannot honestly decide what its standard library guarantees. It can sketch APIs,
write reference implementations, and build benchmark proofs, but it should not
pretend the library is real before the language can enforce the promises on the
page.

The standard library has to be treated as part of the language design, not a bag
of useful helpers.

## Standard Library Principle

Hum's standard library should expose intent first and implementation second.

A programmer should say what semantic contract they need:

```text
store sessions: map SessionId -> Session {
  expects:
    up to 20 million active sessions

  optimizes:
    lookup speed
    memory density
    collision resistance

  protects:
    attacker cannot force unbounded probe chains

  needs:
    stable addresses are not required
}
```

The compiler and stdlib can then choose or reject an implementation:

- dense hash table
- ordered B-tree map
- arena-backed map
- cache-semantic table
- pointer-stable node map
- concurrent sharded map
- GPU-backed table for batched workloads

This is how Hum avoids C++'s problem of making the user manually know 50 subtly
different container tradeoffs before they can store data.

See [OPTIMIZATION_AND_DSA_STRATEGY.md](OPTIMIZATION_AND_DSA_STRATEGY.md) for the
research intake pipeline and optimization admission gate.

See [PAVED_ROAD_DOCTRINE.md](PAVED_ROAD_DOCTRINE.md) for the default-path philosophy, [STDLIB_CONSTITUTION.md](STDLIB_CONSTITUTION.md) for stable `std` admission rules, and [STDLIB_PRIMITIVE_RESEARCH_2026.md](STDLIB_PRIMITIVE_RESEARCH_2026.md) for the first primitive research sweep.

## What The 2026 Research Actually Means

### Hash Tables

Andrew Krapivin, Martin Farach-Colton, and William Kuszmaul showed in 2025 that
open addressing without reordering can do better than older theory suggested.
That is a serious signal, but not an immediate drop-in stdlib map.

See [HASH_TABLE_RESEARCH_2501_02305.md](HASH_TABLE_RESEARCH_2501_02305.md) for
the detailed Hum research note.

Hum should create a `map-lab` benchmark track for hash-table research:

- SwissTable/hashbrown-style dense baseline
- secure/adversarial hashing baseline
- stable low-associativity tables
- open-addressing-without-reordering experiments
- elastic-hashing-inspired insert-heavy experiments
- funnel-hashing-inspired greedy experiments
- cache-semantic GPU tables for batched workloads

The default `Map` must be safe under hostile input. A faster but HashDoS-weak
map can exist only when the source says the keys are trusted.

### Constraint And Verification Data Structures

Krapivin's 2026 cardinality-constraint work matters more for Hum's verifier than
for ordinary user containers. Smaller CNF encodings and grid-compression-style
ideas can help `hum prove` generate tighter obligations for `needs:`, `ensures:`,
and `keeps:`.

This belongs in the compiler/prover toolchain before it belongs in the user
stdlib.

### GPU And Accelerator Data Structures

HierarchicalKV shows a key lesson: some tables should have cache semantics, not
dictionary semantics. A full table can evict or reject by policy instead of
rehashing or failing.

Hum should make that semantic distinction visible:

```text
store embeddings: cache FeatureId -> Vector {
  capacity:
    80 GB device memory

  evicts:
    lowest score first

  optimizes:
    batched lookup throughput
    stable full-capacity performance
}
```

But this should not be in core v0. GPU-backed structures are powerful and easy to
get wrong. They need a separate `std.accel` tier.

### Hardware Memory Safety

CHERI is the most important hardware direction to respect. It extends ISAs with
capabilities for fine-grained memory protection and compartmentalization.

Hum should not depend on CHERI, but its pointer, FFI, allocator, and unsafe
models should be capable of mapping onto CHERI-like hardware later. That means:

- capabilities should be explicit in the type/effect system
- FFI should have visible trust boundaries
- unsafe code should carry `needs:`, `protects:`, and `proves:` blocks
- allocators should preserve provenance and bounds information when possible

### C++26 Is A Warning And A Menu

C++26 adds library pieces Hum should study: SIMD, inplace vectors, hive-like
containers, linear algebra, RCU, hazard pointers, execution control, contracts,
and reflection.

The warning is that bolting features onto an old language creates complexity.
The menu is useful: these are real systems needs, and Hum should design them as
first-class intent-aware APIs instead of afterthought headers.

## Standard Library Layers

### `std.core`

No heap, no IO, no hidden panics.

Should include:

- primitive types
- `Bool`, integers, floats, checked arithmetic, saturating arithmetic
- `Option`, `Result`
- slices, spans, ranges
- fixed arrays
- bit operations
- comparison, hashing traits/capabilities
- compile-time constants

### `std.alloc`

Memory policy is a systems-language feature, not an implementation detail.

Should include:

- global allocator interface
- arena allocator
- bump allocator
- slab allocator
- pool allocator
- page allocator
- secure zeroing allocator
- fallible allocation by default in low-level contexts
- allocation telemetry hooks

### `std.data`

Default data structures should optimize for cache locality, memory density, and
clear contracts.

Starter set:

- `Vec`
- `InlineVec`
- `SmallVec`
- `Deque`
- `Map`
- `Set`
- `OrderedMap`
- `OrderedSet`
- `BitSet`
- `BloomFilter`
- `Graph`
- `StableGraph`
- `Text`
- `Bytes`

The standard library should prefer flat/dense storage where pointer stability is
not required. Pointer stability should be a declared requirement, not an
accidental default.

### `std.sync`

Concurrency primitives need contracts, cancellation, and memory reclamation.

Should include:

- mutexes and read-write locks
- atomics with named memory-order intent
- channels
- structured tasks
- cancellation tokens
- RCU
- hazard pointers
- epoch reclamation
- lock-order declarations

### `std.io`

IO should be capability-based and effect-visible.

Should include:

- files
- paths
- streams
- sockets
- buffered IO
- async IO adapters
- deterministic replay hooks

A task that touches IO should say so in `uses:`.

### `std.crypto`

Cryptography must be misuse-resistant, boring, and strongly typed.

Should include:

- secure random
- hashes
- MACs
- AEAD encryption
- key derivation
- constant-time comparisons
- secret memory wrappers

Do not expose footgun primitives as the default path. Crypto should have a paved
road for ordinary secure use, not a tray of sharp primitives with equal status.

### `std.parse`

Parsing deserves first-class support because agents, compilers, protocols, and
security-sensitive systems all need it.

Should include:

- zero-copy parsing
- arena-backed AST construction
- binary protocol primitives
- UTF-8 validation
- bounded input contracts
- structured errors
- fuzz target generation

### `std.simd` And `std.hw`

Hardware features should be portable by default and explicit when specialized.

Should include:

- portable vector types
- CPU feature detection
- runtime dispatch
- cache-line constants
- alignment tools
- prefetch hints only behind measured APIs
- hardware counter/profiling hooks

Hum should never make users write five copies of a loop just to get SSE, AVX,
NEON, SVE, or RISC-V Vector support.

### `std.accel`

Accelerators should be a separate tier.

Possible future modules:

- GPU buffers
- GPU maps/caches
- batched kernels
- tensor and linear algebra adapters
- device/host transfer contracts

This should be built after the CPU semantics are solid.

## How We Build It

### Phase 0: Library Constitution

Before implementing containers, write the rules:

- every stdlib API has a contract
- every allocation is declared or inferred into an allocation effect
- every unsafe block has proof obligations
- every optimized structure has a reference implementation
- every performance claim has a static cost contract, benchmark, or explicit reason it cannot be checked
- every security-sensitive API has misuse tests
- every data structure documents pointer stability, iteration order, memory use,
  and adversarial behavior
- every structure-facing API passes the optimization and DSA admission gate
- every primitive names its paved-road default and any explicit side roads

### Phase 1: Reference Implementations Outside Hum

Use Rust first.

That is not a betrayal of Hum. It is discipline. Rust gives us memory safety,
benchmarks, fuzzing, and real performance while Hum's compiler is still forming.

Each primitive gets:

- Hum-facing API sketch
- Rust reference implementation
- property tests
- fuzz targets
- benchmark suite
- competitor baselines
- design note with tradeoffs

### Phase 2: Semantic Fixtures

Create `.hum` examples that express desired stdlib contracts before they compile.

These fixtures drive the parser, semantic graph, effect checker, and test
generator.

### Phase 3: Backend Selection

Once the compiler has enough semantic information, make `store` declarations
choose strategies.

Example:

```text
store users: map UserId -> User {
  expects:
    millions of entries

  needs:
    stable addresses are required

  optimizes:
    lookup speed
    memory density
}
```

The compiler should be allowed to say:

```text
error: stable addresses conflict with selected dense map strategy
help: use pointer-stable map, arena-backed values, or remove stable-address need
```

### Phase 4: Self-Hosting Stdlib Pieces

Only after the language is strong enough should core library pieces move into
Hum itself.

The first self-hosted targets should be small and proof-friendly:

- `Option`
- `Result`
- slices
- checked arithmetic
- `InlineVec`
- simple `Map` interface over a host implementation

## Benchmark Rules

No stdlib structure should enter `std` because it is elegant.

It enters because it wins or because it is the safest clear default.

Benchmark against:

- Rust standard library
- `hashbrown`
- Abseil containers
- C++ standard library containers
- Folly where relevant
- Zig standard library where relevant
- Go standard library where relevant
- domain-specific research prototypes where relevant

Measure:

- wall-clock time
- memory bytes per element
- allocations per operation
- insertion probes
- present-key lookup probes
- missing-key lookup probes
- tail latency at high load factors
- cache misses
- branch misses
- compile time
- binary size
- adversarial input behavior
- deterministic replay behavior

## Brutal Critiques Of Our Current Direction

1. "Simpler than Python, faster than Rust" is a slogan, not a design. We only
   earn it by narrowing choices and making the compiler enforce intent.
2. The stdlib cannot chase every new paper. Research becomes standard only after
   it survives benchmarks, adversarial testing, maintenance, and explanation.
3. Natural-language `does:` is dangerous if it is executable too early. The
   compiler needs a small, precise core language underneath the readable surface.
4. `why:` and `watch for:` are only revolutionary if tools consume them. If they
   become comments, Hum loses its reason to exist.
5. A tiny language with a huge magical stdlib is still a huge language. Keep the
   core small, make tiers explicit, and let advanced modules prove themselves.
6. Do not promise double or triple RAM. Promise measurable memory density,
   allocation visibility, and data-structure choices that avoid waste.
7. Do not make AI correctness part of the trusted base. Agents can propose;
   compiler, tests, proofs, and benchmarks decide.

## Near-Term Deliverables

1. Use [STDLIB_CONSTITUTION.md](STDLIB_CONSTITUTION.md) as the admission gate.
2. Create `examples/stdlib/*.hum` semantic fixtures for the first primitive set.
3. Build fixtures for `Result`/`Option`, `Vec`/`Slice`/`Span`, `Map`/`Set`, and `Text`/`Bytes`.
4. Build a Rust benchmark lab for Map strategies.
5. Define the JSON schema for stdlib contract diagnostics.
6. Add hum std explain Map as a future tool goal.
7. Create the `map-lab` benchmark track described in
   [OPTIMIZATION_AND_DSA_STRATEGY.md](OPTIMIZATION_AND_DSA_STRATEGY.md).

## Sources

- Krapivin, Farach-Colton, Kuszmaul, "Optimal Bounds for Open Addressing Without Reordering": https://arxiv.org/abs/2501.02305
- Krapivin, Przybocki, Subercaseaux, "Near-Optimal Encodings of Cardinality Constraints": https://arxiv.org/abs/2603.28954
- HierarchicalKV GPU cache-semantic hash table: https://arxiv.org/abs/2603.17168
- CHERI project overview, University of Cambridge: https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/
- PICASSO colored capabilities for CHERI temporal safety: https://arxiv.org/abs/2602.09131
- PoisonCap for CHERI temporal and initialization safety: https://arxiv.org/abs/2605.13210
- CHERI-D object IDs for temporal safety: https://arxiv.org/abs/2606.19055
- Abseil container guide: https://abseil.io/docs/cpp/guides/container
- Rust hashbrown README: https://github.com/rust-lang/hashbrown
- C++26 library overview: https://en.cppreference.com/cpp/26
- Hum stdlib constitution: STDLIB_CONSTITUTION.md
- Hum stdlib primitive research sweep: STDLIB_PRIMITIVE_RESEARCH_2026.md

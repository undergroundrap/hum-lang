# Hash Table Research Note: arXiv 2501.02305

Date: 2026-07-06

Paper: Martin Farach-Colton, Andrew Krapivin, and William Kuszmaul,
"Optimal Bounds for Open Addressing Without Reordering".

Source: https://arxiv.org/abs/2501.02305

## Why This Paper Matters

Hash tables are one of the most important standard-library data structures in
any systems language.

If Hum wants a world-class `std.data.Map`, it cannot copy whatever Rust, C++,
Go, Zig, or Java happen to ship today. It needs a research pipeline that can
take new theory, measure it against real workloads, and decide whether it belongs
in `std`, in a trusted-key fast path, in a lab package, or nowhere.

This paper matters because it changes old assumptions about open addressing
without reordering. It shows that better probe bounds are possible than the
classic mental model suggested.

That is exactly the kind of research Hum should respect.

It is not, by itself, a production-ready default map.

## What The Paper Shows

The paper studies open-addressed hash tables where inserted elements are not
reordered after insertion.

The important results:

- It disproves a central conjecture left by Yao's "Uniform Hashing is Optimal".
- It gives an open-addressed hash table without reordering, called elastic
  hashing, with amortized expected probe complexity `O(1)`.
- Elastic hashing also gives worst-case expected probe and insertion complexity
  `O(log(delta^-1))`, where `delta` is the unused-space fraction.
- It gives a greedy open-addressing scheme, called funnel hashing, with
  worst-case expected probe and insertion complexity `O(log^2(delta^-1))`.
- It proves matching lower bounds for the models studied.

This is a theory result with real standard-library implications: "simple
uniform probing is optimal" is not the end of the design space.

## Core Idea Hum Should Steal

The deepest idea is not merely "use this hash table".

The deepest idea is:

```text
separate the cost paid while inserting from the cost paid while searching
```

Elastic hashing may probe further during insertion than the final position used
for search. That decoupling lets the table spend work while placing an element
without permanently making every future lookup pay that same cost.

For Hum, this suggests that a `cost:` block for data structures must be more
precise than "hash map lookup is O(1)".

It should distinguish:

- present-key lookup
- missing-key lookup
- insertion
- deletion
- iteration
- memory overhead
- tail latency
- adversarial-key behavior
- address and slot stability
- reordering behavior
- resize behavior

That precision is how Hum becomes more honest than today's average stdlib docs.

## Why This Is Not Automatically `std.data.Map`

This is the part we have to be ruthless about.

The paper is not a complete production map story for Hum.

Missing or unresolved for a default standard-library map:

- key-value API details
- deletion-heavy workloads
- infinite-horizon insert/delete behavior
- negative-query performance for all variants
- adversarial keys and HashDoS resistance
- deterministic iteration order
- concurrency and memory reclamation
- cache locality on real CPUs
- SIMD probing details
- resize strategy and allocation policy
- constant factors
- implementation complexity
- benchmark evidence against hashbrown, SwissTable, Abseil, Go, Zig, and C++
- security review
- beginner explanation
- semantic graph representation

So the right conclusion is not "Hum has solved maps".

The right conclusion is:

```text
Hum needs one paved Map API with profile-backed strategies, not many equal-looking map defaults.
```

## Hum Map Profiles

Profiles are not a request to expose 50 public map types. They are the evidence
surface that lets the compiler and stdlib choose or reject a strategy while the
ordinary user still reaches for `Map` first.

Hum should eventually let programmers express map intent like this:

```text
store sessions: map SessionId -> Session {
  expects:
    up to 20 million active sessions
    present-key lookups dominate
    deletes are rare

  needs:
    stable addresses are not required
    deterministic iteration is not required

  protects:
    untrusted keys cannot force unbounded probe chains

  optimizes:
    lookup speed
    memory density
    bounded tail latency

  cost:
    present lookup is expected O(1)
    insert is expected O(log load_slack_inverse)
    missing lookup is bounded by selected strategy
}
```

The compiler, stdlib, and `chirp` should be able to say:

```text
warning[H09xx]: requested map profile is hostile-key safe, but selected strategy assumes trusted hash distribution
help: use std.data.SecureMap, declare keys trusted, or relax the speed goal
```

Or:

```text
error[H09xx]: stable addresses conflict with dense reordering map strategy
help: use pointer-stable map, arena-backed values, or remove stable-address need
```

## Map Strategy Candidates

Hum should compare multiple strategies inside `map-lab`:

- secure default map for untrusted keys
- SwissTable/hashbrown-style dense map
- open-addressing-without-reordering lab map
- elastic-hashing-inspired insert-heavy map
- funnel-hashing-inspired greedy map
- ordered B-tree map
- pointer-stable arena map
- sharded concurrent map
- cache-semantic map for bounded-capacity workloads
- GPU/batched lookup table in `std.accel`

No single strategy wins all of these.

The language win is letting the source state enough intent that tools can pick,
reject, or explain a strategy.

## Benchmark And Proof Requirements

Before any map strategy enters stable `std`, it needs:

- a reference implementation
- property tests
- fuzz tests
- adversarial key tests
- deletion and tombstone tests
- resize tests
- memory-pressure tests
- pointer-stability tests
- deterministic replay tests where promised
- benchmarks against real language libraries
- separate present-key and missing-key benchmarks
- bytes-per-entry reporting
- cache-miss and branch-miss reporting where available
- semantic graph output explaining the selected strategy
- documentation that a beginner can read

For elastic/funnel-inspired maps specifically, `map-lab` should track:

- load factor
- `delta` / slack fraction
- insertion probes
- final search probes
- worst observed probe count
- expected probe estimate
- high-percentile tail latency
- whether elements ever move after insertion
- how deletion changes the guarantee

## What This Changes For Hum

Hum's data-structure design should gain three explicit ideas.

### 1. Cost Has Dimensions

`O(1)` is not enough.

For maps, Hum should treat operation-specific cost as a first-class contract:

```text
cost:
  present lookup is expected O(1)
  missing lookup is expected O(log load_slack_inverse)
  insert is expected O(log load_slack_inverse)
  memory is at most 1.25x payload plus control bytes
```

### 2. Reordering Is A Promise

Some tables move elements. Some do not. Some preserve pointer stability. Some
preserve slot stability. Some preserve neither.

Hum should make this visible:

```text
needs:
  values may move during resize
  entries do not move after insertion within a generation
  stable addresses are not required
```

### 3. Research Enters Through Labs

The Hum standard library should have a clear route for serious papers:

```text
paper -> lab implementation -> Hum fixtures -> benchmarks -> adversarial tests -> explanation -> candidate -> std
```

This is how Hum can benefit from algorithmic breakthroughs without becoming a
museum of half-proven cleverness.

## Verse Connection

Verse is relevant here in a different way.

Verse pushes toward expressing desired truth and letting the language reason
about success, failure, and choice. This hash-table paper pushes toward a
similar design lesson for data structures: state the desired behavior precisely
enough that the implementation can be selected or rejected.

For Hum, the synthesis is:

```text
Verse-like intent for what must be true.
Systems-language contracts for cost, memory, failure, and trust.
Research-gated stdlib implementation behind that intent.
```

Hum should not copy hidden search into systems hot paths. But it should copy the
ambition of making code express obligations instead of merely issuing commands.

## Brutal Assessment

This paper could make Hum's map story much stronger.

It will not make every stdlib package perfect.

The path to a great Hum stdlib is not one breakthrough. It is a process that
keeps absorbing breakthroughs without overfitting to them:

- every data structure has visible assumptions
- every performance claim has a benchmark or proof
- every security claim has adversarial tests
- every clever implementation has a simple explanation
- every default survives production-shaped workloads

If Hum does that, then papers like this become fuel instead of hype.

## Source Notes

- arXiv page: https://arxiv.org/abs/2501.02305
- PDF: https://arxiv.org/pdf/2501.02305
- The paper's arXiv page lists v1 submitted 2025-01-04 and v2 revised
  2025-02-28.
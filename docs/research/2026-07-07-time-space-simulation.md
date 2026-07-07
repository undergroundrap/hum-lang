# Time-Space Simulation Research Notes

Date: 2026-07-07
Status: research snapshot, not a language standard

## Purpose

This note records the Hum design consequences of Ryan Williams' 2025
space-efficient simulation result and the immediate follow-up literature.

Hum should learn from breakthroughs like this, but the language must not turn a
complexity-theory theorem into marketing fog. The correct takeaway is that
space, time, recomputation, caching, and evidence should be first-class design
facts.

## Sources Read

- R. Ryan Williams, "Simulating Time With Square-Root Space", arXiv:2502.17779, submitted 2025-02-25: https://arxiv.org/abs/2502.17779
- Yakov Shalunov, "Improved Bounds on the Space Complexity of Circuit Evaluation", arXiv:2504.20950, submitted 2025-04-29, revised 2025-06-19: https://arxiv.org/abs/2504.20950
- Alexandra Henzinger, Edward Pyne, Seyoon Ragavan, "Catalytic Tree Evaluation From Matching Vectors", arXiv:2602.14320, submitted 2026-02-15, revised 2026-02-18: https://arxiv.org/abs/2602.14320
- Logan Nye, "TIME[t] subset SPACE[O(sqrt(t))] via Tree Height Compression", arXiv:2508.14831, submitted 2025-08-20, withdrawn 2026-01-01: https://arxiv.org/abs/2508.14831
- Vahid R. Asadi and Richard Cleve, "Polynomial-Time Almost Log-Space Tree Evaluation by Catalytic Pebbling", arXiv:2604.02606, submitted 2026-04-03, withdrawn 2026-04-07: https://arxiv.org/abs/2604.02606

## Facts

### Williams 2025

Williams proves that for every time bound `t(n) >= n`, deterministic multitape
Turing machines running in time `t` can be simulated in space
`O(sqrt(t log t))`.

The paper presents this as a major improvement over the Hopcroft-Paul-Valiant
`O(t / log t)` space simulation from the 1970s. It also gives consequences for
bounded fan-in circuit evaluation and time lower bounds for linear-space
problems.

The construction reduces time-bounded multitape computation to implicitly
defined Tree Evaluation instances. This is a space simulation, not a generic
runtime speedup theorem: it can reduce the memory needed to simulate a broad
class of computations, but the simulation may trade away time through replay or
recomputation. The useful intuition for Hum is the block view: split a long
computation into blocks, represent dependencies as a tree, and evaluate enough
boundary information to recover what is needed without storing the whole trace.

### Model Caveat

This is not a direct claim about arbitrary real-world RAM programs, modern CPUs,
cache hierarchies, operating systems, GPUs, or distributed systems.

Williams explicitly notes that arbitrary random-access models are not known to
fall under the theorem. The paper gives an extension for oblivious random-access
models where the read/write pattern can be computed in small space.

Hum should therefore treat this result as design pressure, not as a generic
optimizer guarantee.

### Circuit Follow-Up

Shalunov gives a direct circuit-evaluation route to related bounds, improving
size-`s` bounded fan-in circuit evaluation to `O(sqrt(s log s))` space and
avoiding some abstraction layers through Turing-machine simulation.

For Hum, this reinforces a compiler-IR lesson: many optimization and evidence
questions are graph evaluation questions. If Hum preserves dependency graphs,
profiles, and source spans, future tools can reason about resource tradeoffs
without scraping code text.

### Current Research Watchlist

Some exciting follow-up claims are unstable. Nye's claimed `O(sqrt(t))` result
was withdrawn in January 2026 because the proof of the main theorem was
incorrect. Asadi and Cleve's 2026 almost-log-space TreeEval result was also
withdrawn because the advertised polynomial-time bound did not hold.

Henzinger, Pyne, and Ragavan's 2026 catalytic TreeEval work remains a useful
watchlist item, but catalytic-space results should not become Hum doctrine until
they have enough review and implementation relevance.

This is a good reminder: Hum can be research-native only if it records source
status, dates, withdrawals, and confidence.

## Hum Design Consequences

### 1. Cost Claims Must Be Multidimensional

A serious `cost:` block cannot only say `time: O(n)`.

Hum should eventually distinguish:

```text
cost:
  time: O(work)
  peak space: O(boundary)
  scratch: bounded
  allocations: none in hot path
  recomputes: pure derived values only
  caches: bounded by profile memory budget
  check: bench
```

Milestone 0 does not need to implement this vocabulary immediately. The design
lesson is that time, peak memory, scratch memory, caching, recomputation, and
allocation policy are separate promises.

### 2. Recompute Is A Capability, Not A Trick

Space-efficient algorithms often recompute values instead of storing them.
Hum should support that path, but only when it is semantically safe.

A future optimizer may recompute only when the relevant computation is:

- pure or explicitly replayable
- deterministic under the active profile
- free of hidden IO, time, randomness, mutation, and external authority
- inside a declared cost and allocation budget
- explainable in the semantic graph

Recomputing `clock.now`, `random.secure`, network reads, file reads, or mutable
state is not an optimization. It is a behavior change unless the source captured
and declared the replay boundary.

### 3. Space-Pressure Profiles Should Be First-Class

Hum should eventually support profiles that prefer lower peak memory over raw
latency. Examples:

- embedded parser
- safety-critical controller
- offline audit tool
- memory-capped batch job
- mobile or edge runtime

Those profiles can guide library choices and optimizer passes, but they need
source-visible tradeoffs:

```text
optimizes:
  peak memory over throughput

tradeoffs:
  recomputing canonical forms is acceptable under memory pressure
```

### 4. Dependency Graphs Are Performance Evidence

The theorem's high-level story is a dependency-graph story. Hum already wants a
semantic graph for agents, security, and editor tooling. This research adds a
performance reason: resource tools need dependency facts, not terminal prose.

Future graph versions should expose enough information for:

- call graph cost propagation
- pure subgraph identification
- bounded cache and checkpoint opportunities
- profile memory budgets
- benchmark and proof evidence links
- stale evidence invalidation when dependencies change

### 5. Compiler Guarantees Need Confidence Labels

Hum should not reject code merely because it misses a theoretical lower bound.
It should reject visible contradictions, require evidence for high-risk claims,
and label confidence honestly.

Useful statuses:

```text
resource_status: declared
resource_status: statically_disproved
resource_status: statically_supported
resource_status: benchmark_supported
resource_status: proof_supported
resource_status: unverified
```

This matches Hum's evidence-native direction better than pretending all resource
facts are equally proven.

## Non-Claims

Hum should not claim:

- any arbitrary program can run in square-root memory on real hardware
- the compiler can always choose the optimal time-space tradeoff
- static analysis can reject all inefficient code
- recomputation is always better than caching
- theoretical asymptotics replace measurement
- withdrawn or unreviewed follow-ups are doctrine

The honest claim is stronger: Hum is built to absorb research by turning it into
checked resource intent, graph facts, profile policy, and evidence obligations.

## Near-Term Hum Actions

1. Keep `cost:`, `allocates:`, `optimizes:`, `avoids:`, and `tradeoffs:` as the
   user-facing resource-intent vocabulary.
2. Add resource evidence linking after security/trust evidence linking exists.
3. Preserve dependency graph facts before attempting smart recompute/cache
   optimization.
4. Add profile vocabulary for memory pressure before adding any automatic
   space-time optimizer.
5. Treat breakthrough papers as dated inputs with confidence status, not as
   permanent claims.
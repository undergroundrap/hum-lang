# Hum Math Engine Boundary

Date: 2026-07-07
Status: architecture boundary, not an integration contract

## Purpose

Hum should be able to use external mathematical verification engines without
turning any one engine into the language's source of truth.

Truth Harness, SMT tools, proof assistants, model checkers, benchmark harnesses,
and future math engines can all be useful evidence producers. They must sit
behind boring schemas, local execution rules, explicit assumptions, and Hum's
own evidence policy.

## Boundary Rule

Hum owns source semantics, graph facts, obligation IDs, spans, profiles,
effect/resource facts, and final evidence policy.

External math engines own attempts to prove, refute, explain, or classify those
obligations under declared assumptions.

```text
hum source -> hum graph -> hum obligations export -> external verifier -> evidence receipt -> hum evidence
```

The verifier can produce a receipt. It cannot silently optimize Hum programs,
rewrite Hum semantics, or make an uncheckable claim true.

## What Hum Emits

Hum may eventually export math obligations with:

- source span
- graph node id
- obligation id
- source claim text
- normalized claim kind
- assumptions
- allowed effects
- resource model
- profile
- program-shape classification
- requested confidence
- timeout and memory budget

The first useful obligation kinds should stay small:

- `allocation_freedom`
- `peak_memory_bound`
- `purity_replayability`

Resource optimization and standard-library proof work should wait until these
basic obligations have stable receipts.

## What A Math Engine Returns

An external engine may return:

- `proved`
- `refuted`
- `unknown`
- `unsupported`
- `timeout`

`unknown` is not a failure. It is an honest result.

`proved` requires a certificate, independently checkable trace, or verifier
artifact Hum policy accepts for the active profile. An LLM explanation, benchmark
run, cached success, or unchecked proof sketch is not a proof.

A result should preserve:

- obligation id
- assumptions used
- model limitations
- affected source spans
- counterexample when refuted
- certificate or checkable trace when proved
- trust label
- confidence level

## Program Shape Classification

Hum should distinguish program shapes before asking for strong resource or proof
claims:

| Shape | Expected proof strength | Rule |
| --- | --- | --- |
| streaming or sequential | high | Good target for peak memory and allocation claims. |
| tree, DAG, or circuit-like | high | Good target for replay, checkpoint, and space-first reasoning. |
| oblivious random access | medium-high | Promising when access patterns are explicit and deterministic. |
| arbitrary random access | medium-low | Degrade unless invariants and bounds are explicit. |
| pointer or mutation heavy | low | Needs ownership, separation, or aliasing proof first. |
| IO or effectful | low | Replay needs logged environment assumptions. |
| concurrent | low-medium | Needs scheduler/model assumptions. |
| hardware-specific | evidence only | Benchmarks and profile artifacts, not theorem claims. |

## Williams Rule

Ryan Williams' 2025 square-root-space result is a space-simulation theorem, not
a generic runtime speedup theorem. It can inspire block decomposition, tree
or DAG evaluation, recomputation, checkpoint placement, and bounded scratch
regions only when Hum has enough source facts to justify replay.

Before any Williams-inspired lowering is enabled, Hum should require evidence
for:

- deterministic execution
- replayable block semantics
- no hidden IO, time, randomness, volatile reads, or mutation
- bounded input and state model
- decomposable computation graph
- explicit block boundaries
- bounded scratch regions
- declared recompute/cache tradeoff

Until those are present, the compiler may report a resource-strategy candidate,
but it must not sell it as a verified optimization.

## Local Observability, Not User Telemetry

Math-engine integration may produce local developer-facing artifacts such as
obligation maps, proof receipts, counterexamples, budget reports, solver timing,
cache keys, and stale-evidence notices.

That is repo/tool observability. It is not user-data collection, training data,
cloud telemetry, or hidden network behavior.

## Non-Goals

Hum math-engine integration must not:

- require cloud access
- collect hidden telemetry
- treat an LLM proof as proof
- treat benchmark evidence as a theorem
- hide assumptions
- hide `unknown`
- trust cached proof success as release evidence
- make randomized hashing a deterministic safety claim
- make Williams' theorem a generic optimization slogan
- force one verifier to be the whole verification story

## Near-Term Integration Path

Do not add a hard dependency on Truth Harness or any external verifier in
Milestone 0.

The safe sequence is:

1. Let the external engine define schema fixtures and local receipts.
2. Review the schemas against Hum's graph and evidence policy.
3. Add a Hum obligation export only after the schema boundary is stable.
4. Import verifier receipts into `hum evidence` as evidence artifacts.
5. Add CI fixtures that prove `proved`, `refuted`, `unknown`, `unsupported`, and
   `timeout` remain distinct.

The first Hum-side code should be schema validation and fixture handling, not a
solver, optimizer, or proof assistant.
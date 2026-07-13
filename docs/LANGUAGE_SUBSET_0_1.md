# Hum Language Subset 0.1

Date: 2026-07-06

## Purpose

This document pins the source surface allowed by `offline-tool@0.1`.

Exact diagnostic allocations come only from
[`src/diagnostic_catalog.rs`](../src/diagnostic_catalog.rs); the checked human
projection is [DIAGNOSTICS.md](DIAGNOSTICS.md).

The alpha subset should be small enough to audit and boring enough to explain.
Anything outside this document is not part of the public alpha promise.

## Allowed Top-Level Items

The alpha subset may accept:

- `task`
- `type`
- `store`
- `test`

`app` may continue to parse for existing Milestone 0 examples, but it is not an
alpha execution boundary until explicitly added here.

## Allowed Task Sections

The alpha subset may use these task sections, in canonical order:

```text
why
uses
changes
needs
ensures
protects
trusts
fails when
watch for
cost
avoids
tests
proves
does
```

The first executable profile should treat unsupported sections as warnings while
Milestone 0 remains parser/checker-only. Before public alpha, unsupported
sections inside an executable `offline-tool@0.1` task should fail with stable
diagnostics.

## Required Task Sections

An alpha executable task must have:

- `why:`
- `uses:`
- `changes:`
- `needs:`
- `ensures:` or a documented no-return/no-output reason
- `cost:`
- `tests:` or generated test obligations
- `does:`

Security-sensitive tasks must also have:

- `protects:`
- `fails when:`
- `watch for:`

## Allowed Capabilities

`offline-tool@0.1` may model these capability families:

```text
files.read
evidence.write
json.parse
json.emit
hash.sha256
manifest.compare
sbom.compare
provenance.compare
policy.check
hum.graph
hum.diagnostics
hum.tests
```

A task may only read or write through declared capability entries.

## Denied Capabilities

The profile denies:

```text
network.*
process.*
ffi.*
unsafe.*
thread.*
clock.wall
random.*
env.read
plugin.*
dynamic_load.*
```

A denied capability should produce a stable registered diagnostic when one is
implemented. The reserved profile family currently allocates no exact code, so
the subset must not invent one; a more specific existing security/effects
diagnostic remains preferable when applicable.

## Allowed Control Shape

The first executable subset may allow:

- local immutable bindings
- explicit local `change` bindings
- `if`
- `match` over simple tagged results
- loops only over bounded collections already loaded from declared inputs
- `return`
- typed failure through `Result`-like shape

The subset should not allow:

- recursion
- unbounded `while`
- async
- threads
- macros
- dynamic dispatch
- generic user-defined abstractions
- unsafe
- FFI
- hidden allocation promises the compiler cannot report

## Allowed Data Shape

The alpha subset should prioritize:

- `Text`
- `Bool`
- bounded integer types
- `Bytes`
- `List<T>` for bounded input-derived collections
- records
- tagged result/failure values
- strict JSON values
- file digests

## Alpha Rule

If an executable construct cannot be represented in the semantic graph, effect
report, profile report, diagnostics schema, and evidence bundle, it is outside
`offline-tool@0.1`.

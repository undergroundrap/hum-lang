# Hum Resource Report Schema

Date: 2026-07-07

Current schema: `hum.resource_report.v0`

## Purpose

`hum resource-report` turns source-declared resource and optimization intent into
machine-readable facts.

This is the first compiler-owned inventory for the doctrine that resource,
layout, compile-time, interop, and agent-facing power must be explicit before it
is trusted. The report does not prove performance, allocation freedom, memory
bounds, or optimal layout. It records what the source claims, where it was said,
and how Hum classifies it today.

## Command

```powershell
hum resource-report [--format human|json] <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- resource-report examples/control_flow.hum
cargo run -- resource-report --format json examples/control_flow.hum
```

The human output is for quick review. The JSON output is for editors, agents,
CI wrappers, profile gates, benchmark planners, and future verifier bridges.

## Report Shape

```json
{
  "schema": "hum.resource_report.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "summary": {
    "files": 1,
    "tasks": 2,
    "resource_claims": 13,
    "errors": 0,
    "warnings": 0
  },
  "resource_claims": []
}
```

## Resource Claim Shape

Each entry is a source-declared claim, not proof:

```json
{
  "id": "hum_res_time_complexity_find_active_session_cost_26_5",
  "task": "find active session",
  "graph_node_id": "item:examples/control_flow.hum:3:1:task-find-active-session",
  "source_section": "cost",
  "claim_kind": "time_complexity",
  "resource_dimension": "time",
  "text": "time: O(sessions)",
  "source_span": {
    "file": "examples/control_flow.hum",
    "start_line": 26,
    "start_column": 5,
    "end_line": 26,
    "end_column": 22
  },
  "normalized_claim": {
    "representation": "hum_resource_claim_v0",
    "dimension": "time",
    "claim": "time:O(sessions)"
  },
  "verification_status": "declared",
  "proof_status": "not_proven",
  "benchmark_status": "not_measured",
  "related_math_obligation_kind": null
}
```

Exact IDs and spans are source-derived and may change when source paths, names,
or lines change.

## Current Classification Rules

Milestone 0 collects meaningful task lines from these sections:

- `cost:`
- `allocates:`
- `avoids:`
- `tradeoffs:`
- `optimizes:`

Current `claim_kind` values:

- `time_complexity`: `cost:` lines beginning with `time:`
- `space_complexity`: `cost:` lines beginning with `space:` or `memory:`
- `allocation_behavior`: `allocates:` lines or `cost:` lines beginning with
  `allocates:`
- `check_strategy`: `cost:` lines beginning with `check:`
- `optimization_priority`: `optimizes:` lines
- `avoided_shape`: `avoids:` lines
- `accepted_tradeoff`: `tradeoffs:` lines
- `cost_claim`: other `cost:` lines
- `resource_claim`: fallback for future resource sections

When the source declares a conservative allocation-free shape such as
`allocates: nothing`, the report sets `related_math_obligation_kind` to
`allocation_freedom`. The actual verifier-facing obligation lives in
[MATH_OBLIGATIONS_SCHEMA.md](MATH_OBLIGATIONS_SCHEMA.md); the resource report
only cross-references the kind.

## Honesty Rules

Every V0 claim is reported as:

- `verification_status`: `declared`
- `proof_status`: `not_proven`
- `benchmark_status`: `not_measured`

A later checker, benchmark harness, profile gate, or external verifier may add
stronger receipts, but this report must not silently upgrade prose into proof.

## Privacy And Dependency Rules

The command is local-first:

- no network
- no cloud
- no telemetry
- no solver dependency
- no benchmark dependency
- no generated code execution

## Non-Goals For V0

V0 does not infer hidden allocations, prove complexity, measure runtime, choose
algorithms, validate layout, or import external evidence. It is an inventory of
source-visible resource intent so humans, agents, and future tools stop guessing.
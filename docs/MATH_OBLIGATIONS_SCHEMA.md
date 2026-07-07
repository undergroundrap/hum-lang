# Hum Math Obligations Schema

Date: 2026-07-07

Current report schema: `hum.math_obligations.v0`
Current obligation schema: `hum.math_obligation.v0`

## Purpose

`hum math-obligations` exports verifier-facing math obligation candidates from
source facts Hum already owns. It is the first Hum-side bridge to external
validators such as Truth Harness.

This command does not run a solver, optimizer, proof assistant, benchmark, or
network service. It exports explicit claims and assumptions so another local
tool can validate shape first, then eventually attempt proof or refutation.

## Command

```powershell
hum math-obligations [--format human|json] [--out-dir <dir>] <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- math-obligations --format json examples/control_flow.hum
cargo run -- math-obligations --out-dir target/hum-math-obligations examples/control_flow.hum
```

The JSON output is a Hum report. Files written through `--out-dir` are individual
`hum.math_obligation.v0` objects that contract validators can read one at a
time.

## Report Shape

```json
{
  "schema": "hum.math_obligations.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "summary": {
    "files": 1,
    "tasks": 2,
    "obligations": 1,
    "errors": 0,
    "warnings": 0,
    "emitted_schema": "hum.math_obligation.v0"
  },
  "obligations": []
}
```

## Individual Obligation Shape

Each obligation file uses the Truth Harness V0 contract shape. Example generated
from `examples/control_flow.hum`:

```json
{
  "schema_version": "hum.math_obligation.v0",
  "obligation_id": "hum_obl_allocation_freedom_find_active_session_28_5",
  "obligation_kind": "allocation_freedom",
  "source_span": {
    "file": "examples/control_flow.hum",
    "start_line": 28,
    "start_column": 5,
    "end_line": 28,
    "end_column": 23
  },
  "graph_node_id": "item:examples/control_flow.hum:3:1:task-find-active-session",
  "claim_text": "task `find active session` declares no heap allocation: allocates: nothing",
  "normalized_formal_claim": {
    "representation": "hum_static_claim_v0",
    "expression": "heap_allocations(find_active_session) == 0",
    "variables": [
      {
        "name": "heap_allocations",
        "sort": "nat",
        "unit": "allocations"
      }
    ],
    "expected_bound": "0"
  },
  "assumptions": [
    {
      "assumption_id": "assume_declared_no_allocation_find_active_session_28",
      "evidence_class": "compiler_fact",
      "text": "Hum source declares no allocation; Milestone 0 exports this as evidence, not proof.",
      "source_ref": "hum_graph:item:examples/control_flow.hum:3:1:task-find-active-session"
    }
  ],
  "allowed_effects": ["read_memory", "time"],
  "program_shape": {
    "primary": "io_effectful",
    "access_pattern": "sequential",
    "mutation": "none",
    "io": "declared",
    "concurrency": "single_threaded",
    "hardware": "abstract"
  },
  "resource_model": {
    "machine_model": "hum_abstract_machine_v0",
    "word_bits": 64,
    "integer_overflow": "checked",
    "allocation_model": "none",
    "peak_memory_unit": "bytes"
  },
  "confidence_requested": "evidence_only",
  "timeout_budget": {
    "timeout_ms": 1000,
    "max_solver_memory_bytes": 67108864,
    "max_steps": 10000
  },
  "privacy": {
    "local_first": true,
    "network_access": "none",
    "cloud_access": "none",
    "telemetry": "none"
  }
}
```

Exact IDs and spans are source-derived and may change when source paths, names,
or lines change.

## Current Export Rules

Milestone 0 exports only conservative allocation-freedom candidates:

- `allocates: nothing` in a task `cost:` block
- `nothing`, `none`, `no allocation`, `no heap allocation`, or `zero allocations`
  in a task `allocates:` block

These become `allocation_freedom` obligations with
`confidence_requested: evidence_only`.

Hum deliberately does not export weaker prose such as `allocates: one task` as a
math obligation yet. Peak memory, replayability, pointer-heavy code, IO-heavy
code, concurrency, and hardware-specific claims need stronger source facts and
checker support before they become more than visible evidence candidates.

## Privacy And Dependency Rules

The command is local-first:

- no network
- no cloud
- no telemetry
- no Truth Harness dependency
- no solver dependency
- no benchmark-as-proof upgrade

External validators may accept or reject the emitted files. Hum must continue to
own source semantics, graph facts, obligation IDs, spans, and evidence policy.

## Non-Goals For V0

V0 does not prove allocation freedom. It does not infer hidden allocation. It
does not validate a result receipt. It does not import external evidence into
`hum evidence` yet.

The goal is a narrow but real bridge: Hum can emit explicit, validation-shaped
math obligations without coupling to any verifier.
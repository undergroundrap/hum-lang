# Hum Evidence Report Schema

Date: 2026-07-07

Current schema: `hum.evidence.v0`

## Purpose

`hum evidence` summarizes security and trust evidence obligations without
requiring humans or agents to inspect raw `hum graph` JSON.

It is a report surface over current graph facts. It is not a proof checker, not a
runtime telemetry collector, and not a replacement for the full semantic graph.
A linked item means a first-class evidence artifact names the same coverage
target; it does not prove the security property or trust boundary by itself.

## Command

```powershell
hum evidence <file-or-dir>...
hum evidence --format json <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- evidence examples/reference_surface.hum
cargo run -- evidence --format json examples/reference_surface.hum
```

The human output is for local terminals. The JSON output is the adapter, CI, and
agent contract.

## Top-Level Shape

```json
{
  "schema": "hum.evidence.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "summary": {},
  "evidence_obligations": []
}
```

## Summary Fields

- `files`: number of input files loaded
- `tasks`: number of task items inspected, including nested tasks
- `evidence_obligations`: number of generated security/trust obligations
- `linked`: obligations with at least one matching evidence artifact
- `unverified`: obligations without matching evidence
- `errors`: source diagnostics with error severity
- `warnings`: source diagnostics with warning severity

## Evidence Obligation Fields

Each `evidence_obligations` entry contains:

- `id`: source-derived evidence obligation ID
- `task`: task name that owns the obligation
- `kind`: `security_property` for `protects:` or `trust_boundary` for `trusts:`
- `blame`: owner category for the obligation
- `source_section`: source section that generated it
- `text`: source line text
- `span`: source span for the line
- `covers`: generated coverage phrase, such as `<task> protects <claim>`
- `coverage_key`: conservative canonical coverage key
- `suggested_evidence`: starter evidence name for humans and agents
- `verification_status`: `linked` or `unverified`
- `linked_evidence`: matching evidence artifacts

Current `linked_evidence` entries have `kind: test` and include `name`,
`modifiers`, `covers`, `coverage_key`, `match`, and `span`.

## Relationship To The Semantic Graph

`hum graph` remains the complete machine-readable source of parsed facts. `hum
evidence` is a narrower report over the same obligation and coverage-matching
rules. If these surfaces disagree, the implementation is wrong and the graph and
report must be brought back into agreement.

## Adapter Rules

- Query `hum capabilities --format json` before assuming `hum evidence` exists.
- Treat `linked` as traceability, not as proof of correctness.
- Treat `unverified` as work to schedule, generate, review, or prove.
- Ignore unknown fields.
- Preserve source spans when routing obligations into editors, CI annotations,
  agent prompts, or review packets.

## Non-Goals For V0

V0 does not promise:

- proof checking
- sanitizer or profiler evidence ingestion
- threat-model packet import
- SARIF, SPDX, OpenTelemetry, or external compliance export
- final schema stability

Those can be added as explicit evidence kinds later without changing the basic
report purpose.
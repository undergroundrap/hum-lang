# Hum Effect Report Schema 0.1

Date: 2026-07-06

## Purpose

The effect report is the first alpha artifact that proves Hum is more than a
pretty source syntax. It shows declared authority beside what the checker or
runner observed.

Exact diagnostic allocations come only from
[`src/diagnostic_catalog.rs`](../src/diagnostic_catalog.rs); the checked human
projection is [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Schema Name

```text
hum.effect_report.v0.1
```

## Top-Level Shape

```json
{
  "schema": "hum.effect_report.v0.1",
  "profile": "offline-tool@0.1",
  "toolchain": {},
  "inputs": [],
  "outputs": [],
  "tasks": [],
  "summary": {}
}
```

## Task Shape

Each task entry should contain:

- `task_id`
- `name`
- `source_span`
- `declared`
- `observed`
- `denied`
- `unresolved`
- `diagnostic_codes`
- `claim_ids`

## Declared Effects

`declared` should include:

- `reads`
- `writes`
- `mutates`
- `emits`
- `trusts`

## Observed Effects

`observed` should include the same categories when the checker or runner can
observe them.

For parser/checker-only phases, observed effects may be incomplete. In that
case, the report must place gaps under `unresolved` rather than pretending they
are proven.

## Denied Effects

`denied` records attempted or statically visible authority that the profile does
not allow:

```json
{
  "capability": "network.connect",
  "reason": "offline-tool@0.1 denies network access",
  "diagnostic_codes": []
}
```

The empty list is honest: no exact runtime-profile diagnostic is allocated.
The reserved profile family does not authorize inventing a placeholder code.

## Summary

`summary` should include:

- `status`: `pass`, `fail`, or `unknown`
- `declared_reads`
- `observed_reads`
- `declared_writes`
- `observed_writes`
- `denied_count`
- `unresolved_count`

## Alpha Rule

A public alpha demo must not say "no hidden effects" unless the effect report
shows declared, observed, denied, and unresolved effects for the demo run.

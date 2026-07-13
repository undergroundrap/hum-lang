# Hum Diagnostics Schema 0.1

Date: 2026-07-06

## Purpose

This document defines the machine-readable diagnostic target for
`offline-tool@0.1`.

The current compiler emits stable codes in terminal output, `hum check --format
json`, and graph output. The alpha schema narrows what evidence bundles must
preserve.

Exact allocations come only from
[`src/diagnostic_catalog.rs`](../src/diagnostic_catalog.rs); the checked human
projection is [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Schema Name

Working schema name:

```text
hum.diagnostics.v0.1
```

## Diagnostic Shape

```json
{
  "schema": "hum.diagnostics.v0.1",
  "diagnostics": [
    {
      "code": "H0201",
      "title": "save target not declared in changes",
      "severity": "error",
      "message": "task `save_item` saves into `tasks` without listing it in `changes:`",
      "span": {
        "file": "examples/humgate/gate.hum",
        "line": 12,
        "column": 3
      },
      "help": "Add `tasks` under `changes:` or avoid mutating it.",
      "profile": "offline-tool@0.1",
      "claim_ids": ["HA02"],
      "evidence_paths": []
    }
  ]
}
```

## Required Fields

Every alpha diagnostic must include:

- `code`
- `title`
- `severity`
- `message`
- `span` when source-backed
- `help` when a local repair exists

Alpha profile and evidence diagnostics should also include:

- `profile`
- `claim_ids`
- `evidence_paths` when the diagnostic is tied to generated artifacts

## Severity

Allowed severities:

- `error`
- `warning`
- `note`

`offline-tool@0.1` denied authority must be an error.

## Alpha Code Use

The alpha should use only exact codes already allocated in the canonical
registry. Relevant active families include:

- `H0200-H0299` (`declared_state_effects`)
- `H0400-H0499` (`security_trust`)
- `H0500-H0599` (`test_evidence`)

`H1100-H1199` is the reserved `runtime_profile_policy` family, but it allocates
no exact profile diagnostic yet.

Do not add an alpha-only diagnostic code without documenting it in
[DIAGNOSTICS.md](DIAGNOSTICS.md).

## Agent Rule

An agent should be able to repair alpha diagnostics by reading the diagnostic
JSON plus the semantic graph. If not, the diagnostic is underspecified.

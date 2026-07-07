# Hum Diagnostics Schema 0.1

Date: 2026-07-06

## Purpose

This document defines the machine-readable diagnostic target for
`offline-tool@0.1`.

The current compiler emits stable codes in terminal output, `hum check --format
json`, and graph output. The alpha schema narrows what evidence bundles must
preserve.

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
      "code": "H1001",
      "title": "profile denies capability",
      "severity": "error",
      "message": "task requests denied capability `network.connect` under `offline-tool@0.1`",
      "span": {
        "file": "examples/humgate/gate.hum",
        "line": 12,
        "column": 3
      },
      "help": "Remove the network access or move the task out of `offline-tool@0.1`.",
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

The alpha should prefer existing ranges:

- `H020x`: effects and mutation
- `H040x`: security and trust boundaries
- `H050x`: tests and regressions
- `H100x`: runtime profile and certification policy violations

Do not add an alpha-only diagnostic code without documenting it in
[DIAGNOSTICS.md](DIAGNOSTICS.md).

## Agent Rule

An agent should be able to repair alpha diagnostics by reading the diagnostic
JSON plus the semantic graph. If not, the diagnostic is underspecified.

# Hum Profile Report Schema 0.1

Date: 2026-07-06

## Purpose

The profile report records whether a run obeyed `offline-tool@0.1` policy.

The effect report says what authority appeared. The profile report says whether
that authority was allowed.

## Schema Name

```text
hum.profile_report.v0.1
```

## Top-Level Shape

```json
{
  "schema": "hum.profile_report.v0.1",
  "profile": "offline-tool@0.1",
  "status": "pass",
  "allowed": [],
  "denied": [],
  "required_evidence": [],
  "unresolved": [],
  "diagnostics": []
}
```

## Status

Allowed status values:

- `pass`
- `fail`
- `unknown`

Use `unknown` only when the tool cannot yet observe enough facts. A public demo
should not treat `unknown` as success.

## Allowed Entries

Each `allowed` entry should contain:

- `capability`
- `source`
- `reason`
- `evidence_path` when generated

## Denied Entries

Each `denied` entry should contain:

- `capability`
- `source`
- `reason`
- `diagnostic_code`
- `claim_ids`

## Required Evidence

`offline-tool@0.1` should require evidence for:

- semantic graph
- diagnostics
- effect report
- profile report
- run trace
- file hashes
- generated tests or test skeletons
- HumGate comparison outputs

## Alpha Rule

A task that requests a denied capability fails the profile even if the run did
not reach that code path.

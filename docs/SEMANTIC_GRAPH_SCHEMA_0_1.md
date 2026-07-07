# Hum Semantic Graph Schema 0.1

Date: 2026-07-06

## Purpose

This document defines the semantic graph facts that `offline-tool@0.1` evidence
must expose beyond the current `hum.semantic_graph.v0` prototype.

The graph is the shared truth for humans, tools, CI, and agents. It must not be
terminal prose in JSON clothing.

## Schema Name

Working schema name:

```text
hum.semantic_graph.v0.1
```

The current compiler still emits `hum.semantic_graph.v0`. This document is the
alpha target.

## Required Top-Level Fields

```json
{
  "schema": "hum.semantic_graph.v0.1",
  "toolchain": {},
  "profile": {},
  "summary": {},
  "files": [],
  "evidence_obligations": [],
  "diagnostics": []
}
```

## Toolchain Facts

`toolchain` should contain:

- `hum_version`
- `compiler_commit`
- `target`
- `build_profile`
- `feature_flags`

## Profile Facts

`profile` should contain:

- `name`: `offline-tool@0.1`
- `allowed_capabilities`
- `denied_capabilities`
- `status`: `pass`, `fail`, or `unknown`

## Item Facts

Each executable alpha task should expose:

- stable item ID
- item kind
- source path
- source span
- params and result
- declared sections
- section line facts with text, span, and meaningful/comment status
- declared capabilities from `uses:` and `changes:`
- inferred capability needs where the checker can see them
- declared tests
- generated test obligations
- declared cost facts
- profile restrictions that apply

## Test Obligation Facts

Alpha graph `test_obligations` should contain:

- `id`
- `kind`
- `blame`
- `source_section`
- `text`
- `span`
- `covers`
- `coverage_key`
- `suggested_test`
- `link_status`
- `linked_tests` with `match` set to `exact` or `canonical`

Milestone 0 emits task-level obligations and links them to actual top-level
tests when a meaningful `covers:` line either exactly matches the obligation
coverage phrase after whitespace normalization or shares its conservative
`coverage_key`. Richer traceability and semantic proof of paraphrased coverage
remain later alpha work.

## Effect Facts

Graph effect facts should use the same vocabulary as
[EFFECT_REPORT_SCHEMA_0_1.md](EFFECT_REPORT_SCHEMA_0_1.md):

- `reads`
- `writes`
- `mutates`
- `emits`
- `denies`
- `unresolved`

## Evidence Obligations

The graph should list obligations created by source promises:

- missing test for `needs:`
- missing test for `ensures:`
- missing negative fixture for `watch for:`
- missing protection evidence for `protects:`
- missing profile evidence for denied authority
- missing benchmark evidence for checked `cost:`

## Stability Rule

Field names used by evidence bundles are alpha API. Do not rename them without a
schema version bump.

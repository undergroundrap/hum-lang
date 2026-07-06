# Hum Semantic Graph Schema

Date: 2026-07-06

Current schema: `hum.semantic_graph.v0`

The semantic graph is the first stable-ish machine surface for Hum tools and
agents. It is not a final IR. It is a structured summary of what the compiler
understood from source.

## Command

```powershell
hum graph <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- graph examples/task_list.hum
```

## Top-Level Shape

```json
{
  "schema": "hum.semantic_graph.v0",
  "summary": {},
  "files": [],
  "diagnostics": []
}
```

## Summary

`summary` contains counts for quick tool routing:

- `files`
- `items`
- `tasks`
- `tests`
- `errors`
- `warnings`

## Files

Each file contains:

- `path`: source path
- `module`: module name or `null`
- `items`: parsed top-level items

## Items

Each item contains:

- `kind`: `app`, `type`, `store`, `task`, or `test`
- `name`: source-level item name
- `span`: source location of the item header
- `sections`: intent blocks captured from the item body

Additional fields depend on kind.

### App

Apps also contain nested `items`.

### Type

Types contain `fields`:

- `name`
- `type`
- `span`

### Store

Stores contain:

- `type`

### Task

Tasks contain:

- `params`
- `result`
- `sections`
- `test_obligations`

Params contain:

- `name`
- `type`
- `span`

### Test Obligations

Task `test_obligations` are generated from meaningful lines in `needs:`,
`ensures:`, `watch for:`, and `tests:` sections. Each obligation contains:

- `id`: stable-ish source-derived obligation ID
- `kind`: `precondition`, `postcondition`, `edge_case`, or `declared_test`
- `source_section`
- `text`
- `span`
- `covers`: coverage phrase a test can use
- `suggested_test`: human-readable generated test name

These are not executable tests yet. They are graph facts that future Hum test
generation, LSP actions, CI, and agents can use.

### Test

Tests contain:

- `params`
- `modifiers`
- `sections`

Known modifiers in Milestone 0:

- `unit`
- `property`
- `fuzz`
- `regression`
- `integration`
- `model`

## Sections

Each section contains:

- `name`
- `lines`: count of non-empty captured lines

Milestone 0 intentionally keeps this compact. Future versions should expose
section line spans and normalized contracts separately.

## Diagnostics

Diagnostics contain:

- `code`: stable diagnostic code, such as `H0201`
- `title`: short stable diagnostic title
- `severity`: `error` or `warning`
- `message`
- `span`, when available
- `help`, when available

See [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Current Checks Feeding The Graph

Milestone 0 currently checks:

- tasks and tests must have `why:` and `does:`
- duplicate sections produce warnings
- tasks returning values should have `ensures:`
- tasks should declare `needs:`
- tasks should declare `cost:`
- `save ... in resource` requires `resource` under `changes:`
- `set name = ...` requires a local `change name: ...` or top-level `changes:` entry
- `check: compile` plus `time: O(1)` rejects visible `for each`
- `check: compile` rejects unbounded-looking `while`
- security-sensitive resources should pair with `protects:`
- regression tests should include a `regression:` note
- known task and test sections should follow canonical order

## Non-Goals For V0

V0 does not yet promise:

- full expression parsing
- full type checking
- ownership checking
- borrow checking
- executable generated tests
- stable JSON formatting
- final package/module semantics

V0 exists to prove that Hum source can become structured meaning.

## Future Core Graph

[FORMAL_CORE.md](FORMAL_CORE.md) defines the first executable core Hum should
lower into. Future graph versions should expose enough core facts for tools,
agents, verifiers, debuggers, and profilers to work without guessing:

- lowered core task identity
- typed params and result
- declared and inferred effects
- mutable places
- reads and writes
- loop nodes
- call nodes
- failure variants
- profile restrictions
- proof and test obligations

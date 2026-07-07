# Hum Diagnostics

Date: 2026-07-06

## Purpose

Diagnostics are part of the language.

Hum is trying to make systems programming readable, safe, fast, and friendly to
agents. That only works if the compiler explains problems with stable facts, not
just terminal prose.

A Hum diagnostic must help five readers at once:

- a beginner who needs plain language
- a senior engineer who wants precise blame
- an IDE or LSP that needs stable structure
- a build system that needs machine-readable failure data
- an agent that needs repair instructions without guessing

## Diagnostic Rule

Every diagnostic should answer:

```text
What promise was broken?
Where is the smallest useful source span?
Which block is responsible?
How can a human repair it?
How can a tool recognize this class of issue next year?
```

## Current Shape

Milestone 0 diagnostics contain:

- `code`: stable diagnostic code, such as `H0201`
- `title`: short stable diagnostic title
- `severity`: `error` or `warning`
- `message`: human-readable explanation
- `span`: source file, line, and column when available
- `help`: optional repair guidance

Terminal shape:

```text
examples/task.hum:14:1: error[H0201]: task `save task` saves into `tasks` without listing it in `changes:`
  help: Add `tasks` under `changes:` or avoid mutating it.
```

JSON shape in `hum check --format json` and `hum graph`:

```json
{
  "code": "H0201",
  "title": "save target not declared in changes",
  "severity": "error",
  "message": "task `save task` saves into `tasks` without listing it in `changes:`",
  "span": { "file": "examples/task.hum", "line": 14, "column": 3 },
  "help": "Add `tasks` under `changes:` or avoid mutating it."
}
```

Diagnostic JSON commands:

```powershell
hum check --format json examples
hum explain H0201
hum explain H0201 --format json
hum diagnostics
hum diagnostics --format json
```

`hum check --format json` reads source and emits source-backed diagnostics as
`hum.check.v0` for editors, CI, and agents. `hum explain` and `hum diagnostics`
are offline and do not read source files. `hum explain` returns the stable code
title, default severity, plain-language explanation, and repair guidance for one
code. `hum diagnostics` returns the whole current catalog. The JSON schemas are
`hum.check.v0`, `hum.diagnostic_explain.v0`, and `hum.diagnostic_catalog.v0`.

JSON shape in `hum check --format json`:

```json
{
  "schema": "hum.check.v0",
  "summary": { "files": 1, "errors": 1, "warnings": 0 },
  "diagnostics": [
    {
      "code": "H0201",
      "title": "save target not declared in changes",
      "severity": "error",
      "message": "task `save task` saves into `tasks` without listing it in `changes:`",
      "span": { "file": "examples/task.hum", "line": 14, "column": 3 },
      "help": "Add `tasks` under `changes:` or avoid mutating it."
    }
  ]
}
```

JSON shape in `hum explain --format json`:

```json
{
  "schema": "hum.diagnostic_explain.v0",
  "code": "H0201",
  "title": "save target not declared in changes",
  "default_severity": "error",
  "explanation": "A `does:` body saves into a resource that is not listed under `changes:`, so mutation would be hidden from readers and tools.",
  "repair": "Add the resource under `changes:` if the mutation is intended, or remove the save."
}
```

JSON shape in `hum diagnostics --format json`:

```json
{
  "schema": "hum.diagnostic_catalog.v0",
  "count": 32,
  "diagnostics": [
    {
      "code": "H0201",
      "title": "save target not declared in changes",
      "default_severity": "error",
      "explanation": "A `does:` body saves into a resource that is not listed under `changes:`.",
      "repair": "Add the resource under `changes:` if the mutation is intended, or remove the save."
    }
  ]
}
```

## Stability Rules

Diagnostic codes are user-facing API.

Rules:

- Do not renumber a code after release.
- Do not reuse a removed code for a different meaning.
- If the meaning changes materially, add a new code.
- Message text may improve without changing the code.
- Help text may improve without changing the code.
- Severity may change only through an edition, feature gate, or clearly documented release policy.

## Code Ranges

Current ranges:

- `H000x`: parser and source shape
- `H010x`: item shape and intent block discipline
- `H020x`: effects, mutation, and declared state changes
- `H030x`: cost and performance contracts
- `H040x`: security and trust boundaries
- `H050x`: tests and regression obligations
- `H120x`: backend, target, and debug metadata

Future ranges should be reserved before broad use:

- `H060x`: type checking
- `H070x`: ownership and borrowing
- `H080x`: packages, capabilities, and Nectar
- `H090x`: unsafe, FFI, ABI, and provenance
- `H100x`: runtime profile and certification policy violations
- `H110x`: concurrency and memory ordering

## Current Codes

### Parser And Source Shape

| Code | Severity | Title | Meaning |
|---|---|---|---|
| `H0001` | warning | unexpected top-level line | Source has a top-level line Hum does not understand yet. |
| `H0002` | error | nested item extends past containing block | A nested item crosses its parent block boundary. |
| `H0003` | error | item header missing opening brace | An item header is malformed and does not end with `{`. |
| `H0004` | error | item block missing closing brace | An item starts but never closes with `}`. |
| `H0005` | warning | unknown item kind | The parser found an item-like header with an unknown kind. |
| `H0006` | warning | unexpected callable signature text | A task/test signature has trailing text Hum did not expect. |
| `H0007` | error | callable signature missing close parenthesis | A callable parameter list starts but does not close. |
| `H0008` | error | parameter missing type | A parameter lacks an explicit type. |

### Intent Block Discipline

| Code | Severity | Title | Meaning |
|---|---|---|---|
| `H0101` | warning | app missing why section | An app lacks a visible purpose. |
| `H0102` | warning | type missing shape | A type has no fields or invariant. |
| `H0103` | warning | store missing type | A store does not declare what it contains. |
| `H0104` | warning | store missing purpose | A store lacks `why:` or `expects:`. |
| `H0105` | error | item missing required section | A task/test lacks a required section such as `why:` or `does:`. |
| `H0106` | warning | duplicate section | The same section appears more than once in one item. |
| `H0107` | warning | task missing needs section | A task lacks preconditions. |
| `H0108` | warning | section out of order | A known section appears after a later canonical section. |
| `H0109` | warning | task return missing ensures section | A returning task lacks postconditions. |
| `H0110` | warning | hollow contract line | A contract-like line is too generic, tautological, or placeholder-shaped to catch a wrong implementation. |

### Effects And Mutation

| Code | Severity | Title | Meaning |
|---|---|---|---|
| `H0201` | error | save target not declared in changes | `does:` saves into a resource not listed under `changes:`. |
| `H0202` | error | set target not declared mutable | `does:` sets a name that is neither locally `change` nor declared in `changes:`. |

### Cost Contracts

| Code | Severity | Title | Meaning |
|---|---|---|---|
| `H0301` | warning | task missing cost section | A task does not declare cost expectations. |
| `H0302` | warning | cost missing check level | A `cost:` block lacks `check:`. |
| `H0303` | error | compile cost missing time claim | `check: compile` is requested without a `time:` claim. |
| `H0304` | error | constant cost claim has iteration | A task claims `time: O(1)` but visibly iterates. |
| `H0305` | error | compile cost has unbounded-looking while | A compile-checked cost block contains a `while` without an obvious bound. |

### Security And Trust

| Code | Severity | Title | Meaning |
|---|---|---|---|
| `H0401` | warning | security-sensitive task missing protects | A task touches security-sensitive resources without `protects:`. |
| `H0402` | warning | trust boundary missing protects | A task declares `trusts:` without a matching safety/security promise. |

### Tests And Regressions

| Code | Severity | Title | Meaning |
|---|---|---|---|
| `H0501` | warning | test missing covers section | A test does not say what promise it covers. |
| `H0502` | warning | regression test missing regression note | A regression test does not record the bug shape. |

### Target And Backend Metadata

| Code | Severity | Title | Meaning |
|---|---|---|---|
| `H1201` | error | unknown target fact record | `targets:` names a target record Hum does not publish. |
| `H1202` | error | unknown capability family | `targets:` names a capability family Hum does not publish. |
| `H1203` | error | unsupported target declaration | `targets:` contains a meaningful line with no current formal key. |

## Contract Quality Warnings

`H0110` is a Milestone 0 warning for obviously hollow claims in contract-like task sections such as `needs:`, `ensures:`, `protects:`, `trusts:`, `watch for:`, `allocates:`, and `optimizes:`.

The checker does not try to prove contracts yet. It only catches shapes that are visibly too weak to be useful, such as `true`, `works`, `safe`, `todo`, or `result == result`. The repair is not to add more words. The repair is to state a claim that could reject a real mistake.

## Canonical Section Order

Hum uses section order as pedagogy and review support. The order should read like
a senior engineer thinking through the work.

Current task order:

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
allocates
avoids
tradeoffs
optimizes
tests
does
```

Current test order:

```text
why
uses
needs
regression
covers
avoids
cost
does
```

`H0108` is a warning in Milestone 0. Hum should keep this as a warning until the
community has enough examples to prove the order is right.

## Severity Philosophy

Errors block compilation when Hum cannot trust the program meaning.

Warnings are used when the code is understandable but weakens readability,
reviewability, safety, or future tooling.

A warning may become an error only when:

- the rule has proved stable
- the rule catches real defects
- the repair path is clear
- `humfmt`, `chirp`, and LSP code actions can help
- the change is edition-gated or clearly announced

## LSP Mapping

`hum lsp` should map diagnostics directly:

- `code` -> LSP diagnostic code
- `title` -> short hover title or code description
- `severity` -> LSP severity
- `span` -> range
- `help` -> quick fix or code action hint

The LSP must not invent codes that the compiler did not emit.

## Agent Contract

Agents should treat diagnostic codes as repair handles.

Good agent behavior:

- fix by code, not message substring
- preserve the source span unless moving code is required
- explain the promise being repaired
- add tests when the diagnostic reveals a missing promise
- avoid broad rewrites for local diagnostics

Bad agent behavior:

- scrape terminal prose when JSON is available
- silence warnings without improving the source
- add `changes:` or `protects:` claims that are not true
- weaken `cost:` just to pass a check without recording a tradeoff

## Test Requirements

Every new diagnostic code needs:

- at least one parser/checker test that emits the code
- a fixture or example when the code affects public syntax
- JSON coverage before the code is used by LSP or agents
- documentation in this file

## Brutal Standard

If a diagnostic would make a beginner feel stupid, rewrite it.

If a diagnostic would make a senior engineer ask "but where is the real blame?",
rewrite it.

If an agent cannot repair from the code, span, message, and help, enrich the
machine form before adding clever syntax elsewhere.

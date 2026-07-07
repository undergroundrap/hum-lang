# Hum Language Reference

Date: 2026-07-06
Version: 0.0.1 pre-alpha
Status: reference spine, not a stable standard

## Purpose

This is the traditional language reference spine for Hum.

It exists so the project does not become only a pile of research notes, doctrine
docs, examples, and implementation details. This document answers the ordinary
language questions first:

- What can a `.hum` file contain?
- What are the top-level forms?
- What do sections mean?
- What does Milestone 0 parse and check today?
- Which deeper document owns each unfinished area?

If this document conflicts with [ARCHITECTURE.md](ARCHITECTURE.md), the
architecture is the design bug detector and the conflict must be resolved.

## Normative Levels

Hum uses explicit stability language:

- `current`: implemented or recognized by the Rust bootstrap today
- `reference`: intended language rule, but not fully implemented yet
- `alpha`: intended for the `offline-tool@0.1` public alpha subset
- `future`: design direction, not a compatibility promise
- `rejected`: should not enter Hum without a new accepted design decision

Milestone 0 is local, offline-first, parser/checker/graph only. It does not
execute Hum programs, generated code, build scripts, packages, plugins, or
foreign code.

## Reading Map

- [ARCHITECTURE.md](ARCHITECTURE.md): ground-truth map
- [PAVED_ROAD_DOCTRINE.md](PAVED_ROAD_DOCTRINE.md): default-path doctrine
- [LANGUAGE_CONSTITUTION.md](LANGUAGE_CONSTITUTION.md): rules Hum must not violate
- [MILESTONE_0_GRAMMAR.md](MILESTONE_0_GRAMMAR.md): current Rust bootstrap parser grammar contract
- [FORMAL_CORE.md](FORMAL_CORE.md): precise executable core direction
- [LANGUAGE_SUBSET_0_1.md](LANGUAGE_SUBSET_0_1.md): pinned alpha subset
- [SEMANTIC_GRAPH_SCHEMA.md](SEMANTIC_GRAPH_SCHEMA.md): graph JSON emitted today
- [DIAGNOSTICS.md](DIAGNOSTICS.md): stable diagnostic code contract
- [FORMATTER.md](FORMATTER.md): canonical formatting direction

## Checked Reference Fixture

[../examples/reference_surface.hum](../examples/reference_surface.hum) is the
canonical Milestone 0 reference fixture. It is intentionally small and should
stay parseable by the Rust bootstrap CLI.

Use it when changing this reference, the grammar contract, syntax highlighting,
or graph facts. If prose says a current construct exists, the fixture should
show the ordinary spelling unless a smaller focused example owns that case.

A healthy reference fixture passes `hum check` without diagnostics and produces no
unlinked obligations from `hum test-skeletons`.

## Source Files

A Hum source file uses the `.hum` extension.

Current source files are UTF-8 text without BOM. Public docs and tools should
use repo-relative paths and portable examples unless documenting a platform
boundary.

A file may start with a module declaration:

```hum
module examples.task_list
```

Milestone 0 treats the module path as source metadata. Future module and
visibility rules are tracked in [CORE_LANGUAGE_SHAPE.md](CORE_LANGUAGE_SHAPE.md).

## Lexical Shape

Hum is line-oriented and block-structured.

Current rules:

- top-level items use braces
- normal source does not use semicolons
- section headers end in `:`
- section body lines are indented under the section header
- comments may start with `#` or `//` inside sections
- two-space indentation is the formatter direction

Current parser behavior is intentionally small. It captures many section body
lines as text for graph facts and future lowering. It is not the final grammar
engine. See [MILESTONE_0_GRAMMAR.md](MILESTONE_0_GRAMMAR.md) for the exact
Milestone 0 parser contract.

## Top-Level Forms

Current Milestone 0 recognizes these item kinds:

- `app`
- `type`
- `store`
- `task`
- `test`

The `offline-tool@0.1` alpha subset may accept `task`, `type`, `store`, and
`test`. `app` remains parsed for examples but is not yet an alpha execution
boundary.

### `app`

```hum
app Name {
  why:
    explain the application
}
```

An `app` groups program-level intent and may contain nested items. Milestone 0
parses it and emits graph facts. Runtime application semantics are future work.

### `type`

```hum
type Task {
  title: Text
  done: Bool
}
```

A `type` describes data. Milestone 0 parses field names, field types, section
counts, section spans, and section line facts. Rich type checking is future
work.

### `store`

```hum
store tasks: list Task {
  why:
    remember the user's tasks
}
```

A `store` declares persistent or shared data intent. The type after `:` is
currently captured as text. Future stdlib/profile work decides the concrete
strategy, such as a paved-road `Map` with profile-backed internals.

### `task`

```hum
task add task(title: Text) -> Result Task, TaskError {
  why:
    let the user remember something to do

  needs:
    title is not empty

  ensures:
    new task is saved

  does:
    return task
}
```

A `task` is Hum's function-like unit. It combines a callable header, checked
intent sections, and a `does:` body. Milestone 0 parses and checks task shape
but does not execute the body.

### `test`

```hum
test add task rejects empty title regression {
  covers:
    add task fails when title is empty

  does:
    expect add task("") fails with TaskError.empty_title
}
```

A `test` is first-class evidence. Milestone 0 parses tests, modifiers, `covers:`
lines, and links exact or conservative canonical coverage phrases to generated task obligations.

Known current test modifiers:

- `unit`
- `property`
- `fuzz`
- `regression`
- `integration`
- `model`

## Names

Hum names are intentionally human-readable. Item names may contain spaces in
today's examples, such as `add task`.

Milestone 0 stores names as parsed source text. Later grammar work must pin the
exact identifier rules for modules, item names, fields, parameters, variants,
and paths before editor grammars and self-hosting depend on them.

## Parameters And Results

Task and test parameters use this current shape:

```hum
task name(input: Type, other: Type) -> Output {
  does:
    return value
}
```

Current parser facts:

- parameter name
- parameter type text
- source span
- optional result type text for tasks

Parameter and result types are not yet fully type-checked in Milestone 0.

## Sections

Sections are named intent blocks inside items.

```hum
needs:
  title is not empty
```

Milestone 0 graph output includes:

- section name
- section span
- non-empty line count
- `line_items` with text, span, and meaningful/comment status

Comment-only section lines are preserved in graph output and marked
`meaningful: false`.

`hum syntax` emits a machine-readable section catalog with hover text and a
semantic-token legend so editor and LSP adapters can explain and color the
current surface without duplicating language copy.

## Task Sections

Common task sections:

| Section | Status | Meaning |
| --- | --- | --- |
| `why:` | current | human and agent purpose |
| `uses:` | current | read dependencies and capabilities |
| `changes:` | current | mutation/write permissions |
| `needs:` | current | preconditions and generated test obligations |
| `ensures:` | current | postconditions and generated test obligations |
| `protects:` | current | safety/security promises and evidence obligations |
| `trusts:` | current | trust assumptions and evidence obligations |
| `fails when:` | current | explicit failure modes |
| `watch for:` | current | edge cases and generated test obligations |
| `cost:` | current | time, space, allocation, and check claims |
| `allocates:` | current | visible allocation expectations and limits |
| `avoids:` | current | implementation shapes to avoid |
| `tradeoffs:` | current | accepted engineering compromises |
| `optimizes:` | current | performance or quality priorities when tradeoffs conflict |
| `tests:` | current | declared test obligations |
| `proves:` | reference | formal or semi-formal proof obligations |
| `does:` | current | captured body text; executable semantics are future work |

Additional sections such as `creates:`, `deletes:`, `assumes:`, `keeps:`,
`benchmarks:`, and `calls:` are part of the broader design direction and appear
in design docs. They require more checker and graph work before becoming stable
executable promises.

## Canonical Section Order

Milestone 0 warns when known task and test sections are out of canonical order.

Preferred task order follows [FORMATTER.md](FORMATTER.md). The current checker warning order is narrower and is listed in [DIAGNOSTICS.md](DIAGNOSTICS.md):

```text
why
uses
changes
creates
deletes
needs
assumes
ensures
keeps
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
benchmarks
proves
does
```

## Effects And Capabilities

Hum does not intend to hide IO, mutation, allocation, randomness, time, network,
or unsafe behavior behind innocent-looking calls.

Current Milestone 0 checks are small:

- save-like mutation in `does:` should refer to declared `changes:` targets
- known sections should appear in canonical order
- tasks should have important context such as `why:` and `does:`
- contract-like lines should not be obviously hollow, tautological, or placeholder-shaped
- simple cost claims should not contradict visible loop shape

Future effect reports are tracked in [EFFECT_REPORT_SCHEMA_0_1.md](EFFECT_REPORT_SCHEMA_0_1.md).

## Contracts And Blame

Contract sections assign responsibility:

- `needs:`: caller or context must satisfy the precondition
- `ensures:`: task must satisfy the postcondition on success
- `fails when:`: task exposes a typed failure condition
- `changes:`: task may mutate only declared targets
- `cost:`: implementation claims must be checkable or explicitly deferred
- `protects:` and `trusts:`: security boundaries must be named

Full blame semantics are still design work, but the reference rule is simple:
important claims belong in checked sections, not comments. A checked section line
should be specific enough that a future verifier, test, or reviewer could notice
when an implementation breaks it.

## Test Obligations And Coverage

Milestone 0 generates task `test_obligations` from meaningful lines in:

- `needs:`
- `ensures:`
- `watch for:`
- `tests:`

`hum graph` links obligations to top-level tests when a meaningful `covers:`
line exactly matches the generated coverage phrase after whitespace
normalization or shares its conservative coverage key. The canonical key absorbs
case, punctuation, filler words, hyphenation such as `non-empty`, and small
section aliases such as `requires` for `needs`; it does not prove broad semantic
paraphrases.

`hum test-skeletons` prints Hum `test` blocks for unlinked obligations. It does
not execute code or write files.

## Evidence Obligations

Milestone 0 generates task `evidence_obligations` from meaningful lines in:

- `protects:`
- `trusts:`

`protects:` lines become `security_property` obligations with
`security_boundary` blame. `trusts:` lines become `trust_boundary` obligations
with `trust_boundary` blame. Current obligations include source spans,
generated `covers` phrases, canonical `coverage_key` values,
`suggested_evidence`, `verification_status`, and `linked_evidence`.

`hum graph` links evidence obligations to top-level tests when a meaningful
`covers:` line exactly matches the generated coverage phrase after whitespace
normalization or shares its conservative coverage key. `verification_status` is
`linked` when at least one evidence artifact matches and `unverified` when none
do.

This is intentionally not a proof claim. A linked test means the test names the
same coverage target; it does not prove the protection or trust boundary is
fully verified. Future evidence kinds include proof exports, threat-model
notes, review packets, sanitizer runs, and profile evidence.

## Executable Body Status

The `does:` block is the future executable body.

In Milestone 0, body lines are parsed as section text for checks and graph
facts. They are not executed. Any executable syntax must lower into
[FORMAL_CORE.md](FORMAL_CORE.md) before it becomes stable.

Starter executable forms are tracked in [CORE_LANGUAGE_SHAPE.md](CORE_LANGUAGE_SHAPE.md).

## Diagnostics

Milestone 0 diagnostics have stable `H####` codes, severity, message, optional
span, and optional help. Diagnostics appear in terminal output and `hum graph`
JSON.

See [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Semantic Graph

The semantic graph is the machine-readable form of what the compiler currently
understands. It is the source for agents, editor tools, future formatters,
coverage checks, and evidence bundles.

Current command:

```powershell
hum graph <file-or-dir>...
```

During bootstrap:

```powershell
cargo run -- graph examples
```

See [SEMANTIC_GRAPH_SCHEMA.md](SEMANTIC_GRAPH_SCHEMA.md).

## Formatting

Hum will have one canonical format. The formatter should be strict, boring, and
low-configuration.

Current direction:

- two-space indentation
- canonical section order
- braces for item bodies
- no semicolons in ordinary source
- preserve comments and intent blocks

See [FORMATTER.md](FORMATTER.md).

## Paved Road Rule

Language syntax and standard-library APIs should follow the
[Paved Road Doctrine](PAVED_ROAD_DOCTRINE.md): one obvious safe default, with
explicit side roads only when evidence and source-visible intent justify them.

This means the language reference should prefer one clear spelling for common
programs. Alternatives need a reason that appears in diagnostics, graph facts,
profiles, or evidence.

## Rejected Or Delayed In Core

Rejected for core Hum unless a future accepted design reverses it:

- C/C++ headers
- C-style `for (init; condition; step)` loops
- implicit null
- hidden exceptions for normal failure
- fallthrough switch statements
- C-style preprocessor macros
- hidden unsafe behavior

Delayed until the formal core, graph, diagnostics, and tooling can explain them:

- user macros
- closures
- async/await surface syntax
- operator overloading
- inheritance
- dynamic dispatch by default
- compile-time reflection

## Current Commands

```powershell
hum check <file-or-dir>...
hum graph <file-or-dir>...
hum test-skeletons <file-or-dir>...
hum syntax
hum syntax --format textmate
```

Bootstrap examples:

```powershell
cargo run -- check examples
cargo run -- graph examples/reference_surface.hum
cargo run -- graph examples
cargo run -- test-skeletons examples
cargo run -- syntax
cargo run -- syntax --format textmate
```

## Open Reference Gaps

This reference is intentionally incomplete. The next gaps to close are:

- final identifier and path grammar beyond the Milestone 0 parser contract
- exact expression grammar for the first executable subset
- type grammar for records, results, options, lists, and maps
- import and visibility rules
- formal lowering from surface constructs into Core Hum
- stable examples for every accepted syntax form beyond `examples/reference_surface.hum`
- generated editor grammar and syntax-highlight keyword list beyond the current TextMate snapshot

Until those are pinned, broad syntax expansion should stay in design docs, not
in stable Hum.

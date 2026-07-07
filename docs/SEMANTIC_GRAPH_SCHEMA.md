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

## Node IDs

`id` fields are source-derived handles for tools, editor sessions, agents, and
future debugger facts. They are local to `hum.semantic_graph.v0`; they are not
cryptographic hashes, package-global identities, or proof that a renamed node is
semantically unchanged.

Current IDs use a normalized source path plus line, column, kind, and label
where useful. They should remain stable while the path, source location, and
node label stay stable. Moving code, renaming a node, or changing the line that
declares it may change the ID. Keep using `span` for display and blame; use
`id` when a tool needs to refer back to the same graph node.

Lines and columns are one-based. The Milestone 0 parser reports the first
visible source column for line-oriented constructs such as item headers,
sections, section lines, fields, and diagnostics. It does not yet emit full
start/end token ranges.

## Files

Each file contains:

- `id`: source-derived file node ID
- `path`: source path
- `module`: module name or `null`
- `folding_ranges`: section ranges for editor and LSP adapters
- `symbols`: document-symbol outline for editor and LSP adapters
- `items`: parsed top-level items

## Folding Ranges

File `folding_ranges` are editor-friendly ranges for collapsing intent sections.
They are derived from section headers and captured section lines, not from a
second parser.

Each folding range contains:

- `id`: section node ID
- `kind`: currently `section`
- `name`: section name, such as `why`, `does`, or `cost`
- `owner`: item `id`, `kind`, and `name`
- `span`: section header source location
- `start_line`: one-based source line where the section starts
- `end_line`: one-based source line where the captured section body ends

Milestone 0 folding ranges cover intent sections only. Item-body ranges need
item end spans and should wait until the parser records them honestly. LSP
adapters should convert these one-based graph lines to protocol-specific ranges
at the adapter boundary.

## Symbols

File `symbols` are an editor-friendly outline derived from the same AST as
`items`. They are not a second parser or alternate source of truth.

Each symbol contains:

- `id`: source-derived graph node ID
- `kind`: `app`, `type`, `store`, `task`, `test`, or `field`
- `name`: display name
- `span`: source location
- `children`: nested symbols

Milestone 0 symbols contain top-level items, nested app items, and type fields.
Module names remain file metadata until module declarations carry their own
source spans.

## Items

Each item contains:

- `id`: source-derived item node ID
- `kind`: `app`, `type`, `store`, `task`, or `test`
- `name`: source-level item name
- `span`: source location of the item header
- `sections`: intent blocks captured from the item body

Additional fields depend on kind.

### App

Apps also contain nested `items`.

### Type

Types contain `fields`:

- `id`
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
- `evidence_obligations`

Params contain:

- `id`
- `name`
- `type`
- `span`

### Test Obligations

Task `test_obligations` are generated from meaningful lines in `needs:`,
`ensures:`, `watch for:`, and `tests:` sections. Each obligation contains:

- `id`: stable-ish source-derived obligation ID
- `kind`: `precondition`, `postcondition`, `edge_case`, or `declared_test`
- `blame`: current owner category for the obligation; `caller` for `needs:`,
  `callee` for `ensures:`, and `evidence` for `watch for:` or `tests:`
- `source_section`
- `text`
- `span`
- `covers`: coverage phrase a test can use
- `coverage_key`: conservative canonical key used for non-exact matching
- `suggested_test`: human-readable generated test name
- `link_status`: `linked` when at least one exact or canonical `covers:` line matches, otherwise `unlinked`
- `linked_tests`: covering test references with `name`, `modifiers`, `covers`, `coverage_key`, `match`, and `span`

These are not executable tests yet. They are graph facts that future Hum test
generation, LSP actions, CI, and agents can use. The `blame` field is not a full
proof verdict; it is the current repair owner a future checker, test runner, or
agent should start from when that obligation is missing or violated. Milestone 0 links obligations
to top-level `test` items when a meaningful `covers:` line either exactly
matches the obligation `covers` phrase after whitespace normalization or shares
the same conservative `coverage_key`. Canonical matching tolerates case,
punctuation, filler words, hyphenation such as `non-empty`, and small section
aliases such as `requires` for `needs`; it does not prove broad semantic
paraphrase equivalence.

### Evidence Obligations

Task `evidence_obligations` are generated from meaningful lines in `protects:`
and `trusts:` sections. Each obligation contains:

- `id`: stable-ish source-derived evidence obligation ID
- `kind`: `security_property` for `protects:` or `trust_boundary` for `trusts:`
- `blame`: current owner category for the obligation; `security_boundary` for
  `protects:` and `trust_boundary` for `trusts:`
- `source_section`
- `text`
- `span`
- `suggested_evidence`: human-readable starting point for a proof, test,
  review packet, or threat-model note
- `verification_status`: currently `unverified`

These are not proof results yet. They make security and trust promises visible
as machine facts so future test generation, proof export, CI, IDE actions, and
agents do not have to scrape prose. Milestone 0 emits declared obligations and
marks them `unverified` until a later evidence linker can connect them to
specific tests, proofs, reviews, sanitizer runs, or profile facts.

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

- `id`
- `name`
- `span`
- `lines`: count of non-empty captured lines
- `line_items`: non-empty captured section lines

Each `line_items` entry contains:

- `id`
- `text`
- `span`
- `meaningful`: `false` for comment-only lines that start with `#` or `//`; `true` for lines that feed graph facts such as obligations and coverage

Milestone 0 keeps normalized contract facts, such as `test_obligations` and
`evidence_obligations`, separate from raw section lines so tools can choose
either source-faithful or normalized views.

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
- obviously hollow contract-like lines produce `H0110` warnings
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

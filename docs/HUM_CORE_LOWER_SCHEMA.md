# Hum Core Lower Schema

Date: 2026-07-08

Current schema: `hum.core_lower.v0`

## Purpose

`hum core-lower` emits the first source-mapped Core Hum artifact boundary. It
sits after `hum core-preview`, `hum resolve`, and `hum type-check`, and before
Hum IR, interpreters, or backend adapters.

This command is intentionally not an interpreter, not Hum IR, not an optimizer,
not an effect checker, and not a safety proof. It serializes the tiny Core
Hum-shaped subset the bootstrap can currently lower, keeps every operation tied
to source spans, and reports blockers where source text cannot yet become Core
Hum.

## Command

```powershell
hum core-lower [--format human|json] [--timings] <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- core-lower examples/reference_surface.hum
cargo run -- core-lower --format json examples/reference_surface.hum
```

The human output is for terminals. The JSON output is for agents, CI wrappers,
`hum core-verify`, future interpreters, and future Hum IR lowering.

## Top-Level Shape

```json
{
  "schema": "hum.core_lower.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "lowering_status": "unverified_core_artifact_v0",
  "milestone": "0 semantic graph",
  "core_contract_schema": "hum.core_contract.v0",
  "core_preview_schema": "hum.core_preview.v0",
  "resolve_schema": "hum.resolve.v0",
  "type_check_schema": "hum.type_check.v0",
  "ir_contract_schema": "hum.ir_contract.v0",
  "summary": {},
  "core_items": [],
  "non_goals_v0": []
}
```

## Fields

- `schema`: schema name, currently `hum.core_lower.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: maturity label such as `pre-alpha`
- `lowering_status`: currently `unverified_core_artifact_v0`
- `milestone`: current implementation milestone
- `core_contract_schema`: Core Hum contract this artifact targets
- `core_preview_schema`: source preview facts consumed before lowering
- `resolve_schema`: checked resolver facts that must be clean before future
  executable use
- `type_check_schema`: declaration and trivial return type facts that must be
  clean before future executable use
- `ir_contract_schema`: future Hum IR consumer contract
- `summary`: aggregate file, item, task, operation, blocker, diagnostic, and
  readiness counts
- `core_items`: source items with a `does:` section and their lowered operation
  rows or blockers
- `non_goals_v0`: claims this command must not make

## Summary Shape

`summary` includes:

- `files`, `items`, `tasks`, and `tests`
- `core_items`, `lowered_items`, and `blocked_items`
- `lowered_operations` and `blocked_operations`
- `execution_ready`: always `0` in V0
- `ir_ready`: always `0` in V0
- `errors`, `warnings`, `resolver_errors`, and `type_errors`
- `preview_blocked_statements`: blocker count inherited from `hum.core_preview.v0`

## Core Item Shape

Each `core_items` entry has:

- `id`: source-derived Core item row id
- `kind`: source item kind, currently `task` or `test` when it has `does:`
- `name`: source item name
- `source_span`: source file, line, and column for the item
- `status`: lowering status for the item
- `verification_status`: currently `unverified_v0`
- `execution_ready`: always `0` in V0
- `body_status` and `grammar_status`: body grammar facts consumed from the
  partial Core body grammar
- `params` and `result`: source signature facts preserved for future type gates
- `source_sections`: source section names present on the item
- `operations`: source-mapped Core operation rows
- `blockers`: source-mapped blockers that prevent future execution or IR claims

Current item statuses:

- `lowered_unverified_core_v0`: all current rows became unverified Core operation
  rows and no source, resolver, type, body, or brace blockers were found
- `blocked_by_source_errors`: source diagnostics contain errors
- `blocked_by_resolver_errors`: checked resolver facts contain errors
- `blocked_by_type_errors`: declaration/trivial-return type facts contain errors
- `blocked_before_core_execution`: at least one operation or block shape is not
  lowerable in V0
- `empty_body`: the `does:` section has no meaningful body lines

## Operation Shape

Each operation row has:

- `id`: source-derived operation row id
- `index`: zero-based operation index inside the candidate body
- `source_span`: source file, line, and column for the body row
- `surface_text`: original source line text
- `source_kind` and `source_status`: partial body grammar facts
- `core_operation`: Core Hum operation family or `blocked_surface_statement`
- `status`: `lowered_unverified_operation_v0` or `blocked_operation_v0`
- `expression`: compact expression preview root, or `null`
- `reason`: optional blocker or honesty reason

An H0906 unsupported-`try` fact lowers only as
`blocked_unsupported_try_expression` with `blocked_operation_v0`, a null
expression, and the shared stable reason. It is never a lowered executable
operation.

Session AF lowers accepted shared predicate facts as
`checked_contract_predicate_v2` with a non-null `predicate_ast_v2` expression
whose root is `predicate_comparison`, and preserves malformed/semantic H0704 facts as
`blocked_contract_predicate_v2` with the exact shared reason and source status.
The lowerer consumes the analyzer fact and does not recognize contract strings
independently.

Lowered V0 operation families include:

- `return`
- `fail`
- `let_binding`
- `mutable_binding`
- `set_place`
- `if_statement`
- `while_loop`
- `for_each`
- `for_index`
- `loop`
- `block_close`

Blocked V0 operation families include record-construction fields, nested intent
headers inside `does:`, test expectations, store writes, and unknown body lines.

## Expression Shape

`expression` is intentionally compact. It carries the expression preview root
needed by future type/effect/lowering work:

- `text`, `kind`, and `status`
- `ast_status`, `root_form`, `operator`, and `node_count`
- `type_status`, `type_text`, and `type_source`
- `effect_status`
- `reason`

Type slots may be populated only from checked trivial return facts already
reported by `hum.type_check.v0`. They are not broad expression inference.

### Bounded Structured Expression

For a validated parser-owned `Binary(Add, Identifier, Identifier)` return,
`structured_expression` is an additive object inside the existing expression
row. The validated Core-body transport associates the canonical expression
directly with its owning statement; it does not use an index-correlated side
vector. Structural transport does not depend on a checked expression type.
Other expression shapes retain the existing flat preview and emit
`structured_expression: null`.

The bounded object contains:

- `provenance`: `parser_owned_canonical_expression_v0`;
- `parser_node_id`: the original parser-owned binary-root identity;
- `source_range`: the original file, one-based line and column, and
  `byte_length` of that parser node;
- `kind`: `binary`;
- `operator`: `add`;
- `children`: exactly two ordered child rows.

Each child row contains its zero-based `index`, semantic `role`, original
`parser_node_id`, parser-owned `source_range`, `kind`, and identifier spelling.
Index 0 is role `left`; index 1 is role `right`. The public object is a
projection of the retained parser tree, not independent authority. The
in-memory Core-lower expression privately retains an immutable copy of the
parser-owned canonical expression for Core verification. That private
authority is absent from human and JSON output. Identities and ordering are
not inferred from names, source-range adjacency, or rendered `surface_text`.

The range records identify source bytes; they are not bytecode ranges or
backend offsets. Core verification compares the projected root and child
ranges exactly with private parser authority. Same-file containment and
left-to-right order remain additional sanity checks. Coherent foreign or
relocated projections fail even when internally consistent, and candidate
range arithmetic fails closed on overflow.

The structured object contains no type field. The existing outer expression
type fields remain authoritative. For `examples/core/minimal_add.hum` they are
`type_status: not_type_checked_v0`, `type_text: null`, and `type_source: null`.
The task's declared `Int` result is not expression-type proof.

## Honesty Rules

- `hum core-lower` must not execute code.
- It must not emit Hum IR, bytecode, backend IR, native code, or generated source.
- It must not claim independent type checking, effect checking, ownership
  checking, optimization, memory safety, or profile enforcement.
- It may emit unverified Core Hum operation rows, source spans, signature facts,
  compact expression preview roots, the bounded parser-owned ordered add tree,
  selected checked return-expression type slots, and explicit blockers.
- It must block future execution or IR claims when source, resolver, type, body,
  or brace blockers exist.
- It must stay in sync with `hum.core_contract.v0`, `hum.core_preview.v0`,
  `hum.core_verify.v0`, `hum.ir_readiness.v0`, `hum capabilities --format json`, and `hum version
  --format json`.

## Privacy And Dependency Rules

The command is local-first:

- no network
- no cloud
- no telemetry
- no solver dependency
- no backend dependency
- no generated code execution

## Non-Goals For V0

V0 does not produce executable Core Hum, Hum IR, bytecode, machine code, backend
adapter input, proof artifacts, optimized code, executable behavior, broad type
inference, effect facts, ownership facts, profile enforcement, or safety claims.
The emitted non-goals include `no Hum IR emission`.
It is the first durable source-to-Core artifact boundary future passes must
verify before Hum can honestly run code.

## Session AL Callable Nodes

The lowered report preserves the same callable type/value/application IDs,
resolver relationship, result type, failure root, and closed-row identity. It
does not lower closures, captures, handlers, or open rows.

# Hum Core Preview Schema

Date: 2026-07-07

Current schema: `hum.core_preview.v0`

## Purpose

`hum core-preview` emits the first Core Hum-shaped view of parsed `does:` body
lines. It sits between the partial body grammar in `hum ir-readiness` and future
true Core Hum lowering.

This command is intentionally not an interpreter, not a type checker, not an
effect checker, not Hum IR, and not a backend. It reports conservative candidate
operations, block previews, expression preview atoms, AST previews, operators,
and blockers so humans, agents, and future compiler passes can see what the
current bootstrap can map toward Core Hum without pretending the body has
executable meaning.

## Command

```powershell
hum core-preview [--format human|json] [--timings] <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- core-preview examples/reference_surface.hum
cargo run -- core-preview --format json examples/reference_surface.hum
```

The human output is for terminals. The JSON output is for agents, CI wrappers,
compiler-roadmap checks, and future Core Hum lowering/verifier work.

## Top-Level Shape

```json
{
  "schema": "hum.core_preview.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "milestone": "0 semantic graph",
  "core_contract_schema": "hum.core_contract.v0",
  "summary": {},
  "core_candidates": [],
  "non_goals_v0": []
}
```

## Fields

- `schema`: schema name, currently `hum.core_preview.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: maturity label such as `pre-alpha`
- `milestone`: current implementation milestone
- `core_contract_schema`: Core Hum contract this report targets
- `summary`: file, item, task, test, candidate, execution-ready, diagnostic,
  statement-preview, block-preview, expression-preview, expression-atom,
  expression-AST-node, and compound-expression preview counts
- `core_candidates`: task or test bodies with a `does:` section
- `non_goals_v0`: claims this command must not make

## Candidate Shape

Each `core_candidates` entry has:

- `id`: stable-ish source-derived preview row ID
- `kind`: Hum item kind, currently usually `task` or `test`
- `name`: source item name
- `source_span`: file, line, and column for the source item
- `status`: aggregate preview status
- `core_contract_schema`: owning Core Hum contract schema
- `body_status`: partial body grammar aggregate status
- `grammar_status`: body grammar maturity, currently `partial_v0`
- `summary`: meaningful body lines, statement status counts, expression preview
  counts, block preview counts, expression atom counts, expression AST node counts,
  and compound expression preview counts
- `source_sections`: sections seen on the source item
- `block_preview`: conservative nested block tree over statement indexes
- `statements`: one row per meaningful `does:` body line

Current candidate statuses:

- `lowerable_preview_v0`: all meaningful body lines map to Core Hum candidate
  operations, but are not typed or executable
- `contextual_preview_v0`: the body contains lines that need surrounding context,
  such as record fields or test expectations
- `preview_with_blockers`: at least one meaningful line is blocked before Core
  lowering
- `empty_body`: the `does:` section has no meaningful lines
- `blocked_by_source_errors`: source diagnostics include errors

## Statement Shape

Each `statements` row has:

- `source_span`: source file, line, and column
- `text`: original meaningful body line text
- `source_kind`: partial body grammar kind from `hum.ir_readiness.v0`
- `source_status`: source grammar status, such as `recognized_v0` or
  `unsupported_v0`
- `core_operation`: candidate Core Hum operation family
- `status`: statement preview status
- `expression_kind`: coarse expression shape from the body grammar when available
- `expression_preview`: structured expression preview, or `null` when the row has
  no standalone expression text
- `reason`: optional blocker or context reason

## Block Preview Shape

`block_preview` is a syntax/control-flow preview only. It groups the flat
`statements` rows by explicit `{` and `}` structure so future Core lowering,
type-checking, effect-checking, and diagnostics can consume block shape without
re-parsing source text.

This example is abbreviated to show the shape without repeating every nested field:

```json
{
  "status": "block_preview_v0",
  "block_count": 3,
  "max_depth": 2,
  "unmatched_closes": 0,
  "unclosed_blocks": 0,
  "root": {
    "node_kind": "block",
    "id": "hum_core_preview_task_find_active_session_3_1_block_root",
    "block_kind": "root",
    "status": "block_preview_v0",
    "header_statement_index": null,
    "closing_statement_index": null,
    "reason": null,
    "children": [
      { "node_kind": "block", "block_kind": "for_each", "children": [] },
      {
        "node_kind": "statement_ref",
        "statement_index": 5,
        "core_operation": "fail",
        "status": "lowerable_preview_v0",
        "reason": null
      }
    ]
  }
}
```

Block preview fields:

- `status`: aggregate block status, currently `block_preview_v0` or
  `block_preview_with_mismatch_v0`
- `block_count`: root plus nested block count
- `max_depth`: maximum nested block depth, with root at depth `0`
- `unmatched_closes`: number of `}` rows found at root level
- `unclosed_blocks`: number of opened blocks missing a closing `}`
- `root`: root block node

Block node fields:

- `node_kind`: `block` or `statement_ref`
- `id`: source-derived preview block identifier for block nodes
- `block_kind`: `root`, `if_statement`, `while_loop`, `for_each`, `for_index`,
  `loop`, or `record_construction`
- `status`: `block_preview_v0`, `contextual_block_preview_v0`,
  `unclosed_block_preview_v0`, or `block_preview_with_mismatch_v0` on the root
  node when aggregate brace mismatches are present
- `header_statement_index`: index into `statements` for the opening statement, or
  `null` for root
- `closing_statement_index`: index into `statements` for the closing `}`, or
  `null` when root or unclosed
- `children`: nested block nodes or statement refs
- `reason`: optional context or mismatch reason

Block preview honesty rules:

- It does not lower blocks to executable Core Hum.
- It does not type-check branch conditions, loop bounds, mutation, returns, or
  failures.
- It does not prove reachability, termination, allocation behavior, or effects.
- It only reflects explicit brace structure recognized by the partial body
  grammar.

Statement statuses:

- `lowerable_preview_v0`: can be represented as a Core Hum operation candidate
- `contextual_preview_v0`: recognized, but not a standalone Core Hum operation
- `blocked_v0`: blocked before Core Hum lowering

Examples of Core Hum candidate operations:

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
- `record_construction_field`
- `record_construction_close`

Named V0 blockers include:

- `store_write_deferred` with `surface_save_requires_store_lowering`
- `unknown` with `not_in_core_preview_v0`

## Expression Preview Shape

`expression_preview` is a syntax preview only. It does not type-check, evaluate,
resolve names, prove effects, or choose overloads.

```json
{
  "text": "title is empty",
  "kind": "condition_or_surface_binary",
  "status": "compound_preview_v0",
  "atoms": [
    { "text": "title", "kind": "name", "status": "atom_preview_v0" },
    { "text": "empty", "kind": "name", "status": "atom_preview_v0" }
  ],
  "operators": ["is"],
  "ast": {
    "status": "ast_preview_v0",
    "type_status": "not_type_checked_v0",
    "effect_status": "not_effect_checked_v0",
    "node_count": 3,
    "root": {
      "id": "expr_root_title_is_empty",
      "form": "binary_operation_candidate",
      "text": "title is empty",
      "operator": "is",
      "type_status": "not_type_checked_v0",
      "effect_status": "not_effect_checked_v0",
      "reason": null,
      "children": [
        {
          "id": "expr_atom_0_title",
          "form": "name_ref",
          "text": "title",
          "operator": null,
          "type_status": "not_type_checked_v0",
          "effect_status": "not_effect_checked_v0",
          "reason": null,
          "children": []
        },
        {
          "id": "expr_atom_1_empty",
          "form": "name_ref",
          "text": "empty",
          "operator": null,
          "type_status": "not_type_checked_v0",
          "effect_status": "not_effect_checked_v0",
          "reason": null,
          "children": []
        }
      ]
    }
  },
  "reason": null
}
```

Expression fields:

- `text`: expression text extracted from the statement
- `kind`: preview family such as `name`, `bool_literal`, `int_literal`,
  `text_literal`, `path_or_field_read`, `call_like`, `record_literal_start`,
  `binary_expression`, `condition_or_surface_binary`, or `surface_text`
- `status`: expression preview maturity
- `atoms`: syntax atoms found inside the expression preview
- `operators`: recognized operator families such as `returns`, `fails_with`,
  `is`, `does`, arithmetic operators, comparisons, `and`, or `or`
- `ast`: Core expression AST preview with explicit unchecked type/effect slots
- `reason`: optional context or limitation reason

Expression statuses:

- `atom_preview_v0`: single syntax atom, with no type or name resolution claim
- `compound_preview_v0`: split into preview atoms and operator families, with no
  precedence, type, or executable semantics claim
- `contextual_preview_v0`: needs surrounding statement or block context, such as a
  record literal start
- `surface_phrase_preview_v0`: human-oriented surface phrase preserved as text
  because V0 cannot honestly lower it yet

Atom fields:

- `text`: atom text
- `kind`: preview kind such as `name`, `callee_name`, `bool_literal`,
  `int_literal`, `text_literal`, `path_or_field_read`, `call_like`, or
  `surface_text`
- `status`: atom preview status

## Expression AST Preview Shape

The expression AST preview is a syntax tree boundary, not a checked Core Hum
expression. It exists so later passes can fill in type, effect, name-resolution,
and lowering facts without changing the JSON shape again.

AST fields:

- `status`: `ast_preview_v0`, `contextual_ast_preview_v0`, or
  `surface_ast_preview_v0`
- `type_status`: currently always `not_type_checked_v0`
- `effect_status`: currently always `not_effect_checked_v0`
- `node_count`: total root plus child node count
- `root`: root `CoreExpressionNode` preview

Node fields:

- `id`: source-derived expression node identifier, stable enough for preview use
- `form`: node family such as `name_ref`, `path_or_field_read`, `call_candidate`,
  `record_construction_candidate`, `binary_operation_candidate`, or
  `surface_phrase`
- `text`: source expression slice represented by this node
- `operator`: operator family for compound nodes, or `null`
- `type_status`: currently always `not_type_checked_v0`
- `effect_status`: currently always `not_effect_checked_v0`
- `reason`: optional context or limitation reason
- `children`: child expression nodes

AST honesty rules:

- The AST preview does not define precedence beyond the recognized V0 binary
  candidate shape.
- It does not resolve names or fields.
- It does not type-check calls, literals, paths, records, or operators.
- It does not infer effects or prove purity.
- It is allowed to preserve surface phrases as `surface_phrase` nodes when V0
  cannot honestly lower the text yet.

## Honesty Rules

- `hum core-preview` must not execute code.
- It must not claim type checking, effect checking, ownership checking,
  optimization, or backend readiness.
- It must not emit Hum IR.
- It may report Core Hum candidate operation families, source spans, coarse
  expression kinds, block previews, expression preview atoms, expression AST
  previews, operators, and explicit blockers.
- It must stay in sync with `hum.core_contract.v0`, `hum.ir_readiness.v0`, `hum
  capabilities --format json`, and `hum version --format json`.

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
adapter input, proof artifacts, optimized code, or executable behavior. It is a
conservative preview of what the next true lowering pass must make precise.
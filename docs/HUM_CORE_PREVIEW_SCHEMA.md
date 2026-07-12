# Hum Core Preview Schema

Date: 2026-07-07

Current schema: `hum.core_preview.v0`

## Purpose

`hum core-preview` emits the first Core Hum-shaped view of parsed `does:` body
lines. It sits between the partial body grammar in `hum ir-readiness` and future
true Core Hum lowering.

This command is intentionally not an interpreter, not an independent type
checker, not an effect checker, not Hum IR, and not a backend. It reports
conservative candidate operations, candidate-local name previews, block previews,
expression preview atoms, AST previews, operators, and blockers so humans,
agents, and future compiler passes can see what the current bootstrap can map
toward Core Hum without pretending the body has executable meaning. When
`hum.type_check.v0` has already checked a trivial task return expression,
`core-preview` may copy that checked fact into the expression AST type slots with
explicit provenance.

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
  "type_check_schema": "hum.type_check.v0",
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
- `type_check_schema`: checked return fact schema this report may consume for
  selected expression type slots
- `summary`: file, item, task, test, candidate, execution-ready, diagnostic,
  statement-preview, block-preview, name-preview, expression-preview,
  expression-atom, expression-AST-node, compound-expression, and typed-expression
  preview counts
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
  counts, block preview counts, name definition/reference counts, expression atom
  counts, expression AST node counts, compound expression preview counts, and
  typed expression preview counts
- `source_sections`: sections seen on the source item
- `name_status`: aggregate status for candidate-local name preview facts
- `name_preview`: conservative definition/reference preview over the candidate body
- `block_preview`: conservative nested block tree over statement indexes
- `statements`: one row per meaningful `does:` body line

Current candidate statuses:

- `lowerable_preview_v0`: all meaningful body lines map to Core Hum candidate
  operations, but are not executable; selected return expression type slots may
  reflect checked facts from `hum.type_check.v0`
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

Session W's shared typed-failure analysis maps every H0906 unsupported-`try`
fact to `core_operation: unsupported_try_expression`, `status: blocked_v0`, and
the same stable reason. Such a row is not `lowerable_preview_v0`.

Session AF adds contract rows from the shared Predicate v2 fact:
`checked_contract_predicate_v2` is lowerable preview evidence with the accepted
comparison AST, typed Bool root, operand children, and structured place atoms,
`blocked_contract_predicate_v2` preserves the exact H0704 reason, and honest
prose remains contextual `unchecked_prose_contract_v0`. Core preview does not
re-parse contract text.

## Name Preview Shape

`name_preview` is a candidate-local binding/reference preview only. It walks the
candidate in source-statement order and reports the names currently visible to
future lowering work without claiming full module, type, overload, field, or
checked name resolution.

```json
{
  "status": "name_preview_v0",
  "scope_model": "lexical_block_scope_preview_v0",
  "scope_id": "hum_core_preview_task_add_task_4_1_scope_root",
  "checked_resolver_status": "not_run_v0",
  "resolver_diagnostic_status": "preview_facts_only_v0",
  "resolver_diagnostic_count": 0,
  "scope_count": 2,
  "definition_count": 2,
  "reference_count": 3,
  "resolved_reference_count": 2,
  "unresolved_reference_count": 0,
  "external_reference_count": 1,
  "shadowed_definition_count": 0,
  "scopes": [],
  "definitions": [],
  "references": []
}
```

Name preview fields:

- `status`: `name_preview_v0`, `name_preview_with_shadowing_v0`, or
  `name_preview_with_unresolved_v0`
- `scope_model`: currently `lexical_block_scope_preview_v0`
- `scope_id`: source-derived root scope identifier for this preview candidate
- `checked_resolver_status`: currently `not_run_v0`; this command has not run
  the future checked name resolver
- `resolver_diagnostic_status`: currently `preview_facts_only_v0`; name preview
  rows are facts for later passes, not compiler diagnostics
- `resolver_diagnostic_count`: currently `0`; checked resolver diagnostics must
  come from a future resolver pass, not from preview rows
- `scope_count`: number of preview lexical scopes reported
- `definition_count`: number of preview definitions reported
- `reference_count`: number of preview references reported
- `resolved_reference_count`: references resolved to a visible candidate-local
  definition
- `unresolved_reference_count`: references not found in the candidate-local
  visible set
- `external_reference_count`: unresolved capitalized, path-root, or callee names
  reported as external/type/global preview references because V0 has no module or
  global resolver yet
- `shadowed_definition_count`: definitions that shadow a previously visible name
- `scopes`: preview lexical scope rows
- `definitions`: preview definition rows
- `references`: preview reference rows

Scope rows include:

- `id`: source-derived scope id
- `parent_scope_id`: parent lexical scope id, or `null` for the root scope
- `scope_kind`: `root`, `if_statement`, `while_loop`, `for_each`, `for_index`, or
  `loop`
- `block_id`: matching block preview id when the scope is tied to a block
- `header_statement_index`: opening statement index for block scopes, or `null`
  for root
- `closing_statement_index`: closing statement index when present

Definition rows include:

- `id`: source-derived definition id
- `name`: source name text
- `normalized_name`: snake-style comparison key used by this preview
- `definition_kind`: `parameter`, `declared_use`, `declared_change`,
  `let_binding`, `mutable_binding`, `for_each_binding`, or `for_index_binding`
- `scope_id`: owning preview scope
- `statement_index`: index into `statements`, or `null` for item header or
  section declarations
- `source_span`: source location for the definition
- `status`: `defined_preview_v0`, `shadowed_definition_preview_v0`, or
  `duplicate_declaration_preview_v0`
- `shadowed_definition_id`: previous visible definition id when shadowing or
  duplicate declaration is detected
- `reason`: optional honesty reason such as `definition_shadows_visible_name` or
  `declaration_already_available_in_candidate_scope`

Reference rows include:

- `id`: source-derived reference id
- `name`: source name text
- `normalized_name`: snake-style comparison key used by this preview
- `reference_kind`: `name_ref`, `path_root_ref`, `callee_ref`, or
  `mutation_target`
- `scope_id`: owning preview scope
- `statement_index`: index into `statements`
- `source_span`: source location for the containing statement
- `resolution_status`: `resolved_preview_v0`, `unresolved_preview_v0`, or
  `external_reference_preview_v0`
- `resolved_definition_id`: visible definition id when resolved locally
- `reason`: optional honesty reason such as `name_not_in_candidate_scope` or
  `global_or_type_name_resolution_not_implemented`

Name preview honesty rules:

- `resolution_status` values are preview facts only. `unresolved_preview_v0` is
  not a compiler error until a future checked resolver emits a diagnostic.
- It previews lexical scopes for explicit control-flow blocks: `if`, `while`,
  `for each`, `for index`, and `loop`.
- It does not treat record-construction braces as lexical scopes.
- It does not resolve modules, imports, globals, type names, enum variants,
  overloads, fields, stores, or traits.
- It does not prove definite assignment, liveness, ownership, effects, or
  mutation safety.
- It reports capitalized unresolved names as external/type/global preview facts
  so later resolvers have explicit work to do.
- It consumes names already surfaced by the expression preview; it does not
  re-parse arbitrary source text.

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

`expression_preview` is primarily a syntax preview. It does not independently
type-check, evaluate, resolve names by itself, prove effects, or choose
overloads. Candidate-level `name_preview` consumes expression atoms separately to
report conservative local binding/reference facts. V0 may also expose checked
return-expression type slots copied from `hum.type_check.v0`; these are
provenance-bearing facts, not broad expression inference.

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
    "type_text": null,
    "type_source": null,
    "effect_status": "not_effect_checked_v0",
    "node_count": 3,
    "root": {
      "id": "expr_root_title_is_empty",
      "form": "binary_operation_candidate",
      "text": "title is empty",
      "operator": "is",
      "type_status": "not_type_checked_v0",
      "type_text": null,
      "type_source": null,
      "effect_status": "not_effect_checked_v0",
      "reason": null,
      "children": [
        {
          "id": "expr_atom_0_title",
          "form": "name_ref",
          "text": "title",
          "operator": null,
          "type_status": "not_type_checked_v0",
          "type_text": null,
          "type_source": null,
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
          "type_text": null,
          "type_source": null,
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
- `ast`: Core expression AST preview with explicit type/effect slots; type slots
  are unchecked unless populated from checked return facts
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
expression. It exists so later passes can fill in type, effect, checked
name-resolution, and lowering facts without changing the JSON shape again.

AST fields:

- `status`: `ast_preview_v0`, `contextual_ast_preview_v0`, or
  `surface_ast_preview_v0`
- `type_status`: `not_type_checked_v0`, `checked_trivial_return_type_v0`, or
  `checked_trivial_return_type_mismatch_v0`
- `type_text`: checked expression type text when available, otherwise `null`
- `type_source`: source of the checked type fact when available, otherwise `null`
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
- `type_status`: `not_type_checked_v0`, `checked_trivial_return_type_v0`, or
  `checked_trivial_return_type_mismatch_v0`
- `type_text`: checked node type text when available, otherwise `null`
- `type_source`: source of the checked node type fact when available, otherwise
  `null`
- `effect_status`: currently always `not_effect_checked_v0`
- `reason`: optional context or limitation reason
- `children`: child expression nodes

AST honesty rules:

- The AST preview does not define precedence beyond the recognized V0 binary
  candidate shape.
- It does not resolve names or fields; candidate `name_preview` reports separate
  candidate-local preview facts.
- It does not independently type-check calls, literals, paths, records, or
  operators.
- It may copy `hum.type_check.v0` checked return facts onto the expression root
  when statement text and source span match.
- It does not infer effects or prove purity.
- It is allowed to preserve surface phrases as `surface_phrase` nodes when V0
  cannot honestly lower the text yet.

## Honesty Rules

- `hum core-preview` must not execute code.
- It must not claim independent type checking, broad expression type inference,
  effect checking, ownership checking, optimization, or backend readiness.
- It must not emit Hum IR.
- It may report Core Hum candidate operation families, source spans, coarse
  expression kinds, candidate-local name previews, block previews, expression
  preview atoms, expression AST previews, selected checked return-expression type
  slots, operators, and explicit blockers.
- It must not claim module, global, type, overload, field, or lexical block name
  resolution.
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
adapter input, proof artifacts, optimized code, executable behavior, independent
type checking, broad expression type inference, module or global name resolution,
or checked name resolution. It is a conservative preview of what the next true
lowering pass must make precise.

## Session AL Callable Nodes

`callable_facts.core_nodes` previews stable `callable_type`, `callable_value`,
and `callable_application` nodes with the shared closed-row identity.

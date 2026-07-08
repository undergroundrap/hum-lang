# Hum Syntax Surface Schema

Date: 2026-07-07

Current schema: `hum.syntax_surface.v0`

## Purpose

The syntax surface is Hum's editor-neutral metadata for the current source
shape. It gives tools the shared names, orders, hover copy, semantic-token
legend, and generated TextMate grammar inputs they need without asking every
editor plugin to rediscover Hum syntax.

It is not an AST, a final grammar, or a source-specific token stream. For parsed
source facts, spans, symbols, folds, obligations, and diagnostics, use
`hum graph` and `hum check --format json`.

## Commands

```powershell
hum syntax
hum syntax --format textmate
```

During the Rust bootstrap:

```powershell
cargo run -- syntax
cargo run -- syntax --format textmate
```

`hum syntax` emits JSON. `hum syntax --format textmate` emits a generated
TextMate grammar built from the same syntax surface.

## Top-Level Shape

```json
{
  "schema": "hum.syntax_surface.v0",
  "source_extension": ".hum",
  "module_keyword": "module",
  "item_kinds": [],
  "identifiers": { "value": "[a-z_][a-z0-9_]*", "type": "[A-Z][A-Za-z0-9]*" },
  "comment_prefixes": [],
  "test_modifiers": [],
  "parameter_permission_modes": ["borrow", "change", "consume"],
  "section_headers": {},
  "semantic_tokens": {}
}
```

## Top-Level Fields

- `schema`: schema name, currently `hum.syntax_surface.v0`
- `source_extension`: canonical source extension, currently `.hum`
- `module_keyword`: keyword used for module declarations
- `item_kinds`: top-level and nested declaration kinds recognized by the
  Milestone 0 parser
- `identifiers`: current value-name and type-name regex patterns from decision 0012
- `comment_prefixes`: line comment prefixes recognized inside sections
- `test_modifiers`: recognized leading modifiers for `test` declarations
- `parameter_permission_modes`: parameter permission words recognized in callable signatures; unmarked parameters default to `borrow`
- `section_headers`: section order, obligation mapping, and hover catalog
- `semantic_tokens`: shared token legend and Hum role mapping for editor and LSP
  adapters

## Section Headers

`section_headers` has this shape:

```json
{
  "task_order": ["why", "uses", "changes", "needs", "ensures", "does"],
  "test_order": ["why", "uses", "needs", "covers", "does"],
  "task_obligations": [
    { "name": "needs", "kind": "precondition", "blame": "caller" }
  ],
  "evidence_obligations": [
    { "name": "protects", "kind": "security_property", "blame": "security_boundary" }
  ],
  "section_catalog": [
    {
      "name": "why",
      "applies_to": ["app", "type", "store", "task", "test"],
      "hover": "Explains why this item exists and what value it should provide."
    }
  ]
}
```

`task_order` and `test_order` are canonical-order hints for diagnostics,
formatting, docs, and editor organization. They do not mean every listed section
is mandatory.

`task_obligations` maps task sections to generated test-obligation kinds and a
first blame owner. Current kinds are `precondition`, `postcondition`,
`edge_case`, and `declared_test`. Current blame values are `caller`, `callee`,
and `evidence`.

`evidence_obligations` maps task sections to generated evidence-obligation
kinds and a first blame owner. Current kinds are `security_property` for
`protects:` and `trust_boundary` for `trusts:`. Current blame values are
`security_boundary` and `trust_boundary`.

`section_catalog` is the source of truth for section hover text. Adapters should
use it instead of copying prose into each plugin.

## Semantic Tokens

`semantic_tokens` has this shape:

```json
{
  "token_types": ["namespace", "type", "function", "keyword"],
  "token_modifiers": ["declaration", "definition", "documentation"],
  "rules": [
    {
      "source": "section:uses",
      "token_type": "keyword",
      "modifiers": ["documentation", "readonly"],
      "hum_role": "effect_read"
    }
  ]
}
```

`token_types` and `token_modifiers` form the current LSP semantic-token legend.
`rules` maps Hum-specific syntax sources to that legend and gives each source a
stable `hum_role` for adapters, docs, and future richer views.

Milestone 0 emits the legend and rules, not per-file semantic token ranges.
Precise ranges should wait until the parser and semantic graph can report them
honestly.

## TextMate Grammar

`hum syntax --format textmate` emits the generated baseline grammar used by
[../editors/textmate/hum.tmLanguage.json](../editors/textmate/hum.tmLanguage.json).
Refresh the snapshot with:

```powershell
.\tools\update_textmate_grammar.ps1
```

Do not hand-edit the generated TextMate snapshot. Change `hum syntax` first, then
regenerate.

## Adapter Rules

- Use `hum.syntax_surface.v0` for global syntax metadata, hover copy, grammar
  generation, and semantic-token legend setup.
- Use `hum graph` for parsed source facts, source spans, document symbols,
  folding ranges, and section line facts.
- Use `hum check --format json` for source-backed diagnostics.
- Ignore unknown top-level fields and unknown semantic token rules.
- Treat array order as meaningful where the field name says order, such as
  `task_order` and `test_order`.
- Do not infer source-specific semantic ranges from this schema alone.
- Keep plugins thin: they may adapt this schema to editor APIs, but they should
  not fork Hum syntax definitions.

## Non-Goals For V0

V0 does not promise:

- a complete grammar
- file-specific token ranges
- AST nodes
- final item or section lists
- stable JSON formatting
- theme colors
- executable semantics

The surface exists so editors and tools can start from one honest source while
Hum's parser and compiler mature.

# Hum LSP Capability Matrix

Date: 2026-07-07

## Purpose

Hum editor support should come from first-party language facts, not from every
editor plugin rediscovering the language. This matrix records which Language
Server Protocol capabilities are ready to adapt today, which are planned, and
which must wait for more compiler truth.

The rule is:

```text
hum source -> first-party CLI/schema -> hum lsp -> thin editor adapter
```

VS Code, Cursor, Visual Studio, JetBrains IDEs, Eclipse, Neovim, Helix, Zed,
Sublime Text, and future notebook tools should consume the same Hum facts.

## Status Vocabulary

- Current: the Rust bootstrap already emits the source facts.
- Adapter-ready: the source facts are current, but `hum lsp` or editor glue must
  map them to protocol shapes.
- Planned: the capability is part of the near-term LSP contract, but the source
  facts are not complete enough yet.
- Deferred: Hum needs executable semantics, package graphs, precise token
  ranges, or debugger/runtime work first.

## Capability Matrix

| Capability | Status | Hum source of truth | Adapter behavior | Notes |
| --- | --- | --- | --- | --- |
| Toolchain capability discovery | Current | `hum capabilities --format json`, `hum.capabilities.v0` | Confirm schema and adapter-surface support before enabling editor features. | This is toolchain discovery, not runtime authority. |
| Diagnostics | Adapter-ready | `hum check --format json`, `hum.check.v0` | Publish LSP diagnostics from source-backed spans. | Use `code`, `title`, `severity`, `message`, `span`, and `help`; preserve stable codes such as `H0201`. |
| Diagnostic explanations | Adapter-ready | `hum explain <H####> --format json`, `hum.diagnostic_explain.v0`; `hum diagnostics --format json`, `hum.diagnostic_catalog.v0` | Show detail panels, quick-help text, or command results without reparsing source. | Works offline and should not read project files. |
| Document symbols | Adapter-ready | `hum graph`, `hum.semantic_graph.v0`, file `symbols` | Map top-level and nested symbols to `textDocument/documentSymbol`. | Current symbols cover apps, tasks, tests, types, stores, and type fields. |
| Folding ranges | Adapter-ready | `hum graph`, file `folding_ranges` | Map intent-section ranges to `textDocument/foldingRange`. | Current ranges cover sections only; item-body ranges wait for honest item end spans. |
| Section hover | Adapter-ready | `hum syntax`, `hum.syntax_surface.v0`, `section_catalog` | Explain section keywords such as `why:`, `does:`, `needs:`, `ensures:`, `cost:`, and `protects:`. | See [SYNTAX_SURFACE_SCHEMA.md](SYNTAX_SURFACE_SCHEMA.md); this should be identical across editors. |
| Declared-name hover | Planned | `hum graph` item, field, param, and section facts | Show conservative hover on declaration lines first. | Full token-range hover waits for precise symbol references and end ranges. |
| Semantic-token legend | Current | `hum syntax`, `semantic_tokens` | Use the shared legend for client themes and adapter mapping. | See [SYNTAX_SURFACE_SCHEMA.md](SYNTAX_SURFACE_SCHEMA.md); actual semantic-token ranges are deferred until source ranges are precise. |
| TextMate highlighting | Current | `hum syntax --format textmate`, `editors/textmate/hum.tmLanguage.json` | Provide basic highlighting for editors before LSP is installed. | Regenerate with `tools/update_textmate_grammar.ps1`; do not hand-edit the generated grammar. |
| Editor recovery fixtures | Current | `fixtures/editor`, `tools/check_editor_fixtures.ps1` | Keep diagnostics and graph generation useful on broken or half-written source. | Editors live in invalid code, so recovery fixtures are part of the contract. |
| Formatting | Planned | `humfmt` | Provide `textDocument/formatting` and format-on-save once trivia handling is lossless. | Do not ship a formatter that rewrites user intent or comments unpredictably. |
| Code actions | Planned | `hum.check.v0`, diagnostic catalog, semantic graph facts | Offer repairs for missing or malformed `uses:`, `changes:`, `why:`, `cost:`, `protects:`, and related sections. | Each action should cite the diagnostic code it repairs. |
| Go to definition | Deferred | Future symbol table and reference graph | Resolve references to definitions. | Needs exact reference spans and module/package identity. |
| Find references | Deferred | Future symbol table, reference graph, and Nectar workspace graph | Enumerate symbol uses across files and packages. | Must avoid regex-only reference guesses. |
| Rename | Deferred | Future symbol table, reference graph, and validation pass | Rename only when all affected facts remain valid. | Needs graph-backed conflict checks and previewable edits. |
| Workspace symbols | Deferred | Future Nectar workspace and package graph | Provide cross-file and cross-package symbol search. | Wait until package/module semantics stabilize. |
| Code lens | Deferred | Future test/proof/benchmark/evidence facts | Show generated tests, proof status, benchmark status, and evidence links. | Depends on alpha evidence bundle work. |
| Debug adapter bridge | Deferred | Future `hum debug`, core lowering, effect reports, and source maps | Pair LSP source facts with Debug Adapter Protocol sessions. | LSP should not invent runtime facts. |
| Jupyter kernel | Deferred | Future safe execution model and profile enforcement | Run notebook cells only when execution is reproducible and authority is explicit. | Notebooks wait until Hum can execute safely. |

## Adapter Rules

- Query `hum capabilities --format json` before enabling optional features.
- Treat the Hum CLI output as the source of truth.
- Do not reimplement parsing, diagnostics, type rules, formatting, or graph
  logic inside editor adapters.
- Convert protocol details at the boundary. Hum graph spans currently use
  one-based line and column numbers; LSP positions are zero-based.
- Preserve stable diagnostic codes in editor UI and code actions.
- Run only trusted local Hum binaries or signed embedded Hum binaries.
- Do not run arbitrary project scripts, hidden network calls, or telemetry.
- Keep editor-local state, machine paths, caches, and user settings out of the
  core language repo.
- Prefer graceful degradation: syntax highlighting should still work without
  LSP, diagnostics should still work without semantic tokens, and the LSP should
  still answer basic requests when source is incomplete.

## First Adapter Sequence

1. Ship generated TextMate grammar for baseline `.hum` highlighting.
2. Build `hum lsp` around diagnostics, document symbols, folding ranges, section
   hover, and the semantic-token legend.
3. Wire VS Code and Cursor first because they reach many users quickly and can
   reuse the TextMate grammar.
4. Document Neovim, Helix, Zed, Sublime Text, and other LSP clients as thin
   configuration examples.
5. Add Visual Studio, JetBrains, and Eclipse adapter shells only when `hum lsp`
   is stable enough for enterprise install trust.
6. Add Jupyter integration only after safe execution and profile enforcement
   exist.

## Non-Goals

- No editor-specific parser.
- No regex-only semantic navigation.
- No formatter until trivia and comments are preserved.
- No notebook kernel before safe execution.
- No plugin behavior that changes source, runs scripts, or leaves the workspace
  without explicit user action.
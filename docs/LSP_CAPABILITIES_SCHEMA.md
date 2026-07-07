# Hum LSP Capabilities Schema

Date: 2026-07-07

Current schema: `hum.lsp_capabilities.v0`

## Purpose

`hum lsp --capabilities` exposes the LSP-shaped editor capability map for the
current Hum binary without claiming that a full JSON-RPC LSP server exists yet.
It gives editor adapters a stable command to query while Hum keeps parser,
diagnostic, semantic graph, and syntax facts in first-party CLI surfaces.

This is a preview schema for adapter planning. It is not an LSP initialize
response, not a running stdio server, and not a promise that every listed method
is implemented inside `hum lsp` today.

## Command

```powershell
hum lsp --capabilities
hum lsp --capabilities --format json
```

During the Rust bootstrap:

```powershell
cargo run -- lsp --capabilities
cargo run -- lsp --capabilities --format json
```

Plain `hum lsp` intentionally fails until server mode exists.

## Top-Level Shape

```json
{
  "schema": "hum.lsp_capabilities.v0",
  "server_command": "hum lsp",
  "server_status": "planned",
  "json_rpc_server": false,
  "source_capabilities_schema": "hum.capabilities.v0",
  "capabilities": []
}
```

## Fields

- `schema`: schema name, currently `hum.lsp_capabilities.v0`
- `server_command`: command reserved for the future LSP server
- `server_status`: current server maturity, currently `planned`
- `json_rpc_server`: whether `hum lsp` starts a JSON-RPC server today
- `source_capabilities_schema`: toolchain discovery schema this preview aligns
  with
- `capabilities`: method-shaped LSP feature facts

## Capability Entries

Each capability entry has:

- `method`: LSP method or notification name
- `status`: `adapter-ready`, `planned`, or `deferred`
- `source`: first-party Hum command or future tool that owns the facts
- `schema`: schema associated with the source, or `none`
- `note`: short caveat for adapter authors

Current adapter-ready methods are backed by these first-party surfaces:

- `textDocument/publishDiagnostics`: `hum check --format json`, `hum.check.v0`
- `textDocument/documentSymbol`: `hum graph`, `hum.semantic_graph.v0`
- `textDocument/foldingRange`: `hum graph`, `hum.semantic_graph.v0`
- `textDocument/hover`: `hum syntax`, `hum.syntax_surface.v0` for section
  hovers

Current planned/deferred entries include semantic-token ranges, formatting, and
workspace symbols.

## Adapter Rules

- Query `hum lsp --capabilities --format json` before assuming `hum lsp` can
  serve a method.
- If `json_rpc_server` is `false`, adapters should call the underlying CLI
  surfaces directly or wait for server mode.
- Treat `adapter-ready` as enough source facts for a thin adapter, not proof that
  first-party server mode exists.
- Ignore unknown fields and unknown methods.
- Keep LSP adapter logic thin; do not duplicate parser, diagnostic, graph, or
  syntax semantics inside an editor plugin.

## Non-Goals For V0

V0 does not promise:

- a running LSP JSON-RPC loop
- completions
- go-to-definition
- rename
- workspace-wide package semantics
- final LSP extension methods

The preview exists so editor work can begin with precise boundaries instead of a
vague `hum lsp someday` placeholder.
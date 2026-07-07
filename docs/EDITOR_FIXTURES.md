# Hum Editor Fixtures

Date: 2026-07-07

## Purpose

Editors live in unfinished files.

The fixtures in [../fixtures/editor](../fixtures/editor) keep Milestone 0 honest
about broken and half-written Hum source. They are intentionally not clean
examples. Their job is to prove that `hum graph` still emits parseable
`hum.semantic_graph.v0` JSON and stable diagnostics while a user is typing,
moving blocks, or recovering from malformed syntax.

These fixtures are for editor, LSP, Tree-sitter, formatter, agent, and debugger
work. They must stay local, offline, and non-executing.

## Fixture Contract

Run:

```powershell
.\tools\check_editor_fixtures.ps1
```

The script expects the debug compiler at `target/debug/hum.exe` on Windows or
`target/debug/hum` elsewhere. `tools/check_all.ps1` builds that compiler before
running the fixture check.

Each fixture must satisfy three rules:

- `hum graph <fixture>` emits valid JSON even when diagnostics are errors.
- The graph schema is `hum.semantic_graph.v0`.
- The graph diagnostics include the expected stable code.

Current fixtures:

| Fixture | Expected code | Why it exists |
| --- | --- | --- |
| `mid_edit_missing_does.hum` | `H0105` | A task exists but a required intent block is not written yet. |
| `incomplete_task_header.hum` | `H0007` | A callable header is missing a close parenthesis while the user is typing. |
| `missing_close_brace.hum` | `H0004` | A file ends before an item block closes. |
| `malformed_nested_item.hum` | `H0003` | A nested item header is present but not yet opened. |
| `orphan_body_line.hum` | `H0001` | A section body is temporarily stranded at the top level. |

## Rules For New Fixtures

- Keep fixtures small and source-readable.
- Do not add code that would execute if a future tool gains execution support.
- Prefer one primary expected diagnostic per fixture.
- Add a fixture before claiming editor recovery for a new syntax shape.
- Update this document and `tools/check_editor_fixtures.ps1` together.
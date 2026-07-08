# Hum Agent Instructions

This repo is the Hum language design seed and Milestone 0 Rust bootstrap.

## Operating Rules (BDFL-accepted 2026-07-08)

These rules override any older habit in this file or in session memory. The
active work order is `WORKORDER.md` at the repo root; execute it top to bottom
before proposing new work.

1. Definition of done: a session's deliverable is a program that runs, a check
   that fires on a real mistake, or a decision record that kills alternatives.
   A session that ends with only new prose, a new schema, or a new report
   surface is a failed session.
2. Doc discipline: the original moratorium expired when `hum run` shipped
   (2026-07-08). The durable rule: new `docs/*.md` files, `hum.*.v0`
   schemas, CLI subcommands, and pipeline gates require an explicit mandate
   in the active WORKORDER.md. Research snapshots in `docs/research/` and
   decision records in `docs/decisions/` are always allowed. Editing
   existing docs to stay honest is allowed and required.
3. Decisions over deferrals: when design options exist, pick one, write the
   decision record, and state what dies. Do not park choices in "future work"
   sections. If a decision is genuinely BDFL-level, ask the BDFL directly with
   a recommendation.
4. Write Hum, not just Rust: real `.hum` programs are design instruments. When
   the work order asks for example programs, write them by hand and record
   what they revealed about the design.
5. Vertical slice over horizontal layers: prefer one thin parse -> check ->
   run path for a small subset over another full-width pass that covers
   everything and executes nothing.
6. Adversarial pass: when asked to review the design, argue against it in
   earnest; do not volunteer praise structure.

## Ground Truth

- Start with `docs/ARCHITECTURE.md` before changing design direction.
- Preserve Milestone 0 as local, offline-first, non-executing, and safe on the maker's machine.
- Major language changes need semantics, diagnostics, graph facts, tooling impact, profile impact, verification story, performance story, and pedagogy story.
- Keep Windows-first proof behind explicit platform capability boundaries so the design stays portable.

## Editing Rules

- Before editing, inspect `git status --short` and preserve existing user changes.
- Prefer `apply_patch` for hand edits.
- If `apply_patch` fails in the Windows sandbox, use a guarded non-interactive PowerShell writer only inside the repo root, and write with `[System.IO.File]::WriteAllText(..., (New-Object System.Text.UTF8Encoding($false)))`.
- Do not use long interactive PowerShell here-strings for docs; terminal line wrapping and pasted control characters can corrupt files.
- Do not use `Set-Content -Encoding UTF8` in Windows PowerShell 5.1 for repo text files because it writes a UTF-8 BOM.
- Default to ASCII unless a file already requires non-ASCII.
- Keep setup docs editor-agnostic. Prefer `.editorconfig`, `.gitattributes`, Cargo commands on `PATH`, and repo-relative paths.
- Do not commit local editor state such as `.vscode/`, `.cursor/`, `.idea/`, `.vs/`, `.fleet/`, `*.code-workspace`, or `*.iml`.
- Use forward slashes for repo-relative paths in public docs unless documenting a platform-specific boundary.

## Text Hygiene

- Repo text files must be valid UTF-8 without BOM.
- Reject terminal control characters, replacement characters, and suspicious mojibake.
- Markdown links to local files must resolve.
- After editing Markdown, Hum source, Rust source, TOML, or PowerShell tooling, run:

```powershell
.\tools\check_text_hygiene.ps1
```

For implementation details, see `docs/TEXT_HYGIENE_WORKFLOW.md`.

- Before an initial commit or public snapshot, run:

```powershell
.\tools\check_public_readiness.ps1
```
- Before a local commit, public snapshot, release-style handoff, or tag
  decision, prefer the full preflight:

```powershell
.\tools\check_all.ps1
```

It wraps Rust checks, fixture coverage, JSON parsing, generated grammar drift
detection, text hygiene, public readiness, and release readiness.

- Never create or push remote tags without explicit user approval.

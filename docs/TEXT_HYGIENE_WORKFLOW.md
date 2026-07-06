# Text Hygiene Workflow

Date: 2026-07-06

## Purpose

Hum's design docs are source artifacts. They must stay machine-readable for the compiler roadmap, semantic graph work, future agents, and human review.

The repo standard is:

```text
valid UTF-8, no BOM, no mojibake, no terminal control characters, local Markdown links resolve
```

## Why This Exists

Windows editing paths can quietly create text damage:

- Windows PowerShell 5.1 `Set-Content -Encoding UTF8` writes a UTF-8 BOM.
- Interactive terminal here-strings can capture pasted line wrapping or control characters.
- Encoding mistakes can turn punctuation, symbols, or hidden BOMs into mojibake.
- Broken local links make the architecture map less useful as project ground truth.

## Required Check

Run this after editing Markdown, Hum source, Rust source, TOML, plain text, or PowerShell tooling:

```powershell
.\tools\check_text_hygiene.ps1
```

The check scans repo text files, excluding `.git` and `target`, for:

- UTF-8 BOM
- invalid UTF-8 byte sequences
- terminal or binary control characters
- Unicode replacement characters
- common mojibake marker characters produced by encoding mixups
- broken local Markdown links

## Safe Edit Fallback

Prefer `apply_patch` for hand edits. If the Windows sandbox refuses `apply_patch`, use a non-interactive PowerShell writer with explicit repo-root checks and UTF-8 without BOM:

```powershell
$RepoRoot = (Resolve-Path .).Path
$Path = Join-Path $RepoRoot 'docs\example.md'
$Resolved = [System.IO.Path]::GetFullPath($Path)
if (-not $Resolved.StartsWith($RepoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
  throw "Refusing to write outside repo root: $Resolved"
}
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($Resolved, $Content, $Utf8NoBom)
```

Avoid `Set-Content -Encoding UTF8` in Windows PowerShell 5.1 for repo text. It writes a BOM.

## Repair Routine

When the hygiene check fails:

1. Inspect the named file and line before replacing text.
2. Remove BOMs by reading the file as text and rewriting with `UTF8Encoding(false)`.
3. Fix mojibake from source context, not by blind global replacement.
4. Re-run `.\tools\check_text_hygiene.ps1`.
5. Run the normal compiler checks if source files changed.

## Codex Turn Guardrail

Repo-local agent instructions in `AGENTS.md` make this workflow part of future Codex turns. A separate global Codex skill can come later if this pattern proves useful across multiple repos; the repo-level guardrail is the first source of truth for Hum.

## Public Snapshot Check

Before an initial commit meant to become the public baseline, or before publishing the repository, run:

```powershell
.\tools\check_public_readiness.ps1
```

This check rejects local developer paths, pending repository placeholders, and git identity artifacts that do not belong in source files. Legal attribution in `LICENSE`, `NOTICE.md`, and the README license section is intentional and should not be removed by this check.
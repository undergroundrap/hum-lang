# R11 obstacle evidence — retained 2026-10-03 (Ocean-directed, Codex-reproduced)

## Root cause

`tools/test_timing_ledger.ps1` (R10-era, line 910 in the R11 worktree) defined a
test-only `Set-HumDurableText` override inside the `Test-Control27` scope:

```powershell
function Set-HumDurableText([string]$Path, [string]$Content) { [IO.File]::WriteAllText($Path, $Content, [Text.UTF8Encoding]::new($false)) }
```

The genuine production writer (`tools/run_fast_evidence.ps1:456-460`) appends LF:

```powershell
function Set-HumDurableText {
  param([string] $Path, [string] $Text)
  $Encoding = New-Object System.Text.UTF8Encoding($false, $true)
  Set-HumDurableBytes $Path $Encoding.GetBytes($Text + "`n")
}
```

The override shadowed the genuine writer for every `Set-HumDurableText` call made
by `Invoke-HumBinaryCapture` inside the extracted `Invoke-HumPreflightCapture`,
so `manifest.txt` was written WITHOUT the final LF. The production validator
(`Read-HumStrictUtf8 -RequireFinalLf`, run_fast_evidence.ps1:475, reached via
`Assert-HumCaptureComplete`) then threw:

```
Exception: /home/hatch/workspace/hum-timing-observability/tools/run_fast_evidence.ps1:475
Line |
 475 |      throw "final LF required: $Path"
      |      ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
      | final LF required:
      | /tmp/hum-timing-controls-<guid>/control27-fixtures/q-capture/manifest.txt
```

This failure is environmental to the test scope (the shadow), not a production
defect: control 12's direct `Invoke-HumBinaryCapture` call succeeds because it
runs outside the shadowed scope.

## Byte-level demonstration (scratch, no test run)

Genuine writer output for text "abc": bytes `61 62 63 0a` (LF present).
Override writer output for text "abc": bytes `61 62 63` (LF absent).

## Disposition

- Override REMOVED from `tools/test_timing_ledger.ps1` (was line 910).
- Genuine production writer used as-is; no production helper changed.
- No LF appended after the fact; validator not weakened; no quiescence data fabricated.
- Regression added in control 27: asserts the writer's definition is unchanged
  (no replacement leaked) and the produced manifest ends with LF.

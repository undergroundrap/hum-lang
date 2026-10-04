# Review R19 — Hygiene correction + Unit C runtime diagnosis

## Deliverable A: Hygiene correction (uncommitted)

**File:** `tools/test_timing_ledger.ps1`, lines 833–834

**Change:** Windows fake-pwsh fixture representation — replaced literal
`` `r`n `` PowerShell escape sequences with `$([char]13)$([char]10)`
subexpressions.

```diff
-    [IO.File]::WriteAllText($Fake1Exe, "@exit /b 42`r`n", $Utf8NoBom)
-    [IO.File]::WriteAllText($Fake2Exe, "@exit /b 43`r`n", $Utf8NoBom)
+    [IO.File]::WriteAllText($Fake1Exe, "@exit /b 42$([char]13)$([char]10)", $Utf8NoBom)
+    [IO.File]::WriteAllText($Fake2Exe, "@exit /b 43$([char]13)$([char]10)", $Utf8NoBom)
```

**Preserved:**
- Generated `.cmd` bytes: byte-identical (`40-65-78-69-74-20-2F-62-20-34-32-0D-0A`)
- BOM-less, actual CRLF, exits 42/43, selection controls
- Hygiene validator untouched (not weakened)

**Verification:**
- `check_text_hygiene.ps1`: passed (659 files)
- `check_public_readiness.ps1`: passed (659 files)
- `test_timing_ledger.ps1`: **148 passed, 0 failed** (Linux/pwsh 7.6.6)
- Control 26 (fake-pwsh fixtures): all assertions passed

**Status:** Uncommitted on `wip/timing-observability-candidate`, index empty.

## Deliverable B: Unit C runtime diagnosis (read-only)

See `unit-c-diagnosis.md` for the full read-only reconciliation of run
37177745673's Windows 3000s timeout.

**Summary:** 887 balanced pairs with no orphan START; the 98-second gap
after WO25 I07 restoration was the **untimed** Unit C shell-equivalence
child (`check_all.ps1:1718`) running `test_fast_evidence_capture.ps1`
under Windows PowerShell 5.1 via untimed `$Process.WaitForExit()`.
Smallest coverage-preserving fix: wrap the child invocations in the
timing ledger. No implementation performed.

## No scope expansion

No deadline increase, coverage cuts, new process model, Full campaign,
PR #69 advancement, CI dispatch, watcher, merge, N9, or WO31.

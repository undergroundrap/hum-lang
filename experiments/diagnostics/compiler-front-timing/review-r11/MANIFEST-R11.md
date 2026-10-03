# R11 manifest — review-only correction (2026-10-03)

Baseline: 827eec2865cf886c979de9e60e26f3e6b60154b8 (explicit SHA, never FETCH_HEAD)
Prior carrier: ea856f9503e05f3427006c3a1b57b10a445f146b (R10)

## Eight candidate files (blob SHA-1)

| File | Blob | Status |
|------|------|--------|
| .github/workflows/ci.yml | 911525192825306bf2a886de378277ac25f232de | frozen at R10 |
| tools/check_all.ps1 | 00c9a53b1dff91e0cfeafd627705b1151590f700 | frozen at R10 |
| tools/test_exact_rust_selector.ps1 | 6b7dd305ab467e984b29c05ee653a518c40d5e4f | frozen at R10 |
| tools/run_fast_evidence.ps1 | d612763f0b5d87b78bf221c21029b04d719df837 | frozen at R10 |
| tools/hum_timing_ledger.ps1 | 4abd3749508f6b98ef6edf2944b5717b8aa239f2 | frozen at R10 |
| tools/test_ci_policy.ps1 | ef44acdf75a77771f9dce1f67be18262fe29e146 | frozen at R10 |
| crates/hum-dev/src/shell.rs | 99f54498af18d1aec6a7389fa0f4937d19fd3b06 | frozen at R10 |
| tools/test_timing_ledger.ps1 | 854ecc49c207102dda73d6b194ca7af89b34b425 | **R11 corrected** |

## What changed in R11 (only tools/test_timing_ledger.ps1)

1. **Removed the R10 test-only `Set-HumDurableText` override** (was line 910).
   The override omitted the final LF that the genuine production writer appends,
   breaking `Assert-HumCaptureComplete` ("final LF required"). The genuine writer
   from dot-sourced `run_fast_evidence.ps1` is now active. See
   `obstacle-set-hum-durable-text-shadow.md` and `writer-byte-proof.txt`.

2. **Control 27 now mechanically extracts, defines, and CALLS:**
   - the Fast-tier guard (extracted, executed for set/unset);
   - Full's `Save-HumPreflightDiagnostics` (extracted, defined, CALLED via capture);
   - Full's `Invoke-HumPreflightCapture` (extracted, defined, CALLED with timeout + success children);
   - Full's exact journal finalizer (extracted, EXECUTED with real capture data).

3. **Regression added:** pins the genuine writer's definition before capture;
   asserts it is unchanged after (no replacement leaked); asserts the produced
   manifest ends with LF; asserts `Assert-HumCaptureComplete` passes.

4. **Fixed the finalizer extractor:** the YAML-stripped body has 6-space (not 16-space)
   indent; the extractor now takes the second `} else {` (for `if ($Quiescent)`)
   and extends to its closing brace.

## Evidence retained (~/workspace/r11-evidence/)

- candidate.diff (explicit 827eec28 baseline; apply-check exit 0; apply exit 0)
- reconstructed-blobs.txt (all 8 verified)
- test_timing_ledger-stdout-stderr.log (126 passed, 0 failed; exit 0)
- test_ci_policy-stdout-stderr.log (policy suite on final bytes)
- obstacle-set-hum-durable-text-shadow.md (root-cause retention)
- writer-byte-proof.txt (genuine 61 62 63 0a vs override 61 62 63)

## R10 unsupported claims — WITHDRAWN

R10's report claimed a working harness and a valid apply-check. Both are withdrawn:
- R10 never invoked `$ExtractedCapture` or `$ExtractedFinalizer` (only checked nonempty).
- R10's candidate.diff header referenced ci.yml blob f1b9b6b (belongs to base a01f4dc2);
  the exact 827eec28 baseline has ci.yml blob 1c575e87.
- R10 retained no raw apply-check/reconstruction output.
- R10's `Set-HumDurableText` override broke capture authentication (found by Codex).

R11 corrects all four. The R10 report is preserved unmodified alongside this notice;
its harness/apply claims are withdrawn.

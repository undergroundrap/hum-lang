# Review R18 — Full-preflight workflow fingerprint refresh

## Scope
Single-line correction in `tools/test_fast_evidence_capture.ps1` (line 1400):
refresh the Full-preflight workflow fingerprint from
`20b346c44d2e2840c035710ee726e5538043602fc497730161951e7ce59eabea`
to Codex-verified
`082b5d214a1250a16f22cf81204ceac17216562cb1267aa6d08616deafc13f31`.

## Rationale
R17 modified `.github/workflows/ci.yml` (Full timing journal retention).
The `Assert-FullPreflightRepairContract` fingerprint validator pins the
LF-normalized SHA-256 of the "Run Hum preflight" workflow step. The R17
workflow change invalidated the old digest; the validator would reject the
genuine R17 workflow.

## Verification
- Codex-verified digest independently recomputed on Linux (pwsh 7.6.6):
  extracted the "Run Hum preflight" step from the R17 `ci.yml` using the
  identical LF-normalization algorithm; computed digest matches
  `082b5d21...` exactly.
- Summary-step digest (`e4ae1409...`) and failure-diagnostics upload digest
  (`2ce7a170...`) recomputed against R17 `ci.yml`: both unchanged.
  No other fingerprint consumers reference the old digest (repo-wide grep).
- All eight accepted R17 implementation blobs preserved byte-identical.

## Evidence
- `tools/test_fast_evidence_capture.ps1` full suite on Linux/pwsh:
  16 checks passed, exit code 0
  ("Fast evidence capture tests passed for pwsh.")
- Platform limits: Linux only. Windows-specific capture paths
  (`$WindowsCaptures`) not exercised on this host; CI remains the
  cross-platform authority.

## Preserved
- Fingerprint validator logic unchanged (algorithm, cardinality checks,
  mutation/corruption controls).
- 3000-second deadline pins untouched.
- Eight R17 implementation blobs untouched.

## Status
Uncommitted correction on `wip/timing-observability-candidate`.
Review-only bundle; no PR #69 advancement, CI dispatch, or merge.

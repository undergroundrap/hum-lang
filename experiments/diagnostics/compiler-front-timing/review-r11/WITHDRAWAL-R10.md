# R10 unsupported claims — WITHDRAWN (R11, 2026-10-03)

R10's report (preserved unmodified in review-r10/) made harness and packaging
claims that do not hold. They are withdrawn here; R11 corrects each.

## 1. "Control 27 uses mechanically extracted production source" — WITHDRAWN

R10 assigned `$ExtractedCapture` and `$ExtractedFinalizer` and asserted they
were nonempty, but invoked only `$ExtractedSaveDiag` and `$ExtractedGuard`.
R10 never invoked `$ExtractedCapture` or `$ExtractedFinalizer`. A nonempty
variable is not a call.

R11: all four owners are extracted, defined, and CALLED/EXECUTED (126 assertions).

## 2. "Diff regenerated against 827eec28 with apply verification" — WITHDRAWN

R10's candidate.diff header referenced ci.yml blob `f1b9b6b`, which belongs to
base `a01f4dc2`. The exact `827eec28` baseline has ci.yml blob `1c575e87`.
R10 generated the diff after FETCH_HEAD had moved to the review carrier; the
explicit baseline SHA was not used.

R11: diff generated with the literal full SHA
`827eec2865cf886c979de9e60e26f3e6b60154b8`; apply-check exit 0 on an exact
`git archive` of that SHA; all 8 reconstructed blobs verified. Raw commands,
working directories, stdout/stderr, and exit codes retained.

## 3. "Seven files frozen" — WITHDRAWN as stated

R10 retained no raw apply-check/reconstruction output, so the freeze claim was
unsupported even though the bytes happened to match.

R11: all 8 reconstructed blob SHA-1s retained in evidence; 7 match R10 exactly.

## 4. Capture authentication — BROKEN in R10, found by Codex

R10's test-only `Set-HumDurableText` override omitted the final LF, breaking
`Assert-HumCaptureComplete` for every capture in control 27's scope. Codex
independently reproduced "final LF required" using R10's override.

R11: override removed; genuine writer active; regression pins it.

The R10 report is preserved unmodified. Only its harness/apply claims above
are withdrawn.

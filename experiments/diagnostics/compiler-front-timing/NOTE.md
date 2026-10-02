# Review snapshot: compiler-front timing kit

This directory is an **unaccepted diagnostic review snapshot**, not
production tooling. It is published for Codex review only. Nothing here
is wired into CI, builds anything, or judges anything.

- `measure-compiler-front-kit.ps1` — scratch-only timing kit sampling the
  four compiler-front execution families (F0 launch control, F1 exact Rust
  selectors, F2 compiler-mutation work, F3 native/CLI probes; 28 commands).
  Measures only; never repairs. Requires a separately authorized honest
  build and a dedicated scratch source copy; awaits authorized execution.
- `validate-kit-harness.ps1` — synthetic (non-Hum) harness validation for
  the kit's mechanics and finalization path.
- `validation-results.txt` — retained output of the harness validation run
  (55 checks, all passing).
- `SHA256SUMS` — SHA-256 checksums of the files in this directory.

Context: PR #69 (undergroundrap/hum-lang#69) Windows validation failed
when the compiler-front group exceeded the 3000s capture deadline.
Pins: head `a01f4dc242cd075d7aff91090280ce9c38bcc8a7`,
base `2dfe3d9c230f2d45336bbde60a526c87c6ee1c5b`.

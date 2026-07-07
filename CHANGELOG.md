# Changelog

All notable Hum repository changes are tracked here.

Hum is pre-alpha. Versions before `1.0.0` are release snapshots, not stability
promises.

## [Unreleased]

No unreleased changes yet.

## [0.0.1] - 2026-07-07

Status: pre-alpha.

### Added

- Rust bootstrap CLI with `#![forbid(unsafe_code)]` and no third-party crates.
- Milestone 0 `.hum` parser, source spans, stable diagnostics, and intent
  checks.
- `hum check`, `hum graph`, `hum test-skeletons`, `hum syntax`,
  `hum diagnostics`, `hum explain`, `hum capabilities`, `hum lsp
  --capabilities`, and `hum doctor` preview surfaces.
- Machine-readable schemas for semantic graph, diagnostics, syntax surface,
  toolchain capabilities, LSP capability preview, and setup health.
- Editor-neutral TextMate grammar generation and editor recovery fixtures.
- Portable setup, text hygiene, public-readiness, release-readiness,
  clean-checkout, tag-readiness, and release-manifest guardrails.
- Architecture, language reference, release, security, governance, setup,
  editor, stdlib, backend, platform, research, and alpha planning docs.

### Changed

- The repo now treats Cargo as the Rust bootstrap build path, not Hum's final
  identity or distribution strategy.
- Release discipline now requires SemVer, a clean tree, full preflight,
  clean-checkout smoke, and a non-publishing tag gate before annotated tags.

### Known Risks

- Hum is not executable yet and does not compile programs to native code.
- Public APIs, schemas, syntax, and docs are still pre-alpha and may change.
- `hum lsp` has only a capability preview; it is not a running JSON-RPC server.
- Release notes still require the verified commit hash printed by
  `tools/check_tag_readiness.ps1` at tag time.

### Verification

- `tools/check_all.ps1`
- `tools/check_clean_checkout.ps1`
- `tools/check_tag_readiness.ps1`

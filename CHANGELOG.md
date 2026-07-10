# Changelog

All notable Hum repository changes are tracked here.

Hum is pre-alpha. Versions before `1.0.0` are release snapshots, not stability
promises.

## [Unreleased]

### Added

- A checked Milestone 1 interpreter for the current formal Core subset,
  including arithmetic, control flow, records, lists, runtime contracts, and
  stable failure/trap exit behavior.
- Resolver, type, Core preview/lower/verify, full-type, effect, ownership,
  resource, profile, target, backend-contract, and IR-readiness report gates
  with explicit honesty locks.
- Narrow checked ownership slices for moves and parameter permissions,
  Transaction-shaped exactly-once resources, returned-view provenance, local
  field and element views with invalidation, and straight-line writable field
  aliases with H0801-H0809 diagnostics.
- Predicate v1 runtime contracts with `old(...)` pre-state capture and
  `list_len(...)` expressions.
- Explicit nominal typed-failure propagation and causal caller-root wrapping,
  including H0901-H0907 diagnostics and outer-to-root runtime source chains.
- Hand-written ownership and failure corpora plus adversarial misuse fixtures;
  the repository CI matrix is configured to run the full preflight on Windows
  and Ubuntu.

### Changed

- Core numerics are `Int` and `UInt`; the provisional `Number` type was
  removed.
- Ownership follows accepted decision 0014, with broad safety claims locked to
  the exact fixture-proven subsets.
- Runtime contract policy and explicit causal typed failure are recorded by
  accepted decisions 0015 and 0016.
- Agent governance now uses cold-start role runbooks, independent session
  review, delegated rulings with a BDFL veto, and evidence-gated bake-offs.

### Known limitations

- Hum remains pre-alpha: no native backend, first-class `Result`, failure
  recovery, general call typing, complete borrow or memory-safety proof,
  general aliases/internal references, or executable IO capability slice is
  claimed yet.

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

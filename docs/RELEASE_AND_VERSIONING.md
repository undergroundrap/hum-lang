# Release And Versioning

Date: 2026-07-06

## Purpose

Hum should treat version numbers, tags, and release evidence like product surfaces from the beginning. The project is still pre-alpha, but the release process should already feel legible to an enterprise maintainer, security reviewer, package manager, and future contributor.

## Current Version

Current repo version: `0.0.1`.

Hum is in major-version-zero development. No public API is stable yet. The version exists to practice discipline, make snapshots traceable, and avoid vague tags.

## Version Sources

The version must match in:

- `VERSION`
- `Cargo.toml` package version

Do not add a third version source unless a release tool owns synchronization across all version files.

The bootstrap CLI reports the Cargo package version and schema identity:

```powershell
hum version
hum version --format json
hum explain H0201 --format json
hum diagnostics --format json
hum capabilities --format json
hum lsp --capabilities --format json
hum doctor --format json
```

## Distribution Stance

Hum's bootstrap compiler is a Rust package because Rust is the current
implementation language. That is normal and useful for early development.

Cargo can help Rust developers build and install the bootstrap CLI, but it
should not define Hum's identity or adoption plan. Hum itself is a language and
toolchain, not merely a Cargo crate.

Long-term distribution should include:

- prebuilt signed or checksummed binaries
- Windows, macOS, and Linux installer/package paths
- editor adapters that invoke first-party Hum tools
- private-release smoke tests from a clean clone
- a public package only after naming, license, security posture, and release
  evidence are ready

Do not publish to a public package registry until the release checklist says the
repo is ready for that channel.

## SemVer Policy

Hum follows Semantic Versioning 2.0.0 once public APIs exist.

During `0.0.x`:

- patch changes may include docs, guardrails, and bootstrap fixes
- breaking language-design changes are allowed but must be documented
- tags are snapshots, not compatibility promises
- release notes must say `pre-alpha`

During `0.x.y`:

- minor bumps mark meaningful new preview capabilities
- patch bumps fix or clarify existing preview behavior
- compatibility is still not promised unless a specific surface is marked candidate

At `1.0.0`:

- the public API and tool surfaces must be declared
- compatibility rules must be enforceable
- migration and deprecation policy must exist
- the compiler, formatter, LSP, package metadata, and semantic graph contracts must have stable lanes

## Tag Policy

Use annotated Git tags for release snapshots:

```powershell
git tag -a v0.0.1 -m "Hum 0.0.1 pre-alpha"
```

The tag name may use the common `v` prefix, but the semantic version itself is `0.0.1`.

Do not tag until:

1. `tools/check_release_readiness.ps1` passes.
2. `tools/check_text_hygiene.ps1` passes.
3. `tools/check_public_readiness.ps1` passes.
4. `cargo fmt --check` passes.
5. `cargo test` passes.
6. `cargo clippy --all-targets -- -D warnings` passes.
7. `cargo run -- check examples` passes.
8. `git status --short` is clean except for intentional release edits before the release commit.

## Private Remote Then Public Remote

Recommended sequence:

1. Keep working locally until the repo has a clean initial history and release discipline.
2. Push to a private remote first.
3. Use the private remote to verify clone/setup from a clean machine or clean directory.
4. Create pre-alpha tags only after release-readiness checks pass.
5. Delay public release until the README, setup, license, security posture, and first demos explain what Hum is and is not.

No remote push is part of the local release check. Publishing is a separate human decision.

## Release Notes

Every release tag should have a short note containing:

- version
- date
- status: pre-alpha, alpha, beta, release candidate, or stable
- highlights
- compatibility notes
- known risks
- verification commands run
- commit hash

## Enterprise Bar

Before Hum is presented like a serious new language from a Microsoft-scale or Google-scale team, it should have:

- versioned CLI behavior through `hum version`, `hum version --format json`, `hum capabilities --format json`, and `hum doctor --format json`
- versioned semantic graph schema
- documented LSP capability matrix at [LSP_CAPABILITY_MATRIX.md](LSP_CAPABILITY_MATRIX.md)
- editor integration strategy
- supported platform tiers
- security policy
- contribution policy
- release checklist
- signed or at least annotated tags
- reproducible local verification

## Sources

- Semantic Versioning 2.0.0: https://semver.org/spec/v2.0.0.html
- Git tag documentation: https://git-scm.com/book/en/v2/Git-Basics-Tagging
- Cargo manifest documentation: https://doc.rust-lang.org/cargo/reference/manifest.html

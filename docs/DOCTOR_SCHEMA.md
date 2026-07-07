# Hum Doctor Schema

Date: 2026-07-07

Current schema: `hum.doctor.v0`

## Purpose

`hum doctor` is the first-party setup health report for the Hum repo. It gives
beginners, editors, CI wrappers, installers, and agents one command to ask
whether the current directory looks like a portable Hum checkout.

The command checks repo-local facts only. It should not record absolute machine
paths, home directories, editor install locations, secrets, tokens, or personal
workspace state.

## Command

```powershell
hum doctor
hum doctor --format json
```

During the Rust bootstrap:

```powershell
cargo run -- doctor
cargo run -- doctor --format json
```

The human output is for terminals. The JSON output is the adapter, installer,
CI, and agent contract.

## Top-Level Shape

```json
{
  "schema": "hum.doctor.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "milestone": "0 semantic graph",
  "workspace": "current_directory",
  "summary": {},
  "checks": [],
  "next_steps": []
}
```

## Fields

- `schema`: schema name, currently `hum.doctor.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: maturity label such as `pre-alpha`
- `milestone`: current implementation milestone
- `workspace`: intentionally generic workspace label, not an absolute path
- `summary`: pass, warn, and fail counts plus aggregate status
- `checks`: individual setup facts
- `next_steps`: portable next actions

Each `checks` entry has:

- `id`: stable check identifier
- `label`: short human label
- `status`: `pass`, `warn`, or `fail`
- `required`: whether a failure should block a release-style handoff
- `message`: portable explanation without machine-specific paths

## Current Checks

Current `checks` includes:

- `repo_manifest`: `Cargo.toml`
- `version_file`: `VERSION` matches the Cargo package version
- `license_notice`: `LICENSE` and `NOTICE.md`
- `text_hygiene_policy`: `.editorconfig`, `.gitattributes`, and
  `tools/check_text_hygiene.ps1`
- `public_readiness_policy`: `.gitignore` and
  `tools/check_public_readiness.ps1`
- `preflight_script`: `tools/check_all.ps1`
- `clean_checkout_smoke`: `tools/check_clean_checkout.ps1`
- `tag_readiness`: `tools/check_tag_readiness.ps1`
- `hosted_ci`: `.github/workflows/ci.yml` guarded hosted preflight
- `setup_docs`: `docs/SETUP.md` and `docs/TEXT_HYGIENE_WORKFLOW.md`
- `editor_assets`: editor strategy, LSP matrix, and generated TextMate grammar
- `reference_fixtures`: reference source and editor recovery fixtures

## Rules

- Run from the repo root.
- Treat `fail` as a blocker before a commit, tag, public snapshot, or release
  handoff.
- Treat `warn` as visible debt that may be acceptable only for local private
  development.
- Keep output portable. Do not add absolute local paths to committed examples.
- Keep this separate from semantic source checks. `hum doctor` checks the
  checkout and guardrails; `hum check` checks `.hum` source.

## Non-Goals For V0

V0 does not promise:

- package installation repair
- editor-specific configuration mutation
- network checks
- registry checks
- OS package manager checks
- a replacement for `tools/check_all.ps1`

The surface exists so setup health can become machine-readable before Hum has
installers, package managers, or full editor plugins.

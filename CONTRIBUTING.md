# Contributing To Hum

Repository orchestration requires an explicit absolute PowerShell 7 executable.
Pass it as `--pwsh <absolute-path>`; a bare `pwsh`, ambient PATH, profile, cache,
or `HUM_DEV_PWSH` value is not authority. Do not rely on Windows
PowerShell 5.1, ambient shell aliases, or Unix text utilities for commit-message
validation. Use `hum-dev commit-message check --file <message>`; pass executable
paths, environment, credentials, and repository trust explicitly and
process-locally.

Date: 2026-07-06
Status: pre-alpha contributor policy

## Start Here

Hum is an intent-first, evidence-native systems language in pre-alpha.
Milestone 0 is local, offline-first, non-executing, and safe on the maker's
machine.

Before proposing a change, read:

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): ground-truth language map
- [docs/LANGUAGE_REFERENCE.md](docs/LANGUAGE_REFERENCE.md): current source surface
- [examples/reference_surface.hum](examples/reference_surface.hum): checked source fixture
- [docs/GOVERNANCE.md](docs/GOVERNANCE.md): evolution and feature gates
- [docs/SETUP.md](docs/SETUP.md): portable setup and editor guidance
- [SECURITY.md](SECURITY.md): reporting path and supported security scope

If another document disagrees with the architecture, treat that as a design bug
to resolve rather than a fork in the truth.

## Local Setup

Install Rust with `rustup`, open a fresh terminal, then run from the repo root:

```powershell
$Pwsh = Join-Path $PSHOME $(if ($IsWindows) { 'pwsh.exe' } else { 'pwsh' })
cargo run -p hum-dev -- evidence focused --pwsh $Pwsh
```

That is the paved-road verification command. It runs Rust checks, example checks,
reference fixture coverage, graph and syntax JSON parsing, generated grammar
drift detection, text hygiene, public readiness, and release readiness.

Keep personal editor state, local tool paths, secrets, `.env` files, and scratch
notes out of the repo.

## Change Types

Small implementation or documentation fixes should stay narrow and include the
matching tests or docs updates.

Language, standard-library, runtime-profile, backend, package, security, or
tooling direction changes must explain:

- semantics
- diagnostics
- semantic graph facts
- tooling impact
- runtime profile or evidence impact
- verification story
- performance story
- pedagogy story

Use [docs/RFC_TEMPLATE.md](docs/RFC_TEMPLATE.md) for large proposals. Syntax
without diagnostics, graph facts, and formatter/LSP implications is not ready
for stable Hum.

## Reference Fixture Rule

If a change alters current source syntax, accepted sections, test obligations,
or ordinary examples, update [examples/reference_surface.hum](examples/reference_surface.hum)
or explain why a smaller focused fixture owns the case.

A healthy reference fixture passes `hum check` without diagnostics and produces
no unlinked obligations from `hum test-skeletons`.

## Commit Style

Use scoped Conventional Commits:

```text
type(scope): short imperative summary
```

Examples:

```text
docs(language): clarify task sections
test(graph): cover obligation links
fix(tooling): keep text scanner portable
```

Allowed types are `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`,
`build`, `ci`, `chore`, and `revert`.

## Pull Requests

A good PR should:

- describe the problem, not only the edit
- keep unrelated refactors out
- update docs when behavior or promises change
- add tests or fixtures when a checkable contract changes
- run `tools/check_all.ps1` before review
- avoid secrets, personal paths, editor state, generated junk, and local machine
  assumptions

Do not publish packages, create tags, push release artifacts, or change remote
settings as part of an ordinary contribution.

## Review Standard

Hum prefers one clear paved road over many clever alternatives. A contribution
is stronger when it removes ambiguity, improves evidence, helps diagnostics,
or makes the language easier to teach without hiding effects, allocation,
mutation, failure, unsafe behavior, or platform authority.

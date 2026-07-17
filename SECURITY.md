# Hum Security Policy

Date: 2026-07-06
Status: pre-alpha security response policy

## Scope

Hum is pre-alpha. Milestone 0 is a local, offline-first Rust bootstrap
front-end that parses `.hum` files, emits diagnostics, emits graph JSON,
generates test skeletons, and emits syntax metadata. It does not execute Hum
programs, build packages, run plugins, fetch dependencies, or publish release
artifacts.

Security issues are still important because the repo contains compiler code,
tooling scripts, examples, CI configuration, and docs that future users may
copy.

## Supported Versions

| Version | Security support | Notes |
| --- | --- | --- |
| `main` | Best-effort pre-alpha review | No compatibility promise. |
| `0.0.x` snapshots | Best-effort pre-alpha review | Tags are snapshots, not stable releases. |
| older commits | Not supported | Reproduce on `main` when possible. |

## What To Report

Please report:

- ways to make repo tooling execute unexpected commands
- path traversal, path leak, or public-readiness bypasses
- UTF-8, BOM, mojibake, or terminal-control bypasses in text scanners
- malformed `.hum` input that crashes the CLI or causes unbounded resource use
- graph, diagnostic, or test-skeleton output that hides security-relevant facts
- CI or release workflow behavior that could publish or trust the wrong artifact
- unsafe, FFI, package, or execution claims that overstate current capability

Do not report ordinary missing language features as security issues unless they
create a concrete safety, integrity, availability, privacy, or supply-chain risk.

## How To Report

Do not post exploit details, secrets, token values, or weaponized proofs in a
public issue.

Preferred reporting path:

1. Use GitHub's private vulnerability reporting: open the repository's Security
   tab and choose "Report a vulnerability." This opens a private advisory
   visible only to the maintainer.
2. If private reporting is unavailable, open a minimal public issue asking for
   private security coordination. Do not include technical exploit details.
3. Include the affected commit, platform, command, expected behavior, observed
   behavior, and a minimal reproduction inside the private channel.

Do not add personal emails, private chat handles, secrets, or machine-specific
details to this repository.

## Response Expectations

Pre-alpha response is best effort. The intended process is:

- acknowledge the report once received through an appropriate private channel
- reproduce the issue from a clean checkout when possible
- identify whether the bug affects parser/checker behavior, graph output, repo
  tooling, CI, docs, or future security claims
- fix with tests or scanner coverage when practical
- run `tools/check_all.ps1` before landing the fix
- document any remaining risk in the relevant design or release docs

## Security Claims

Hum does not claim broad production security today. Current claims are limited
to the checked repository state and the Milestone 0 scope.

Do not describe Hum as secure, memory-safe, sandboxed, certified, or production
ready without a matching threat model, evidence artifact, profile, and release
note.

See:

- [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md)
- [docs/UNSAFE_POLICY.md](docs/UNSAFE_POLICY.md)
- [docs/ALPHA_THREAT_MODEL_0_1.md](docs/ALPHA_THREAT_MODEL_0_1.md)
- [docs/RELEASE_AND_VERSIONING.md](docs/RELEASE_AND_VERSIONING.md)

## Safe Research Rules

Keep testing local and non-destructive:

- use small local fixtures
- do not attack third-party services
- do not test against systems you do not control
- do not include real secrets or private data in reports
- do not rely on network access for a Milestone 0 reproduction

If a reproduction needs a file path, use repo-relative paths such as
`examples/reference_surface.hum`.

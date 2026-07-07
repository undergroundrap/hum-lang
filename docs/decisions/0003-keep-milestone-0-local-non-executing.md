# 0003 Keep Milestone 0 Local And Non-Executing

Status: accepted
Date: 2026-07-06

## Context

Early language tools can harm users if they execute untrusted code, fetch
network resources, run build scripts, or mutate outside declared boundaries
before the safety model exists.

Hum also needs public trust in its own development process. The first milestone
should prove parser, diagnostics, graph facts, text hygiene, and repository
discipline before claiming execution safety.

## Decision

Milestone 0 remains local, offline-first, parser/checker/graph only, and
non-executing.

It may parse `.hum` files, emit diagnostics, emit graph JSON, generate test
skeletons, emit syntax metadata, and run local repository checks. It must not
execute Hum programs, build packages, run plugins, fetch dependencies, or
publish artifacts.

## Consequences

- Security posture stays honest during pre-alpha.
- The reference fixture can be checked without running user code.
- CI and local preflight can run safely from a clean checkout.
- Execution waits for formal core, profiles, effects, and evidence gates.

## Alternatives Rejected

- Add a quick interpreter before the formal core is pinned.
- Add package or plugin execution during Milestone 0.
- Market parser/checker output as a public alpha runtime.

## BDFL Note

Hum should earn trust by refusing risky shortcuts. A quiet non-executing
milestone is more credible than a flashy unsafe one.

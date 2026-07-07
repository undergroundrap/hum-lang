# 0004 Make Tests First-Class Evidence

Status: accepted
Date: 2026-07-06

## Context

Hum intent blocks such as `needs:`, `ensures:`, `watch for:`, and `tests:`
state promises that should not remain prose. If those promises are not linked
to evidence, future agents and reviewers have to guess whether the source is
covered.

Milestone 0 already emits graph facts and generated test skeletons for unlinked
obligations.

## Decision

`test` is a first-class top-level item, and test coverage is part of the
semantic graph. Task obligations generated from meaningful section lines must
link to exact or conservative canonical `covers:` lines when present.

The checked reference fixture must keep its generated test obligations and
current security/trust evidence obligations linked so the current surface has a
healthy example.

## Consequences

- `hum graph` exposes task test obligations, evidence obligations, and linked tests.
- `hum test-skeletons` can propose missing evidence without writing files.
- `tools/check_all.ps1` fails when the reference fixture has unlinked security or trust evidence obligations.
- Docs and examples must treat tests as language facts, not comments.
- Future LSP, CI, and agents can reason from graph facts instead of prose.

## Alternatives Rejected

- Treat tests as external files only.
- Leave `needs:` and `ensures:` as unchecked documentation.
- Let agents infer coverage from unconstrained similar wording instead of explicit graph links.

## BDFL Note

Hum should make promises visible and then ask for evidence. Tests are not an
aftermarket accessory; they are how intent becomes reviewable.

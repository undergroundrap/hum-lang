# 0005 Keep Verifiers As Evidence Producers

Status: accepted
Date: 2026-07-07

## Context

Hum will need mathematical verification help for resource claims, allocation
freedom, purity/replayability, unsafe boundaries, data-structure invariants,
space-time strategies, and safety-critical profiles.

Truth Harness and similar engines are promising, but coupling Hum directly to a
specific verifier would create several risks:

- the verifier could become hidden compiler authority
- proof attempts could be mistaken for proof
- cached or LLM-generated proof prose could become marketing
- Hum could inherit external tool assumptions without source-visible evidence
- local-first and no-hidden-telemetry rules could erode

Hum needs a boundary before it needs integration.

## Decision

Hum treats external verifiers, math engines, model checkers, benchmark harnesses,
and proof assistants as evidence producers, not as the source of language truth.

Hum emits obligations. External engines return receipts: `proved`, `refuted`,
`unknown`, `unsupported`, or `timeout`.

A `proved` result is accepted only when it includes a certificate,
independently checkable trace, or verifier artifact allowed by the active Hum
profile. `unknown` is an honest result and must remain distinct from failure.

No external engine may silently optimize Hum programs, rewrite Hum semantics,
require cloud access, collect hidden telemetry, or hide assumptions.

The architecture boundary is documented in
[../MATH_ENGINE_BOUNDARY.md](../MATH_ENGINE_BOUNDARY.md).

## Consequences

- Hum can integrate with Truth Harness later without becoming dependent on it.
- `hum graph` and future obligation exports remain the source of source-backed
  facts.
- `hum evidence` can record verifier receipts as evidence artifacts.
- Solver, proof, benchmark, heuristic, and model-assumption evidence must stay
  separate.
- Safety-critical profiles can reject weak or assumption-heavy receipts.
- CI should validate schema fixtures before any clever solver integration.

## Alternatives Rejected

- Build a proof assistant into Hum before the executable core is stable.
- Let a verifier mutate compiler optimization decisions without explicit source
  intent and evidence.
- Treat LLM proof prose as proof.
- Treat benchmark results as theorem evidence.
- Require a specific external verifier for normal Hum development.
- Allow cloud-only proof services as part of the default toolchain.

## BDFL Note

A verifier is a witness, not a monarch. Hum should welcome strong witnesses,
but the language must keep its own court records.
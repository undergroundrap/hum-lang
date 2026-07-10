# Hum Decision Records

Date: 2026-07-06
Status: active decision index

## Purpose

Decision records explain why major Hum design rails exist. They do not replace
[../ARCHITECTURE.md](../ARCHITECTURE.md), but they preserve the reasoning
behind choices that future contributors should not reopen casually.

Use this directory for accepted or rejected decisions that materially affect
language semantics, diagnostics, graph facts, tooling, profiles, security,
standard library direction, backend strategy, or release posture.

## Records

| ID | Status | Decision |
| --- | --- | --- |
| [0001](0001-adopt-evidence-native-architecture.md) | accepted | Adopt evidence-native architecture. |
| [0002](0002-use-rust-bootstrap-until-self-hosting.md) | accepted | Use Rust bootstrap until staged self-hosting is proven. |
| [0003](0003-keep-milestone-0-local-non-executing.md) | accepted | Keep Milestone 0 local, offline-first, and non-executing. |
| [0004](0004-make-tests-first-class-evidence.md) | accepted | Make tests first-class evidence with graph-linked obligations. |
| [0005](0005-keep-verifiers-as-evidence-producers.md) | accepted | Keep external verifiers as evidence producers, not compiler authority. |
| [0006](0006-make-resource-layout-and-comptime-explicit.md) | accepted | Make resource, layout, and compile-time power explicit. |
| [0007](0007-adopt-progressive-disclosure-and-migration-discipline.md) | accepted | Adopt progressive disclosure and migration discipline. |
| [0008](0008-adopt-swappable-backend-ladder.md) | accepted | Adopt a swappable backend ladder. |
| [0009](0009-adopt-formal-readability-not-english-mimicry.md) | accepted | Adopt formal readability, not English mimicry. |
| [0010](0010-adopt-explicit-state-model.md) | accepted | Adopt an explicit state model. |
| [0011](0011-add-checked-resolver-before-execution.md) | accepted | Add checked resolver before execution. |
| [0012](0012-adopt-snake-case-identifiers.md) | accepted | Adopt snake_case identifiers, retire spaced names. |
| [0013](0013-remove-number-type.md) | accepted | Remove the Number type; core numerics are Int and UInt. |
| [0014](0014-adopt-ownership-model.md) | accepted (delegated, veto open) | Adopt ownership and borrowing as the core model; Session V earns only the exact local direct-field writable-alias slice. |
| [0015](0015-adopt-classified-runtime-contract-policy.md) | accepted (delegated, veto open) | Classify runtime contracts by proof and trust boundary before any elision. |

## Template

A decision record should include:

- status
- context
- decision
- consequences
- alternatives rejected
- BDFL note

For feature proposals before a decision exists, use
[../RFC_TEMPLATE.md](../RFC_TEMPLATE.md).

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

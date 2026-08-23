# 0002 Use Rust Bootstrap Until Self-Hosting

Status: accepted
Date: 2026-07-06

## Context

Hum needs a trustworthy compiler front end before Hum can execute programs or
compile itself. Rust gives the bootstrap compiler memory safety, portable
tooling, mature tests, and a boring build path while the language design is
still moving.

Self-hosting too early would turn language design uncertainty into compiler
risk.

## Decision

The Milestone 0 compiler front end stays in Rust until Hum proves self-hosting
through staged differential tests and clearer compiler code than the Rust
version.

The original Milestone 0 decision admitted no third-party crates and forbade
unsafe compiler code. That deliberate rule kept the immature trust root small.

### WO22 Unit B Amendment

WO22 preserves that small-trust-root rule as history and narrowly graduates the dependency rule for native code generation.
Exactly five pinned direct Cranelift `0.133.1` crates (`cranelift-codegen`,
`cranelift-frontend`, `cranelift-jit`, `cranelift-module`, and
`cranelift-native`) plus their locked transitive graph provide the first
practical backend. Hum retains language semantics, verification, capability
authority, and backend admission. Cranelift is bounded and replaceable; this
decision promises neither its removal nor an undecided LLVM migration.

`#![deny(unsafe_code)]` remains the compiler-wide default with exactly one
reviewed, locally allowed JIT invocation boundary. This amendment grants no
general dependency, unsafe, FFI, backend, build-script, proc-macro, or hidden-generation permission.

## Consequences

- Cargo is a normal early build path, but not Hum's identity.
- Hum can use Rust to build parser, diagnostics, graph, and tooling quickly.
- Self-hosting remains a proof milestone, not a prestige milestone.
- Backends are targets; they do not own Hum semantics.

## Alternatives Rejected

- Start self-hosted before the formal core exists.
- Use C or C++ for the bootstrap compiler.
- Present Hum as merely a Cargo crate or Rust DSL.

## BDFL Note

Rust is scaffolding. Good scaffolding is not a weakness; refusing to remove it
when the building can stand would be.

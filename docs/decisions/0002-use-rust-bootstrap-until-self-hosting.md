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

The Rust bootstrap must stay conservative: no third-party crates today and
`#![forbid(unsafe_code)]` for the bootstrap compiler.

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

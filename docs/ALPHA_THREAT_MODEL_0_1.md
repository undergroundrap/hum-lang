# Hum Alpha 0.1 Threat Model

Date: 2026-07-06

## Purpose

This document scopes the threats Hum `0.1-alpha` must handle for the
`offline-tool@0.1` profile.

The goal is not to solve all systems-security problems. The goal is to make the
first executable Hum profile small enough that its authority and evidence can be
audited.

## Protected Assets

The alpha protects:

- source files under analysis
- declared input bundles
- generated evidence artifacts
- diagnostics and graph output
- user filesystem outside declared inputs and outputs
- build and run reproducibility claims
- public project credibility

## Trust Boundaries

The alpha trust boundaries are:

- Hum source entering the parser
- declared input directories entering a Hum run
- declared output directory receiving evidence artifacts
- JSON input and output parsing
- SHA-256 digest computation
- host filesystem reads and writes
- Rust bootstrap compiler implementation
- user shell invoking `hum`

## Attacker Model

Assume an attacker can:

- provide malformed Hum source
- provide malformed JSON files
- create deep or large directory trees
- use confusing file names and path traversal attempts
- include symlinks, junctions, or platform-specific path edge cases
- try to trigger undeclared reads or writes
- try to make output nondeterministic
- try to hide risk behind vague intent blocks
- compare public claims against actual evidence

The alpha must not assume inputs are polite.

## Profile Denials

For `offline-tool@0.1`, the checker or runner must deny:

- network access
- process execution
- dynamic loading
- FFI
- unsafe
- threads
- wall-clock-dependent logic
- randomness
- environment-variable reads by default
- writes outside the declared output directory
- hidden mutation
- undeclared file reads

## Required Evidence

The alpha evidence bundle should make these facts inspectable:

- declared inputs
- observed inputs
- declared outputs
- observed outputs
- declared effects
- inferred or observed effects
- denied authority attempts
- diagnostics
- semantic graph facts
- profile status
- run trace
- hashes of relevant files

## Out Of Scope

The alpha does not defend against:

- malicious Rust compiler or standard library
- malicious operating system
- malicious shell launching Hum
- hardware compromise
- concurrent attackers mutating input files during a run
- production supply-chain attacks against a public registry
- network attacks
- unsafe or foreign code attacks
- certification or compliance misrepresentation by downstream users

Out-of-scope items may become future profile work. They must not be hidden in
alpha claims.

## Failure Rule

If Hum cannot prove an authority decision in `0.1-alpha`, it must fail closed or
state the unresolved assumption in the evidence bundle.

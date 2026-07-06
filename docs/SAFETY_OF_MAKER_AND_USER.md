# Safety Of Maker And User

Date: 2026-07-06

## Purpose

Hum treats safety as a two-sided promise.

The project must protect the person building the language and the people who
will eventually build, run, deploy, audit, and depend on Hum programs.

```text
The language is not safe if building it is unsafe.
The tool is not safe if using it quietly expands authority.
The ecosystem is not safe if packages can surprise the machine.
```

This is a design philosophy, a project gate, and a future product standard.

## Two-Sided Safety

Hum must protect:

- the maker: the developer machine, repo, build process, maintainers,
  contributors, release process, and local data
- the user: program users, operators, downstream systems, package consumers,
  auditors, and deployment environments

These are not separate concerns. A language that trains its toolchain to run
surprising code will eventually train its ecosystem to do the same.

## Maker Safety

Hum development must stay safe on real primary developer machines.

Rules:

- local-first by default
- offline-first until a network threat model exists
- no destructive filesystem operations without explicit approval
- no writes outside the repo except approved build caches and temp fixtures
- no generated code execution in Milestone 0
- no dependency build scripts in Milestone 0
- no registry access, package downloads, telemetry, or background services by
  default
- no shelling out based on `.hum` source
- no secrets, tokens, private keys, recovery codes, or `.env` files in the repo
- tests must use fixtures under the repo or isolated temp directories
- risky commands need dry-run behavior before real mutation

Milestone 0 tools may parse Hum source, validate it, produce diagnostics, and
emit semantic graph JSON. That is enough power for the current proof.

## User Safety

Hum programs should make authority visible before it can surprise the user.

Rules:

- effects are part of the interface
- mutation is explicit
- allocation and lifetime behavior are visible where they matter
- failure is typed and cannot be ignored accidentally
- network, filesystem, process, clock, randomness, unsafe, and foreign-code
  authority require named capabilities
- secrets use distinct types and must not fall into ordinary text APIs
- foreign code is marked in source and graph output
- package code cannot gain ambient authority by being installed
- security claims are scoped to a threat model and evidence packet
- the safe path should be the short path

Hum should make risky behavior loud, local, and reviewable.

## Contributor And Maintainer Safety

Hum must also protect the people maintaining the project.

Rules:

- every major decision leaves a written trail
- generated artifacts are reviewable or reproducible
- release steps are scripted, minimal, and eventually reproducible
- CI should use least privilege when it arrives
- no workflow may publish, sign, upload, or change remote state without an
  explicit release gate
- dependency changes need review, provenance, and rollback thinking
- logs must not contain secrets
- tools should fail closed when trust evidence is missing

Maintainer safety is language safety. Burned-out, overloaded, or surprised
maintainers make worse security decisions.

## Agent Safety

Agents are collaborators, not trusted authorities.

Hum should be unusually good for agents because it exposes structured intent,
semantic graph facts, source spans, effect graphs, ownership facts, diagnostics,
and repair hints.

But agents must never be the reason a program is accepted.

Rules:

- agents may propose code
- agents may explain diagnostics
- agents may generate tests
- agents may search graph facts
- agents may not bypass compiler checks
- agents may not bypass tests, fuzzing, proofs, or benchmarks
- generated code is untrusted until checked by the same pipeline as human code

The compiler carries trust. Agents carry acceleration.

## Product Principle

Hum's toolchain should make safe behavior easier than risky behavior.

That means:

- the default command is local and read-only when possible
- capabilities are visible before use
- diagnostics explain authority, mutation, allocation, and failure
- package metadata can be inspected without running package code
- generated fixes show what they change before applying
- unsafe and foreign code require named justification
- performance shortcuts cannot silently weaken safety

The language should feel powerful because the dangerous parts are named.

## Admission Checklist

Before accepting a language or toolchain feature, ask:

```text
does it protect the developer machine?
does it protect the eventual program user?
does it make authority visible?
does it execute code, download code, or load plugins?
does it read secrets or write outside fixtures?
does it lower the amount of trusted code?
does it increase the amount of ambient authority?
does it have a local proof?
does it have a rollback or migration story?
does it make the safe path easier?
```

If the answer is unclear, the feature stays experimental or waits.

## Examples

Good:

```text
uses:
  filesystem.read

changes:
  cache.files

protects:
  source files are never modified
  secrets are not printed in diagnostics
```

Bad:

```text
does:
  run build script
  download dependency
  update generated files
```

The bad version may become valid someday, but only after it declares
capabilities, inputs, outputs, trust source, rollback behavior, and evidence.

## BDFR Decision

Safety is part of Hum's taste.

Features that feel clever but quietly expand authority are not Hum.

Features that protect only the compiler while surprising the developer machine
are not Hum.

Features that protect only local development while passing risk to downstream
users are not Hum.

Hum should keep the maker safe and everyone else safe.
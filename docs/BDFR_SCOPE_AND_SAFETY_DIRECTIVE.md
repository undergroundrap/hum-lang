# Hum BDFR Scope And Safety Directive

Date: 2026-07-06

## Purpose

This document turns the BDFR's scope and safety concerns into hard project constraints.

Hum can have a huge vision, but the project must stay safe on a real developer
machine, scoped enough to finish, and practical enough to adopt existing
ecosystems before trying to replace them.

## Brutal Answer

The missing piece is not another language feature.

The missing piece is a control system for the project itself:

```text
vision is A
safe proof path is B

A without B becomes fantasy or risk
B without A becomes a small tool nobody cares about
Hum needs A through B
```

That means the project must prioritize the smallest local proof that preserves
the big design, while checking new work against
[LANGUAGE_PROJECT_RISK_REGISTER_2026.md](LANGUAGE_PROJECT_RISK_REGISTER_2026.md).

This directive is paired with
[SAFETY_OF_MAKER_AND_USER.md](SAFETY_OF_MAKER_AND_USER.md): Hum must protect the
person building the toolchain and the people who will eventually run, deploy,
audit, and depend on Hum programs.

## Non-Negotiables

### 1. Protect The Developer Machine

Hum development happens on real primary developer machines, so the project must behave like
mission-critical local software from day one.

Rules:

- no destructive filesystem operations without explicit approval
- no writing outside the repo except approved build caches
- no deleting, moving, or rewriting user files as part of compiler tests
- no generated code execution in Milestone 0
- no package build scripts in Milestone 0
- no background services unless explicitly requested
- no network access from Hum tools until a threat model, tests, and capability
  gates exist
- no secrets, tokens, keys, recovery codes, or `.env` files in the repo
- tests use fixtures under the repo or temp directories only
- every future command that can mutate the host needs dry-run behavior first

The compiler may inspect `.hum` source, parse it, validate it, and emit local
diagnostics or graph JSON. That is enough for Milestone 0.

The same rule applies to future users: Hum should make authority explicit before
tools, packages, or programs can affect their machines or systems.

### 2. Offline-First Until Proven Otherwise

Hum should be local-only by default.

This applies to:

- compiler
- formatter
- linter
- semantic graph generation
- tests
- benchmarks
- docs generation
- package metadata inspection

Future network features must require explicit capabilities:

```text
uses:
  network.client

changes:
  remote service config
```

Nectar should start as an offline package/build metadata tool. It should not
download packages, run remote scripts, or execute dependency build code until
the supply-chain model exists.

Important honesty: no serious project can prove "zero vulnerabilities" in the
absolute sense. Hum should instead require:

- no known vulnerabilities under the current scoped threat model
- no ambient network authority
- no hidden code execution
- fuzzed parsers before untrusted input promises
- reproducible local tests
- explicit profile gates before risky capabilities

### 3. Scope Beats Sprawl

Hum should rank work by safety and functionality, not excitement.

Priority order:

1. Local machine safety.
2. Parser correctness.
3. Stable diagnostics.
4. Semantic graph truth.
5. Intent block validation.
6. Generated local tests.
7. Tiny executable core.
8. Ownership and effects.
9. Unsafe and FFI policy.
10. Interop wrappers.
11. Portability.
12. Standard library labs.
13. Native code generation.
14. Package downloads and registry behavior.
15. Self-hosting.

Anything that jumps ahead must explain why it reduces risk or unlocks the next
proof.

### 4. Adopt Before Rewrite

Hum will not get adoption if it refuses the existing world.

The right rule is:

```text
use existing libraries through safe boundaries first
rewrite only after measurement, safety, and ergonomics justify it
```

Hum should plan interop with:

- C ABI
- C++ libraries through explicit wrappers
- Rust crates through stable wrapper boundaries, not by pretending Rust ABI is
  stable
- Python through extension/module boundaries for data and ML adoption
- Wasm for portable sandboxed components
- system libraries through capability-limited foreign APIs

But foreign code is dangerous. Every interop boundary needs:

- ABI and layout contract
- ownership transfer rules
- failure and panic rules
- thread-safety rules
- versioning rules
- security/trust section
- tests and fuzzing where input-facing
- graph facts that mark the boundary as foreign

Hum should use the world before replacing the world.

### 5. Portability Is A Product Feature

Hum should be portable in two senses:

- source portability: Hum code means the same thing across targets unless a
  target/profile says otherwise
- tool portability: Hum tooling works on real developer machines without heroic
  setup

Early target tiers:

```text
tier 0: Windows x86_64 dev host for this repo
tier 1: Windows, Linux, macOS on x86_64 and arm64
tier 2: Wasm sandbox targets
tier 3: no-std, firmware, kernel, and realtime profiles
```

Portability requires:

- target triples
- explicit endian/layout behavior
- no hidden OS assumptions in core
- path APIs that understand platform differences
- reproducible toolchain metadata
- profile-specific standard-library availability
- CI eventually, but local proof first

### 6. Fun Matters After Trust

Hum should be fun to read and write. That is part of the mission.

But fun cannot outrank:

- safety
- correctness
- portability
- diagnostics
- predictable performance
- interop honesty

The language becomes fun because the compiler carries the terrifying parts.

## Current Scope Lock

Until Milestone 0 is complete, Hum may do only this:

- parse `.hum`
- validate known top-level forms
- validate known sections
- emit diagnostics
- emit semantic graph JSON
- run local tests
- add local examples
- add docs and design gates

Milestone 0 must not:

- execute Hum programs
- generate native code
- execute generated code
- download packages
- connect to registries
- run dependency build scripts
- mutate files outside approved fixtures
- shell out based on `.hum` input
- load plugins from the network

That keeps the project safe while the language shape is still plastic.

## What This Means For Next Work

The previous next steps still matter:

1. `OPERATIONS_MODEL.md`
2. `NETWORK_MODEL.md`
3. `NUMERIC_AND_TENSOR_MODEL.md`
4. `PACKAGE_AND_BUILD.md`

But this directive adds two gates that should come before risky implementation:

1. `INTEROP_AND_PORTABILITY.md`
2. local safety checks in the compiler/tooling docs

Nectar should not become an internet-connected package manager yet. Nectar
should first become a local manifest, profile, evidence, and reproducibility
tool.

## BDFR Decision

Hum stays ambitious, but the work proceeds through local, reversible,
auditable proof.

The project should say no to impressive features until the core can prove:

- it is safe on the developer machine
- it preserves structured meaning
- it explains every diagnostic clearly
- it does not hide authority, mutation, allocation, failure, network, unsafe, or
  foreign code

That is the path from vision to a language people can trust.

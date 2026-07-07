# Hum Evidence-Native Adoption Strategy 2026

Date: 2026-07-06

## Purpose

This document distills the current adoption research into product doctrine. It
does not replace the architecture. It tells the architecture what the outside
world will demand before Hum can be taken seriously.

Primary research snapshot:

- [research/2026-07-06-evidence-native-systems-language.md](research/2026-07-06-evidence-native-systems-language.md)
- [research/2026-07-06-offline-tool-alpha.md](research/2026-07-06-offline-tool-alpha.md)
- [research/2026-07-07-lattner-compiler-lessons.md](research/2026-07-07-lattner-compiler-lessons.md)

## Thesis

Hum should compete as an evidence-native systems language.

That means the language should produce more than binaries:

- machine-checkable intent
- explicit effect and capability reports
- semantic graph facts for tools and agents
- diagnostics with stable machine codes
- profile reports
- unsafe and power-boundary review packets
- provenance, SBOM, and reproducible build evidence
- deployable artifacts for real environments

Readable syntax is necessary. It is not enough.

## Competition Stance

Hum should eventually compete with Rust, C++, Go, and other serious systems
tools. The alpha strategy should not pretend that has already happened.

The right posture is:

```text
compete later as a full systems language;
win first as an evidence-native offline tool profile.
```

That means Hum should avoid premature "Rust replacement" claims while still
building toward a language that can challenge incumbents on safety, evidence,
readability, performance truth, and toolchain coherence.

## First Adoption Wedges

The strongest early wedges are:

1. Cybersecurity tools and secure internal services.
2. DevOps and SRE utilities that replace fragile script glue.
3. Defense, military, and air-gapped offline tooling.

These audiences naturally value memory safety, explicit effects, offline
distribution, provenance, auditability, and safe defaults before they require a
massive third-party ecosystem.

Healthcare, finance, embedded, robotics, and safety-critical systems can become
serious later wedges, but only after Hum can emit credible evidence bundles and
run real code. Games and AI/ML are useful stress tests, not the first adoption
center.

## Public Alpha Gate

Milestone 0 is not a public alpha. It is a semantics and evidence seed.

A credible public alpha should start with
[OFFLINE_TOOL_ALPHA_0_1.md](OFFLINE_TOOL_ALPHA_0_1.md): a deterministic
`offline-tool@0.1` profile for local security and change-review tools.

It needs:
- one executable safe profile
- native Windows output
- one portable artifact path, preferably WASI or a component-style target
- basic standard library coverage for files, strings, JSON, time, hashing, and tests
- formatter, LSP, test runner, structured diagnostics, and graph output
- package manifest, lockfile, vendoring, and offline rebuild
- effect report, provenance, and SBOM output
- one C ABI or practical interop path
- one demo that beats a Rust, Go, or Python alternative on evidence and reviewability

Do not call a parser-only or checker-only state a public alpha.

Do not call an unstable alpha stable by implication. If source, schemas, or
tool outputs may break, say so plainly and provide a migration path for
intentional breaks.

## Top Blockers

The highest-risk blockers are:

- executable backend and runtime profile
- precise memory, effect, and capability model
- credible FFI and interop
- reproducible builds, lockfiles, vendoring, and offline installs
- signed provenance and SBOM generation
- minimal but coherent standard library
- formatter, LSP, docs, test runner, and benchmark runner
- stable diagnostics and machine-readable output
- unsafe and power-feature governance
- concurrency and async model

The pattern is clear: Hum's adoption risk is product completeness more than
syntax novelty.

## Killer Demos

The first serious demos should be evidence demos, not language-tour demos:

- secure service with effect report, SBOM, provenance, and threat model
- offline defense-style utility with vendored dependencies and reproducible build report
- DevOps/SRE release utility with dry-run, change trace, and signed artifacts
- regulated workflow skeleton with risk trace, tests, and documentation bundle
- agent-safe tool execution through a capability-restricted portable component

Each demo should show the source, what the compiler understood, what evidence it
produced, and what risk it reduced.

## Deferrals

Defer these until the narrow alpha is credible:

- broad syntax expansion
- large general-purpose standard library ambition
- custom public registry
- full async ecosystem
- GPU and accelerator claims
- broad formal verification promises
- fancy macro systems
- vague AI-native branding

## Design Consequences

Every major feature proposal must say which evidence it improves. If it cannot
improve evidence, safety, portability, performance truth, or beginner clarity,
it should wait.

Every adoption proposal must also say how a team can try Hum incrementally:
one module, one tool, one wrapper boundary, one hot path, or one
safety-critical component at a time.

Hum can still be playful and readable, but the product promise is serious:

```text
intent -> semantics -> graph -> evidence -> artifact
```

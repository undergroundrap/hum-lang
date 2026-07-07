# 0001 Adopt Evidence-Native Architecture

Status: accepted
Date: 2026-07-06

## Context

Hum is not trying to win only by being nicer syntax over existing systems
languages. The adoption thesis in [../ADOPTION_STRATEGY_2026.md](../ADOPTION_STRATEGY_2026.md)
says serious users need evidence: intent, effects, diagnostics, profile facts,
provenance, tests, and deployable artifacts.

A language whose only output is a binary would compete mostly on runtime,
syntax, and ecosystem maturity. That is not enough to justify a new systems
language.

## Decision

Hum will be an intent-first, evidence-native systems language.

The core path is:

```text
human-readable intent -> precise formal core -> semantic graph -> checks, profiles, evidence, and tools -> portable backends and platform artifacts
```

Language features must preserve machine-checkable meaning in diagnostics, graph
facts, profile reports, tests, or release evidence.

## Consequences

- Docs, examples, checks, and graph schemas are product surfaces.
- Important claims belong in checked sections, not comments.
- Features that create new power must create new evidence.
- Tooling cannot be an afterthought; it is part of the language.
- Public security or adoption claims need scoped evidence.

## Alternatives Rejected

- Treat Hum as a nicer front end over an existing language.
- Prioritize syntax growth before diagnostics, graph facts, and evidence.
- Defer provenance, profiles, and release artifacts until after execution.

## BDFL Note

Hum should feel human-readable, but the real taste line is evidence. If a
feature makes reviewers, tools, or agents guess, it is moving away from Hum.

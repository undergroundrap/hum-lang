# Hum Paved Road Doctrine

Date: 2026-07-06

## Purpose

Hum should not become a language of endless knobs, duplicate APIs, and folklore
performance tricks.

The language should make the best ordinary path obvious, checked, optimized,
and explainable. Advanced paths may exist, but they must be rare, explicit,
and backed by evidence.

## Core Doctrine

```text
Hum has paved roads, not endless knobs.
```

A paved road is a default path that is:

- safe for ordinary use
- readable to beginners
- precise enough for systems engineers
- represented in the semantic graph
- supported by diagnostics and repair hints
- optimized by the compiler, stdlib, and profiles where possible
- backed by tests, benchmarks, fuzzing, or research evidence where relevant

If Hum offers a choice, the default choice must be the one the language is
willing to recommend.

## Rules

1. There should be one obvious default API for the common case.
2. Alternatives must remove a real tax, not satisfy taste.
3. Faster but riskier APIs require source-visible trust, profile gates, or lab
   status.
4. Bad patterns should produce diagnostics before they become culture.
5. Compiler and stdlib optimizations should follow declared intent instead of
   requiring users to memorize tricks.
6. Research enters through labs, benchmarks, and adversarial tests before it
   changes a stable default.
7. The semantic graph must expose what path was chosen and why it is valid.
8. A feature that needs a long style guide to be safe is not paved.

## Side Roads

Hum can have specialized APIs, profiles, and strategy choices when the domain
requires them. They are side roads, not competing defaults.

A side road must declare at least one of:

- a trusted input boundary
- a hardware or runtime profile
- a memory, allocation, or concurrency constraint
- a benchmark-proven workload shape
- an unsafe, foreign, realtime, certified, or accelerator boundary

Side roads should be visible in source, graph facts, docs, diagnostics, and
benchmarks. If a side road becomes the best common path, it should graduate
into the paved road and the old road should become legacy or lab-only.

## Diagnostics Are Product

Hum should warn when code is legal but suspicious:

- a cost claim conflicts with visible loops or allocation
- an API choice conflicts with declared trust boundaries
- a data structure choice conflicts with workload intent
- an optimization goal sacrifices a stronger safety claim
- a test obligation is missing, stale, or not linked to source intent

A diagnostic should prefer a repair path over scolding. The best diagnostic
says what Hum believes the paved road is and how to get there.

## Standard Library Consequence

The standard library is where this doctrine matters most.

For example, Hum should not expose many equal-looking map types and force
beginners to choose. It should provide an excellent default `Map`, keep it safe
under hostile input, and let advanced profiles select specialized strategies
only when the source and evidence justify them.

Research such as elastic or funnel hashing can make Hum better, but only after
it passes through `map-lab`, competitor baselines, adversarial tests, and a
semantic graph story. A paper is a signal to investigate, not a license to add
another permanent public knob.

## Anti-Goals

Hum should avoid:

- multiple equivalent ways to express the same ordinary program
- default APIs that are fast only for friendly inputs
- clever APIs whose safety depends on tribal knowledge
- hidden allocation, mutation, IO, unsafe, or trust assumptions
- research-driven churn without migration and evidence
- configuration surfaces that replace language design

## Architecture Answers

Which architecture layer does this belong to?

- Surface Hum, semantic graph, checks and evidence, tooling, and standard
  library labs.

Which existing docs does it constrain?

- ARCHITECTURE.md
- LANGUAGE_CONSTITUTION.md
- STDLIB_CONSTITUTION.md
- STDLIB_STRATEGY.md
- OPTIMIZATION_AND_DSA_STRATEGY.md
- HASH_TABLE_RESEARCH_2501_02305.md

Which semantic graph facts does it require?

- chosen default path
- declared trust assumptions
- optimization intent
- cost claims
- selected stdlib strategy where relevant
- diagnostics and evidence links for non-default choices

Which profile or evidence gates does it change?

- Specialized paths require profiles, trust declarations, benchmark packets,
  adversarial tests, or lab status before they can become stable defaults.

What must Milestone 0 ignore for now?

- Real strategy selection, executable optimization, benchmark enforcement, and
  stdlib implementation. Milestone 0 should only preserve the doctrine, emit
  graph facts, and keep diagnostics ready for later enforcement.

# 0009: Adopt Formal Readability, Not English Mimicry

Date: 2026-07-07
Status: accepted

## Context

Hum wants source that humans can read, agents can repair, and compilers can
check. That does not mean Hum should imitate casual English.

COBOL proved an important lesson: English-like spelling can make code feel
approachable while hiding a rigid formal language underneath. The result can be
verbose, redundant, harder to scan, and deceptively familiar to beginners who
recognize the words but not the constrained semantics.

Hum's risk is similar because its intent blocks use ordinary phrases. Without a
hard rule, readability work could drift into synonym-heavy syntax, grammar by
prose convention, or executable natural language.

## Decision

Hum will optimize for formal readability, not English mimicry.

Accepted rules:

1. Stable executable syntax must lower to precise Core Hum operations. A phrase
   that cannot lower precisely is a diagnostic, not a feature.
2. Hum may use readable words where they name one formal concept, such as
   `task`, `does:`, `fails when:`, `for each`, `from`, `until`, and `through`.
3. Hum must reject synonym sets for core concepts. There should not be many
   English ways to spell the same executable operation.
4. Section prose can guide humans and agents, but it is not executable authority
   until the compiler owns a checked grammar and graph fact for it.
5. Readable names are allowed only as source identity where the grammar can
   tokenize them deterministically. They must not become grammar-critical prose.
6. Diagnostics should teach the formal model behind friendly text instead of
   pretending that ordinary English is being executed.
7. Hum examples must stay scannable. If the algorithm is buried under filler
   words, the syntax failed.
8. Agent-assisted authoring may begin from natural language, but the compiler
   must force the result into precise Hum source before it counts.

## Consequences

Hum can still be beginner-friendly, but the beginner path teaches programming
concepts rather than hiding them behind familiar words.

The formatter, linter, parser, semantic graph, Core Hum preview, and future LSP
must prefer one canonical spelling per concept.

Milestone 0 may continue capturing some names, types, and body lines as text,
but those captures are not stable language semantics. Stabilization requires a
lossless syntax/token model and Core Hum lowering rules.

Future grammar work should treat every readable phrase as a candidate formal
operator, keyword, section label, name token, or rejected ambiguity. No phrase
gets accepted just because it sounds friendly.

## Alternatives Rejected

- Let arbitrary English execute.
- Make the language verbose so it looks approachable in screenshots.
- Accept many synonyms for the same operation and rely on style guides later.
- Treat parser permissiveness in Milestone 0 as stable language design.
- Hide formal concepts from beginners until they hit production bugs.
- Let agents convert vague English directly into trusted behavior.

## BDFL Note

Hum should feel humane because it is honest, not because it cosplays as prose.
The best version is closer to a clean engineering checklist with executable
sections than to a paragraph pretending to be a program.
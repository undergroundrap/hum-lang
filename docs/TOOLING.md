# Hum Tooling

Date: 2026-07-06

## Thesis

Hum's tools should be part of the language design, not an afterthought.

Rust has `rustfmt` and Clippy. Hum should have equivalents from the beginning,
because readability, consistency, linting, and teachable diagnostics are central
to the language promise.

## Tool Names

Working first-party tools:

```text
hum       compiler and source-level checks
humfmt    formatter
chirp     linter, mentor, and code-quality checker
nectar    package manager and build tool
hum lsp   first-party Language Server Protocol implementation
hum debug first-party Debug Adapter Protocol implementation
```

See [TOOLCHAIN_2050.md](TOOLCHAIN_2050.md) for the long-horizon editor,
debugger, syntax highlighting, profiler, and agent tooling strategy.

## Why `humfmt`

`humfmt` should be boring and obvious.

Formatting should not be branded so cleverly that users have to remember what it
means. A formatter is infrastructure. The name should be predictable.

Commands:

```powershell
humfmt file.hum
humfmt examples
nectar fmt
nectar fmt --check
```

## Why `chirp`

`chirp` is the working Clippy-style name.

A chirp is small, sharp, and attention-getting. That fits a linter: it should
warn without feeling like a wall of punishment.

Commands:

```powershell
chirp file.hum
chirp examples
nectar chirp
nectar chirp --fix
```

Possible output tone:

```text
chirp[readability.unlisted-effect]: this task reads `clock.now` but does not list it under `uses:`

why:
  Hum makes outside dependencies visible so readers and tools know what this task relies on.

fix:
  add `clock.now` under `uses:` or remove the read.
```

`chirp` should be stricter than a style checker and kinder than a compiler error.
It should be a mentor tool.

## Tool Responsibilities

### `hum`

Compiler-level truth:

- parse
- check
- graph
- version and schema identity
- explain diagnostics
- eventually run single-file experiments
- serve LSP facts through `hum lsp`
- serve debugger facts through `hum debug`

### `humfmt`

Formatting truth:

- stable layout
- no semantic edits
- preserve comments
- preserve intent blocks
- format examples in docs
- support `--check` for CI

### `chirp`

Quality and mentorship:

- readability lints
- suspicious cost claims
- missing `why:` quality
- vague `needs:` or `ensures:`
- unsafe review-packet completeness
- duplicated intent
- overly clever syntax
- beginner-hostile naming
- auto-fixes only when mechanically safe

### `nectar`

Project truth:

- package metadata
- dependencies
- lockfile
- project check/test/build
- registry operations
- package graph timings
- project-wide semantic graph
- workspace indexes for LSP, debugger, docs, and agents

## Lint Levels

Chirp should support levels:

```text
allow   accepted, silent
note    useful teaching note
warn    should usually fix
error   policy violation; fails CI when configured
deny    project forbids this pattern
```

Hum should avoid making every opinion a compiler error. Put style and pedagogy in
`chirp`, unless the issue breaks a language promise.

## Rule Names

Rule names should be readable:

```text
readability.vague-why
intent.unlisted-use
intent.unlisted-change
safety.missing-protects
unsafe.missing-trusts
cost.unchecked-claim
tests.missing-regression-note
agent.missing-semantic-context
```

Stable rule codes can come later, but the names should be understandable now.

## Formatter Principles

Humfmt should:

- use one canonical layout
- avoid configuration wars early
- keep blocks visually scannable
- make nested `does:` blocks obvious
- keep long intent lines readable
- never hide `uses:` and `changes:` far from a task header

Hum should not become a language where every project has a different style.

## Logo And Mascot Direction

Hum's mascot can be an original, hand-drawn hummingbird.

Tool personality:

- `hum`: calm compiler
- `humfmt`: tidy, invisible helper
- `chirp`: small bird warning at the edge of the branch
- `nectar`: package/build food source

Keep it charming, but do not let cuteness outrun seriousness.

## Brutal Rule

If Hum claims to be readable, `humfmt` and `chirp` are not optional side quests.
They are core language infrastructure.
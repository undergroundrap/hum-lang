# Nectar Package Manager

Date: 2026-07-06

## Decision

The working name for Hum's package manager and project tool is **Nectar**.

Hum is the language. Nectar is what Hum projects feed on.

```text
hum       compiler and source-level tools
nectar    package manager, build tool, lockfile, registry client
```

## Why Nectar Works

Nectar fits the language world:

- Hum suggests human-readable code and humming machines.
- Nectar naturally pairs with a hummingbird mascot.
- The name is short and memorable.
- It feels warmer than `humpm`, `humget`, or `hpm`.
- It can own project-level workflows without confusing the compiler name.

## CLI Shape

Expected commands:

```powershell
nectar init
nectar add <package>
nectar remove <package>
nectar check
nectar test
nectar build
nectar build --release
nectar graph
nectar timings
nectar publish
```

`nectar check` should call into `hum check` across the project graph.

`nectar graph` should aggregate `hum.semantic_graph` output across packages.

## Files

Working names:

```text
nectar.toml
nectar.lock
```

`nectar.toml` should describe package intent as well as mechanics.

Example sketch:

```toml
[package]
name = "session-server"
version = "0.1.0"
edition = "2026"
why = "safe session storage example for Hum"

[dependencies]
http = "0.1"
time = "0.1"

[build]
default-target = "native"
```

## Package Manager Principles

Nectar should be stricter than most package managers.

1. Lockfiles by default.
2. Reproducible builds by default.
3. Dependency graph visible to humans and agents.
4. Package intent and trust metadata visible.
5. Fast local checks before full builds.
6. Security advisories as first-class diagnostics.
7. Build scripts restricted and auditable.
8. Native dependencies explicit, not discovered by surprise.

## Build Profiles

Nectar should separate feedback loops:

```text
nectar check             fastest semantic check
nectar test              local tests
nectar build             fast dev build
nectar build --release   optimized build
nectar timings           explain where build time went
```

## Security Metadata

Eventually packages should be able to declare:

```toml
[trust]
uses-network = false
uses-filesystem = true
uses-unsafe = false
has-build-script = false
```

This should not replace compiler checks. It should make package review cheaper.

## Hummingbird Logo Direction

The mascot should be an original, simple, hand-drawn hummingbird.

Style goals:

- black-and-white first
- crude but recognizable
- hacker-native, not corporate-slick
- works as a tiny terminal/README mark
- distinct from GNU art while respecting the mascot tradition

The GNU head is a useful inspiration for mascot seriousness and simplicity, but
Hum should not copy GNU's image or trade dress. The bird should be ours.

## Naming Risk

Nectar is a common word and appears in other technical projects. Before a public
release, do a deeper conflict pass across:

- GitHub
- crates.io
- npm
- PyPI
- domains
- package registries
- trademarks

Until then, Nectar is the working project-tool name.

## Brutal Rule

Nectar should not become a slow, magical wrapper around the compiler.

It should make project state, dependency state, build state, and trust state more
visible than other ecosystems do.
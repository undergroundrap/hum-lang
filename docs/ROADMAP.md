# Hum Roadmap And Pedagogy

Date: 2026-07-06

## Brutal Roadmap Principle

Do not build the impressive thing first.

Build the thing that proves the language is not vague.

The first useful Hum compiler does not need native code generation. It needs to
parse Hum, understand intent blocks, reject broken promises, and emit a semantic
graph that humans and agents can trust.

The unifying map for this roadmap is [ARCHITECTURE.md](ARCHITECTURE.md).

## Teaching Order

Hum should be taught in this order:

1. `task`: the unit of intent.
2. `why:`: why this code exists.
3. `uses:` and `changes:`: the capability boundary.
4. `needs:` and `ensures:`: preconditions and postconditions.
5. `fails when:`: typed failure.
6. `watch for:`: edge cases that become tests.
7. `does:`: precise execution.
8. Bindings and explicit mutation.
9. Control flow: `if`, `match`, `while`, `loop`, `for each`, `for index`.
10. Core operators, ranges, collection literals, and ergonomic expression rules.
11. Types and null-free values.
12. Effects.
13. Ownership: `owned`, `borrow`, `change`, `shared`.
14. Stores and data-structure intent.
15. Unsafe boundaries.
16. Concurrency and cancellation.
17. Performance contracts and benchmarks.
18. Backend lowering.
19. Self-hosting only after the compiler can prove itself against the Rust bootstrap.

This order teaches the mental model before the machinery.

## Milestone 0: Semantic Graph

Goal: prove Hum source can become structured meaning.

Must do:

1. Parse `.hum` files.
2. Build an AST.
3. Validate top-level forms.
4. Validate task block names and order.
5. Emit a JSON semantic graph.
6. Record source spans for every block.
7. Check that `does:` only changes names listed in `changes:`.
8. Generate test skeletons from `needs:`, `ensures:`, and `watch for:`.
9. Produce human diagnostics and machine diagnostics.
10. Preserve `cost:`, `avoids:`, and `tradeoffs:` in the semantic graph.
11. Parse top-level `test` blocks and link them to `tests:` obligations.
12. Record research gates for features that touch safety, unsafe, profiles, scheduling, or verification.
13. Reserve graph fields for dry-run, rollback, idempotence, telemetry, desired/observed state, numeric tolerance, shape, dtype, device, OS/platform authority, deployment, container, agent-tool, and reproducibility facts.
14. Check new feature work against `LANGUAGE_PROJECT_RISK_REGISTER_2026.md`.

Must not do:

- native code generation
- generics
- macros
- async
- full borrow checking
- LLVM integration

Current prototype status:

- Rust bootstrap package exists.
- `hum check` parses `.hum` files and runs first intent checks.
- `hum graph` emits `hum.semantic_graph.v0` JSON.
- `--timings` reports file read, parse, check, and total time.
- AST items, sections, fields, params, and diagnostics keep source spans.
- Diagnostics have stable `H####` codes in terminal output, `hum check --format json`, and `hum graph` JSON.
- Known task and test sections get canonical-order warnings.
- Current examples check cleanly.
- `hum graph` emits source-derived node IDs for files, items, params, fields, sections, and section lines.
- Parser spans report one-based first visible columns for line-oriented constructs.
- `hum graph` emits per-file document symbols for editor and LSP adapters.
- `hum graph` emits section folding ranges for editor and LSP adapters.
- `fixtures/editor` covers broken and half-written source that must still emit graph JSON and stable diagnostics.
- `hum graph` emits section `line_items` with text, spans, and meaningful/comment status.
- `hum graph` emits task-level `test_obligations` from `needs:`, `ensures:`, `watch for:`, and `tests:` lines, with exact or conservative canonical `covers:` links to top-level `test` blocks when present.
- `hum test-skeletons` prints Hum `test` blocks for unlinked obligations without executing code or writing files.
- `hum syntax` emits `hum.syntax_surface.v0` JSON for editor and tool adapters, documented in [SYNTAX_SURFACE_SCHEMA.md](SYNTAX_SURFACE_SCHEMA.md).
- `hum syntax` emits section hover metadata so adapters can explain intent blocks from one source.
- `hum syntax` emits a semantic-token legend so adapters share token names before range emission exists.
- `hum syntax --format textmate` and `editors/textmate/hum.tmLanguage.json` provide the first generated highlighting surface.

Deferred beyond Milestone 0:

- broad semantic paraphrase proof for coverage; Milestone 0 now supports exact links plus conservative canonical matching for case, punctuation, filler words, hyphenation, and small section aliases

Milestone 0 is about truth, not speed.

Milestone 0 is also about local safety. Per [BDFR_SCOPE_AND_SAFETY_DIRECTIVE.md](BDFR_SCOPE_AND_SAFETY_DIRECTIVE.md), this milestone must stay offline, must not execute Hum programs, must not run generated code, must not download packages, and must not mutate files outside approved fixtures.

## Public Alpha Gate

Milestone 0 is not a public alpha. It is the evidence and semantic graph seed
that makes a future alpha coherent.

A credible public alpha must include executable artifacts, an explicit safe
profile, basic standard library coverage, formatter and LSP paths, structured
diagnostics, offline rebuild support, interop, and evidence outputs such as
effect reports, provenance, and SBOMs.

The recommended first alpha profile is `offline-tool@0.1`: deterministic,
file-only, no-network, no-unsafe local tooling that emits evidence artifacts.

See [ADOPTION_STRATEGY_2026.md](ADOPTION_STRATEGY_2026.md) and
[OFFLINE_TOOL_ALPHA_0_1.md](OFFLINE_TOOL_ALPHA_0_1.md).

## Milestone 1: Executable Core

Goal: run a tiny precise subset.

The gate for this milestone is [FORMAL_CORE.md](FORMAL_CORE.md). New executable
syntax must lower into that core or stay experimental.

Add:

- literals
- local bindings
- core operator table and precedence
- arithmetic
- conditionals
- simple loops
- ranges and checked slices
- `try` failure propagation
- `Result`
- simple records
- collection literals
- interpreter or Cranelift prototype

Checks:

- `uses:` and `changes:`
- typed failure handling
- basic effects
- profile declarations parsed into the semantic graph
- runtime contract checks in debug mode
- executable unit tests

## Milestone 2: Ownership And Effects

Goal: make systems safety real.

Add:

- `owned`
- `borrow`
- `change`
- `shared`
- move checking
- exclusive mutation
- no hidden allocation
- effect propagation
- allocation, panic, and failure policy hooks for profiles

This is where Hum starts earning the right to talk near Rust.

## Milestone 3: Stores And Stdlib Lab

Goal: make data-structure intent real.

Add:

- `store`
- `Vec`
- `Map`
- `Set`
- `Text`
- `Bytes`
- arena allocation
- benchmark harness
- adversarial tests
- pointer-stability diagnostics
- `map-lab` research track
- `ops-lab` plan/dry-run/rollback fixtures
- `net-lab` typed address and protocol parsing fixtures
- `numeric-lab` exact/approx/unit/tolerance fixtures
- `tensor-lab` shape/dtype/device fixtures
- present-key, missing-key, insertion, deletion, resize, and iteration cost profiles
- Bevy-inspired declared-access scheduling experiments
- change-detection fixtures

This milestone should produce the first serious comparison against Rust,
hashbrown, Abseil, Zig, Go, Java, and C++ containers.

See [STDLIB_STRATEGY.md](STDLIB_STRATEGY.md),
[STDLIB_CONSTITUTION.md](STDLIB_CONSTITUTION.md),
[STDLIB_PRIMITIVE_RESEARCH_2026.md](STDLIB_PRIMITIVE_RESEARCH_2026.md),
[PRACTITIONER_PAIN_SWEEP_2026.md](PRACTITIONER_PAIN_SWEEP_2026.md),
[OPTIMIZATION_AND_DSA_STRATEGY.md](OPTIMIZATION_AND_DSA_STRATEGY.md), and
[ERGONOMICS_AND_OPERATORS.md](ERGONOMICS_AND_OPERATORS.md).

## Milestone 4: Unsafe, FFI, Interop, And Portability

Goal: make unsafe code, foreign libraries, and target portability reviewable.

The gates for this milestone are [SECURITY_MODEL.md](SECURITY_MODEL.md),
[UNSAFE_POLICY.md](UNSAFE_POLICY.md),
[INTEROP_AND_PORTABILITY.md](INTEROP_AND_PORTABILITY.md), and
[OS_AND_PLATFORM_MODEL.md](OS_AND_PLATFORM_MODEL.md). Unsafe, foreign, or
platform-authority code exists only when the review packet, profile policy,
graph facts, wrappers, and tests/proofs are ready.

Add:

- `unsafe task`
- `foreign`
- explicit trust boundaries
- provenance checks where possible
- required `needs:`, `protects:`, `proves:`, and `watch for:`
- sanitizer integration
- C ABI wrapper strategy
- Rust crate interop through stable wrapper boundaries
- Python extension/module boundary sketch
- Wasm sandbox boundary sketch
- target-tier policy and portability fixtures
- Windows-first, portable-by-design OS capability fixtures
- service/driver/install/update authority modeled as graph facts

Unsafe should feel like entering a sealed lab, not flipping a casual switch.

## Milestone 5: Compile-Time Discipline, Nectar, And Editor Spine

Goal: make fast feedback, package discipline, and editor integration real before native optimization dominates the project.

Add:

- `humfmt`
- `chirp`
- `hum lsp`
- `hum debug` protocol sketch
- TextMate grammar sketch
- Tree-sitter grammar design note
- semantic tokens
- LSP diagnostics, document symbols, folding, hover, and code actions
- debugger data model for tasks, tests, promises, effects, and source spans
- `nectar.toml`
- `nectar.lock`
- `nectar check`
- `nectar timings`
- package graph caching
- semantic graph cache keys
- no-op check benchmarks
- feature compile-time budgets
- runtime profile metadata in Nectar
- reproducible build manifest sketch
- OCI/container metadata sketch
- SBOM and provenance evidence sketch
- deployment-plan and generated-artifact evidence
- numeric/tensor reproducibility evidence
- dry-run and rollback evidence for ops-facing packages

See [COMPILE_TIME_STRATEGY.md](COMPILE_TIME_STRATEGY.md), [NECTAR_PACKAGE_MANAGER.md](NECTAR_PACKAGE_MANAGER.md), [TOOLING.md](TOOLING.md), [FORMATTER.md](FORMATTER.md), [TOOLCHAIN_2050.md](TOOLCHAIN_2050.md), [RUST_LESSONS_2026.md](RUST_LESSONS_2026.md), [ERGONOMICS_AND_OPERATORS.md](ERGONOMICS_AND_OPERATORS.md), [OPTIMIZATION_AND_DSA_STRATEGY.md](OPTIMIZATION_AND_DSA_STRATEGY.md), [SAFETY_CRITICAL_AND_ENGINE_EDGECASES.md](SAFETY_CRITICAL_AND_ENGINE_EDGECASES.md), [RUNTIME_PROFILES.md](RUNTIME_PROFILES.md), [COMPUTING_LESSONS_SWEEP_2026.md](COMPUTING_LESSONS_SWEEP_2026.md), and [CROSS_LANGUAGE_REGRET_LEDGER.md](CROSS_LANGUAGE_REGRET_LEDGER.md).

## Milestone 6: Native Backend

Goal: compile useful code.

Preferred path:

1. Cranelift for first native backend.
2. LLVM for optimized AOT builds.
3. MLIR later for hardware-aware, vector, tensor, sparse, and accelerator paths.

The backend comes after Hum IR can preserve intent.

## Milestone 7: Self-Hosting Preparation

Goal: prepare for Hum compiling Hum without making the compiler fragile.

Add:

- stable package/build model
- stable semantic graph schema
- stable diagnostic codes
- golden graph tests
- differential tests between Rust and Hum implementations
- parser fuzzing
- Hum implementations of non-critical compiler tools
- stage0/stage1/stage2 build plan

Do not rewrite the parser, type checker, borrow checker, optimizer, or codegen in
Hum until smaller compiler tools written in Hum are easier to understand and at
least as trustworthy as their Rust versions.

See [SELF_HOSTING_PLAN.md](SELF_HOSTING_PLAN.md),
[EXTERNAL_ADVICE_REVIEW.md](EXTERNAL_ADVICE_REVIEW.md), and
[RESEARCH_MAP_2026.md](RESEARCH_MAP_2026.md).

## Milestone 8: Agent-Native Tooling And 2050 Developer Experience

Goal: make Hum the easiest systems language for humans, IDEs, debuggers, profilers, and agents to repair correctly.

Add:

- `hum graph`
- `hum explain`
- `hum risks`
- `hum tests`
- `hum agent docs`
- `tree-sitter-hum`
- VS Code extension
- Neovim/Helix/Zed support path
- DAP debug adapter prototype
- profiler source mapping
- stable diagnostic codes
- repair hints
- counterexample-guided repair loop

Agents should never scrape terminal prose when the compiler can hand them facts.

## First User Story

The first beginner demo should be a tiny task list:

- define a task
- remember tasks
- add a task
- reject an empty title
- show each task

It teaches the mental model without requiring systems background.

The first impressive systems demo should not be a game, web server, or compiler.

It should be a small security-sensitive session store:

- create session
- validate session
- expire session
- resist guessed tokens
- bound memory
- generate tests from `watch for:`
- produce a semantic graph
- reject an undeclared mutation

That demo explains Hum in one screen.

## Pedagogy Rule

Every tutorial must show:

- the readable Hum source
- what the compiler understood
- what it checked
- what test or proof it generated
- what machine code path it would eventually choose

Hum should teach programmers to think in promises, capabilities, effects, and
measurements.

# Hum

Hum is an intent-first systems language design draft.

The goal is low-level Rust/C++ power with Python-like readability, static types,
explicit effects, memory safety by default, and compiler-generated context for
humans and coding agents.

Hum source should read like a human-readable set of promises:

```text
task create session(user: User, device: Device) -> SessionToken {
  why:
    let a verified user stay signed in without sending their password again

  uses:
    clock.now
    random.secure
    sessions

  changes:
    sessions

  needs:
    user is verified
    device is allowed

  ensures:
    token belongs to user
    token expires in 30 days
    token does not reveal user secrets

  protects:
    session id cannot be guessed
    expired token cannot work

  watch for:
    attacker may create many sessions quickly
    hash collisions must not make lookup slow
    memory use must stay bounded

  optimizes:
    lookup speed
    memory density
    security before speed

  does:
    make secure token
    create session for user
    save session in sessions
    return token
}
```

## Status

This repository is a language design seed with a Milestone 0 Rust bootstrap compiler front-end.

Current version: `0.0.1` pre-alpha.

Start with [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the ground-truth map
of how the design docs fit together.

Current artifacts:

- [examples/reference_surface.hum](examples/reference_surface.hum): checked Milestone 0 source fixture covering the current reference surface
- [SPEC.md](SPEC.md): broad language design draft
- [docs/LANGUAGE_REFERENCE.md](docs/LANGUAGE_REFERENCE.md): traditional reference spine for source files, items, sections, and current language status
- [docs/MILESTONE_0_GRAMMAR.md](docs/MILESTONE_0_GRAMMAR.md): current Rust bootstrap parser grammar contract
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): unifying architecture ground truth across language, graph, tools, profiles, stdlib, backend, and platform design
- [docs/decisions/README.md](docs/decisions/README.md): accepted design decision record index
- [docs/SETUP.md](docs/SETUP.md): portable setup for beginners, experts, terminals, and editors
- [CONTRIBUTING.md](CONTRIBUTING.md): contributor workflow, design gates, commit style, and PR rules
- [SECURITY.md](SECURITY.md): supported security scope, reporting path, and claim limits
- [docs/EDITOR_AND_INTEGRATION_STRATEGY.md](docs/EDITOR_AND_INTEGRATION_STRATEGY.md): editor, LSP, plugin, Visual Studio, Eclipse, and Jupyter integration strategy
- [docs/RELEASE_AND_VERSIONING.md](docs/RELEASE_AND_VERSIONING.md): SemVer, `0.0.1`, release checks, and tag policy
- [.github/workflows/ci.yml](.github/workflows/ci.yml): hosted preflight workflow for pushes and pull requests
- [docs/LANGUAGE_CONSTITUTION.md](docs/LANGUAGE_CONSTITUTION.md): rules Hum must not violate
- [docs/PAVED_ROAD_DOCTRINE.md](docs/PAVED_ROAD_DOCTRINE.md): one obvious default path, explicit side roads, and evidence-backed optimization doctrine
- [docs/BDFR_SCOPE_AND_SAFETY_DIRECTIVE.md](docs/BDFR_SCOPE_AND_SAFETY_DIRECTIVE.md): BDFR safety, scope, offline-first, interop, and portability directive
- [docs/SAFETY_OF_MAKER_AND_USER.md](docs/SAFETY_OF_MAKER_AND_USER.md): two-sided safety philosophy for makers, users, maintainers, and agents
- [docs/LANGUAGE_PROJECT_RISK_REGISTER_2026.md](docs/LANGUAGE_PROJECT_RISK_REGISTER_2026.md): ground-truth failure modes and safety gates for building Hum itself
- [docs/FORMAL_CORE.md](docs/FORMAL_CORE.md): precise executable core that surface Hum lowers into
- [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md): cybersecurity threat model and secure-by-default language rules
- [docs/UNSAFE_POLICY.md](docs/UNSAFE_POLICY.md): unsafe review packets, profile bans, provenance, and FFI-adjacent rules
- [docs/INTEROP_AND_PORTABILITY.md](docs/INTEROP_AND_PORTABILITY.md): C/C++/Rust/Python/Wasm interop and target portability strategy
- [docs/OS_AND_PLATFORM_MODEL.md](docs/OS_AND_PLATFORM_MODEL.md): Windows-first, portable-by-design OS authority and platform model
- [docs/TEXT_HYGIENE_WORKFLOW.md](docs/TEXT_HYGIENE_WORKFLOW.md): UTF-8, no-BOM, mojibake, control-character, and local-link hygiene workflow
- [docs/BOOTSTRAP_COMPILER.md](docs/BOOTSTRAP_COMPILER.md): Rust bootstrap rules and compiler architecture
- [docs/SELF_HOSTING_PLAN.md](docs/SELF_HOSTING_PLAN.md): when Hum can start compiling Hum
- [docs/MEMORY_SAFETY_MODEL.md](docs/MEMORY_SAFETY_MODEL.md): Rust-inspired safety model and Hum-specific safety promises
- [docs/COMPILE_TIME_STRATEGY.md](docs/COMPILE_TIME_STRATEGY.md): fast check/build philosophy and timing budgets
- [docs/RUST_LESSONS_2026.md](docs/RUST_LESSONS_2026.md): what Hum should learn from Rust's successes and retrofits
- [docs/CROSS_LANGUAGE_REGRET_LEDGER.md](docs/CROSS_LANGUAGE_REGRET_LEDGER.md): C++, Rust, Go, Zig, Python, JS/TS, and other regrets Hum must avoid
- [docs/LANGUAGE_PAIN_SWEEP_2026.md](docs/LANGUAGE_PAIN_SWEEP_2026.md): researched pain sweep across Rust, Zig, C++, Python, Go, TypeScript, Verse, and more
- [docs/PRACTITIONER_PAIN_SWEEP_2026.md](docs/PRACTITIONER_PAIN_SWEEP_2026.md): sysadmin, DevOps, network, low-level, AI/ML, and numeric workload pain sweep
- [docs/RESEARCH_MAP_2026.md](docs/RESEARCH_MAP_2026.md): papers and systems that gate Hum language, compiler, stdlib, profile, and verification design
- [docs/ADOPTION_STRATEGY_2026.md](docs/ADOPTION_STRATEGY_2026.md): evidence-native adoption thesis, early wedges, alpha gate, and demo strategy
- [docs/OFFLINE_TOOL_ALPHA_0_1.md](docs/OFFLINE_TOOL_ALPHA_0_1.md): `offline-tool@0.1` alpha profile, HumGate demo, and alpha cut line
- [docs/ALPHA_CHARTER_0_1.md](docs/ALPHA_CHARTER_0_1.md): `0.1-alpha` promise, non-promises, public artifact gate, and claim rule
- [docs/ALPHA_THREAT_MODEL_0_1.md](docs/ALPHA_THREAT_MODEL_0_1.md): `offline-tool@0.1` protected assets, boundaries, attacker model, and denied authority
- [docs/ALPHA_CLAIMS_MATRIX_0_1.md](docs/ALPHA_CLAIMS_MATRIX_0_1.md): human-readable alpha claims matrix backed by JSON validation
- [docs/LANGUAGE_SUBSET_0_1.md](docs/LANGUAGE_SUBSET_0_1.md): pinned source subset for the `offline-tool@0.1` alpha
- [docs/SEMANTIC_GRAPH_SCHEMA_0_1.md](docs/SEMANTIC_GRAPH_SCHEMA_0_1.md): target semantic graph facts for alpha evidence
- [docs/DIAGNOSTICS_SCHEMA_0_1.md](docs/DIAGNOSTICS_SCHEMA_0_1.md): target diagnostic JSON contract for alpha evidence
- [docs/EFFECT_REPORT_SCHEMA_0_1.md](docs/EFFECT_REPORT_SCHEMA_0_1.md): declared, observed, denied, and unresolved effect report shape
- [docs/PROFILE_REPORT_SCHEMA_0_1.md](docs/PROFILE_REPORT_SCHEMA_0_1.md): `offline-tool@0.1` profile pass/fail report shape
- [docs/EVIDENCE_BUNDLE_LAYOUT_0_1.md](docs/EVIDENCE_BUNDLE_LAYOUT_0_1.md): HumGate alpha evidence directory layout
- [docs/research/README.md](docs/research/README.md): repeatable research workflow, prompts, and normalized snapshots
- [docs/COMPUTING_LESSONS_SWEEP_2026.md](docs/COMPUTING_LESSONS_SWEEP_2026.md): 2026 sweep across hardware, Docker/OCI, Kubernetes, WASI, MCP, observability, networking, storage, and deployment lessons
- [docs/OPTIMIZATION_AND_DSA_STRATEGY.md](docs/OPTIMIZATION_AND_DSA_STRATEGY.md): research-gated optimization and data-structure strategy
- [docs/HASH_TABLE_RESEARCH_2501_02305.md](docs/HASH_TABLE_RESEARCH_2501_02305.md): deep note on elastic/funnel hashing and Hum's `Map` research path
- [docs/ERGONOMICS_AND_OPERATORS.md](docs/ERGONOMICS_AND_OPERATORS.md): operator, QoL, and Bevy-inspired data-access strategy
- [docs/TOOLCHAIN_2050.md](docs/TOOLCHAIN_2050.md): LSP, debugger, highlighting, profiler, and long-horizon toolchain strategy
- [editors/textmate/hum.tmLanguage.json](editors/textmate/hum.tmLanguage.json): generated TextMate grammar for basic `.hum` highlighting
- [docs/TOOLING.md](docs/TOOLING.md): `humfmt`, `chirp`, and first-party tooling principles
- [docs/FORMATTER.md](docs/FORMATTER.md): canonical formatting rules for `humfmt`
- [docs/NECTAR_PACKAGE_MANAGER.md](docs/NECTAR_PACKAGE_MANAGER.md): working plan for Nectar, Hum's package manager
- [docs/SEMANTIC_GRAPH_SCHEMA.md](docs/SEMANTIC_GRAPH_SCHEMA.md): `hum graph` JSON contract
- [docs/DIAGNOSTICS.md](docs/DIAGNOSTICS.md): stable diagnostic codes and repair contract
- [docs/ROADMAP.md](docs/ROADMAP.md): build order and teaching order
- [docs/CORE_LANGUAGE_SHAPE.md](docs/CORE_LANGUAGE_SHAPE.md): expected core constructs and control flow
- [docs/ENGINEERING_LENS.md](docs/ENGINEERING_LENS.md): cost, avoids, and tradeoff blocks
- [docs/PERFORMANCE_CONTRACTS.md](docs/PERFORMANCE_CONTRACTS.md): compile-time and benchmark performance enforcement
- [docs/TESTING_STRATEGY.md](docs/TESTING_STRATEGY.md): built-in tests, generated tests, fuzz, and properties
- [docs/SYSTEMS_LANGUAGE_AUDIT.md](docs/SYSTEMS_LANGUAGE_AUDIT.md): missing ecosystem pieces and systems checklist
- [docs/EXTERNAL_ADVICE_REVIEW.md](docs/EXTERNAL_ADVICE_REVIEW.md): outside systems-language advice turned into Hum design commitments
- [docs/GOVERNANCE.md](docs/GOVERNANCE.md): BDFL, RFC, stability, and evolution process
- [docs/RFC_TEMPLATE.md](docs/RFC_TEMPLATE.md): feature proposal template
- [docs/BEGINNER_EXPERIENCE.md](docs/BEGINNER_EXPERIENCE.md): path from English intent to checked Hum code
- [docs/BEGINNER_GLOSSARY.md](docs/BEGINNER_GLOSSARY.md): plain-language terms for non-programmers
- [docs/NAME_SEARCH.md](docs/NAME_SEARCH.md): naming search notes
- [docs/VOW_COMPARISON.md](docs/VOW_COMPARISON.md): what Hum should learn from Vow
- [docs/STDLIB_STRATEGY.md](docs/STDLIB_STRATEGY.md): 2026 standard library strategy
- [docs/STDLIB_CONSTITUTION.md](docs/STDLIB_CONSTITUTION.md): admission law for stable `std` APIs
- [docs/STDLIB_PRIMITIVE_RESEARCH_2026.md](docs/STDLIB_PRIMITIVE_RESEARCH_2026.md): primitive-by-primitive stdlib research sweep
- [docs/SAFETY_CRITICAL_AND_ENGINE_EDGECASES.md](docs/SAFETY_CRITICAL_AND_ENGINE_EDGECASES.md): safety-critical, realtime, and engine-grade edge-case audit
- [docs/RUNTIME_PROFILES.md](docs/RUNTIME_PROFILES.md): named profile model for normal, realtime, engine, and certified builds
- [docs/BACKEND_STRATEGY.md](docs/BACKEND_STRATEGY.md): LLVM, MLIR, and Cranelift strategy
- [examples/session_server.hum](examples/session_server.hum): first syntax sketch
- [examples/control_flow.hum](examples/control_flow.hum): loop and control-flow sketch
- [examples/task_list.hum](examples/task_list.hum): beginner-friendly program sketch
- [examples/task_list_tests.hum](examples/task_list_tests.hum): first-class test sketch

## Bootstrap Compiler

The first compiler front-end is written in Rust with `#![forbid(unsafe_code)]` and no third-party crates.

Cargo is the bootstrap build and install path for now. Hum itself should not be
positioned as "just a Cargo crate"; long-term distribution needs prebuilt
toolchains, OS package managers, editor adapters, and first-party Hum tools.

For editor and environment setup, see [docs/SETUP.md](docs/SETUP.md).

With Rust installed and Cargo on `PATH`, the full local preflight is:

```powershell
.\tools\check_all.ps1
```

Useful individual commands while developing:

```powershell
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- check examples
cargo run -- version
cargo run -- version --format json
cargo run -- graph examples/reference_surface.hum
cargo run -- graph examples/task_list.hum
cargo run -- test-skeletons examples
cargo run -- syntax
cargo run -- syntax --format textmate
```

Current CLI:

- `hum check <file-or-dir>...`: parse Hum and run Milestone 0 intent checks
- `hum version [--format human|json]`: print toolchain identity, version, target, and schema names
- `hum graph <file-or-dir>...`: emit `hum.semantic_graph.v0` JSON for tools and agents, including section line facts, task test obligations, and exact or conservative canonical `covers:` links
- `hum test-skeletons <file-or-dir>...`: print Hum `test` blocks for unlinked test obligations without executing code or writing files
- `hum syntax`: emit `hum.syntax_surface.v0` JSON for editor and tool adapters
- `hum syntax --format textmate`: emit a generated TextMate grammar from the same syntax surface
- add `--timings` to print read/parse/check timing data

## Text Hygiene

Hum docs and source files are UTF-8 without BOM. After editing Markdown,
Hum source, Rust source, TOML, or PowerShell tooling, run:

```powershell
.\tools\check_text_hygiene.ps1
```

The check rejects BOMs, invalid UTF-8, suspicious mojibake, terminal control
characters, and broken local Markdown links. Before a commit, public snapshot, or
release-style handoff, run:

```powershell
.\tools\check_all.ps1
```

See [docs/TEXT_HYGIENE_WORKFLOW.md](docs/TEXT_HYGIENE_WORKFLOW.md).

## Name

The working language name is **Hum**.

The name fits the design because it points at human-readable systems code, sits
inside the word "human", and suggests code that is smooth enough to hum along
with the machine. It is short, easy to say, and sits well next to names like
Rust, Zig, Swift, Mojo, and Carbon.

Initial web search did not find an obvious active systems programming language
named Hum or HumLang, but exact package names such as `hum` are not globally
available. See
[docs/NAME_SEARCH.md](docs/NAME_SEARCH.md).

## License

Copyright (c) 2026 Ocean Bennett

This project is licensed under the GNU Affero General Public License v3.0 with an additional visible attribution requirement. See:

- `LICENSE` for the full AGPL-3.0 license text
- `NOTICE.md` for the visible attribution requirement

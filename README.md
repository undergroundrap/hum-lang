# Hum

Hum is an intent-first systems language design draft.

The goal is low-level Rust/C++ power with Python-like readability, static types,
explicit effects, memory safety by default, and compiler-generated context for
humans and coding agents.

Hum source should scale from the small form most code wants to the explicit
contract form safety-critical work deserves.

Minimal form:

<!-- hum-example:start examples/core/minimal_add.hum -->
```hum
task add(a: Int, b: Int) -> Int {
  does:
    return a + b
}
```
<!-- hum-example:end -->

Full-contract form:

<!-- hum-example:start examples/reference_surface.hum -->
```hum
task remember_work_item(title: Text) -> Result WorkItem, WorkError {
  why:
    let a user capture work without losing the reason it matters
    # comments inside sections are preserved as section facts

  targets:
    triple: wasm32-wasi-preview1
    requires: os.clock
    requires: os.filesystem
    denies: os.network

  uses:
    clock

  changes:
    work_items

  needs:
    title is not empty

  ensures:
    new work item is saved
    new work item is not done

  protects:
    user work history

  trusts:
    local profile storage

  fails when:
    title is empty

  watch for:
    title may contain only spaces

  cost:
    time: O(1)
    space: O(1)
    check: warn

  allocates:
    one work item

  avoids:
    saving empty work items

  tradeoffs:
    local persistence is enough for the reference surface

  optimizes:
    clear review facts over clever implementation

  tests:
    remember_work_item rejects empty title

  does:
    if title is empty {
      fail WorkError.empty_title
    }

    let item = WorkItem {
      id: clock.now_text
      title: title
      done: false
    }

    save item in work_items
    return item
}
```
<!-- hum-example:end -->

## The Magic Comment Problem

In most systems languages, the sentence that matters most is the one the
compiler cannot see:

```c
// result must equal a + b -- do not change this without updating callers!
int add(int a, int b) { return a - b; }  /* nobody noticed */
```

The comment is a promise with no enforcement. The next edit — human or
AI agent — can silently turn it into a lie.

In Hum, that sentence is a checked contract. This fixture ships in the
repo with a deliberately sabotaged body:

<!-- hum-example:start fixtures/run/wrong_add_contract.hum -->
```hum
task add(a: Int, b: Int) -> Int {
  why:
    prove ensures catches a sabotaged implementation

  ensures:
    result == a + b

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    return a - b
}
```
<!-- hum-example:end -->

Running it catches the lie and blames the right party:

```text
fixtures/run/wrong_add_contract.hum:8:5: error[H0703]: task `add` did not satisfy ensures: result == a + b
  help: Fix the task body or change the contract; task blame means the caller met entry conditions but the implementation broke its promise.
```

The same discipline covers ownership words. `borrow`, `change`, and
`consume` on parameters are checked promises. Local views are narrow also:
`let view = borrow record.field` is invalidated by a later write to that
exact field, and `let view = borrow list[0]` is invalidated by later
`list_append` growth. Copying the value first is just an ordinary value.
Session V adds one equally narrow writable form: `let alias = change
record.field`. It stores the field place rather than a copy, so `set alias =
value` writes through. The alias is live only through its last straight-line
syntactic use; H0808 rejects overlapping access and H0809 rejects escape or
unsupported shapes. Distinct direct fields remain usable. This is not general
aliasing or internal-reference support.
Using a value after it moved is caught with the move site named:

```text
error[H0801]: value `value` was used after it was moved
  help: `value` moved at fixtures/ownership_check/session_j_use_after_move_fail.hum:17:5; use it before that move or create a fresh owned value.
```

Contracts can also reach back in time. `old(...)` captures a parameter's
value at task entry, so a swap that never swaps is caught by its own
promise:

```text
fixtures/run/session_t_wrong_swap_contract.hum:13:5: error[H0703]: task `swap_xy` did not satisfy ensures: result.x == old(point.y)
```

What Hum does not yet claim: `cost:`, `allocates:`, `protects:`, and
`trusts:` lines are recorded intent, graph facts, and generated
obligations today — not enforced proofs. Every checker report emits an
explicit non-claims list so the boundary between checked and declared
stays visible. The roadmap is to keep moving lines from the second
category into the first, and to never blur which is which.

## Status

This repository is a language design seed with a Milestone 0 Rust bootstrap compiler front-end.
Milestone 1 execution has started with `hum run` interpreting the first Formal Core fixtures; the report/check gates remain honest non-executing evidence surfaces.

Current version: `0.0.1` pre-alpha.

Start with [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the ground-truth map
of how the design docs fit together.

Current artifacts:

- [examples/reference_surface.hum](examples/reference_surface.hum): checked Milestone 0 source fixture covering the current reference surface
- [examples/core](examples/core): first Milestone 1 executable fixtures for `hum run` (`add`, `divide`, and `count_completed`)
- [examples/probes/writable_field_aliases.hum](examples/probes/writable_field_aliases.hum): Session V write-through, distinct-field, and straight-line last-use evidence for Program 8
- [SPEC.md](SPEC.md): broad language design draft
- [docs/LANGUAGE_REFERENCE.md](docs/LANGUAGE_REFERENCE.md): traditional reference spine for source files, items, sections, and current language status
- [docs/LANGUAGE_BUILDER_OPERATING_MODEL.md](docs/LANGUAGE_BUILDER_OPERATING_MODEL.md): operating model for how Hum selects problems, prototypes, adopts ecosystems, validates claims, and grows without losing coherence
- [docs/MILESTONE_0_GRAMMAR.md](docs/MILESTONE_0_GRAMMAR.md): current Rust bootstrap parser grammar contract
- [docs/SYNTAX_SURFACE_SCHEMA.md](docs/SYNTAX_SURFACE_SCHEMA.md): `hum.syntax_surface.v0` schema for editor-neutral syntax metadata
- [docs/CAPABILITIES_SCHEMA.md](docs/CAPABILITIES_SCHEMA.md): `hum.capabilities.v0` schema for toolchain surface discovery
- [docs/TARGET_FACTS_SCHEMA.md](docs/TARGET_FACTS_SCHEMA.md): `hum.target_facts.v0` and `hum.target_fact_record.v0` portability field and fixture schema
- [docs/EVIDENCE_REPORT_SCHEMA.md](docs/EVIDENCE_REPORT_SCHEMA.md): `hum.evidence.v0` security/trust evidence report schema
- [docs/MATH_ENGINE_BOUNDARY.md](docs/MATH_ENGINE_BOUNDARY.md): boundary for Truth Harness-style verifier integrations
- [docs/MATH_OBLIGATIONS_SCHEMA.md](docs/MATH_OBLIGATIONS_SCHEMA.md): `hum.math_obligations.v0` and `hum.math_obligation.v0` export surface for external validators
- [docs/RESOURCE_REPORT_SCHEMA.md](docs/RESOURCE_REPORT_SCHEMA.md): `hum.resource_report.v0` source-declared resource, layout, and optimization claim inventory
- [docs/HUM_CORE_CONTRACT_SCHEMA.md](docs/HUM_CORE_CONTRACT_SCHEMA.md): `hum.core_contract.v0` Core Hum executable subset and surface-to-core acceptance contract
- [docs/HUM_CORE_PREVIEW_SCHEMA.md](docs/HUM_CORE_PREVIEW_SCHEMA.md): `hum.core_preview.v0` Core Hum candidate operation and blocker preview
- [docs/HUM_CORE_LOWER_SCHEMA.md](docs/HUM_CORE_LOWER_SCHEMA.md): `hum.core_lower.v0` unverified source-mapped Core Hum artifact rows and blockers
- [docs/HUM_CORE_VERIFY_SCHEMA.md](docs/HUM_CORE_VERIFY_SCHEMA.md): `hum.core_verify.v0` non-executing Core Hum artifact invariant checks
- [docs/HUM_FULL_TYPE_CHECK_SCHEMA.md](docs/HUM_FULL_TYPE_CHECK_SCHEMA.md): `hum.full_type_check.v0` recognized Core/body statement type gate
- [docs/HUM_EFFECT_CHECK_SCHEMA.md](docs/HUM_EFFECT_CHECK_SCHEMA.md): `hum.effect_check.v0` recognized Core/body effect gate
- [docs/HUM_OWNERSHIP_CHECK_SCHEMA.md](docs/HUM_OWNERSHIP_CHECK_SCHEMA.md): `hum.ownership_check.v0` recognized local ownership fact gate
- [docs/HUM_RESOURCE_CHECK_SCHEMA.md](docs/HUM_RESOURCE_CHECK_SCHEMA.md): `hum.resource_check.v0` declared allocation/resource intent gate
- [docs/HUM_IR_CONTRACT_SCHEMA.md](docs/HUM_IR_CONTRACT_SCHEMA.md): `hum.ir_contract.v0` Hum IR ownership, carried-fact, and pass-boundary contract
- [docs/HUM_IR_READINESS_SCHEMA.md](docs/HUM_IR_READINESS_SCHEMA.md): `hum.ir_readiness.v0` source readiness and blocker report after profile checking, before Hum IR verification and lowering
- [docs/BACKEND_CONTRACT_SCHEMA.md](docs/BACKEND_CONTRACT_SCHEMA.md): `hum.backend_contract.v0` backend ladder and adapter preservation contract
- [docs/LSP_CAPABILITIES_SCHEMA.md](docs/LSP_CAPABILITIES_SCHEMA.md): `hum.lsp_capabilities.v0` preview schema for LSP adapter support
- [docs/DOCTOR_SCHEMA.md](docs/DOCTOR_SCHEMA.md): `hum.doctor.v0` setup health schema for portable repo guardrails
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): unifying architecture ground truth across language, graph, tools, profiles, stdlib, backend, and platform design
- [docs/decisions/README.md](docs/decisions/README.md): accepted design decision record index
- [docs/decisions/0006-make-resource-layout-and-comptime-explicit.md](docs/decisions/0006-make-resource-layout-and-comptime-explicit.md): accepted rule that resource, layout, compile-time, interop, and agent-facing power must stay explicit and evidence-backed
- [docs/decisions/0007-adopt-progressive-disclosure-and-migration-discipline.md](docs/decisions/0007-adopt-progressive-disclosure-and-migration-discipline.md): accepted rule that Hum must stay progressively teachable, incrementally adoptable, and evidence-backed before backend claims
- [docs/decisions/0008-adopt-swappable-backend-ladder.md](docs/decisions/0008-adopt-swappable-backend-ladder.md): accepted staged backend rule: interpreter first, Cranelift for first native proof, LLVM for mature optimized AOT, MLIR/custom only when justified by Hum facts
- [docs/decisions/0009-adopt-formal-readability-not-english-mimicry.md](docs/decisions/0009-adopt-formal-readability-not-english-mimicry.md): accepted rule that Hum readability means precise, scannable formal structure rather than executable English
- [docs/SETUP.md](docs/SETUP.md): portable setup for beginners, experts, terminals, and editors
- [CONTRIBUTING.md](CONTRIBUTING.md): contributor workflow, design gates, commit style, and PR rules
- [SECURITY.md](SECURITY.md): supported security scope, reporting path, and claim limits
- [docs/EDITOR_AND_INTEGRATION_STRATEGY.md](docs/EDITOR_AND_INTEGRATION_STRATEGY.md): editor, LSP, plugin, Visual Studio, Eclipse, and Jupyter integration strategy
- [docs/LSP_CAPABILITY_MATRIX.md](docs/LSP_CAPABILITY_MATRIX.md): current and planned LSP/editor capabilities mapped to first-party Hum CLI facts
- [docs/EDITOR_FIXTURES.md](docs/EDITOR_FIXTURES.md): broken and half-written source fixtures for editor/LSP recovery
- [docs/RELEASE_AND_VERSIONING.md](docs/RELEASE_AND_VERSIONING.md): SemVer, `0.0.1`, release checks, and tag policy
- [CHANGELOG.md](CHANGELOG.md): human-readable release history
- [docs/releases/v0.0.1.md](docs/releases/v0.0.1.md): draft `v0.0.1` pre-alpha release notes
- [docs/RELEASE_MANIFEST_SCHEMA.md](docs/RELEASE_MANIFEST_SCHEMA.md): `hum.release_manifest.v0` schema for machine-readable release manifests
- [docs/releases/v0.0.1.manifest.json](docs/releases/v0.0.1.manifest.json): machine-readable `v0.0.1` release candidate manifest
- [.github/workflows/ci.yml](.github/workflows/ci.yml): guarded hosted preflight workflow for `main`, `v*` release tags, and manual runs
- [docs/LANGUAGE_CONSTITUTION.md](docs/LANGUAGE_CONSTITUTION.md): rules Hum must not violate
- [docs/PAVED_ROAD_DOCTRINE.md](docs/PAVED_ROAD_DOCTRINE.md): one obvious default path, explicit side roads, and evidence-backed optimization doctrine
- [docs/BDFR_SCOPE_AND_SAFETY_DIRECTIVE.md](docs/BDFR_SCOPE_AND_SAFETY_DIRECTIVE.md): BDFR safety, scope, offline-first, interop, and portability directive
- [docs/SAFETY_OF_MAKER_AND_USER.md](docs/SAFETY_OF_MAKER_AND_USER.md): two-sided safety philosophy for makers, users, maintainers, and agents
- [docs/LANGUAGE_PROJECT_RISK_REGISTER_2026.md](docs/LANGUAGE_PROJECT_RISK_REGISTER_2026.md): ground-truth failure modes and safety gates for building Hum itself
- [docs/FORMAL_CORE.md](docs/FORMAL_CORE.md): precise executable core that surface Hum lowers into
- [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md): cybersecurity threat model and secure-by-default language rules
- [docs/UNSAFE_POLICY.md](docs/UNSAFE_POLICY.md): unsafe review packets, profile bans, provenance, and FFI-adjacent rules
- [docs/INTEROP_AND_PORTABILITY.md](docs/INTEROP_AND_PORTABILITY.md): C/C++/Rust/Python/Wasm interop and target portability strategy
- [docs/PORTABILITY_BOUNDARY_MODEL.md](docs/PORTABILITY_BOUNDARY_MODEL.md): target facts, platform adapters, capability absence, and artifact portability evidence
- [fixtures/target_facts](fixtures/target_facts): machine-readable target fact fixtures for Windows, Linux, WASI, and embedded-shaped targets
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
- [docs/research/2026-07-07-lattner-compiler-lessons.md](docs/research/2026-07-07-lattner-compiler-lessons.md): distilled LLVM, Swift, MLIR, and Mojo lessons for Hum's adoption, backend, and complexity discipline
- [docs/research/2026-07-07-rad-debugger-lessons.md](docs/research/2026-07-07-rad-debugger-lessons.md): distilled RAD Debugger lessons for Hum debug info, visualizers, stepping, and probe-site design
- [docs/research/2026-07-07-bellard-systems-lessons.md](docs/research/2026-07-07-bellard-systems-lessons.md): distilled Bellard lessons for small, deterministic, portable infrastructure under real constraints
- [docs/research/2026-07-07-systems-legends-lessons.md](docs/research/2026-07-07-systems-legends-lessons.md): distilled Thompson, Ritchie, Kernighan, Torvalds, Carmack, Abrash, Joy, Hejlsberg, Wirth, Stallman, and Kildall lessons for durable systems-language taste
- [docs/COMPUTING_LESSONS_SWEEP_2026.md](docs/COMPUTING_LESSONS_SWEEP_2026.md): 2026 sweep across hardware, Docker/OCI, Kubernetes, WASI, MCP, observability, networking, storage, and deployment lessons
- [docs/OPTIMIZATION_AND_DSA_STRATEGY.md](docs/OPTIMIZATION_AND_DSA_STRATEGY.md): research-gated optimization and data-structure strategy
- [docs/HASH_TABLE_RESEARCH_2501_02305.md](docs/HASH_TABLE_RESEARCH_2501_02305.md): deep note on elastic/funnel hashing and Hum's `Map` research path
- [docs/ERGONOMICS_AND_OPERATORS.md](docs/ERGONOMICS_AND_OPERATORS.md): operator, QoL, and Bevy-inspired data-access strategy
- [docs/TOOLCHAIN_2050.md](docs/TOOLCHAIN_2050.md): LSP, debugger, highlighting, profiler, and long-horizon toolchain strategy
- [docs/DEBUGGABILITY_DOCTRINE.md](docs/DEBUGGABILITY_DOCTRINE.md): rules that preserve source/value/layout/effect identity so a fast Hum debugger can exist later
- [docs/DEBUG_INFO_AND_VISUALIZER_MODEL.md](docs/DEBUG_INFO_AND_VISUALIZER_MODEL.md): target `hum.debug_info.v0` model for source maps, visualizers, probe sites, and native debug links
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
- [docs/BACKEND_STRATEGY.md](docs/BACKEND_STRATEGY.md): swappable backend ladder across interpreter, Cranelift, LLVM, MLIR, Wasm/C, and future custom backend options
- [examples/session_server.hum](examples/session_server.hum): first syntax sketch
- [examples/control_flow.hum](examples/control_flow.hum): loop and control-flow sketch
- [examples/task_list.hum](examples/task_list.hum): beginner-friendly program sketch
- [examples/task_list_tests.hum](examples/task_list_tests.hum): first-class test sketch
- [examples/probes/capability_root.hum](examples/probes/capability_root.hum): checked exact app/task capability closure with no host operation

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
cargo run -- explain H0201
cargo run -- explain H0201 --format json
cargo run -- diagnostics
cargo run -- diagnostics --format json
cargo run -- capabilities
cargo run -- capabilities --format json
cargo run -- target-facts
cargo run -- target-facts --format json
cargo run -- core-contract
cargo run -- core-contract --format json
cargo run -- core-preview examples/reference_surface.hum
cargo run -- core-preview --format json examples/reference_surface.hum
cargo run -- core-lower examples/reference_surface.hum
cargo run -- core-lower --format json examples/reference_surface.hum
cargo run -- core-verify examples/reference_surface.hum
cargo run -- core-verify --format json examples/reference_surface.hum
cargo run -- full-type-check fixtures/full_type_check/simple_pass.hum
cargo run -- full-type-check --format json fixtures/full_type_check/simple_pass.hum
cargo run -- effect-check fixtures/effect_check/simple_pass.hum
cargo run -- effect-check --format json fixtures/effect_check/simple_pass.hum
cargo run -- ownership-check fixtures/ownership_check/simple_pass.hum
cargo run -- ownership-check --format json fixtures/ownership_check/simple_pass.hum
cargo run -- run examples/probes/writable_field_aliases.hum --entry swap_xy_with_aliases --args '{x:1,y:2}'
cargo run -- run examples/probes/pure_app_entry.hum --args hello
cargo run -- ownership-check fixtures/ownership_check/session_v_program8_overlap_write_fail.hum
cargo run -- resource-check fixtures/resource_check/simple_pass.hum
cargo run -- resource-check --format json fixtures/resource_check/simple_pass.hum
cargo run -- profile-check fixtures/profile_check/simple_pass.hum
cargo run -- profile-check --format json fixtures/profile_check/simple_pass.hum
cargo run -- ir-contract
cargo run -- ir-contract --format json
cargo run -- backend-contract
cargo run -- backend-contract --format json
cargo run -- evidence examples/reference_surface.hum
cargo run -- evidence --format json examples/reference_surface.hum
cargo run -- math-obligations examples/control_flow.hum
cargo run -- math-obligations --format json examples/control_flow.hum
cargo run -- math-obligations --out-dir target/hum-math-obligations examples/control_flow.hum
cargo run -- resource-report examples/control_flow.hum
cargo run -- resource-report --format json examples/control_flow.hum
cargo run -- ir-readiness examples/reference_surface.hum
cargo run -- ir-readiness --format json examples/reference_surface.hum
cargo run -- lsp --capabilities
cargo run -- lsp --capabilities --format json
cargo run -- doctor
cargo run -- doctor --format json
cargo run -- graph examples/reference_surface.hum
.\tools\check_editor_fixtures.ps1
.\tools\check_clean_checkout.ps1
.\tools\check_tag_readiness.ps1
cargo run -- graph examples/task_list.hum
cargo run -- test-skeletons examples
cargo run -- syntax
cargo run -- syntax --format textmate
```

Current CLI:

- `hum check <file-or-dir>...`: parse Hum and run Milestone 0 intent checks
- `hum check --format json <file-or-dir>...`: emit `hum.check.v0` diagnostics JSON for editors, CI, and agents
- `hum run <file> [--entry <task>] [--args ...]`: run one structural app root when no entry is named, preserve legacy direct-task probing with `--entry`, use typed failure exit 1, and reserve exit 2 for runtime traps
- Session W's failure slice requires explicit same-root `try` or explicit
  caller-root causal wrapping; multi-call failures retain every recognized
  call site and the root origin
- `hum evidence [--format human|json] <file-or-dir>...`: emit `hum.evidence.v0` security/trust evidence status for humans, agents, and CI wrappers
- `hum math-obligations [--format human|json] [--out-dir <dir>] <file-or-dir>...`: emit `hum.math_obligations.v0` reports and optional per-obligation `hum.math_obligation.v0` files for external contract validators
- `hum resource-report [--format human|json] <file-or-dir>...`: emit `hum.resource_report.v0` source-declared resource, layout, and optimization claim inventory
- `hum ir-readiness [--format human|json] <file-or-dir>...`: emit `hum.ir_readiness.v0` source readiness and blocker facts after consuming profile-check readiness, while still blocking before IR verification and Hum IR lowering
- `hum version [--format human|json]`: print toolchain identity, version, target, and schema names
- `hum explain <H####> [--format human|json]`: explain a stable diagnostic code for humans, editors, and agents
- `hum diagnostics [--format human|json]`: list the stable diagnostic catalog for humans, editors, and agents
- `hum capabilities [--format human|json]`: list `hum.capabilities.v0` toolchain surfaces for editors, agents, and CI wrappers
- `hum target-facts [--format human|json]`: emit `hum.target_facts.v0` target-fact fields, capability families, and portability fixture records without host probing or target selection
- `hum core-contract [--format human|json]`: emit `hum.core_contract.v0` Core Hum executable subset and surface-to-core acceptance facts
- `hum core-preview [--format human|json] <file-or-dir>...`: emit `hum.core_preview.v0` Core Hum candidate operations and blockers without execution
- `hum core-lower [--format human|json] <file-or-dir>...`: emit `hum.core_lower.v0` unverified Core Hum artifact rows and blockers without execution or IR emission
- `hum core-verify [--format human|json] <file-or-dir>...`: emit `hum.core_verify.v0` non-executing Core Hum artifact invariant checks without execution or IR emission
- `hum full-type-check [--format human|json] <file-or-dir>...`: emit `hum.full_type_check.v0` recognized Core/body statement type facts and explicit blockers without execution or IR emission
- `hum effect-check [--format human|json] <file-or-dir>...`: emit `hum.effect_check.v0` recognized Core/body effect facts and explicit blockers without execution or IR emission
- `hum ownership-check [--format human|json] <file-or-dir>...`: emit `hum.ownership_check.v0` recognized local ownership facts and explicit blockers without execution or IR emission
- `hum resource-check [--format human|json] <file-or-dir>...`: emit `hum.resource_check.v0` declared allocation/resource intent facts and explicit blockers without execution or IR emission
- `hum profile-check [--format human|json] <file-or-dir>...`: emit `hum.profile_check.v0` runtime profile policy facts and explicit blockers without execution or IR emission
- `hum ir-contract [--format human|json]`: emit `hum.ir_contract.v0` Hum IR ownership, carried-fact, pass-boundary, and non-execution facts
- `hum backend-contract [--format human|json]`: emit `hum.backend_contract.v0` backend ladder and adapter preservation facts without selecting or running a backend
- `hum lsp --capabilities [--format human|json]`: list `hum.lsp_capabilities.v0` LSP adapter-preview facts without starting server mode
- `hum doctor [--format human|json]`: emit `hum.doctor.v0` setup health facts for portable repo guardrails
- `hum graph <file-or-dir>...`: emit `hum.semantic_graph.v0` JSON for tools and agents, including source-derived node IDs, source columns, folding ranges, document symbols, section line facts, task test obligations, exact or canonical-token `covers:` links, and a non-executing portability object with source-declared `targets:` facts
- `hum test-skeletons <file-or-dir>...`: print Hum `test` blocks for unlinked test obligations without executing code or writing files
- `hum syntax`: emit `hum.syntax_surface.v0` JSON for editor and tool adapters, including section hover metadata, the exact writable-field-alias form, and a semantic-token legend
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

Before a tag or private-remote promotion, prove the committed repo from a fresh local clone:

```powershell
.\tools\check_clean_checkout.ps1
```

Immediately before creating an annotated release tag, run the non-publishing tag gate:

```powershell
.\tools\check_tag_readiness.ps1
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

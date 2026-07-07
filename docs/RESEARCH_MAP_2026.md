# Hum Research Map 2026

Date: 2026-07-06

## Purpose

Hum should not jump from exciting language ideas straight into permanent syntax.

This document records the research and systems that should shape Hum before the
language, compiler, standard library, unsafe model, and runtime profiles harden.

The goal is not to worship papers. The goal is to convert research into design
constraints, diagnostics, tests, and evidence artifacts.

## Brutal Conclusion

The strongest direction for Hum is not one breakthrough.

It is a stack:

```text
small formal core
+ Rust-like ownership
+ checked intent blocks
+ profile-specific subsets
+ verified or auditable unsafe boundaries
+ reproducible builds
+ semantic graph for humans, tools, and agents
+ measured performance contracts
```

Any feature that cannot fit that stack should stay experimental.

## Current Adoption Snapshot

The current adoption research snapshot is:

- [research/2026-07-06-evidence-native-systems-language.md](research/2026-07-06-evidence-native-systems-language.md)
- [research/2026-07-07-time-space-simulation.md](research/2026-07-07-time-space-simulation.md)
- [research/2026-07-07-lattner-compiler-lessons.md](research/2026-07-07-lattner-compiler-lessons.md)
- [research/2026-07-07-rad-debugger-lessons.md](research/2026-07-07-rad-debugger-lessons.md)
- [research/2026-07-07-bellard-systems-lessons.md](research/2026-07-07-bellard-systems-lessons.md)
- [research/2026-07-07-systems-legends-lessons.md](research/2026-07-07-systems-legends-lessons.md)

It sharpens the product thesis: Hum should not compete as nicer syntax alone.
It should compete as an evidence-native systems language that turns checked
intent into semantic graph facts, effect reports, provenance, SBOMs, profile
reports, review packets, and deployable artifacts.

The near-term adoption wedges are cybersecurity tools, DevOps/SRE utilities,
and defense or air-gapped offline tooling. These wedges fit Hum's current
architecture because they value auditability, explicit effects, offline
operation, provenance, and safe defaults before they demand a huge ecosystem.

See [ADOPTION_STRATEGY_2026.md](ADOPTION_STRATEGY_2026.md) and
[research/README.md](research/README.md).

## Research Clusters

### 1. Verified Compilers And Small Cores

Research signals:

- CompCert shows that a realistic optimizing compiler can have a machine-checked
  proof that generated executable behavior matches source semantics.
- CakeML shows a language, proof ecosystem, proven-correct compiler backend, and
  bootstrapped compiler can be built together.
- Pancake shows a verification-friendly imperative systems language can target a
  verified backend and verify a realistic device driver.
- seL4 shows that high-assurance systems are possible when the trusted core is
  small, specified, and proof-oriented.

Hum lessons:

- Self-hosting is not the main safety proof. A Hum compiler written in Hum can
  still be wrong.
- Hum needs a small formal core before it needs a huge surface language.
- Every backend strategy must say what is trusted, what is checked, and what is
  merely tested.
- Long term, Hum should investigate a proof-oriented core IR or a verified
  checker for compiler outputs.

Concrete design gates:

- No feature reaches stable without a core-semantics sketch.
- `hum graph` must preserve enough information to connect source promises to IR
  nodes.
- Backend optimization passes must be explainable in terms of preserved
  contracts, not just benchmark wins.
- Self-hosting remains a milestone after diagnostics, graph stability, testing,
  fuzzing, and differential checks.

### 2. Rust Verification And Unsafe Reality

Research signals:

- RustBelt proves that Rust-style safety depends on a semantic account of unsafe
  abstractions, not just the surface borrow checker.
- Verus, Creusot, Aeneas, and Kani show multiple practical paths for verifying
  Rust programs: SMT-based proof, translation to proof assistants, deductive
  verification, and bounded model checking.
- The 2026 Rust standard library verification campaign shows that even Rust's
  standard library needs machine-checked work around unsafe code.
- New work on Rust pointer-aliasing rules shows that unsafe aliasing remains an
  active research problem in 2026.

Hum lessons:

- Hum should not claim "safer than Rust" unless unsafe, provenance, aliasing, and
  stdlib verification are first-class design topics.
- A borrow checker is necessary but not enough.
- Unsafe Hum must require a safety case, not a comment.
- Standard-library APIs that rely on unsafe internals need proof/test/evidence
  packets before they become stable.

Concrete design gates:

- Keep `docs/UNSAFE_POLICY.md` current before adding `unsafe task`.
- Keep `docs/FORMAL_CORE.md` current before adding complex ownership features.
- Reserve graph fields for proof obligations, lemma dependencies, unsafe blocks,
  trusted assumptions, and verification tool results.
- Treat `std` unsafe internals as audited packages with explicit invariants.

### 3. Memory Safety Beyond The Borrow Checker

Research signals:

- CHERI shows that hardware capabilities can provide fine-grained memory
  protection and compartmentalization with formal ISA models.
- 2026 CHERI temporal-safety work such as PoisonCap and PICASSO shows the field
  is moving toward stronger use-after-free protection with hardware/software
  cooperation.
- MSWasm shows memory safety can be specified as language-independent semantics
  with multiple enforcement strategies.
- Wuffs shows that for a narrower domain, compile-time proofs can eliminate
  buffer overflow, integer overflow, and null pointer classes while producing
  C-usable libraries.

Hum lessons:

- Hum should model provenance, bounds, allocation identity, and lifetimes in its
  semantic graph even before it can enforce all of them.
- Some profiles should be hermetic: no ambient IO, no heap, no syscalls, no hidden
  authority.
- Hum should eventually have target profiles for capability hardware and memory
  tagging.
- Parser/codec-style libraries should be designed closer to Wuffs than to casual
  general-purpose code.

Concrete design gates:

- Add profile hooks for `capability hardware`, `memory tagged`, and `software
  checked` enforcement modes later.
- Make allocation, bounds, initialization, and provenance visible in unsafe
  review packets.
- Standard-library parsers should default to Sans I/O shape: caller owns IO,
  parser owns pure state transitions.

### 4. Safety-Critical Profiles And Coding Standards

Research signals:

- FDA and ISO safety documents emphasize lifecycle evidence, risk management,
  traceability, cybersecurity, maintenance, and release artifacts.
- Ferrocene shows a Rust toolchain can move toward qualified critical-systems use
  through constrained builds and evidence.
- SPARK shows contracts and absence-of-runtime-error proof can be normal language
  practice.
- MISRust 2026 shows many MISRA-style rules are already handled by safe Rust, but
  unsafe reopens many obligations.
- Ravenscar-style realtime subsets show that strict profiles can make concurrency
  and timing analyzable by forbidding dynamic, ambiguous behavior.

Hum lessons:

- Hum needs certifiable subsets, not just a certifiable marketing page.
- Safety profiles should be smaller than normal Hum.
- Deviations should be explicit, reviewed, and emitted in evidence artifacts.
- `panic`, allocation, recursion, dynamic dispatch, floating point, time, and
  concurrency behavior are profile policy topics.

Concrete design gates:

- Add `profile safety critical`, `profile hard realtime`, and `profile certified
  toolchain` examples before adding async or unsafe.
- Nectar must eventually emit dependency evidence, SBOMs, toolchain locks,
  reproducible build manifests, and deviation records.
- `hum graph` must connect `protects:`, `needs:`, `ensures:`, tests, proofs, and
  release artifacts.

### 5. Engine Runtime, ECS, And Determinism

Research signals:

- Bevy ECS shows practical value in typed system parameters, declared mutable
  access, schedule ordering, and conflict detection.
- 2026 ECS formalization work points at deterministic-by-construction concurrent
  ECS subsets and archetype invariants.
- Game-engine determinism research shows engines used for verification workflows
  often need more reproducible behavior than they naturally provide.
- Unreal tooling shows serious engines need profiling, memory tracing, asset
  loading analysis, and data-oriented runtime visibility.

Hum lessons:

- Hum's `uses:` and `changes:` blocks can become scheduling and data-access
  contracts, not just documentation.
- `engine hot path` should require frame budgets, allocation budgets, trace
  labels, replay policy, and data layout promises.
- Order ambiguity should be a diagnostic class.
- Deterministic replay should be a language/toolchain feature for simulation,
  games, robotics, networking, and debugging.

Concrete design gates:

- Create `docs/DATA_ORIENTED_SCHEDULING.md` before adding scheduler syntax.
- Add graph fields for systems, resources, component/storage access, schedule
  edges, order ambiguities, and replay inputs.
- Keep ECS/data-oriented primitives out of core syntax until examples prove they
  beat ordinary library APIs.

### 6. Agent-Assisted Verification

Research signals:

- KVerus argues that LLM proof generation fails when agents lack structural
  dependency information and toolchain-specific metadata.
- VeruSAGE shows agent systems can complete many Rust verification tasks, but
  different models and workflows need different scaffolding.

Hum lessons:

- Hum's semantic graph is not a bonus. It is the agent-proof backbone.
- Agents should receive proof obligations, dependency graphs, lemma indexes,
  diagnostics, and source spans directly.
- The compiler must never trust an agent's proof prose. It should trust checked
  artifacts from verifiers, tests, fuzzers, and compilers.

Concrete design gates:

- Add graph nodes for generated test obligations, proof obligations, proof
  dependencies, verification tool status, and stale-proof invalidation.
- Keep diagnostics stable and machine-readable so agents repair by code, not by
  terminal text.
- Treat agent-generated code as untrusted until `hum check`, tests, fuzzers, and
  proof tools accept it.

### 7. Language Semantics And Verse

Research signals:

- The Verse Calculus is a serious attempt to give functional logic programming a
  small-step rewrite semantics.
- Its own draft warns that details are work in progress and confluence remains
  troublesome.

Hum lessons:

- We should copy Verse's formal humility, not blindly copy its language surface.
- Hum should have a small core calculus or operational semantics for executable
  constructs before clever syntax grows.
- Failure-driven or logic-style constructs are interesting, but they should stay
  outside Milestone 0 and Milestone 1 unless they simplify real systems code.

Concrete design gates:

- `fails when:` should become typed failure first, not logic programming.
- Keep ordinary control flow boring until Hum has executable examples and a clear
  type/effect model.
- If Hum later adds logic/search features, they need profile, determinism, and
  cost rules from day one.

### 8. Secure Compilation And Compartments

Research signals:

- SECOMP extends CompCert with compartmentalized C and machine-checked secure
  compilation properties.
- CHERI and MSWasm show memory safety and compartment boundaries can be designed
  at architecture, IR, runtime, and compiler levels.

Hum lessons:

- Hum should treat compartments and capabilities as first-class security design
  topics, not as framework conventions.
- Undefined behavior or unsafe failure should be containable by profile and
  compartment where possible.
- Foreign interfaces need privilege, lifetime, layout, panic, and callback
  contracts.

Concrete design gates:

- Add capability and compartment ideas to `docs/FFI_AND_ABI.md` and future
  package metadata.
- `trusts:` should distinguish human trust, toolchain trust, hardware trust,
  package trust, and runtime compartment trust.
- Security-sensitive packages should be able to declare which capabilities they
  need and which they must never receive.

### 9. Practitioner Workloads: Ops, Network, Low-Level, And Numeric

Research signals:

- ShellCheck, PowerShell, Ansible, Kubernetes, Terraform, CUE, and Dhall all
  point at the same operations lesson: text-only automation and untyped
  configuration create preventable risk.
- YANG and gNMI show that network management needs typed models, capabilities,
  transactions, telemetry subscriptions, authentication, and rollback semantics.
- Embedded Rust's `no_std` ecosystem shows that low-level users need explicit
  runtime profiles, allocation policy, layout, volatile access, target features,
  and C interop.
- PyTorch and the Python Array API show that AI/ML users need shape, dtype,
  device, determinism, numerical accuracy, zero-copy interchange, and accelerator
  semantics to be visible.
- MLIR shows that tensor, sparse, linalg, vector, shape, GPU, and quantization
  lowering are compiler topics, but they should not leak prematurely into the
  surface language.

Hum lessons:

- `Path`, `Command`, `Secret`, `IpAddr`, `Cidr`, `Port`, `Service`, `Tensor`,
  `Shape`, `DType`, `Device`, `Unit`, and `ErrorBound` should be typed reality,
  not aliases for strings and numbers.
- Dry-run, rollback, idempotence, desired state, observed state, telemetry,
  timeout, retry, determinism, numeric tolerance, and target feature facts must
  be preserved in the semantic graph.
- Most domain power belongs in standard-library and package APIs, not in core
  syntax.

Concrete design gates:

- Create `docs/OPERATIONS_MODEL.md` before hardening Nectar.
- Create `docs/NETWORK_MODEL.md` before adding protocol or device examples.
- Create `docs/NUMERIC_AND_TENSOR_MODEL.md` before adding accelerator or tensor
  language features.
- Reserve graph fields for practitioner facts before building codegen around
  them.

### 10. Time-Space Tradeoffs And Resource Evidence

Research signals:

- Williams' 2025 square-root-space simulation shows that deterministic
  multitape Turing-machine time `t` can be simulated in `O(sqrt(t log t))`
  space by reducing time-bounded computation to Tree Evaluation.
- Shalunov's 2025 circuit-evaluation follow-up reinforces that many resource
  questions are dependency-graph questions.
- Some 2025 and 2026 strengthening attempts have already been withdrawn, so Hum
  must track research confidence and source status instead of adopting exciting
  claims blindly.

Hum lessons:

- `cost:` should eventually separate time, peak space, scratch space,
  allocations, cache policy, checkpoint policy, and recomputation policy.
- Recompute-heavy optimization is valid only for pure, deterministic,
  replayable subgraphs under explicit profile policy.
- The semantic graph is also performance infrastructure: it should preserve
  dependency facts so tools can explain resource tradeoffs.
- Research-native branding is credible only if Hum records dates, withdrawals,
  caveats, and evidence links.

Concrete design gates:

- Keep [research/2026-07-07-time-space-simulation.md](research/2026-07-07-time-space-simulation.md) current before adding any automatic space-time optimizer.
- Add resource evidence linking after security/trust evidence linking exists.
- Add memory-pressure profile vocabulary before recompute/cache optimization.
- Never market theoretical results as direct real-hardware guarantees.

### 11. Debuggers, Visualizers, And Source-Mapped Toolchains

Research signals:

- RAD Debugger practice shows that debugger quality depends on high-density UI,
  fast debug-info reads, process control, linker/debug-info strategy, and domain
  visualizers.
- Existing platform debug formats such as DWARF and PDB are necessary for
  compatibility, but supporting them directly can create expensive abstraction
  layers.
- Large projects can hit real debug-info scale limits, especially through type
  expansion, templates, generated code, closures, lambdas, and generic-heavy
  code.
- Stepping optimized native code is a many-to-many source map problem, not a
  simple line-to-address lookup.
- Conditional breakpoints, dynamic profiling, and manual instrumentation are
  closely related observation workflows.

Hum lessons:

- Debuggability is part of language semantics and backend design, not an editor
  extra.
- Hum should design `hum.debug_info.v0` as the first-party authority for Hum
  facts, with DWARF/PDB as compatibility bridges.
- Type-attached visualizers should be language-visible metadata so packages can
  make domain values inspectable without per-machine debugger setup.
- Debug probe sites should be explicit profile artifacts, source-linked, and
  absent from release profiles that forbid them.
- Optimizers must preserve enough provenance to make stepping, profiling, and
  debugging honest under inlining, reordering, generated checks, and removed
  code.

Concrete design gates:

- Keep [DEBUGGABILITY_DOCTRINE.md](DEBUGGABILITY_DOCTRINE.md) current before
  adding native backend claims.
- Keep [DEBUG_INFO_AND_VISUALIZER_MODEL.md](DEBUG_INFO_AND_VISUALIZER_MODEL.md)
  current before a DAP implementation.
- Add semantic graph slots for debug info ids, visualizer ids, source-map
  provenance, and probe-site ids before lowering becomes executable.
- Treat "debugger faster than logging" as a user-experience gate, not a slogan.

### 12. Bellard-Style Small Infrastructure And Constraint Engineering

Research signals:

- Bellard's public project index spans small compilers, JavaScript engines,
  emulators, codecs, compression, telecommunications, and numeric computation,
  with many projects built around compact, portable, dependency-light artifacts.
- QuickJS demonstrates that a small embeddable runtime can still target strong
  language compatibility, low startup time, and simple distribution.
- MicroQuickJS demonstrates that strict subsets can be a strength for embedded
  budgets when they reject expensive or error-prone behavior.
- TCC demonstrates that integrated compiler pipelines can make compile/run loops
  dramatically faster and usable as dynamic-code infrastructure.
- JSLinux, TinyEMU, QEMU, and FFmpeg show that foundational infrastructure can be
  both portable and useful far beyond its original author.
- TSAC highlights deterministic artifacts across hardware and software
  configurations as a product requirement, not a nice-to-have.

Hum lessons:

- Smallness is a systems feature. Binary size, memory floor, startup time,
  dependency count, and artifact determinism should be reportable facts.
- A strict subset can make Hum stronger when tied to a named profile and useful
  diagnostics.
- Foundational infrastructure beats flashy demos: interpreter, graph, profiler,
  debug info, package/build evidence, and portable runtime artifacts compound.
- Dynamic compilation, eval-like behavior, and native plugins need explicit
  capability and profile boundaries before they become Hum power.
- Portability should eventually be proved by executable artifacts, not only by
  architecture prose.

Concrete design gates:

- Add a `footprint constrained` profile before claiming embedded or tiny-runtime
  credibility.
- Add binary-size, startup-time, memory-floor, dependency-count, and deterministic
  artifact vocabulary to future backend/profile reports.
- Keep early executable semantics small enough that one expert can audit the
  critical path.
- Require serious baselines and hardware details before performance or footprint
  claims.
- Prefer portable process/Wasm/interpreter boundaries before native plugin or
  eval-like power.

### 13. Systems Legends And Durable Taste

Research signals:

- Ritchie and Thompson show that language and operating-system design improve
  together when the core is small enough to port, rebuild, and understand.
- Thompson's trusting-trust lecture makes compiler provenance part of security,
  not a separate supply-chain concern.
- Kernighan shows that documentation, examples, and precise teaching can make a
  terse systems model become shared culture.
- Torvalds shows that infrastructure must survive maintainer review, distributed
  patches, scale, and workflow pressure.
- Carmack and Abrash show that serious performance requires measurement,
  hardware facts, data layout, and algorithmic structure.
- Joy, Hejlsberg, Wirth, Stallman, and Kildall pull in different directions, but
  converge on useful tools, stable boundaries, source availability, simplicity,
  and practical adoption paths.

Hum lessons:

- Durable systems-language taste is not one aesthetic. Hum should synthesize
  small mechanisms, strong tools, clear docs, measured performance, repairable
  artifacts, and explicit platform boundaries.
- The language reference is part of the product, not a later book project.
- Compiler trust, rebuildability, license clarity, and provenance belong in the
  core architecture.
- Adoption requires compatibility with editors, terminals, networks, platforms,
  and existing developer habits, not purity alone.
- Feature subtraction is a design action. A profile that removes expensive or
  risky behavior can be stronger than a surface that permits everything.

Concrete design gates:

- Start a traditional language reference before broadening syntax beyond Core
  Hum.
- Do not promote a major feature until it can pass the combined systems legends
  test in the research note.
- Create a portability-boundary document before claiming serious embedded,
  enterprise, or cross-platform readiness.
- Keep maintainer workflow, documentation examples, and reproducible performance
  reports in the feature admission path.
- Keep hidden cloud, telemetry, proprietary, or unverifiable toolchain steps out
  of the core path.

### 14. Deployment, Containers, Observability, And Agent Tools

Research signals:

- OCI standardizes container image, runtime, and distribution behavior, making
  container compatibility an adoption requirement for serious infrastructure.
- Docker security guidance shows containers rely on namespaces, cgroups, daemon
  protection, capabilities, signatures, and hardening; containers are useful but
  not magic isolation.
- Kubernetes Pod Security Standards show privileged containers, host namespaces,
  host paths, and extra capabilities need explicit policy.
- WASI 0.3 and the Component Model show capability-based sandboxing and
  composable async components are becoming a serious portable runtime story.
- MCP makes agent tool integration mainstream, but its own specification and
  security guidance treat tools as arbitrary code execution with consent,
  authorization, SSRF, token, session, and local compromise risks.
- OpenTelemetry shows traces, metrics, logs, baggage, profiles, and semantic
  conventions are production interfaces, not optional decoration.

Hum lessons:

- Deployment facts must enter the semantic graph before Hum claims production
  readiness.
- Docker/OCI/Kubernetes compatibility is required, but containerization must not
  be treated as a proof of safety.
- Agent-callable tools need schemas, capabilities, dry-run behavior, redaction,
  audit trails, and sandbox boundaries.
- Observability should be source-linked and security-aware.
- Wasm/WASI should be the preferred early untrusted plugin boundary.

Concrete design gates:

- Add `containerized service` and `agent tool sandbox` runtime profiles.
- Create `docs/DEPLOYMENT_AND_CONTAINER_MODEL.md` before image generation or
  Kubernetes examples.
- Create `docs/AGENT_TOOL_SECURITY_MODEL.md` before MCP or agent tool servers.
- Reserve graph fields for ports, mounts, users, capabilities, resource budgets,
  health checks, telemetry schemas, tool schemas, and agent-callable authority.
- Keep Docker, MCP, remote packages, generated code execution, and native plugin
  execution out of Milestone 0.

## Design Rules From The Research

1. Small core first.
2. Profiles before powerful features.
3. Proof obligations before proof marketing.
4. Unsafe review packets before unsafe syntax.
5. Reproducible builds before safety claims.
6. Semantic graph before agent workflows.
7. Deterministic replay before engine claims.
8. Verified or auditable stdlib internals before stable `std` prestige.
9. Compiler trust boundaries must be documented.
10. Research claims need source status, dates, caveats, and confidence labels.
11. Real-world authority, mutation, hardware, network, storage, deployment,
    container, observability, agent-tool, and numeric facts must be typed before
    they are optimized.
12. If a feature cannot explain its behavior to a beginner, a senior engineer, a
    verifier, a debugger, an operator, and an agent, it is not ready.
13. Progressive disclosure, migration tooling, and staged compatibility are part
    of language design, not post-launch cleanup.
14. Debuggability, visualizers, profiling, and source maps must be designed with
    the language and backend, not postponed to editor plugins.
15. Smallness, startup time, memory floor, dependency count, and deterministic
    artifacts are systems-language requirements, not polish.
16. Durable language taste requires small mechanisms, clear documentation,
    measured performance, maintainer workflow, repairable artifacts, and explicit
    platform boundaries at the same time.

## Research Debt Still Open

Hum still needs deeper study before hardening these areas:

- formal executable core semantics
- type system and effect system
- ownership, borrowing, and provenance model
- unsafe, FFI, and ABI policy
- concurrency memory model
- realtime and WCET strategy
- deterministic floating-point policy
- package trust and dependency evidence
- proof language versus external verifier integration
- compiler IR and optimization correctness strategy
- executable debug/profiler implementation and source-map validation
- footprint-constrained profile and deterministic artifact model
- target-fact graph fields and profile diagnostics
- data-oriented scheduling and ECS-like storage contracts
- operations model for dry-run, rollback, idempotence, drift, and telemetry
- network model for typed addresses, protocol parsing, transactions, and
  telemetry subscriptions
- numeric and tensor model for shape, dtype, device, units, tolerance,
  determinism, and accelerator lowering
- deployment and container model for Docker/OCI/Kubernetes, runtime authority,
  resource budgets, image evidence, and observability
- space-time resource model for peak space, scratch space, checkpointing,
  recomputation, cache policy, and memory-pressure profiles
- agent tool security model for MCP/CLI schemas, consent, capabilities, dry-run,
  sandboxing, and audit trails

## Near-Term Recommendation

After the first formal-core and unsafe/security pass, do this order:

1. Keep `docs/FORMAL_CORE.md`, `docs/SECURITY_MODEL.md`, and `docs/UNSAFE_POLICY.md` as gates for new executable, security-sensitive, and unsafe constructs.
2. Write `docs/DEPLOYMENT_AND_CONTAINER_MODEL.md` with Docker/OCI/Kubernetes,
   runtime capabilities, resource budgets, image evidence, health checks,
   signals, and observability.
3. Write `docs/AGENT_TOOL_SECURITY_MODEL.md` with MCP/CLI schemas, consent,
   capabilities, dry-run behavior, sandboxing, prompt-injection risk, and audit
   trails.
4. Write `docs/OPERATIONS_MODEL.md` with dry-run, rollback, idempotence, desired
   state, drift, telemetry, and deployment-plan semantics.
5. Write `docs/NETWORK_MODEL.md` with typed addresses, protocol parsing,
   transactions, telemetry subscriptions, and management-plane security.
6. Write `docs/STORAGE_MODEL.md` with durability, atomicity, crash consistency,
   format versioning, and migration rules.
7. Write `docs/NUMERIC_AND_TENSOR_MODEL.md` with units, exact/approx values,
   shape, dtype, device, tolerance, determinism, and accelerator lowering.
8. Keep `docs/DEBUG_INFO_AND_VISUALIZER_MODEL.md` current as source maps,
   optimized-code honesty states, visualizer hints, and debug probe sites become
   executable facts.
9. Write `docs/PACKAGE_AND_BUILD.md` with Nectar profiles, reproducibility, and
   evidence packets.
10. Write `docs/FFI_AND_ABI.md` with layout, panic, ownership, callback, and
    compartment rules.
11. Write `docs/CONCURRENCY_MODEL.md` with memory-ordering names, lock ordering,
    cancellation, and scheduling assumptions.
12. Write `docs/DATA_ORIENTED_SCHEDULING.md` with Bevy/ECS lessons translated
    into Hum's `uses:` and `changes:` model.
13. Only then expand syntax beyond Milestone 1.

## Brutal Assessment

Hum has enough vision.

Now it needs research discipline.

The biggest risk is pretending that readable syntax plus Rust-like ownership is
already enough. It is not. The world-class move is to make every feature enter
through a gate: semantics, profile impact, diagnostics, tooling, verification,
performance, and pedagogy.

If Hum follows that discipline, the language can be ambitious without becoming
reckless.

## Sources

- CompCert project: https://compcert.org/
- CakeML project: https://cakeml.org/
- seL4 verification overview: https://sel4.systems/Verification/
- RustBelt project and publications: https://plv.mpi-sws.org/rustbelt/
- Verus tutorial and reference: https://verus-lang.github.io/verus/guide/overview.html
- Kani Rust verifier: https://model-checking.github.io/kani/
- Kani: A Model Checker for Rust, 2026: https://arxiv.org/abs/2607.01504
- Verifying the Rust Standard Library, 2026: https://arxiv.org/abs/2606.17374
- Towards verifying unsafe Rust programs against Rust's pointer-aliasing restrictions, 2026: https://arxiv.org/abs/2603.28326
- Verifying Device Drivers with Pancake, 2025: https://arxiv.org/abs/2501.08249
- KVerus, 2026: https://arxiv.org/abs/2605.03822
- VeruSAGE, 2025/2026: https://arxiv.org/abs/2512.18436
- CHERI project: https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/
- PoisonCap, 2026: https://arxiv.org/abs/2605.13210
- PICASSO, 2026: https://arxiv.org/abs/2602.09131
- MSWasm, revised 2026: https://arxiv.org/abs/2208.13583
- Wuffs: https://github.com/google/wuffs
- MISRust, 2026: https://arxiv.org/abs/2605.23490
- Bevy ECS query docs: https://docs.rs/bevy_ecs/latest/bevy_ecs/system/struct.Query.html
- Bevy ECS system docs: https://docs.rs/bevy_ecs/latest/bevy_ecs/system/index.html
- The Essence of Entity Component System, 2026: https://arxiv.org/abs/2606.14919
- Exploring Concurrency in the ECS Pattern, 2025: https://arxiv.org/abs/2508.15264
- On Determinism of Game Engines used for Simulation-based Autonomous Vehicle Verification: https://arxiv.org/abs/2104.06262
- The Verse Calculus: https://simon.peytonjones.org/assets/pdfs/verse-conf.pdf
- SECOMP, 2024/2025: https://arxiv.org/abs/2401.16277
- Williams, "Simulating Time With Square-Root Space", 2025: https://arxiv.org/abs/2502.17779
- Shalunov, "Improved Bounds on the Space Complexity of Circuit Evaluation", 2025: https://arxiv.org/abs/2504.20950
- Henzinger, Pyne, Ragavan, "Catalytic Tree Evaluation From Matching Vectors", 2026: https://arxiv.org/abs/2602.14320
- Nye, withdrawn "TIME[t] subset SPACE[O(sqrt(t))] via Tree Height Compression", 2025/2026: https://arxiv.org/abs/2508.14831
- Asadi and Cleve, withdrawn "Polynomial-Time Almost Log-Space Tree Evaluation by Catalytic Pebbling", 2026: https://arxiv.org/abs/2604.02606
- Practitioner pain sweep sources: see [PRACTITIONER_PAIN_SWEEP_2026.md](PRACTITIONER_PAIN_SWEEP_2026.md)
- Computing lessons sweep sources: see [COMPUTING_LESSONS_SWEEP_2026.md](COMPUTING_LESSONS_SWEEP_2026.md)
- RAD Debugger lessons: see [research/2026-07-07-rad-debugger-lessons.md](research/2026-07-07-rad-debugger-lessons.md)
- Bellard systems lessons: see [research/2026-07-07-bellard-systems-lessons.md](research/2026-07-07-bellard-systems-lessons.md)
- Systems legends lessons: see [research/2026-07-07-systems-legends-lessons.md](research/2026-07-07-systems-legends-lessons.md)

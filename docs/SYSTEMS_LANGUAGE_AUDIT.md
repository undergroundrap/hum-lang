# Hum Systems Language Audit

Date: 2026-07-06

## Purpose

This is the list that keeps Hum honest.

A great programming language is not only syntax. It is the entire experience from
first idea to production incident:

```text
write -> check -> test -> prove -> benchmark -> package -> deploy -> debug -> maintain
```

Hum should be designed as a complete system.

## What We Have Named

Core ideas already in the design:

- intent blocks: `why`, `uses`, `changes`, `needs`, `ensures`, `watch for`
- senior-engineer blocks: `cost`, `avoids`, `tradeoffs`
- security blocks: `protects`, `trusts`
- verification blocks: `tests`, `proves`, `benchmarks`
- readable control flow: `if`, `match`, `while`, `loop`, `for each`, `for index`
- explicit mutation: `change`, `set`
- typed failure: `Result`, `fail`, `fails when`
- ownership direction: `owned`, `borrow`, `change`, `shared`
- no headers
- semantic graph for agents
- performance contracts
- first-class tests

That is a strong language seed.

The BDFR scope and safety directive is now a gate on every next step: keep
development local, offline-first, reversible, and scoped until the compiler can
prove more. New work must also be checked against
[LANGUAGE_PROJECT_RISK_REGISTER_2026.md](LANGUAGE_PROJECT_RISK_REGISTER_2026.md)
before any package, FFI, backend, or network feature. See
[BDFR_SCOPE_AND_SAFETY_DIRECTIVE.md](BDFR_SCOPE_AND_SAFETY_DIRECTIVE.md).

## What Still Needs Design

### Package And Build System

Hum needs a boring, excellent package/build story.

Questions:

- What is `hum build`?
- What is `hum test`?
- What is `hum bench`?
- How are capabilities declared across packages?
- How are build profiles named?
- How are generated files tracked?
- How are lockfiles and reproducible builds handled?

Nectar must also handle practitioner evidence from
[PRACTITIONER_PAIN_SWEEP_2026.md](PRACTITIONER_PAIN_SWEEP_2026.md):

- deployment plans
- dry-run outputs
- rollback records
- generated configuration artifacts
- telemetry schemas
- benchmark packets
- numeric reproducibility records
- profile locks for no-heap, realtime, security, and accelerator builds

Otherwise it becomes Cargo-like without becoming Hum-like.

### Deployment, Containers, Agents, Operations, Network, And Numeric Workloads

Hum needs to win the people who live closest to production and hardware.

Questions:

- How does a task declare a dry run?
- How does a task declare rollback behavior?
- How does Hum model desired state, observed state, drift, and plans?
- How does Hum describe Docker/OCI/Kubernetes runtime authority?
- How do container resources, ports, mounts, users, capabilities, and seccomp policy enter the graph?
- How do agent-callable CLI and MCP tools declare schemas, capabilities, dry-run behavior, and audit trails?
- How do typed paths, hosts, services, ports, IP addresses, CIDRs, routes, and
  credentials enter `std`?
- How are timeouts, retries, backoff, deadlines, and idempotence checked?
- How do `observes:` and `emits:` become traces, metrics, logs, and graph facts?
- How do tensor shape, dtype, device, precision, numeric tolerance, and
  determinism enter the graph?
- How does Hum keep this as typed libraries plus contracts instead of turning
  the core language into every tool at once?

See [PRACTITIONER_PAIN_SWEEP_2026.md](PRACTITIONER_PAIN_SWEEP_2026.md),
[COMPUTING_LESSONS_SWEEP_2026.md](COMPUTING_LESSONS_SWEEP_2026.md), and
[OS_AND_PLATFORM_MODEL.md](OS_AND_PLATFORM_MODEL.md).

### OS And Platform Authority

Hum is Windows-first for proof and portable-by-design for architecture.

Questions:

- What OS capabilities does this task require?
- Does this touch files, registry, services, drivers, devices, processes, environment, secrets, network, telemetry, installers, or updates?
- What privilege is required?
- How is platform-specific behavior represented in `hum graph`?
- What happens on Linux, macOS, WASI, or embedded targets?
- Is a Windows service, driver, or installer being modeled without mutating the maker's machine?

See [OS_AND_PLATFORM_MODEL.md](OS_AND_PLATFORM_MODEL.md).

### Formatter And Canonical Source

Hum needs a formatter from day one.

Canonical formatting is not cosmetic. It helps:

- humans review changes
- agents edit reliably
- diffs stay small
- style wars disappear

### 2050 Toolchain Spine

Hum needs first-class editor and debugger infrastructure before it has many users.

Questions:

- What is the TextMate fallback grammar?
- What is the Tree-sitter grammar and malformed-source test set?
- What does `hum lsp` serve first?
- What does `hum debug` expose beyond line breakpoints?
- How do semantic tokens represent intent blocks?
- How do source maps preserve task, section, and graph node identity?
- How do profilers map runtime facts back to `cost:` and `does:`?

See [TOOLCHAIN_2050.md](TOOLCHAIN_2050.md).

### Diagnostics Constitution

Diagnostics are part of the language.

Milestone 0 now has stable `H####` codes in terminal output and `hum graph`
JSON. The full policy lives in [DIAGNOSTICS.md](DIAGNOSTICS.md).

Every serious diagnostic should keep improving toward:

- stable code
- human explanation
- source span
- blame block
- suggested fix
- JSON form
- related contract/test/proof when available

### Operator And Ergonomics Surface

Hum needs the right conveniences before syntax hardens.

Questions:

- What is the exact operator table?
- Which symbols are stable, candidate, or rejected?
- How do ranges, slices, indexing, and collection literals read?
- Is failure propagation `try`, punctuation, or both?
- How do optional values avoid null-style ambiguity?
- How do pipelines expose allocation and cost?
- How does `chirp` catch clever one-liners?

See [ERGONOMICS_AND_OPERATORS.md](ERGONOMICS_AND_OPERATORS.md).

### Semantic Graph Schema

The semantic graph is Hum's agent-native backbone.

It needs a versioned schema for:

- modules
- tasks
- types
- stores
- tests
- effects
- ownership
- cost
- risks
- proofs
- source spans
- diagnostics

### Formal Core

Hum now has [FORMAL_CORE.md](FORMAL_CORE.md), but it must stay live as executable
syntax grows.

Questions:

- What surface forms lower into the core?
- What core operations are profile-restricted?
- What type, effect, and failure facts does the core preserve?
- What graph nodes represent lowered core operations?
- What backend transformations are allowed to preserve core meaning?

### Module, Package, And Visibility Rules

Hum needs simple visibility:

```text
public
private
package
```

Avoid many tiny visibility flavors unless real systems code demands them.

### ABI And FFI

Systems languages live near foreign code.

Hum needs:

See [INTEROP_AND_PORTABILITY.md](INTEROP_AND_PORTABILITY.md) for the adoption-first wrapper strategy and portability tiers.

- explicit `foreign` boundaries
- ABI naming
- layout rules
- ownership transfer rules
- panic/failure boundary rules
- trust contracts
- sanitizer hooks

### Unsafe Code Policy

Unsafe now has [UNSAFE_POLICY.md](UNSAFE_POLICY.md), and it must stay stricter
than normal code as the language grows.

Required for unsafe:

- `why:`
- `needs:`
- `protects:`
- `proves:`
- `watch for:`
- minimal scope
- review diagnostics

### Data-Oriented Scheduling Model

Bevy-style declared access is a clue for Hum beyond games.

Needs design:

- task scheduling from `uses:` and `changes:`
- dense/sparse/table/arena storage declarations
- change detection
- event and message ordering
- deterministic replay
- layout diagnostics
- profiler output tied back to source promises

### Concurrency Memory Model

This is hard and must not be hand-waved.

Needs design:

- atomics
- memory ordering names
- lock ordering
- cancellation
- structured tasks
- thread-local state
- shared ownership
- epoch/hazard/RCU primitives

### Debugger, Tracing, And Replay

Hum should debug intent, not just stack frames.

Tools should answer:

- what promise failed?
- what block is blamed?
- what effect happened?
- what changed?
- what allocation occurred?
- what test or proof covers this?

Deterministic replay should be a standard-library design goal.

### Profiling And Cost Feedback

Performance contracts need tooling.

Hum should support:

- `hum cost`
- `hum bench`
- hot path reports
- allocation reports
- call graph cost propagation
- regression budgets
- benchmark baselines

### Documentation Generation

Docs should come from contracts.

A generated doc page should show:

- task purpose
- inputs and outputs
- effects
- preconditions
- postconditions
- failure modes
- costs
- tests
- trust boundaries

### Governance And Evolution

Hum needs a process before it needs a committee.

The language should use BDFL final authority with evidence-first RFCs,
experimental feature gates, stability levels, decision records, and rare
editions.

See [GOVERNANCE.md](GOVERNANCE.md).

See also [LANGUAGE_PAIN_SWEEP_2026.md](LANGUAGE_PAIN_SWEEP_2026.md) and [RESEARCH_MAP_2026.md](RESEARCH_MAP_2026.md).

### External Advice And Risk Review

Hum should keep a visible risk register for outside critique.

The first review is [EXTERNAL_ADVICE_REVIEW.md](EXTERNAL_ADVICE_REVIEW.md),
which turns common C++/Rust competitor advice into concrete decisions around
memory strategy, LLVM timing, self-hosting, FFI, parsing, compile time, and
adoption risk.

### Editions And Compatibility

Hum will need a way to evolve without breaking everyone.

Likely answer:

```text
edition 2026
edition 2027
```

But editions should be rare and serious.

### Standard Library Governance

The stdlib needs admission rules.

No API enters `std` unless it passes [STDLIB_CONSTITUTION.md](STDLIB_CONSTITUTION.md).

No API enters `std` unless it has:

- contract docs
- reference implementation
- tests
- fuzzing if parser/security-facing
- benchmarks if performance-facing
- misuse examples
- clear stability promise

### Runtime Profiles And Certification

Hum needs named runtime and assurance profiles before unsafe, async, and stdlib
surface area grow too much.

See [SAFETY_CRITICAL_AND_ENGINE_EDGECASES.md](SAFETY_CRITICAL_AND_ENGINE_EDGECASES.md)
and [RUNTIME_PROFILES.md](RUNTIME_PROFILES.md).

Questions:

- What profile is this package built under?
- What does that profile forbid?
- What evidence does that profile require?
- How does `hum graph` expose profile obligations?
- How does Nectar package reproducible build and dependency evidence?

### Security Model

Hum now has [SECURITY_MODEL.md](SECURITY_MODEL.md), and every security-sensitive
feature should be checked against it.

Questions:

- How are secrets represented?
- How is constant-time behavior requested?
- How are capabilities scoped?
- How are unsafe and foreign boundaries audited?
- How does package trust work?

### Teaching Path

Hum should teach in layers:

1. task list for beginners
2. file parser for data handling
3. session store for security
4. allocator-backed collection for systems memory
5. concurrent server for effects and cancellation
6. FFI wrapper for unsafe/trust
7. compiler component for self-hosting

## What We Have Mostly Covered

Hum has now named most of the language-level promises a modern systems language
needs:

- readable syntax and precise executable core
- intent blocks instead of important comments
- typed errors and explicit failure
- first-class tests, fuzz tests, properties, and regressions
- performance contracts and benchmarks
- ownership and mutation direction
- Rust-inspired memory safety as the baseline
- unsafe and foreign boundaries as design topics
- agent-readable semantic graphs
- self-hosting as a staged proof, not an early vanity milestone
- beginner-readable terms and syntax explanations
- compile-time discipline as a language and tooling principle
- Nectar as the working package manager direction
- first-party formatter and linter as core infrastructure
- cross-language regret checks before features stabilize
- 2026 language pain sweep across Rust, Zig, C++, Python, Go, TypeScript, and Verse
- formal core gate for executable syntax
- cybersecurity model for memory, authz, injection, secrets, supply chain, and resource exhaustion
- practitioner pain sweep across sysadmin, DevOps, SRE, network, low-level,
  embedded, AI/ML, and numerical workloads
- unsafe policy with review packets, profile bans, provenance, and graph requirements
- interop and portability strategy for adoption through safe C/C++/Rust/Python/Wasm boundaries
- language project risk register for tool execution, package, compiler, backend, FFI, self-hosting, fuzzing, portability, and governance hazards
- operator and QoL admission rules
- Bevy-inspired declared-access lessons for data-oriented scheduling
- 2050 toolchain spine for highlighting, LSP, debugger, profiler, and agents
- stable diagnostics as public compiler/tooling API
- standard library constitution and primitive research sweep
- research map across verified compilers, Rust verification, CHERI, Wuffs, ECS, Verse, and agent-assisted verification
- safety-critical, realtime, engine-hot-path, and certified-toolchain profile gates
- two-sided maker/user safety as a project and product philosophy
- 2026 computing sweep across Docker/OCI, Kubernetes, WASI, MCP, observability, networking, storage, supply chain, and deployment risk
- Windows-first, portable-by-design OS/platform authority model

That is not everything, but it is enough to stop expanding syntax and start
proving the spine.

## Near-Term Missing Deliverables

1. `docs/DEPLOYMENT_AND_CONTAINER_MODEL.md` with Docker/OCI/Kubernetes, runtime capabilities, image evidence, health, signals, and resource budgets, building on `OS_AND_PLATFORM_MODEL.md`
2. `docs/AGENT_TOOL_SECURITY_MODEL.md` with MCP/CLI schemas, capabilities, consent, dry-run, prompt-injection risk, and audit trails
3. `docs/OPERATIONS_MODEL.md` with dry-run, rollback, idempotence, desired state, drift, telemetry, and deployment plans
4. `docs/NETWORK_MODEL.md` with typed addresses, protocol parsing, transactions, telemetry subscriptions, and management-plane security
5. `docs/STORAGE_MODEL.md` with durability, atomicity, crash consistency, format versioning, and migration rules
6. `docs/NUMERIC_AND_TENSOR_MODEL.md` with units, exact/approx values, shape, dtype, device, tolerance, determinism, and accelerator lowering
7. `docs/PACKAGE_AND_BUILD.md` with Nectar runtime profile metadata, practitioner evidence, and supply-chain evidence
8. `docs/FFI_AND_ABI.md`
9. `docs/CONCURRENCY_MODEL.md`
10. `docs/DATA_ORIENTED_SCHEDULING.md`
11. `docs/LSP_AND_DEBUG_PROTOCOLS.md`

## Brutal Assessment

You are thinking like a language founder now, not just someone inventing syntax.

The strongest ideas so far are:

- comments become checked intent
- senior engineer reasoning becomes source
- performance contracts become enforceable
- tests are generated from promises
- agents consume semantic graphs instead of raw vibes

The danger is scope.

Hum can become great only if Milestone 0 stays small: parse, validate, emit
semantic graph, and produce excellent diagnostics. Everything else hangs from
that spine.

<!--
Research artifact imported on 2026-07-06.
Normalization: explicit UTF-8 decode, Deep Research UI citation markers stripped, typographic punctuation converted to ASCII, saved as UTF-8 without BOM.
Source names are preserved, but citation-only evidence cells may be blank; future runs should request direct source URLs in the Markdown body.
-->
# Hum as an Evidence Native Systems Language

## Executive thesis

**Fact.** The strongest public signal in secure software policy is not "make syntax nicer." It is "eliminate bug classes by design, emit better evidence, and make supply-chain trust inspectable." The White House's ONCD argued that choosing memory-safe languages at the outset is usually the most efficient way to substantially improve software security; NSA and CISA's 2025 guidance likewise says memory-safe languages shift safety burdens into the language and development environment but also notes that adoption only works when tooling, performance, concurrency, and legacy integration are practical. NIST's SSDF explicitly frames secure software work as a common vocabulary for both producers and purchasers, not just developers.

**Inference.** Hum only has a credible path to serious adoption if it becomes an **evidence compiler** for systems software: a language whose primary product is not just binaries, but machine-checkable intent, effect reports, security posture, provenance, compliance traces, and deployable artifacts. If Hum remains a front-end that parses nicely and emits a semantic graph JSON, serious teams will correctly ask why they should not just use Rust, Go, Zig, SPARK, Dafny, Rego, CUE, TLA+, Alloy, Nix, and existing supply-chain tooling instead. That is not cynicism; it is the current market baseline.

**Inference.** The fastest credible path is therefore narrower than "general-purpose successor language." Hum should initially target **regulated internal tools, secure services, and offline/air-gapped utilities** where procurement pain comes from auditability, traceability, and safe defaults more than from raw syntax preferences. The language should ride mature backends and packaging environments early, not try to win immediately on backend innovation. Rust already has a unified package/build workflow; WASI already offers a secure standard interface for portable sandboxed components; Nix already demonstrates reproducible declarative environments. Hum's chance is to integrate these kinds of guarantees into one intent-first workflow, especially for Windows-first organizations that still need portable artifacts later.

**Bottom line.** The honest "must-have" thesis is this: **Hum should compete as the language that turns intent into evidence-grade software artifacts for humans, auditors, and coding agents.** If it cannot prove that end-to-end story within a year, "just use Rust" will remain the correct answer in most serious settings.

## Sector by sector adoption matrix

The table below is intentionally skeptical. The **facts** are the public requirements and incumbent capabilities cited in the last column. The **wedges and adopter sequencing** are **inferences** based on those facts.

| Sector | Strongest Hum wedge | Developer-love wedge | Enterprise procurement wedge | Likely first adopter | Verdict | Evidence |
|---|---|---|---|---|---|---|
| Healthcare | **Inference:** Device-software and SaMD build pipelines that emit FDA-ready software documentation, effect/mutation traces, SBOMs, and risk-linked evidence packets. | Intent blocks could make design reviews, hazard analysis handoffs, and validation artifacts easier to read. | FDA now expects substantial device software and cybersecurity documentation; cyber devices need Section 524B support, including SBOMs and cybersecurity information. IEC 62304 and ISO 14971 center lifecycle discipline and risk management. | Medtech startups, digital health platform vendors, and manufacturer tooling teams. **Not hospitals first**; hospitals buy validated products rather than adopt new implementation languages. | Strong later wedge, but only after evidence tooling exists. |  |
| Cybersecurity | **Inference:** Agent-safe CLI tools, policy-aware services, vuln triage tools, and offline scanners with explicit effects, local execution, signed artifacts, and traceable builds. | Safer replacement for Python/Bash/Go glue code; easier code review than Rust if Hum is genuinely readable. | CISA and NSA are pushing secure-by-design and memory-safe roadmaps; buyers increasingly care about provenance, SBOMs, signing, and vulnerability-class reduction. | Security startups, internal SOC/platform teams, agent-tool builders. | Best early wedge. |  |
| Finance | **Inference:** Payment and transaction middleware, reconciler services, and model-adjacent control planes where explicit changes/effects and generated audit traces reduce operational risk. | Cleaner, more reviewable systems code than C++/Rust for domain-heavy services. | PCI DSS is a baseline for protecting payment data; FFIEC's updated handbook emphasizes governance, risk management, maintenance, and change management; OCC model-risk guidance emphasizes development, validation, monitoring, governance, and third-party risk. | Fintech infra startups, payment vendors, control-platform teams inside banks. **Not core banking rewrite teams first.** | Medium wedge if audit evidence is excellent. |  |
| Defense and military | **Inference:** Windows-first, offline-first mission tooling, CUI-handling utilities, secure software-factory components, and air-gapped deployment tools that emit provenance and compliance packets. | Readable systems code matters when teams rotate and environments are constrained. | DoD software modernization explicitly emphasizes secure delivery, software factories, cloud environment acceleration, resilience, and cATO-adjacent modernization; CMMC and NIST 800-171 push security controls around CUI. | Defense contractors, integrators, software factory teams, classified-environment tool vendors. | Strong procurement wedge after cyber. |  |
| Cloud infrastructure | **Inference:** Secure control-plane services, plugins, sidecars, and internal platform components that emit SLSA/SBOM/provenance by default. | A more readable low-level language for ops-heavy systems teams. | NIST SSDF, SLSA, Sigstore, and SBOM expectations all reward traceable secure build systems and signed artifacts. | Infra vendors, internal platform engineering groups, supply-chain-focused startups. | Strong wedge if Hum can compile and deploy real services quickly. |  |
| Embedded and robotics | **Inference:** Safety-profiled firmware and robotics control software with explicit mutation/effects, profile-gated power, and analyzable resource behavior. | If Hum feels like readable systems code without borrow-checker pain, it could attract robotics teams. | IEC 61508 and ISO 26262 formalize functional safety lifecycles; AUTOSAR and MISRA remain deeply relevant in automotive-style environments. NSA/CISA also note constrained systems need proof of performance, reliability, real-time guarantees, and tooling. | Robotics startups, industrial automation vendors, tooling teams. **Not automotive OEM core ECUs first.** | High upside, extremely high proof burden. |  |
| Game engines | **Inference:** Engine subsystems, asset tooling, and hot-path gameplay/runtime code where readability and performance both matter. | This is mostly a developer-love wedge: game engineers care about control, profiling, and iteration speed. | Procurement pull is weak relative to other sectors. Vale and Zig already position themselves around fast low-level programming, and Rust is already credible in engine/tooling contexts. | Indie engine teams, tooling makers, ambitious studios experimenting in non-core subsystems. | Secondary wedge only. |  |
| AI, ML, and numerics | **Inference:** Safe host-side runtime, data pipeline, and CPU numeric kernels with explicit effects and provenance for agent-generated code. | Python-like readability is attractive, but only if performance and accelerator interop are real. | Mojo is explicitly targeting CPUs, GPUs, and other accelerators; MLPerf exists because this domain is benchmark-driven and ruthless about throughput and latency. | AI infra startups and agent-tool builders, especially for host-side tools rather than bleeding-edge kernels. | Weak unless Hum gets a compelling accelerator story fast. |  |
| DevOps and SRE | **Inference:** Replace large classes of Python/Bash/PowerShell automation with safe binaries/components that have explicit capabilities, deterministic builds, and signed offline distribution. | This is where readable systems code can win immediately if tooling is frictionless. | Reproducibility, signed provenance, SBOMs, and auditable change behavior are increasingly procurement and platform requirements, not extras. | Internal platform/SRE teams, MSPs, regulated enterprise ops groups. | Very strong early wedge. |  |
| Education | **Inference:** Intro systems language for teaching safety, effects, intent, and verification-oriented thinking together. | Python-like readability helps; explicit intent blocks could be pedagogically powerful. | Procurement pull is low, but education can create future maintainers and research collaborators. NSA/CISA explicitly note CS education is lagging on memory safety. | Universities, advanced systems courses, agent-tooling research labs. | Useful multiplier, not first commercial wedge. |  |

**Inference.** If Hum wants the **fastest credible adoption path**, the first three wedges to prioritize are: **cybersecurity**, **DevOps/SRE**, and **defense/offline tooling**. Those sectors most naturally value memory safety, explicit effects, offline distribution, provenance, and build evidence before they require deep runtime maturity or a massive third-party ecosystem. Healthcare can become a marquee wedge, but not before the evidence story is real. AI/ML can become a wedge, but not before Hum either interoperates cleanly with existing accelerators or deliberately avoids competing there.

## Top blockers ranked by severity

This ranking mixes **facts** about current market expectations with **inference** about what will block Hum specifically.

| Rank | Blocker | Severity | Why it blocks adoption | Must exist by | Evidence |
|---|---|---:|---|---|---|
| 1 | An executable backend and runtime profile | Existential | No serious team adopts a language that cannot produce deployable artifacts; Hum's current front-end-only state is a research milestone, not an adoption milestone. | Public alpha |  |
| 2 | A precise memory model and effect model | Existential | "Safety by default" and "explicit effects" only matter if semantics are crisp enough for tools, reviews, and verification. | Public alpha |  |
| 3 | A credible FFI and interop story | Existential | Systems languages live or die at boundaries: C ABI, OS APIs, existing libraries, and portable components. | Public alpha |  |
| 4 | Reproducible builds, lockfiles, vendoring, and offline installs | Existential | Regulated, secure, and air-gapped teams require deterministic dependency resolution and offline rebuilds. | Public alpha |  |
| 5 | Signed provenance and SBOM generation | Existential | Procurement increasingly expects evidence about what was built, from what, and by whom. | Public alpha |  |
| 6 | A minimal but coherent standard library | Critical | Without stable file, process, serialization, collections, time, hashing, and testing APIs, demos stay toy-grade. | Public alpha |  |
| 7 | Formatter, LSP, docs, test runner, and benchmark runner | Critical | Readability claims mean little without tooling parity. | Public alpha |  |
| 8 | A stable diagnostics contract and machine-readable output | Critical | Hum's differentiator includes compiler-generated context for humans and coding agents; that requires structured diagnostics to be first-class. | Public alpha |  |
| 9 | A realistic unsafe and "power feature" governance model | Critical | Rust succeeded partly because `unsafe` has explicit contracts and social norms; Hum needs an equivalent or stricter story. | Public alpha |  |
| 10 | A concurrency and async model | Critical | Cloud, cyber, agent tools, and modern services all need explicit concurrency semantics. | Production beta |  |
| 11 | Observability primitives | High | Serious operators need logging, metrics, tracing, failure modes, and effect boundaries mapped to runtime behavior. | Production beta |  |
| 12 | Windows-native deployment quality | High | Your project constraint is Windows-first; that means services, installers, signing, debugging, and good UX on Windows must be excellent. | Production beta |  |
| 13 | A portable artifact strategy | High | "Portable by design" must become real through native backends and/or WASI/components. | Production beta |  |
| 14 | Performance methodology and public benchmarks | High | Serious engineers will not trust "Rust/C++ power" without transparent methodology and reproducible benchmarks. | Production beta |  |
| 15 | Registry trust, package vetting, and naming policy | High | One malware or typosquat incident can kill a young language's enterprise reputation. | Production beta |  |
| 16 | Security response process and governance | High | Enterprises buy governance as much as syntax: release policy, embargo handling, CVE process, compatibility guarantees. | Production beta |  |
| 17 | Verification integration that is narrow, honest, and useful | High | Regulated buyers will care about proofs only if Hum can state precisely what is proved and what is not. | Production beta |  |
| 18 | Compliance traceability bundles | High | Medical, defense, finance, and safety-critical teams need traceability from requirement to implementation to evidence. | Regulated use |  |
| 19 | Crypto strategy including FIPS boundary story | High | Federal and defense-like buyers care whether cryptography uses validated modules and where trust boundaries sit. | Regulated use |  |
| 20 | Tool qualification and assessor-facing documentation | High | Aviation, automotive, medical, and some defense paths need documentation around tool use, validation, and assurance. | Regulated use |  |

**Inference.** The top of this list says something uncomfortable but important: Hum's real problem is **not language design novelty**. It is **product completeness**. Most new languages die because they try to sell semantics before they can ship safe execution, interop, packaging, tooling, and trust signals. Hum will die the same way unless it treats those as language features, not ecosystem chores.

## Standards and compliance relevance map

These are the standards and frameworks most relevant to the sectors you named. The dividing line that matters is simple: **a language can reduce classes of defects and generate evidence, but it cannot replace quality systems, organizational controls, audits, validation, or certification.**

| Standard or framework | What serious teams need from it | What Hum could genuinely help with | What Hum cannot do by itself | Sources |
|---|---|---|---|---|
| FDA device software guidance | Strong premarket software documentation tied to risk and device function. | Generate software documentation bundles, effect reports, test trace, change trace, and traceable semantic graphs. | It cannot make a product an FDA-cleared device or replace design controls, submission strategy, clinical evidence, or QMS. |  |
| FDA cybersecurity and Section 524B | Cyber devices need cybersecurity evidence; FDA guidance now ties cybersecurity design and documentation directly into submissions, and SBOMs are required for cyber devices. | Emit SBOMs, vulnerability-handling trace hooks, capability/effect reports, and evidence packets supporting secure product development and postmarket planning. | It cannot satisfy postmarket operations, disclosure handling, patch distribution, or manufacturer obligations without process. |  |
| IEC 62304 | Lifecycle requirements for medical device software. | Map `task`/`test`/intent artifacts to lifecycle evidence and traceability. | It cannot substitute for documented lifecycle processes, maintenance procedures, or organizational controls. |  |
| ISO 14971 | Systematic medical-device risk management across the lifecycle. | Connect `watch for`, `protects`, `ensures`, and effect data to hazard-control evidence. | It cannot perform clinical risk judgment or manufacturer governance. |  |
| HIPAA Security Rule | Administrative, physical, and technical safeguards for ePHI confidentiality, integrity, and availability. | Reduce defect risk in ePHI-handling code; generate audit-friendly build and deployment evidence. | It cannot make an organization HIPAA compliant; that requires administrative and physical controls too. |  |
| PCI DSS | Baseline technical and operational requirements for protecting payment account data. | Enforce constrained capabilities/effects in payment services; emit traceable change evidence and signed artifacts. | It cannot replace segmentation, operational controls, key management operations, or assessor review. |  |
| SOC 2 / Trust Services Criteria | Buyers want evidence around security, availability, processing integrity, confidentiality, and privacy. | Produce better engineering evidence for change management, traceability, and secure SDLC controls. | It cannot produce a SOC opinion without an auditor and broad control environment. |  |
| NIST SSDF | High-level secure software development practices usable by producers and purchasers. | Hum's semantic graph and build outputs could become direct SSDF evidence. | It cannot replace the organization's SDLC. |  |
| SLSA, SBOM, Sigstore, SPDX | Provenance, artifact integrity, software inventory, and verifiable signing. | This is Hum's most natural evidence layer: emit provenance, SBOMs, and signed build metadata by default. | It still needs CI/CD controls, signing infrastructure, and artifact distribution policy. |  |
| FedRAMP | Standardized federal cloud security baselines, continuous monitoring, and documentation. | Help vendors produce repeatable build/deploy evidence and tighter service code assurance. | It cannot grant authorization or replace cloud operational security controls. |  |
| CMMC and NIST 800-171 | Protection of FCI/CUI and assessment-driven defense contractor requirements. | Good fit for offline builds, provenance, access-constrained tooling, and evidence bundles for software handling CUI. | It cannot replace enterprise access control, incident response, or assessor-facing operational evidence. |  |
| DoD software modernization | Resilient software, software factories, speed, cloud, and secure delivery. | Hum could fit as an evidence-oriented software-factory language for internal defense tooling if offline posture is excellent. | It cannot create the software factory, cATO process, or acquisition reform alone. |  |
| DO-178C and FAA AC 20-115D | Airborne software design assurance and certification evidence. | Hum could help create explicit contracts, test traceability, effect boundaries, and analyzable subsets. | It cannot make certification easier unless tools, subsets, and verification claims are extremely disciplined and qualified. |  |
| ISO 26262, IEC 61508, AUTOSAR, MISRA | Functional safety lifecycles, automotive-specific frameworks, and constrained coding practice. | Hum can help only if it offers a safety profile with deterministic behavior, constrained features, and excellent analyzability. | It cannot displace incumbent automotive toolchains quickly; this is a long-game target. |  |
| FIPS 140-3 and Common Criteria | Validated crypto modules and evaluated security functionality/assurance claims for some markets. | Hum could improve the implementation and evidence around non-crypto logic and module boundaries. | It cannot turn homegrown crypto into validated crypto, and certification still sits at product/module level. |  |

**Inference.** The compliance lesson is favorable for Hum, but only in a narrow way: a language that natively emits **traceability, constrained effects, provenance, SBOMs, and profile reports** could materially reduce the cost of preparing evidence for regulated reviews. A language that merely claims safety, without generating these artifacts, does almost nothing for regulated procurement.

## Competitor map

Hum should not try to "beat" each competitor in its strongest domain. It should decide what to **copy**, what to **interoperate with**, what to **avoid**, and where **just use Rust** is the correct advice.

| Competitor | What already wins today | What Hum should do |
|---|---|---|
| Rust | Strong memory safety story, explicit `unsafe` contracts, unified build/package workflow, serious docs, production credibility in Android and the Linux kernel.  | **Copy:** explicit unsafe contract boundaries, Cargo-like workflow, diagnostics quality. **Interoperate:** Rust backend/transpilation early. **Do not** claim to replace Rust broadly until Hum proves equal deployment, tooling, and performance. |
| C | Maximum ubiquity, mature ABI, entrenched embedded/systems footprint. | **Interoperate:** C ABI first. **Avoid:** trying to replace C wholesale in year one. Use Hum to fence and document C boundaries. |
| C++ | Massive installed base for low-level, games, finance, and embedded; still the default many teams know. | **Copy:** migration empathy, zero-cost abstraction ambition. **Avoid:** C++-level surface complexity. **Interoperate:** C++ via C ABI boundaries first, not magical bi-directional interop promises. |
| Zig | Clear low-level positioning, strong toolchain identity, offline docs, WASI/C interop, emphasis on intent/maintainability.  | **Copy:** offline docs, whole-toolchain coherence, "communicate intent" framing. **Avoid:** spreading into too many domains before trust is earned. |
| Go | Operational simplicity, excellent service story, modules, easy deployment, strong fit for infra teams.  | **Copy:** dependency simplicity and boring deployment. **Do not** fight Go on CRUD/web service velocity unless Hum's evidence outputs are unmistakably better. |
| Python | Ubiquity in automation, AI, and education; optional typing rather than enforced static safety.  | **Copy:** readability and beginner approachability. **Avoid:** optional typing and runtime ambiguity if Hum claims high assurance. |
| TypeScript | Strong static checking for JavaScript, gradual adoption, broad developer familiarity.  | **Copy:** accessible type-system ergonomics. **Avoid:** presenting Hum as a "systems TypeScript" unless there is equally graceful interop and tooling. |
| Java and Kotlin | JVM portability, operational maturity, rich libraries, enterprise adoption; Kotlin coroutines give ergonomic concurrency.  | **Copy:** practical concurrency ergonomics and enterprise docs. **Avoid:** trying to out-enterprise the JVM stack on day one. |
| Swift | Strong safety messaging, compile-time race detection work, value semantics, excellent ergonomics.  | **Copy:** humane surface design and safety defaults. **Avoid:** fragile ownership complexity surfacing too early without strong tooling. |
| Ada and SPARK | High-integrity niche credibility; SPARK removes undefined behavior and supports formal specification and proof.  | **Copy:** analyzable subset philosophy and proof-aware design. **Interoperate:** learn from assessor-facing documentation style. |
| Dafny | Built-in specifications, frame conditions, verifier integrated into the compiler, multiple backends.  | **Copy:** verification-aware ergonomics, especially frame/effect declarations. **Avoid:** pretending all Hum code should be theorem-proved. |
| F* and Lean | Deep proof power and formal verification credibility.  | **Interoperate with, not replace.** Hum should export proof obligations or semantic artifacts rather than become a proof assistant. |
| Koka | Explicit effect types and handlers; very relevant conceptually to Hum.  | **Copy:** effect-system clarity. **Avoid:** research-language feel if production adoption is the goal. |
| Vale and Vow | Niche experiments in safe systems programming and agent-oriented contract-heavy development.  | **Learn from:** Vow's machine-readable diagnostics and agent-first outputs; Vale's low-level ambition. **Avoid:** agent-first rhetoric that alienates human maintainers. |
| Carbon | Successor-language framing for C++ with heavy emphasis on community and migration.  | **Copy:** governance seriousness and migration narrative. **Avoid:** grand successor claims before implementations exist. |
| Mojo | Python-like surface, systems concerns, explicit push toward CPUs/GPUs/accelerators.  | **Do not** challenge Mojo in AI compute until Hum has a compelling accelerator and numeric backend story. |
| WebAssembly and WASI | Portable sandboxed component model and secure standard interface for multi-language deployment.  | **Interoperate hard.** WASI should likely be a first-class Hum artifact target for agent-safe and offline execution. |
| CUE, Dhall, and Nix | Constraint-based data validation, maintainable non-Turing-complete configuration, and reproducible declarative systems.  | **Copy:** configuration and supply-chain discipline. **Do not** duplicate them unnecessarily; integrate or emit compatible artifacts. |
| Rego | Declarative policy over structured data, clear audit-friendly semantics.  | **Interoperate:** Hum should call out to policy engines rather than re-implement policy-as-code from scratch. |
| TLA+ and Alloy | High-value modeling for concurrent/distributed systems and design-error elimination before code.  | **Interoperate:** Hum's semantic graph should export models or checks, not replace formal modeling tools. |

### Where just use Rust is correct

**Fact.** Rust now has public evidence of mature secure systems adoption: Android expects Rust to be preferred for most new native projects, Google reports dramatically lower memory-safety vulnerability density and better delivery outcomes for Rust changes, and the Linux kernel has live Rust documentation and testing guidance. Cargo gives Rust a unified package/build workflow, and `unsafe` is explicitly documented as a contract boundary.

**Inference.** "Just use Rust" is the correct answer when a team needs **production-ready low-level services, kernel/driver-adjacent work, mature package/build tooling, existing ecosystem access, and deployability right now**. Hum should not posture against that. It should accept that Rust is the incumbent for safe systems programming and try to win where Rust is not optimized: **human-readable intent, effect summaries, compliance traceability, agent-consumable diagnostics, and evidence artifacts emitted by default.**

### Where Hum could still justify itself

**Inference.** Hum can justify itself if it proves all of the following at once:

1. intent blocks are not comments, but checked inputs to the compiler;
2. the semantic graph is useful to humans, CI, auditors, and agents;
3. powerful features are gated by profiles and evidence;
4. build outputs include provenance, SBOMs, effect reports, and review packets;
5. the language remains pleasant enough that teams choose it for internal tools and secure services, not just for compliance theater.

If Hum cannot prove that bundle, it is not a new category. It is a syntax experiment in a field already crowded with strong incumbents.

## Killer demos and the evidence Hum must produce

### Killer demo portfolio

| Demo | What would make engineers say "wait, this is different" | Required evidence artifact |
|---|---|---|
| Secure service | A small authz or secrets-adjacent service that compiles to native Windows and WASI, forbids hidden effects, emits a signed SBOM and provenance, and produces an effect report showing exact read/write/network boundaries. | Semantic graph, effect report, SBOM, provenance, benchmark report, threat model.  |
| Medical and regulated workflow | A device-software companion or SaMD module that emits FDA-style documentation bundles, links `watch for` hazards to tests and `ensures`, and produces a submission packet skeleton. | Compliance trace, software documentation bundle, risk trace, generated tests, SBOM.  |
| Finance transaction system | A payment or ledger workflow showing explicit mutation, idempotency proof sketch, audit trail generation, and policy gates separated from code. | Effect report, policy trace, generated tests, benchmark report, change log packet.  |
| Defense and air-gapped tool | An offline installer or mission planning utility that builds without internet access, uses vendored dependencies only, emits signed provenance, and runs identically on disconnected Windows hosts. | Offline build report, provenance, SBOM, package mirror manifest, profile report.  |
| Game or engine hot path | A pathfinding, ECS, or animation subsystem with profiler-visible results and readable mutation/effect boundaries. | Benchmark report, profile report, effect report.  |
| Embedded and real-time | A bounded-memory control loop or sensor-processing firmware slice with deterministic profile constraints and no heap after init. | Resource profile, timing report, unsafe review packet, proof sketch for profile constraints.  |
| AI and numeric workload | A host-side data pipeline or CPU numeric kernel that shows Python-like readability but compiles to predictable performance and emits provenance for generated/agent-authored code. | Benchmark report, provenance, effect report, generated tests.  |
| Agent-safe tool execution | A Hum-built tool that an LLM invokes through a capability-restricted WASI component, with machine-readable diagnostics and structured counterexamples on failure. | Semantic graph, capability manifest, structured diagnostics JSON, provenance, effect report.  |
| DevOps and SRE utility | A replacement for a messy Python/PowerShell release utility that emits signed artifacts, deterministic build IDs, and a human-readable "what changed and why" report. | Provenance, SBOM, change trace, policy trace, benchmark/time-to-build report.  |

### Must prove evidence checklist

**Inference.** Before serious outsiders should care, Hum should be able to prove the following in public, repeatable form:

| Evidence item | Why it matters |
|---|---|
| Memory safety by default with a precise statement of scope | Buyers now expect defect-class reduction, not vague safety language.  |
| Explicit effect reports | This is Hum's most distinctive technical claim if done well. |
| Deterministic offline builds | Required for defense, regulated, and reproducible supply chains.  |
| Built-in SBOM and provenance emission | Direct procurement value.  |
| Structured diagnostics for humans and agents | Differentiates Hum from ordinary languages and supports your agent ambitions.  |
| Honest unsafe or power-boundary review packets | Trust depends on explicit exceptional paths.  |
| Public benchmark methodology | Prevents "marketing benchmark" dismissal.  |
| Windows-first operational quality | Required by your non-negotiables and a real differentiator if excellent. |
| At least one compelling regulated evidence demo | Otherwise "compliance-native" is just branding. |
| At least one compelling agent-safe execution demo | Otherwise "compiler-generated context for agents" is hand-waving. |

## Product strategy and first year roadmap

### Recommended build order

**Inference.** The first year should optimize for a **narrow but complete alpha**, not language breadth.

| Time window | Priority | Concrete outputs | Exit criteria |
|---|---|---|---|
| First quarter | Lock the safe core | Formalize mutation/effects/capabilities/memory model for a minimal executable subset; publish Hum profile definitions; define semantic graph schema v1; define diagnostics JSON schema v1. | External readers can tell exactly what a Hum program means and what is out of scope. |
| Second quarter | Make Hum executable | Add one real backend for a narrow subset, preferably via lowering to Rust first and/or a WASI-capable artifact path; ship a tiny stdlib; ship formatter, LSP, test runner, and deterministic local builds. | Hum can build a real CLI/tool on Windows from source to artifact. |
| Third quarter | Make Hum trustworthy | Add lockfiles, vendoring, offline install, SBOM/provenance/signing, profile reports, FFI/C ABI, and explicit power-boundary review packets. | Hum can produce a signed offline build plus evidence bundle. |
| Fourth quarter | Make Hum worth showing | Ship the secure service demo, defense/offline tool demo, and one regulated workflow demo; publish benchmarks and a governance/security response policy; recruit two design partners. | A skeptical staff engineer can run the demos and inspect the evidence without handholding. |

This sequence is shaped by what public standards and incumbents already reward: secure SDLC evidence, reproducible build chains, signed artifacts, and deployable language tooling.

### The smallest credible public alpha

**Inference.** A public alpha is not credible if it only parses and checks examples. The smallest credible alpha is:

- an executable **safe profile** for command-line tools and offline services;
- native Windows output and one portable artifact path, ideally WASI;
- basic stdlib for files, strings, JSON, time, hashing, and tests;
- package manifest, lockfile, vendoring, and offline rebuild;
- formatter, LSP, docs, test runner, structured diagnostics;
- automatic semantic graph, effect report, provenance, and SBOM output;
- one C ABI interop story;
- one demo that clearly beats the "just write it in Rust/Go/Python" alternative on evidence and reviewability.

If any of those are missing, calling it a public alpha may attract attention, but not trust.

### Ground truth documents that must exist

**Inference.** The repo needs a small canon of documents treated as specs, not blog posts:

| Document | Why it is mandatory |
|---|---|
| Language semantics and formal core | Without this, Hum cannot support verification or stable tooling. |
| Memory, effects, and capability model | Core to all security and audit claims. |
| Profiles and feature-gating spec | Needed for "safe by default, power by evidence." |
| Semantic graph schema | This is one of Hum's core products. |
| Diagnostics schema | Required for agent and tooling integration. |
| Build/package/provenance spec | Required for reproducibility and supply-chain trust. |
| Interop and ABI guide | Mandatory for practical adoption. |
| Security model and unsafe policy | Mandatory for trust. |
| Benchmark policy | Mandatory for performance claims. |
| Governance and stability policy | Mandatory for enterprise and research adoption. |

### What to stop doing or defer

**Inference.** To maximize adoption odds, Hum should **defer** the following until the narrow credible alpha exists:

| Defer | Why |
|---|---|
| Broad syntax expansion | Surface area is not the blocker. |
| Large general-purpose stdlib ambition | Start with a profile-scoped stdlib. |
| Custom package registry before trust controls | A registry without signing/vetting is a liability. |
| Full async ecosystem | Solve a narrow concurrency story first. |
| GPU and accelerator ambitions | Mojo already occupies this lane publicly. |
| Grand formal verification promises | Export proof obligations and evidence first; avoid theorem-prover cosplay. |
| Fancy macro systems and metaprogramming | They enlarge the trusted surface too early. |
| Language-wide "AI-native" branding | Show structured diagnostics and agent-safe execution instead of slogans. |

## Contrarian critique and source bibliography

### Contrarian critique

Hum fails under any of these conditions:

**Inference:** If Hum does not ship executable artifacts, then it is a design language, not a programming language, and adoption will stall at curiosity.
**Inference:** If Hum ships executable artifacts but not packaging, offline reproducibility, provenance, and signing, it will lose the exact regulated and high-trust audiences it most needs.
**Inference:** If Hum copies Rust's power without an equally explicit unsafe or capability boundary, its "intent-first" story will collapse under scrutiny.
**Inference:** If Hum chases AI/ML, games, embedded, and regulated software simultaneously, it will become broad before it becomes credible.
**Inference:** If Hum treats the semantic graph as an internal implementation detail instead of a user-facing artifact schema, one of its true differentiators will be wasted.

### Open questions and limitations

Some important source material is only accessible through official summaries or paywalled standards pages, especially parts of IEC and ISO safety standards. The report therefore relies on official abstracts, regulator guidance, and authoritative public summaries where the full text is not freely available. That is enough for roadmap and product strategy decisions, but not enough for final assessor-facing compliance mappings.

The most consequential unresolved design choice is **backend strategy**: whether Hum should first lower to Rust, to C, to WASI, or to more than one. My recommendation is still to lower first to the path that gets you the narrowest trustworthy alpha fastest, even if that means Hum's initial competitive advantage is mostly in the front-end semantics and evidence products rather than native code generation novelty.

### Source bibliography

| Source | Relevance |
|---|---|
| NIST SP 800-218 Secure Software Development Framework | Secure SDLC and purchaser-facing software assurance vocabulary.  |
| ONCD Back to the Building Blocks | U.S. policy case for memory-safe languages and defect-class elimination.  |
| NSA and CISA Memory Safe Languages | Current balanced view of memory-safe language adoption and constraints.  |
| CISA and partner guidance on memory-safe roadmaps | What buyers increasingly expect from secure-by-design manufacturers.  |
| FDA Content of Premarket Submissions for Device Software Functions | Device software documentation expectations.  |
| FDA Cybersecurity in Medical Devices guidance | Cybersecurity and Section 524B implications for medical device software.  |
| IEC 62304 official abstract | Medical device software lifecycle requirements.  |
| ISO 14971 official abstract | Medical device risk management.  |
| HHS HIPAA Security Rule | CIA safeguards for ePHI.  |
| PCI DSS official standard page | Payment data protection baseline.  |
| FFIEC DA&M update | Finance-sector expectations for governance, maintenance, and change management.  |
| OCC revised model risk guidance | Banking expectations for model development, validation, monitoring, and governance.  |
| DoD Software Modernization Strategy and FY25-26 plan | Defense software-factory and resilient delivery priorities.  |
| NIST SP 800-171 Rev. 3 and DoD CMMC pages | CUI protection and defense contractor compliance environment.  |
| FAA AC 20-115D and RTCA DO-178 pages | Airborne software assurance and certification context.  |
| ISO 26262, IEC 61508, AUTOSAR, MISRA official pages | Automotive and industrial functional safety context.  |
| NIST FIPS 140-3 and CMVP pages | Crypto module validation boundaries.  |
| Common Criteria portal | Security certification ecosystem and protection profiles.  |
| SLSA, NTIA SBOM, Sigstore, SPDX | Supply-chain integrity, SBOMs, and signed provenance.  |
| Rust Book, Cargo Book, Android memory-safety docs, Linux kernel Rust docs | Baseline incumbent for systems-language credibility.  |
| Zig docs | Toolchain coherence, offline docs, and low-level maintainability framing.  |
| Swift official page | Safety and value-semantics ergonomics benchmark.  |
| Go modules reference | Operationally simple dependency model benchmark.  |
| TypeScript handbook and Python typing docs | Developer-friendly type-system ergonomics and optional-typing baseline.  |
| SPARK, Dafny, Lean, F*, Koka | High-assurance and effect/verification design reference points.  |
| Rego, TLA+, Alloy, CUE, Dhall, Nix | Policy, modeling, configuration, and reproducibility tools Hum should integrate with or learn from.  |
| WASI and Component Model docs | Best current public story for sandboxed portable components and agent-safe execution.  |
| Mojo docs and vision | Public benchmark for Python-like surface plus systems and accelerator ambitions.  |
| Vow language site | Reference point for agent-oriented contracts, explicit effects, and structured diagnostics.  |
| TechEmpower, CoreMark, MLCommons MLPerf | Public benchmark references for service, embedded, and ML-adjacent performance claims.  |

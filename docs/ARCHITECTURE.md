# Hum Architecture Ground Truth

Date: 2026-07-08

## Purpose

This is the short stable map of Hum. The other docs are allowed to grow; this document says how they fit together and what must stay true when a new design idea arrives.

If another Hum doc disagrees with this one, treat that as a design bug to resolve.

## North Star

Hum is an intent-first, evidence-native systems language:

```text
human-readable intent -> precise formal core -> semantic graph -> checks, profiles, evidence, and tools -> portable backends and platform artifacts
```

Hum should be readable enough for beginners, strict enough for systems engineers, and structured enough for compilers, IDEs, debuggers, profilers, package tools, and coding agents.

Hum should have paved roads, not endless knobs: one obvious safe default path, explicit side roads only when evidence and source-visible intent justify them, and diagnostics that guide users back to the better path. See [PAVED_ROAD_DOCTRINE.md](PAVED_ROAD_DOCTRINE.md).

Readable means formally scannable, not English-like. Hum may use ordinary words when they name one precise construct, but stable executable syntax must lower to Core Hum and avoid synonym-heavy prose. See [decisions/0009-adopt-formal-readability-not-english-mimicry.md](decisions/0009-adopt-formal-readability-not-english-mimicry.md).

Evidence-native means Hum's output is not only a binary. The language and
toolchain should also emit machine-checkable intent, effect reports,
capability boundaries, diagnostics, profile facts, provenance, SBOMs, and
deployment evidence.

See [ADOPTION_STRATEGY_2026.md](ADOPTION_STRATEGY_2026.md).

## Architecture Layers

### 1. Surface Hum

Surface Hum is what people write: `task`, `type`, `store`, `test`, and checked intent blocks such as `why:`, `uses:`, `changes:`, `needs:`, `ensures:`, `fails when:`, `watch for:`, `protects:`, `trusts:`, `cost:`, `allocates:`, `avoids:`, `tradeoffs:`, `optimizes:`, `tests:`, `proves:`, and `does:`.

The surface rule is controlled obviousness: no headers, no semicolons in normal source, no hidden effects, no hidden mutation, no hidden unsafe, and no correctness-critical comments without a checked home.

See [LANGUAGE_REFERENCE.md](LANGUAGE_REFERENCE.md) for the traditional reference spine for source files, items, sections, and current language status.

### 2. Formal Core

Surface Hum lowers into Core Hum. Core Hum defines values, places, mutation,
expressions, statements, typed failure, effects, contracts, profiles, loops, and
backend-preservation rules. New syntax is not stable until it lowers into the
core and preserves graph facts.

The machine-readable Core Hum contract is [HUM_CORE_CONTRACT_SCHEMA.md](HUM_CORE_CONTRACT_SCHEMA.md),
emitted by `hum core-contract --format json`; the first non-executing Core Hum
candidate preview is [HUM_CORE_PREVIEW_SCHEMA.md](HUM_CORE_PREVIEW_SCHEMA.md),
emitted by `hum core-preview --format json`; the first unverified Core Hum
artifact is [HUM_CORE_LOWER_SCHEMA.md](HUM_CORE_LOWER_SCHEMA.md), emitted by
`hum core-lower --format json`; the first non-executing Core Hum verifier is
[HUM_CORE_VERIFY_SCHEMA.md](HUM_CORE_VERIFY_SCHEMA.md), emitted by
`hum core-verify --format json`; checked scope, definition,
reference, and mutable-place identity is reported by [HUM_RESOLVE_SCHEMA.md](HUM_RESOLVE_SCHEMA.md),
emitted by `hum resolve --format json`; declared type-environment facts are reported by
[HUM_TYPE_ENV_SCHEMA.md](HUM_TYPE_ENV_SCHEMA.md), emitted by `hum type-env --format json`;
declaration annotation and trivial return type checking is reported by [HUM_TYPE_CHECK_SCHEMA.md](HUM_TYPE_CHECK_SCHEMA.md),
emitted by `hum type-check --format json`; recognized Core/body statement type checking is reported by
[HUM_FULL_TYPE_CHECK_SCHEMA.md](HUM_FULL_TYPE_CHECK_SCHEMA.md), emitted by `hum full-type-check --format json`; recognized Core/body effect checking is reported by [HUM_EFFECT_CHECK_SCHEMA.md](HUM_EFFECT_CHECK_SCHEMA.md), emitted by `hum effect-check --format json`; recognized local ownership and alias facts are reported by [HUM_OWNERSHIP_CHECK_SCHEMA.md](HUM_OWNERSHIP_CHECK_SCHEMA.md), emitted by `hum ownership-check --format json`; declared allocation and resource intent is checked by [HUM_RESOURCE_CHECK_SCHEMA.md](HUM_RESOURCE_CHECK_SCHEMA.md), emitted by `hum resource-check --format json`; runtime profile policy declarations are checked by [HUM_PROFILE_CHECK_SCHEMA.md](HUM_PROFILE_CHECK_SCHEMA.md), emitted by `hum profile-check --format json`;
the Hum IR ownership contract is [HUM_IR_CONTRACT_SCHEMA.md](HUM_IR_CONTRACT_SCHEMA.md), emitted by `hum ir-contract --format json`;
source progress toward those contracts is reported by [HUM_IR_READINESS_SCHEMA.md](HUM_IR_READINESS_SCHEMA.md),
emitted by `hum ir-readiness --format json`.

See [FORMAL_CORE.md](FORMAL_CORE.md).

#### Current Compiler Spine

As of `0.0.1` pre-alpha, the implemented non-executing compiler spine is:

```text
parse/current
-> semantic_graph/current
-> resolve/checked_report_available
-> type_env/declaration_inventory_available
-> type_check/declaration_and_trivial_return_check_available
-> core_preview/preview_v0
-> core_lower/unverified_core_artifact_v0
-> core_verify/verified_non_executing_core_artifact_v0
-> full_type_check/recognized_core_body_type_gate_available_v0
-> effect_check/recognized_core_effect_gate_available_v0
-> ownership_check/recognized_core_ownership_gate_available_v0
-> resource_check/recognized_core_resource_gate_available_v0
-> profile_check/recognized_core_profile_gate_available_v0
-> ir_readiness/blocked_by_full_type_check_errors_or_effect_check_errors_or_ownership_check_errors_or_resource_check_errors_or_profile_check_errors_or_before_ir_verify
```

`full_type_check` now exists as a narrow recognized Core/body statement type gate, `effect_check` now exists as a narrow recognized Core/body effect gate, `ownership_check` now exists as a narrow local ownership fact gate, `resource_check` now exists as a narrow declared allocation/resource intent gate, and `profile_check` now exists as a narrow runtime profile policy gate. These report gates remain non-executing and must not claim complete type safety, effect safety, ownership safety, memory safety, allocation-freedom proof, strict profile enforcement, IR emission, backend readiness, or safety-critical readiness. `hum run` begins Milestone 1 executable semantics only for the explicitly interpreted first Formal Core subset; it does not turn the report gates into proof, memory-safety, optimization, IR, backend, or certification claims.

Typed-failure doctrine: a known fallible call in any currently executable
expression position never propagates invisibly. Shared analysis finds calls at
the expression root, inside operators and call arguments, and in executable
loop collections. The accepted propagation surface remains only an explicit
direct named call using same-root `try` or caller-root wrapping. Full type owns
nominal compatibility, effect check owns the meaningful `fails when:`
obligation and `avoids: failure` contradiction, Core preserves exact blockers
for unsupported `try` shapes, and runtime owns the causal root/call-site
carrier. This preserves local source blame before IO authority exists; it does
not grant IO, implement recovery, or establish a complete `Result` model. See
[decisions/0016-adopt-explicit-causal-typed-failure.md](decisions/0016-adopt-explicit-causal-typed-failure.md).

Executable app entry is now structural: one top-level app names one directly
nested `Unit` or `Result Unit, E` task through a single meaningful
`starts with:` line. The shared app-entry analysis owns source diagnostics and
runtime selection, so app mode never falls back to global task lookup. App
callable identity stops at the app boundary across resolver, typed-failure,
full-type, effect, and runtime analysis. Before app execution, the existing
resolver, declaration-type, and recognized full-type gates must be clean;
runtime also fails closed rather than hiding a non-Unit value. App success adds
no result display and typed app failure keeps Session W's causal stderr path.
Session Y adds the checked source-authority half under accepted decision 0017.
The only external source capability IDs are `stdout.write`, `clock.replay`, and
`files.read`. App `uses:` is a maximum, each task declares its direct and
transitive closed-call budget, callers cover callees, and the app covers the
start closure. Existing effect boundary rows preserve a stable source-policy ID,
the typed kind/scope/strength/one-run lifetime, severity tier, and every
app/task/call/declaration route site. These facts authorize no host action:
operator grants, denies, prompts, persistence, wildcards, decision/use audit
events, IO, and Path semantics remain later evidence gates. Session Z adds the
one bounded exception: exact one-run `stdout.write` grants intersect with the
source route, exact deny wins, and `stdout_write(Text)` performs immediate
no-newline UTF-8 writes through an injectable adapter under a 1 MiB per-run
limit. Denial, limit, and opaque adapter failure are typed `OutputError` paths;
runtime decision/exercise facts join the stable source-policy ID with the
complete app/start/caller/output route and call occurrences. `stdout_write` is
reserved against user task declarations, and reserved target mappings do not
satisfy `requires:`. Runtime policy selection uses the actual dynamic lexical
call occurrences, including conditionally selected same-callee paths, and uses
shared separator-normalized source identity so equivalent Windows `/` and `\`
inputs cannot change policy selection. Original spans remain display evidence.
H0624
rejects output-reachable recursion because Session Z has no finite exact route
model for it, but only after task/caller/app authority coverage is valid;
H0621/H0618 retain precedence for missing authority. Ordinary recursion is not
thereby banned. This adds no prompt,
persistence, wildcard, broader IO, runtime JSON, or target-availability claim.
`--entry` stays a pure probe and is rejected when its selected closure
carries any pinned source authority.

Session AA adds the second bounded operation under the same authority spine:
`clock_replay_tick() -> Result UInt, ReplayClockError` consumes one value from
an ordered runner-provided sequence only after exact `clock.replay` source
closure and operator consent intersect. Values are input rather than
authority; default or explicit denial calls no replay adapter, and exhaustion
is a typed causal failure. Existing effect rows map the operation to Core
`time` with explicit runner-replay status and complete finite route identity.
Runtime audit facts join policy, decision, exercise, sequence index, and tick.
Replay-reachable recursion fails closed under H0628 only after H0625/H0618/
H0619 authority precedence. This is controlled replay input, not `os.clock`, a
host-time API, deterministic scheduling, randomness, or whole-program
determinism.

Session AB adds an opaque native `Path` identity at the runner/app-entry
boundary without adding a file operation. Only one `Path` parameter on the
structural app start may receive one native `OsString` argument; source cannot
construct, inspect, display, compare, store, return, or transform it, and
direct `--entry` cannot inject it. Windows accepts only a lexically ordinary
drive-letter-rooted `Prefix::Disk` spelling after rejecting relative,
traversal, namespace, ADS, empty-component, trailing-dot/space, and normalized
DOS-device aliases. This evidence is deliberately weaker than locality:
accepted Path arguments and exact native `files.read=<path>` grant payloads
remain `locality_unclassified`. Duplicate identical grants are idempotent,
two distinct payloads reject, and exact deny wins. No metadata, open,
canonicalization, contents, general Path API, or non-Windows file support is
added by that boundary.

Session AC adds one isolated bootstrap adapter that may narrow the internal
status to threat-scoped `fixed_local_v0`. Under a trusted Windows kernel,
storage-driver stack, and non-deceptive hypervisor, the complete observable
backing chain must contain no mapped, network, fabric, file-backed, virtual,
removable, or unknown layer. `GetDriveTypeW` and `QueryDosDeviceW` are only
preliminary evidence. A zero-access synthesized volume handle must report no
type-2 storage dependency and a complete nonempty extent list; every synthesized
physical-disk handle must report non-removable ATA, SATA, or NVMe; and the drive
type/mapping must be identical after inspection. Everything else remains
unclassified. The main crate remains unsafe-free, candidate paths are never
opened, and this is not a portable Path model, target-availability claim, or
filesystem sandbox.

Session AD adds the first bounded file operation:
`files_read_text(path: Path) -> Result Text, FileReadError`. The operation is
reachable only through complete `files.read` task/caller/app source closure and
one exact matching native operator grant; default or explicit deny wins and a
different grant is outside authority before candidate access. The runner
revalidates the AB lexical class and AC `fixed_local_v0` evidence before the
safe file adapter examines components. The adapter rejects every reparse,
symlink, junction, directory, or non-file path, performs one read-only file
open, reads at most 1 MiB, and decodes strict UTF-8. Failures are typed and
causal. Core records `file` and the target mapping remains the reserved
`os.filesystem` family rather than an availability, portability, profile, or
sandbox claim. Concurrent path mutation remains outside the alpha threat
model.

Session AF replaces the split Predicate v1 string recognizer/evaluator with one
shared Predicate v2 fact. The fact owns the exact horizontal-whitespace
grammar, intent signal, bounded delimiter parse, syntactic places, later
resolution/eligibility/type checks, and accepted AST consumed by full type,
downstream gates, Core, graph, and runtime. Predicate v2 adds only exact Text
and ordered `List Text` equality/inequality plus contract-only
`list_count(List Text, Text) -> UInt`. H0701 owns genuine prose, H0704 owns
malformed or ill-typed executable candidates, H0630 retains opaque-Path
precedence, and only recognized typed facts reach H0702/H0703 evaluation.
These recognition statuses are not decision 0015 proof/trust classifications.
Each parsed command memoizes one immutable analysis; resolver/type/Core/graph
reports reference its per-place lexical identities, and runtime indexes that
same analysis for reachable-task preflight, `old(...)` capture, and evaluation.
Runtime rejects every independent H0704 in the selected reachable task subtree
before argument conversion, task bodies, or authority adapters run.

Session AL adds one memoized callable analysis in `src/callable.rs`. The parser
owns delimiter-aware type/body nodes; resolver IDs establish task, parameter,
value-use, and application identity. Resolver, type/effect/ownership/resource,
Core, graph, and runtime consume that immutable relationship. Only one
same-file `task(UInt) -> UInt` value and one `transform(value)` application are
accepted, with `failure_root = none` and an inferred closed-empty latent row.
Runtime carries a private nonescaping resolved-definition handle with no
callable environment. This is not general higher-order typing, capture,
allocation proof, public generalization, open rows, or handling.

Session V's writable-field-alias slice is owned by one shared straight-line
place analysis consumed by `ownership_check` and the interpreter. Resolver and
effect rows recognize the same candidate as writable and defer authority and
overlap to ownership; runtime stores the exact source place rather than a copied
value. This shared fact keeps H0808/H0809 identity, last-use spans, write-through,
and field-view invalidation aligned without creating a general alias subsystem.

### 3. Semantic Graph

The semantic graph is Hum's shared truth for humans, compiler checks, `humfmt`, `chirp`, `hum lsp`, `hum debug`, `hum graph`, Nectar, and agents. Agents should query graph facts, not scrape terminal prose when the compiler can provide structured meaning.

See [SEMANTIC_GRAPH_SCHEMA.md](SEMANTIC_GRAPH_SCHEMA.md) and [DIAGNOSTICS.md](DIAGNOSTICS.md).

### 4. Checks And Evidence

Milestone 0 checks stay small: parse files, validate sections, preserve spans, enforce first mutation and cost promises, emit stable diagnostics, and emit graph JSON with exact or canonical-token `covers:` links between task obligations and tests.

Later checks add generated tests, ownership, borrowing, effect propagation, unsafe review packets, foreign/ABI boundaries, profile restrictions, performance contracts, package evidence, supply-chain evidence, and platform authority checks.

Rule: if a feature creates new power, it must create new evidence.

Cache doctrine: caches can speed builds, semantic graph reads, package checks, proofs, and benchmark setup, but they are never authority. A cached artifact is valid only when its key includes the semantic inputs that make it true: source content, dependency graph, compiler version, profile, target, verifier or solver configuration, ABI seed, and relevant environment facts. Cached proof success must be treated especially carefully; release evidence should come from rerunnable checks, not from trusting a disk entry.

Strong-contract doctrine: a verified contract is valuable only when it would reject meaningful wrong implementations. Hum should eventually detect tautological, vacuous, weak, verifier-shaped, or benchmark-shaped claims and report them as diagnostics or profile gates.

External-verifier doctrine: Truth Harness-style math engines, SMT tools, model checkers, proof assistants, and benchmark harnesses are evidence producers, not compiler authority. Hum emits obligations and records receipts; external engines may prove, refute, or return unknown under explicit assumptions. See [MATH_ENGINE_BOUNDARY.md](MATH_ENGINE_BOUNDARY.md) and [decisions/0005-keep-verifiers-as-evidence-producers.md](decisions/0005-keep-verifiers-as-evidence-producers.md).

Resource-layout-comptime doctrine: resource intent, layout-sensitive representation, compile-time execution, interop, and agent-facing facts must be explicit, graph-visible, profile-aware, and evidence-backed before they become stable Hum power. `hum resource-report` is the current source-declared inventory for these claims, `hum resource-check` is the first narrow gate for declared allocation/resource intent, and `hum profile-check` is the first fail-closed gate for source-visible runtime profile policy declarations. Ergonomic defaults are welcome only when resource behavior stays source-visible and compiler-checkable. See [RESOURCE_REPORT_SCHEMA.md](RESOURCE_REPORT_SCHEMA.md), [HUM_RESOURCE_CHECK_SCHEMA.md](HUM_RESOURCE_CHECK_SCHEMA.md), [HUM_PROFILE_CHECK_SCHEMA.md](HUM_PROFILE_CHECK_SCHEMA.md), and [decisions/0006-make-resource-layout-and-comptime-explicit.md](decisions/0006-make-resource-layout-and-comptime-explicit.md).

Progressive-disclosure doctrine: Hum should keep ordinary code small at the point of use, add power only behind explicit need, avoid special-case syntax, and make adoption incremental through interop and migration tooling. See [research/2026-07-07-lattner-compiler-lessons.md](research/2026-07-07-lattner-compiler-lessons.md) and [decisions/0007-adopt-progressive-disclosure-and-migration-discipline.md](decisions/0007-adopt-progressive-disclosure-and-migration-discipline.md).

Formal-readability doctrine: Hum should be easy to scan because its structure is precise, not because it imitates casual English. Stable syntax gets one canonical spelling per concept, no arbitrary English execution, and no prose-only executable authority. See [decisions/0009-adopt-formal-readability-not-english-mimicry.md](decisions/0009-adopt-formal-readability-not-english-mimicry.md).

State-management doctrine: Hum treats state as visible, permissioned, profile-aware, and evidence-producing. Immutable values are the paved road; mutation, ownership, borrowing, stores, linear resources, shared state, and external authority must have source-visible facts before they become stable power. The current machine-readable state contract is `hum.state_model.v0`, emitted by `hum state-model --format json`. Checked source places begin in `hum.resolve.v0`, emitted by `hum resolve --format json`. See [STATE_MODEL.md](STATE_MODEL.md) and [decisions/0010-adopt-explicit-state-model.md](decisions/0010-adopt-explicit-state-model.md).

Resolution doctrine: checked scope, definition, reference, and place identity comes before execution, type checking, effect checking, ownership, resource checking, borrowing, editor go-to-definition, debugger facts, and IR emission. `hum type-env` must consume resolver definition identity before type checking, `hum type-check` must consume `hum.type_env.v0` before typed-core claims, `hum core-lower` consumes checked resolver, type-check, and core-preview summaries before any Core artifact claim, `hum core-verify` consumes the core-lower artifact before any verified Core artifact claim, `hum full-type-check` consumes the resolver/type/core-verifier summaries before any body type claim, `hum effect-check` consumes the full-type-check summary before any effect claim, `hum ownership-check` consumes the effect-check summary before any ownership claim, `hum resource-check` consumes ownership-check and resource-report summaries before any resource-intent gate claim, `hum profile-check` consumes the resource-check summary and runtime profile catalog before any profile gate claim, and `hum ir-readiness` must consume the profile-check summary before any IR claim. See [HUM_RESOLVE_SCHEMA.md](HUM_RESOLVE_SCHEMA.md), [HUM_TYPE_ENV_SCHEMA.md](HUM_TYPE_ENV_SCHEMA.md), [HUM_TYPE_CHECK_SCHEMA.md](HUM_TYPE_CHECK_SCHEMA.md), [HUM_CORE_PREVIEW_SCHEMA.md](HUM_CORE_PREVIEW_SCHEMA.md), [HUM_CORE_LOWER_SCHEMA.md](HUM_CORE_LOWER_SCHEMA.md), [HUM_CORE_VERIFY_SCHEMA.md](HUM_CORE_VERIFY_SCHEMA.md), [HUM_FULL_TYPE_CHECK_SCHEMA.md](HUM_FULL_TYPE_CHECK_SCHEMA.md), [HUM_EFFECT_CHECK_SCHEMA.md](HUM_EFFECT_CHECK_SCHEMA.md), [HUM_OWNERSHIP_CHECK_SCHEMA.md](HUM_OWNERSHIP_CHECK_SCHEMA.md), [RESOURCE_REPORT_SCHEMA.md](RESOURCE_REPORT_SCHEMA.md), [HUM_RESOURCE_CHECK_SCHEMA.md](HUM_RESOURCE_CHECK_SCHEMA.md), [HUM_PROFILE_CHECK_SCHEMA.md](HUM_PROFILE_CHECK_SCHEMA.md), and [decisions/0011-add-checked-resolver-before-execution.md](decisions/0011-add-checked-resolver-before-execution.md).

Language-builder doctrine: Hum should grow by small proofs, written lessons, graph/report/check surfaces, migration paths, and then public claims. See [LANGUAGE_BUILDER_OPERATING_MODEL.md](LANGUAGE_BUILDER_OPERATING_MODEL.md).

Debuggability doctrine: Hum must preserve source identity, value identity, layout facts, effect facts, profile facts, contract facts, and provenance so a fast intent-aware debugger can exist later. Native debug formats are bridges; Hum debug facts remain the authority for Hum intent. The target debug-info shape is `hum.debug_info.v0`: source maps, step honesty, visualizers, debug probe sites, native debug links, and local-first privacy. See [DEBUGGABILITY_DOCTRINE.md](DEBUGGABILITY_DOCTRINE.md) and [DEBUG_INFO_AND_VISUALIZER_MODEL.md](DEBUG_INFO_AND_VISUALIZER_MODEL.md).

### 5. Runtime Profiles

Profiles are policy bundles for normal apps, containers, agent tools, Windows services, driver candidates, embedded no-heap code, hard realtime code, engine hot paths, safety-critical code, and certified toolchains. Profiles can forbid features, require evidence, narrow stdlib APIs, and change release artifacts.

The current machine-readable catalog is [HUM_RUNTIME_PROFILES_SCHEMA.md](HUM_RUNTIME_PROFILES_SCHEMA.md), emitted by `hum profiles --format json`. V0 is contract-only and must not claim profile enforcement, stdlib narrowing, runtime behavior, target selection, certification, host probing, or footprint measurement.

See [RUNTIME_PROFILES.md](RUNTIME_PROFILES.md).

### 6. OS And Platform Model

Hum is Windows-first for proof on the primary development platform and portable-by-design for architecture. Windows APIs, services, drivers, registry, devices, installers, updates, telemetry, and process authority must be modeled as explicit platform capabilities, not hidden globals.

Target portability is a boundary contract, not a backend marketing claim. Target facts, capability absence, path/time/randomness policy, backend adapters, platform adapters, and artifact evidence are owned by [PORTABILITY_BOUNDARY_MODEL.md](PORTABILITY_BOUNDARY_MODEL.md). The machine-readable V0 field catalog and fixtures are owned by [TARGET_FACTS_SCHEMA.md](TARGET_FACTS_SCHEMA.md) and `hum target-facts --format json`.

See [OS_AND_PLATFORM_MODEL.md](OS_AND_PLATFORM_MODEL.md).

### 7. Ecosystem Tools

The tools are part of the language: `hum`, `humfmt`, `chirp`, `nectar`, `hum lsp`, `hum debug`, and `hum graph`. No serious feature is stable until the tools have a story for it.

See [TOOLING.md](TOOLING.md), [FORMATTER.md](FORMATTER.md), [TOOLCHAIN_2050.md](TOOLCHAIN_2050.md), [DEBUGGABILITY_DOCTRINE.md](DEBUGGABILITY_DOCTRINE.md), [DEBUG_INFO_AND_VISUALIZER_MODEL.md](DEBUG_INFO_AND_VISUALIZER_MODEL.md), and [NECTAR_PACKAGE_MANAGER.md](NECTAR_PACKAGE_MANAGER.md).

### 8. Standard Library Labs

The first stable primitive families are `Result`/`Option`, `Vec`/`Slice`/`Span`, `Map`/`Set`, and `Text`/`Bytes`. Allocators, parsers, sync, SIMD, accelerators, networking, operations, storage, and numeric/tensor APIs go through labs before stable `std`.

The stdlib rule is:

```text
algorithm > data layout > allocation > cache behavior > compiler lowering > instruction tricks
```

See [PAVED_ROAD_DOCTRINE.md](PAVED_ROAD_DOCTRINE.md), [STDLIB_CONSTITUTION.md](STDLIB_CONSTITUTION.md), [STDLIB_PRIMITIVE_RESEARCH_2026.md](STDLIB_PRIMITIVE_RESEARCH_2026.md), and [OPTIMIZATION_AND_DSA_STRATEGY.md](OPTIMIZATION_AND_DSA_STRATEGY.md).

### 9. Backends

The backend order is Rust bootstrap front end, interpreter for first executable semantics, Cranelift for first native proof, LLVM for mature optimized native AOT builds, MLIR for vector/tensor/accelerator work, Wasm/WASI for sandboxed portable components, and custom backend work only if later evidence justifies it. Backends are swappable targets; they are not Hum's semantic soul. The backend adapter contract points back to `hum.ir_contract.v0` as the semantic owner.

See [BACKEND_STRATEGY.md](BACKEND_STRATEGY.md) and [decisions/0008-adopt-swappable-backend-ladder.md](decisions/0008-adopt-swappable-backend-ladder.md).

## Non-Negotiable Decisions

- Rust remains the bootstrap compiler until Hum proves self-hosting through staged differential tests.
- Milestone 0 stays local, offline-first, non-executing, and safe on the maker's machine.
- Important comments become checked intent blocks.
- Unsafe and foreign code require review packets and profile gates.
- Containers, OS sandboxes, and agent tools do not replace language safety.
- Windows is the first tested platform, but platform-specific details stay behind explicit capability boundaries.
- No feature enters stable Hum without semantics, state-model impact, diagnostics, graph facts, tooling impact, profile impact, verification story, performance story, and pedagogy story.
- Defaults must be paved roads; non-default power requires explicit source intent and evidence.
- Resource, layout, compile-time, interop, and agent-facing power must be explicit and graph-visible before it is stable.
- Progressive disclosure and migration discipline are language requirements, not polish.
- Small proof, written lesson, graph/report/check surface, migration path, then public claim is the default growth loop.
- Caches optimize development speed but do not certify correctness, safety, performance, or release readiness.
- No parser-only or checker-only milestone should be presented as a credible public alpha; public adoption requires executable artifacts and evidence bundles.

## Current Build Order

1. Keep Milestone 0 semantic graph, diagnostics, generated test skeleton hardening, coverage matching, and report gates honest as non-executing evidence surfaces.
2. Grow Milestone 1 `hum run` only through the active work order: first the three Formal Core fixtures, then executable contracts, then real probe programs.
3. Keep docs honest by linking every new doctrine back to this architecture and by updating stale status text when execution expands.
4. Strengthen checked resolution, types, effects, ownership, and resource intent before serious unsafe, FFI, profile, or native backend work.
5. Add package/build/profile evidence before networked package behavior.
6. Defer drivers, installers, Windows Update publishing, and kernel work until strict profiles and proof infrastructure exist.

## Update Rule

When adding a major Hum design document, answer:

```text
Which architecture layer does this belong to?
Which existing docs does it constrain?
Which semantic graph facts does it require?
Which profile or evidence gates does it change?
What must Milestone 0 ignore for now?
```
Session AM extends that same analysis with one local, nonrecursive structural
row tail. Exact `change` occurrences are resolver-owned identities, duplicates
remain duplicates, and one application substitution propagates the complete
row. Their normalized label spelling and alpha-stable tail alias are separate
comparison facts, not semantic identity. The slice rejects a second direct
relationship or application for the same resolver-owned receiver. Rows remain
type/effect evidence only: they grant no authority, consent, ownership,
resource lifetime, allocation proof, capture, or handling.

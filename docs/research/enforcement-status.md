# Enforcement status: doctrine claims vs implementation

As of `origin/main` c3ed1ad (2026-09-24).
Status: research snapshot (living document — update when a claim's status changes)

Rows expected to move soon: 0022 → enforced when WO28 #4 lands;
0028/0029 move when implemented.

## Purpose

Every normative claim the project has adopted — in accepted decisions
0001–0029 and in the SPEC (`docs/LANGUAGE_REFERENCE.md`,
`docs/LANGUAGE_SUBSET_0_1.md`, repo-root `SPEC.md`) — with its enforcement
status today:

- **enforced** — code plus fixtures/tests prove it today.
- **partial** — some enforcement exists; known gaps remain (cited).
- **declared-only** — stated in docs/decisions or the SPEC; no code
  enforcement found.
- **not-started** — accepted/specified; no implementation begun.

Each gap names the Work Order or decision that would close it, or "none
assigned". Evidence cites are pinned to `origin/main` as of 2026-09-24
unless noted. This page is a map, not a verdict: a declared-only claim is
not a broken promise, it is an obligation with no check behind it yet.

## How to read the "gap closer" column

- A WO number means the active Work Order already carries the item.
- A decision number means the decision authorizes the work but no WO
  session is scheduled.
- "none assigned" means the gap is recorded here first.

---

## Decisions 0001–0010 (foundations)

### 0001 — Adopt evidence-native architecture (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Language features preserve machine-checkable meaning in diagnostics, graph facts, profile reports, tests, or release evidence | partial | `src/graph.rs` emits graph facts; `src/diagnostic_catalog.rs` H-codes; `tools/check_all.ps1:660,670` fails on unlinked evidence obligations in the reference fixture | none assigned |
| Important claims belong in checked sections, not comments | partial | Honesty-lock warnings H0107/H0109 (`src/diagnostic_catalog.rs:2415,2433`) fire on app tasks (ledger #11); coverage is warnings-only and app-task-scoped | none assigned |
| Features that create new power must create new evidence | declared-only | No code ties a new feature to new evidence; enforced only via review process | none assigned |
| Public security/adoption claims need scoped evidence | partial | `tools/check_public_readiness.ps1` scans docs (hygiene/PII) but does not scope claims | none assigned |

### 0002 — Use Rust bootstrap until staged self-hosting (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| `#![deny(unsafe_code)]` compiler-wide default with exactly one reviewed, locally allowed JIT invocation boundary | enforced | `src/main.rs:1` `#![deny(unsafe_code)]`; exactly one `#[allow(unsafe_code)]` at `src/backend_cranelift.rs:732` | — |
| Exactly five pinned direct Cranelift 0.133.1 crates; no general dependency permission | enforced | Root `Cargo.toml:21-25` pins cranelift-codegen/frontend/jit/module/native at `=0.133.1`; locked in `Cargo.lock` | — |
| Self-hosting only after staged differential tests and clearer compiler code than the Rust version | not-started | `experiments/` holds only feasibility/lowering-contract probes, excluded from the workspace; no self-hosting harness exists | none assigned |

### 0003 — Keep Milestone 0 local, non-executing (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Must not execute Hum programs | partial (superseded) | `hum run` shipped (`src/run.rs`); execution is now gated on the checked resolver — unchecked bodies refused (exit 2 `blocked_by_unchecked_body_types_v0`, per ledger #17) | Decision 0011 (accepted) — the checked-resolver gate that lifted the moratorium |
| Local, offline-first: no network fetch | enforced | No network code in `src/` (no TcpStream/reqwest/hyper); no network crates in root `Cargo.toml` — enforced by absence; no negative test pins it | — |
| Must not build packages, run plugins, fetch dependencies, or publish artifacts | enforced | `hum` command list (`src/main.rs`) has no publish/pack/fetch/plugin subcommand — enforced by absence | — |

### 0004 — Make tests first-class evidence (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| `test` is a first-class top-level item | enforced | `src/parser.rs:2392` (accepts module, app, type, store, task, and test at the top level), `:2529` | — |
| Task obligations link to exact or conservative canonical `covers:` lines when present | enforced | `src/graph.rs:56-77` (`covers` collection, `coverage_match_kind`, `linked_test_count`); `hum evidence` renders linked/unlinked (`src/evidence.rs:54-69`) | — |
| `check_all.ps1` fails when the reference fixture has unlinked security/trust evidence obligations | enforced | `tools/check_all.ps1:660,670` | — |
| `hum test-skeletons` proposes missing evidence without writing files | enforced | `src/test_skeletons.rs`; `check_all.ps1:1872` asserts exit 0 with empty stdout and stderr exactly "no unlinked test obligations" | — |

### 0005 — Keep verifiers as evidence producers (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Hum emits obligations; external engines are evidence producers, not language truth | enforced | `hum math-obligations` (`src/math_obligations.rs`, in `src/main.rs` command list); boundary documented in `docs/MATH_ENGINE_BOUNDARY.md` | — |
| External engines return receipts: proved / refuted / unknown / unsupported / timeout; unknown distinct from failure | declared-only | No receipt type, schema, or ingestion command in `src/` (`math-obligations` exists; no `math-receipt`); `math_obligations.rs` emits timeout budgets but no receipt vocabulary | none assigned |
| `proved` accepted only with certificate / independently checkable trace / profile-allowed artifact | declared-only | No receipt-validation code found | none assigned |
| No external engine may silently optimize, rewrite semantics, require cloud, collect telemetry, or hide assumptions | partial | No external engine integrated at all (enforced by absence for now); boundary stated in `docs/MATH_ENGINE_BOUNDARY.md` — no code gate would catch a future violating integration | none assigned |

### 0006 — Make resource, layout, and compile-time power explicit (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Resource intent belongs in checked source blocks (`cost:`, `allocates:`, …) | partial | `src/resource_check.rs:686-704` parses `allocates:`/`cost:`; diagnostic `task_body_requires_explicit_allocates_intent_v0` at `:631`; `avoids:`/`tradeoffs:`/`optimizes:` unchecked | none assigned |
| Compile-time execution must be explicit, effect-limited, budgeted, provenance-preserving, profile-gated | not-started | No comptime/const-eval code in `src/` (consistent with "future work") | none assigned |
| Interop enters only through explicit trust/ownership/layout/effect/failure/profile contracts | not-started | No FFI surface in `src/` (only the Cranelift JIT boundary, `src/backend_cranelift.rs`) | none assigned |
| Agents get compact schema-backed facts; compiler emits JSON, not terminal prose | enforced | `hum check --format json` (`src/main.rs:2158`), `hum explain` (`src/main.rs:220`), JSON emitters (`core_lower_json`, `backend_probe_json`) | — |
| Layout assumptions exposed as semantic graph facts (ABI, alignment, endian, …) | partial | `hum target-facts` exists, but `src/target_facts.rs` covers capability families, not ABI/alignment/endian/pointer-width | none assigned |

### 0007 — Progressive disclosure and migration discipline (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| No major feature stable until it has syntax, semantics, diagnostics, graph facts, tooling/profile impact, tests/evidence, teaching material | partial | Pre-issuance review gate (repo `AGENTS.md`) checks authority/evidence linkage; process-only, no code gate | none assigned |
| Reject special-case syntax; prefer one general checkable mechanism | partial | Decision 0027 (accepted) is the instrument, cited in the 0028/0029 records; enforcement is reviewer judgment, no automated check | Decision 0027 (accepted) |
| Backend/performance claims require hard evidence (fixtures, targets, harnesses, baselines) | partial | Decision 0024 (accepted, performance north star); Session AG byte-exact assertions (wordfreq) as the working pattern | Decision 0024 (accepted) |
| Major design work begins with a research sweep or design note | partial | `docs/research/` holds sweeps (e.g. agent-repair benchmark design); process-only | none assigned |

### 0008 — Swappable backend ladder (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Interpreter first, to prove executable semantics without backend complexity | enforced | `Interpreter` in `src/run.rs:722`; `hum run` executes through the type gate | — |
| Cranelift next, as first native backend candidate | partial | `src/backend_cranelift.rs` wired into the run path (`src/run.rs:152,158`); `hum backend-probe` (`src/main.rs:338`); coverage narrow (integer-sign/constant-text probes, not general native execution) | none assigned |
| Hum IR owns semantics; backends are adapters behind a narrow contract | partial | `src/backend_contract.rs`, `src/ir_contract.rs` exist; `adapter_must_preserve_or_report_loss` (`backend_contract.rs:78,116`); machine-checking depth of the contract not verified | none assigned |
| Every adapter must preserve-or-report: spans, graph node IDs, typed failure, effects, ownership, resources, profiles, unsafe/foreign boundaries, provenance, unsupported features | partial | Stated in `src/backend_contract.rs:78`; per-item coverage in the Cranelift adapter not verified | none assigned |

### 0009 — Formal readability, not English mimicry (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Stable syntax must lower to precise Core Hum ops; a phrase that cannot lower precisely is a diagnostic, not a feature | partial | `src/core_lower.rs`, `src/core_verify.rs`; `hum core-preview`/`core-lower`/`core-verify`; parser rejects unexpected top-level lines (`src/parser.rs:2385-2392`); core lowering is a preview surface — the execution path checks surface Hum directly, so no "unlowerable ⇒ diagnostic" gate found | none assigned |
| Reject synonym sets for core concepts | declared-only | One keyword per concept in practice, but no synonym-rejection mechanism found | none assigned |
| Section prose is not executable authority until the compiler owns a checked grammar and graph fact for it | partial | Parser/resolver/checker own the section grammars (unknown top-level lines rejected) | none assigned |
| Diagnostics teach the formal model behind friendly text | enforced | `src/diagnostic_catalog.rs` H-codes carry blame-style help; surfaced via `hum explain` (`src/main.rs:220`) | — |

### 0010 — Explicit state model (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| `set name = ...` requires local `change name: ...` or a matching `changes:` entry (V0) | enforced (V0 scope) | `src/check.rs:681,707` fire diagnostics ("saves into … without listing it in `changes:`"; "Use `change {target}: Type = ...`…"); contract documents it (`src/state_model.rs:145`); the V0 mode name `contract_only_partial_declared_mutation_check` itself admits partial coverage | — |
| Immutable default; local mutation via `change`/`set`; external mutation in `changes:`; external reads/authority in `uses:` | partial | set/change/changes rule enforced; `uses:`/authority enforcement not verified (`src/ownership_check.rs`, `src/effect_check.rs` exist but not audited) | none assigned |
| Profiles may narrow state permissions but cannot hide state authority | partial | `src/profile_check.rs`, `src/runtime_profiles.rs` exist; the no-hiding property itself not verified | none assigned |
| `hum state-model --format json` emits `hum.state_model.v0` | enforced | `src/main.rs:185`; `src/state_model.rs` | — |

**Cross-cutting (0001–0010):** 0003's non-execution claim is the only
doctrine claim *superseded* rather than pending — lifted by accepted
decision 0011. Strongest clusters: 0004 (tests-as-evidence), 0002's
trust-root rules, 0010's V0 set/change rule. Weakest: 0005's receipts
(obligations emitted, receipts don't exist in code), 0006's
comptime/interop/layout-facts, 0007's stability checklist (process-only).
Honesty caveat: "enforced by absence" (no network code, no FFI, no publish
command) is real but brittle — no negative test pins it.

## Decisions 0011–0020 (language core)

### 0011 — Add checked resolver before execution (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| `hum resolve` is the first checked name/scope/reference/mutable-place report (V0 schema) | enforced | `src/main.rs:740` `"resolve"` subcommand; `resolve::resolve_json`/`resolve_text`; 10 `H060x` codes in `src/diagnostic_catalog.rs` (H0601 at :2550) | — |
| Resolver runs before lowering/type/effect/ownership checks and before execution | enforced | `src/main.rs` `execute_run_command`: `resolve_has_errors` (~:1228) returns before invoking the runner; stage order resolve → type_check → full_type_check at :744–866 | — |
| `set` targets must resolve to mutable places or declared change permissions | enforced | `src/resolve.rs:2609` `set_target_immutable_v0` ("cannot mutate immutable place") | — |
| V0 honesty: resolver does no type/borrow/effect checking | enforced | `src/resolve.rs:3-16` imports ast, core_body, diagnostic, graph, predicate only — no type/effect/ownership modules | — |
| Downstream stages agree with resolver scope rules | partial | Ledger #15 (resolver admits contract-only builtins the checker doesn't reject), #16 (checker/runtime don't honor resolver block scopes — including a wrong-acceptance probe) | WO28 #15, WO28 #16 |

### 0012 — Adopt snake_case identifiers (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Value identifiers match `[a-z_][a-z0-9_]*`; type names PascalCase | enforced | `src/parser.rs:10243-10254` (`snake_identifier`), :9591, :9657 char rules | — |
| Spaced name is a parse error with a dedicated diagnostic suggesting snake_case | enforced | `INVALID_IDENTIFIER`, Severity::Error (`src/diagnostic_catalog.rs` ~:3171); repair text suggests snake_case spelling (:3266) | — |
| All examples/fixtures/docs migrated; no new spaced-name examples | enforced | Spot check: no spaced `task` names in `fixtures/`, `examples/`, `programs/` | — |
| Coverage-key matching stops absorbing filler words | enforced | `src/graph.rs:237-260`: `coverage_key` does case/punctuation token normalization only — no filler-word list | — |
| Test names may stay multi-word until the test grammar is pinned | unverified | Parser identifier rules checked only; test-grammar pinning status not audited | none assigned |

### 0013 — Remove the Number type (accepted)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| `Number` removed; core numerics are `Int` and `UInt` | enforced | No `Number` Hum type in `src/` (only JSON `Number` in `src/ir_verify.rs`, unrelated); none in `fixtures/`, `examples/`, `programs/` | — |
| No implicit narrowing, no implicit signedness conversion | enforced | No implicit-conversion/narrowing code path in `src/full_type_check.rs`; `Int`/`UInt` are distinct nominal types (:1995) | — |
| Defined overflow diagnostics before the core is called stable | partial | Runtime uses `checked_add/sub/mul/div` failing safe into typed failure (`src/run.rs:1915-1921`); no dedicated overflow H-diagnostic in `src/diagnostic_catalog.rs` (grep "overflow": none) | none assigned |
| `Float` stays outside the trusted core until a float policy is accepted | unverified | `Float` exists in surface type lists; its exclusion from core stages not verified | none assigned |

### 0014 — Adopt ownership and borrowing (accepted under delegated authority, BDFL veto open)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Values owned by default; `borrow`/`change`/`consume` authorities | enforced | `src/ownership_check.rs` tracks borrow/change/consume; H0802 (acquire from borrowed param) in catalog | — |
| Checker is place-based (records, fields, elements, indexes, params, views) | enforced | `src/field_place.rs`, `src/element_place.rs`; place-based facts in `src/ownership_check.rs` | — |
| Session V: local writable aliases `let alias = change owner.field`; H0808/H0809 reject overlap/escape | enforced | `src/diagnostic_catalog.rs:2964` H0808, `:2973` H0809; `writable_field_alias` used by `src/resolve.rs`; probe `examples/probes/writable_field_aliases.hum` | — |
| Linear resources first-class for exactly-once protocols | partial | `linear_resources` map + `linear_double_consume_diagnostic` (`src/ownership_check.rs:208-209,1887-1888`); completeness not established | none assigned |
| Honesty lock: no full memory-safety/borrow-soundness claims until repairs built | enforced (as lock) | `src/ownership_check.rs:29` "no complete borrow checker"; decision's own lock list | — |
| Required repairs: internal references, disjoint-field projection (beyond Session V slice), flow-sensitive borrowing | not-started | Listed as required in the decision; no implementation found | none assigned |

### 0015 — Adopt classified runtime contract policy (accepted under delegated authority, BDFL veto open)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| `hum run` checks every executable predicate contract; no classifier/elision | enforced | `src/run.rs:1724` iterates predicate facts; `:1816` "caller did not satisfy needs:"; no contract-classification code in `src/predicate.rs`/`src/run.rs` | — |
| Classification vocabulary (proved/boundary/unproved/external-trust); unknown fails closed as unproved | declared-only | Defined in the decision text; no classifier implementation found | none assigned |
| Release mode may elide only mechanically proved contracts with evidence | not-started | No release mode / elision path exists | none assigned |
| No body guard called unreachable or removed until classification evidence | enforced | No guard-removal logic in `src/check.rs`; `examples/core/divide.hum` keeps its guard; contracts remain checked | — |
| `needs:` is not a substitute for parsing/validating external input | declared-only | Policy statement; no code cite found | none assigned |
| Contract text literals decode and get H0638 coverage | partial | H0638 exists (`src/diagnostic_catalog.rs:2856`) but ledger #4: contract literals neither reject invalid escapes nor decode | WO28 #4 |

### 0016 — Adopt explicit causal typed failure (accepted under delegated authority, BDFL veto open)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| `try` surface: `try f()`, `try f() or fail OuterError.context`; keyword-bounded | enforced | `src/parser.rs:4761-4795,5261-5306` try parsing; :5716 keyword list (`trying`/`try_value` unaffected) | — |
| Nominal roots: unwrapped `try` needs same root (H0902); wrapping uses caller root (H0903); direct `fail` uses caller root (H0905) | enforced | `src/diagnostic_catalog.rs:2982-3005` H0901–H0906 suite, all owned by `full_type_check` | — |
| No implicit propagation, including nested in operators/arguments/loop collections | enforced | H0901 "fallible call requires try" | — |
| Typed failure is exit-1 (not a trap); causal chain rendered outer-to-root | enforced | `src/run.rs:6271` `causal_failure_chain_keeps_outer_to_root_sites`; `:5138` app-path test; `examples/probes/causal_failures.hum` | — |
| `fails when:` must state a meaningful condition; hollow-contract rule | enforced | H0110 hollow contract line (`src/diagnostic_catalog.rs:2443`); H0107/H0109 intent-shape warnings; `src/check.rs:362` `fails when` section check | — |
| Unsupported `try` shapes are H0906 blockers | enforced | H0906 "unsupported try expression" | — |

### 0017 — Adopt structural app authority boundary (accepted under delegated authority, BDFL veto open)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Exactly one top-level `app`; exactly one `starts with:` with one bare snake_case name; start task returns `Unit`/`Result Unit, E` | enforced | `src/app_entry.rs:58,156,201` diagnostics; ledger #9 notes the `Unit` requirement by design | — |
| App-mode lookup is lexical, never falls back; `--entry` stays a pure-task probe | enforced | `src/app_entry.rs` lexical analysis; no fallback path found in `src/app_entry.rs` | — |
| Source capability vocabulary is exactly `stdout.write`, `clock.replay`, `files.read`; unknown capability-like IDs fail closed | enforced | `src/capability_root.rs:9` `CAPABILITY_IDS`; `:162` `unknown_capabilities` | — |
| Operator grant algebra: declaration never consent; app-max × task-closure × grant intersection; default empty; deny wins | enforced | `src/operator_grant.rs:29` `default_empty_grant_set_v0`; `:28` `exact_deny_overrides_allow_v0`; deny-wins tests at :164, :216 | — |
| Path boundary: opaque `Path`, no source literal; Windows lexical fail-closed rejections; threat-scoped `fixed_local_v0` | enforced | `src/path_boundary.rs:9` diagnostics; `crates/windows-drive-locality`; `src/run.rs:5307` `fixed_local_v0`/`locality_unclassified`; H0618/H0621/H0624 in catalog | Decision 0029 (accepted; implementation queued in WO28 after #7); ledger #17 hosted-runner wall |
| `#![deny(unsafe_code)]` default; exactly one reviewed JIT exception (WO22 Unit B) | enforced | `src/main.rs:1` `#![deny(unsafe_code)]` | — |
| Portable non-Windows file read | not-started | `files_read_text` Windows-only (ledger #7) | WO28 #7 |
| Future audit trail joining source snapshot, operator decision, operation exercise | not-started | Decision text only | none assigned |

### 0018 — Effect polymorphism model (accepted under delegated authority, BDFL veto open)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Model selected: open row-polymorphic latent effects + explicit capture/ownership/authority companions | declared-only | Model selection only; the decision explicitly authorizes no production syntax, compiler, runtime, Core, or graph-fact work | none assigned |
| Honesty locks: no production effect-polymorphism/closure/handler/inference claims until implemented | enforced (as locks) | No row-polymorphism in `src/effect_check.rs` (only external-authority boundary rows); no production effect-row claims in SPEC | — |
| Decision 0016 stays binding through any future effect work | declared-only | Forward constraint in decision text; no future work exists to test it against | none assigned |

### 0019 — Relicense Apache-2.0 (accepted by the BDFL; licensing is a reserved matter)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| `LICENSE` contains verbatim Apache-2.0 text | enforced | `LICENSE` head reads "Apache License, Version 2.0, January 2004" | — |
| `Cargo.toml` declares `license = "Apache-2.0"` | enforced | `Cargo.toml:5` | — |
| `NOTICE.md` carries copyright + courtesy attribution request | enforced | `NOTICE.md`: "Copyright (c) 2026 Ocean Bennett" | — |
| `TRADEMARK.md` states the Hum/hum-lang naming policy | enforced | `TRADEMARK.md`: name policy; Apache-2.0 §6 non-grant cited | — |
| All AGPL/Affero references removed | enforced | Repo-wide grep finds AGPL only inside the 0019 decision doc itself | — |

### 0020 — Termination measures and loop bounds (accepted as design only; authorizes no implementation)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Termination contract (`decreases:`) and quantitative bound (`cost: iterations: at most`) as separate concepts; `exactly` compiler-derived only | not-started | No `decreases` in `src/`; record authorizes no implementation by its own status line | none assigned |
| New termination stage after ownership check in the pipeline order | not-started | No termination stage in `src/` | none assigned |
| Salvage regression matrix becomes permanent fixtures before any public claim | not-started | No such fixtures | none assigned |
| Public `may diverge:` syntax deferred pending a separate decision | declared-only | Deferral recorded in decision text | none assigned |

**Cross-cutting (0011–0020):** WO28 is the main gap-closer for 0011
(#15, #16), 0015 (#4), and 0017 (#7, plus 0029's implementation queued
after #7). 0018 and 0020 are accepted records that deliberately authorize
no implementation — their "not-started" status is by design, not drift.
Notable unassigned gaps: the 0015 contract classifier, 0014's repair
roadmap (internal references, disjoint-field projection, flow-sensitive
borrowing), 0013's overflow diagnostics, and all of 0020.

## Decisions 0021–0029 (recent)

### 0021 — text_split primitive (accepted 2026-09-22)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| `text_split(text, sep)` splits on a single separator, non-overlapping left-to-right, preserving leading/trailing/repeated empty pieces | enforced | `src/run.rs`, `src/full_type_check.rs`; fixtures `examples/probes/text_split*.hum`; commit 9b2e923 (WO27 Part 1a, PR #13) | — |
| A directly written empty separator is checker error H0636; a runtime-computed empty separator raises `TextSplitError.SepEmpty` via try/fail | enforced | H0636/H0637 in `src/diagnostic_catalog.rs`; `src/typed_failure.rs` | — |
| Owned copies per piece are deliberate performance debt, not a settled optimum | declared-only (as debt) | Researcher ledger PD-001; Part 1a commit note | none assigned (hardening batch parked) |

### 0022 — text literal escape sequences (accepted 2026-09-22)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Program text literals support escapes; the runner decodes them; invalid escapes are H0638 | enforced | H0638 `invalid_text_escape` in `src/diagnostic_catalog.rs` (fires in `full_type_check`); commit cf1599a (WO27 Part 1b) | — |
| Contract (`ensures`) text literals get the same escape checking and decoding | partial | Ledger entry 4: `predicate.rs` takes contract literals verbatim — no H0638, no decode | none assigned (checker work beyond wordfreq's authorization) |

### 0023 — dispatched Full counts as integration health (accepted 2026-09-22)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Integration health comes from dispatch-triggered Full runs on main; push-triggered runs don't count; the 03:17 UTC nightly renews the anchor | enforced | `.github/workflows/validation.yml` dispatch schedule; regression test commit f5304f6 | — |

### 0024 — performance north star (accepted 2026-09-22)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Check and CI time are a performance budget too; for agents the tighter one; ~40-min CI is process cost, not compiler speed | declared-only | North-star statement; no compiler gate encodes it | none assigned (CI speed work measures first, per standing direction) |
| Performance debt is recorded in a ledger, not silently absorbed | partial | `docs/research/2026-09-22-performance-debt-ledger.md` exists (PD-001 text_split owned copies, PD-005 verification feedback speed); entries are added manually at commit time, no check enforces ledger updates | none assigned |

### 0025 — CI routing / docs governance / tooling gaps (accepted 2026-09-23)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Ownership registry: `docs/`+`workorders/` classify by path at language rank; renames take max rank; `tools/` hygiene scripts pinned at language rank; the eight `include_str!` docs keep compiler rank | enforced | `tools/check_ci_policy.ps1`; live on main via PR #14; validated by docs PRs #17/#18/#19/#25/#26 on the Language profile | — |
| Skip the Work Order status-boundary suite when no consumer of a changed path changed (the 0025 amendment) | enforced | PR #33 (`fix/boundary-forward`, merged as main 3853972) fixed the forward: classify now forwards `boundary_required` when the event is not `push` and the input is exactly `true`/`false`. Measured on PR #35's run 35967934714 — the first non-consumer `pull_request` validation run after #33 merged (docs/research/ only, Language profile): the "Skipping Work Order status-boundary classifier tests" line printed on both OSes; preflight wall-clock Ubuntu 3m44s, Windows 4m39s (Language docs PRs were ~8/15 min before #33). The suite is skipped exactly when no consumer changed, as the amendment requires | Closed by PR #33 (merged 2026-09-24) |

### 0026 — closed-world accountability as scoped design objective (accepted 2026-09-23)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Accountability is a scoped design objective with a comparative-evidence plan; the non-bolt-on exclusivity claim stays withdrawn | declared-only | `docs/research/` 0026 study design; the enforced-Rust-profile comparison study is not run | none assigned (study not scheduled) |

### 0027 — program-driven language development (accepted 2026-09-23)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Real programs drive the language; friction goes in the program's ledger; missing surface is a STOP, never a workaround; gaps become decision records first, then implementation with tests+docs in the same PR | enforced (as process) | `docs/research/wordfreq-friction-ledger.md` (17 entries, maintained); STOPs exercised → decisions 0021/0022/0028; builder lane never writes decision records | — |
| Default no on additions; the general-vs-specific test before proposing | enforced (as process) | 0028 Option E (concat deferred for lack of motivating evidence); 0029 §10 (CI carve-out rejected) | — |

### 0028 — text rendering: integer conversion now, concat deferred (accepted 2026-09-23, Option E)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| `uint_to_text` / `int_to_text` builtins, one per integer type | not-started | No hits in `src/` as of 2026-09-24 | Builder lane (WO28 follow-up or new WO — none assigned) |
| Concatenation deferred until a program demonstrates it must build Text as a value; reopen trigger is a friction entry | declared-only | Trigger condition stated; concat does not exist | none assigned (config parser is the expected demonstrator) |

### 0029 — trusted-local storage for file reads (accepted 2026-09-24, Option D)

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Fixed-local requires the P1–P4 property (not network-backed, stable identity, no mid-read substitution, ordinary file); each platform proves it its own way | not-started | Accepted 2026-09-24; current code still the ATA/SATA/NVMe proxy (`crates/windows-drive-locality/src/lib.rs:204`) | Builder lane (implementing WO — none assigned); eMMC/SD P3 argument is research-lane evidence for that WO |
| Where unprovable, trust is explicit and labelled: per-invocation, per-path, CLI-only operator grant (no env/config/persistent setting), recorded as external-trust | not-started | — | same as above |
| virtio-blk and guest-invisible backing are always grant, never proof | declared-only | Ruling recorded; no code change yet | same as above |
| Labelled trusted-not-proven evidence satisfies WO28 #13; byte-exact output remains proven | declared-only | Ruling recorded; WO28 #13's done-condition caveat resolved | — |

## SPEC (`docs/LANGUAGE_REFERENCE.md` + `docs/LANGUAGE_SUBSET_0_1.md`)

40 claims checked: 34 enforced, 5 partial, 0 declared-only, 0
not-started, 0 unverified. Every claim has code plus tests behind it;
the partials carry named, WO-assigned gaps.

### Names, lexical, resolution, types

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Value names snake_case `[a-z_][a-z0-9_]*`, type names PascalCase; spaced name → parse error with snake_case help | enforced | `src/parser.rs` `invalid_identifier` (~3248–3270), help via `snake_identifier` (`:10259`); test `rejects_spaced_task_name_with_snake_case_help` (`:14327`); catalog `:2343` | none assigned |
| Source files are UTF-8 without BOM | partial | Invalid UTF-8 fails at load (`src/main.rs:3433` `fs::read_to_string`); but BOM is rejected only by repo CI (`tools/check_text_hygiene.ps1:222`) — the compiler gives a BOM-prefixed `.hum` only an unrelated top-level-line warning, no BOM diagnostic | none assigned |
| Comments may start with `#` or `//` inside sections | enforced | `src/parser.rs:10321` `is_ignorable`; section-body skip at `:5543`, `:5790`; test `preserves_comment_lines_inside_sections` (`:14435`) | none assigned |
| `hum resolve` = first checked pass (scopes, definitions, references, mutable-place targets); emits `hum.resolve.v0` JSON | enforced | `src/main.rs:740`; `src/resolve.rs:18` (`RESOLVE_REPORT_SCHEMA`), `:926`; mutable-place tracking `:302/362/370`; schema tests `:3960`, `:4014` | none assigned |
| Duplicate definitions and undeclared names rejected with stable diagnostics | enforced | `src/resolve.rs:2549–2593`: `UNRESOLVED_NAME` (H0601), `DUPLICATE_NAME_IN_SCOPE` (H0602), `Severity::Error`; test `:3975`; catalog `:2546`, `:2555` | none assigned |
| `hum type-check` validates declaration annotation names only (no expression inference/body checking); `hum full-type-check` validates more | enforced | `src/type_check.rs:22` (`declaration_annotation_and_trivial_return_check_v0`), non-claims `:25–35`; test `:2564` (H0605); `src/full_type_check.rs:21` + `fixtures/full_type_check/` corpus | none assigned |
| Returned-view rule: `from` must name a task parameter; view depending on a local → H0805; non-closed chains rejected, not guessed | enforced | `src/ownership_check.rs:2548–2585` (local source → H0805), `:2811–2840` (non-closed → H0805); catalog `:2937`; fixtures `fixtures/ownership_check/session_l_*`, `session_n_*` | none assigned |
| `hum graph` exposes the declared from-source dependency fact | enforced | `src/json.rs:517` (`return_dependencies` per task, `source_kind` parameter/internal_reference/unknown); schema `docs/SEMANTIC_GRAPH_SCHEMA.md:204` (caveat: no dedicated unit test asserts the fact in graph JSON output) | none assigned |

### Ownership

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Unmarked parameters default to `borrow`; mutation/transfer visible via `change`/`consume` | enforced | `src/parser.rs:10219–10229` (Borrow default); `src/ast.rs:575–579` (enum) | none assigned |
| Writing through a default-borrow param or its direct field → H0802, static and runtime; borrowed param can't supply writable alias | enforced | Static `src/ownership_check.rs:1409` (cause 111 → H0802, catalog `:1109–1117`); runtime `src/run.rs:3622–3642`; alias preflight rejects borrowed owners (`run.rs:1561–1603`); fixtures `session_j_borrow_write_fail.hum`, `session_o_field_write_borrow_fail.hum`; CI `check_all.ps1:4006`, `4057` | none assigned |
| Use-after-move (via `consume` or `return`) → H0801 | enforced | Static `src/ownership_check.rs:2120–2140` (cause 110); runtime `mark_moved` (`run.rs:2515`, `:2019`) + `use_after_move_invariant` (`:3524`); fixture `session_j_use_after_move_fail.hum`; CI `check_all.ps1:2504–2523` | none assigned |
| Transaction-shaped resource unconsumed → H0803; consumed twice → H0804 | enforced | Static `src/ownership_check.rs:2097` (cause 112), `:2194` (cause 113); runtime traps `run.rs:3694`, `:3610`; fixtures `session_k_*`, `session_j_double_consume_fail.hum`; CI `check_all.ps1:4011–4026` | none assigned |
| Violating the V0 returned-view from-parameter rule → H0805 (runtime side) | enforced (V0 narrow rule) | Runtime `ensure_return_dependency` (`run.rs:1540`) → H0805 (`:3827–3842`); internal references fail closed by design (`ownership_check.rs:2561`) | none assigned |
| Appending to a list during active iteration → H0806 | enforced (runtime only) | `src/run.rs:3411` (`iteration_mutation_trap`); fixture `session_p_append_during_iteration_fail.hum`; CI `check_all.ps1:4069`; no static emission (SPEC only says "traps") | none assigned |
| Using a local field/element view after the field was written or the list grew → H0807 | enforced | Static `src/ownership_check.rs:1980–2175`; runtime `stale_view_trap` (`run.rs:3530`); invalidation on `set` (`:2152`) and on `list_append` growth (`:3428`); fixtures `session_r_*`, `session_s_*`; CI `check_all.ps1:4087–4115` | none assigned |
| Writable field aliases: binding→last-use lifetime; overlap → H0808; branch/loop/escape/alias-to-alias/nested/element/rebind/shadow → H0809 | enforced | `src/writable_field_alias.rs:60–90` (cause keys 116/158–160 → H0808; 117/163–176 → H0809); static `ownership_check.rs:1702–1717`; runtime `preflight_writable_aliases` (`run.rs:1552–1640`); 13 `session_v_*` misuse fixtures + `examples/probes/writable_field_aliases.hum`; CI `check_all.ps1:4472–4532`, `5898–6014` | none assigned |
| `hum ownership-check` verifies the narrow parameter-derived returned-view subset | enforced | `src/main.rs:928`; `return_dependency_fact` (`ownership_check.rs:2540–2600`), fails closed on non-parameter sources | none assigned |

### Effects, capabilities, app authority, file I/O

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Denied capabilities (network.*, process.*, ffi.*, unsafe.*, thread.*, clock.wall, random.*, env.read, plugin.*, dynamic_load.*) produce a stable registered diagnostic | enforced (with the subset doc's hedge) | Unknown capability spellings → H0617 `UNKNOWN_SOURCE_CAPABILITY` (`diagnostic_catalog.rs:2656–2664`; `src/capability_root.rs:1703–1740`); sandbox-bypass roots (process/ffi/unsafe/import) get a distinct message + `sandbox_bypass_authority` severity tier (`capability_root.rs:2295–2300`; CI `check_all.ps1:4922–4933`); fixtures `session_y_unknown_capability_fail.hum`, `session_y_entry_transitive_process_fail.hum`. Caveat: `network.*`, `thread.*`, `clock.wall`, `random.*`, `env.read`, `plugin.*`, `dynamic_load.*` get only generic H0617 wording — matching the subset doc's "reserved profile family allocates no exact code" note | none assigned (hedge holds) |
| App `uses:` is the capability maximum; tasks declare direct+transitive budgets; callers cover callee closure; missing task/app authority → H0621; missing caller closure → H0618 | enforced | H0618 (`capability_root.rs:1740/1889`); H0621 (`:1980/2110`); app-maximum H0619 (`:1782/1939`); fixtures `session_y_missing_caller_capability_fail.hum`, `session_y_app_capability_mismatch_fail.hum`; CI `check_all.ps1:3114`, `4901–4902`, `4981–4997` | none assigned |
| Default deny; repeatable `--allow`/`--deny`; duplicates idempotent; deny wins | enforced | `src/operator_grant.rs` denies-first (`:107–115`), `exact_deny_overrides_allow_v0`/`default_empty_grant_set_v0` (`:24–34`); idempotent via BTreeSet (`:59–74`); repeatable parsing `src/main.rs:2081–2092`; unit tests `:161–260`; Session AD/AG denied rows (CI `check_all.ps1:5513`, `:5226`) | none assigned |
| Reserved builtin names: `stdout_write` → H0623; replay names → H0625–H0628; `files_read_text` → H0633; Path outside start param → H0629; source construction/use → H0630 | enforced | H0623 (`src/check.rs:120–129`); H0627 (`:132–143`); H0633 (`:145–154`); H0629 (`src/path_boundary.rs:439`); H0630 (`path_boundary.rs:304/420`, `callable.rs:2386/4504`); H0625/26/28 (`capability_root.rs:2017/2168`, `full_type_check.rs:965/974`); fixtures `session_ad_reserved_file_read_name_fail.hum`, `session_ab_*_fail.hum`; CI `check_all.ps1:5002–5012`, `5265–5327`, `5493`, `5407–5411` | none assigned |
| Output-reachable recursion → H0624 (checked after the authority route is valid); replay-reachable recursion → H0628; not a general recursion ban | enforced | H0624 `capability_root.rs:614–650`; fixture `session_z_output_recursion_fail.hum`; CI `check_all.ps1:5017–5035` (precedence rows prove H0621/H0618 suppress it); H0628 `:675–711`, fixture `session_aa_replay_recursion_fail.hum`, CI `:5321–5327` | none assigned |
| `files_read_text` has exactly `files_read_text(path: Path) -> Result Text, FileReadError` on the structural start task; H0631 = complete `files.read` source closure; H0632 = exact Path signature | enforced | H0632 `src/full_type_check.rs:994–1010` (runner-owned opaque `Path` required); H0631 `src/capability_root.rs:2054–2081` (task AND app authority); fixtures `session_ad_file_read_wrong_type_fail.hum`, `session_ad_missing_file_source_fail.hum`; CI `check_all.ps1:5491–5492`. "Start task only" holds effectively because Path can only enter via the start parameter (H0629/H0630) | none assigned |
| FileReadError taxonomy: denied → outside_grant → unsafe_path/unavailable → not_found/not_file/too_large/invalid_utf8/io_failed, ordered before candidate access | enforced | `src/run.rs:2751–2890` ordered exactly as claimed; adapter errors `src/file_read.rs:17–58`; CI pins Session AD denied (`check_all.ps1:5513`), Session AG denied + locality-refusal `unavailable` (`:5226`, ledger #17) | none assigned |
| Session AC locality: `fixed_local_v0` only on agreeing evidence; zero-access synthesized devices; never opens the candidate path | enforced (current rule) | `crates/windows-drive-locality/src/lib.rs`: `classify_evidence` (`:167–212`) requires before==after observations, empty storage-dependency result, complete bounded extents, non-removable ATA/SATA/NVMe for every backing disk (`:204`); opens only synthesized volume/disk devices with desired access zero (`:225`, `:396–410`, `:582–610`); CI IOCTL allowlist + forbidden-host-surface invariants (`check_all.ps1:2534–2542`) | Decision 0029 (accepted, Option D, PR #32 merged); implementation queued in WO28 after #7 |
| Windows Path: only ordinary drive-letter-rooted candidates; rejects namespace/traversal/ADS/empty-component/trailing-dot-space/DOS-device; non-Windows Path execution unavailable | enforced | `src/native_path.rs:203–290` (`Prefix::Disk` only, ADS/empty/traversal/trailing-dot/DOS-device rejections); non-Windows `UnsupportedHost` (`:198–201`) → `unavailable` (`file_read.rs:73–79`) | Non-Windows success path → WO28 item #7 (ledger #7) |
| `files_read_text` reads ≤ 1 MiB (`too_large`), strict UTF-8 (`invalid_utf8`); `stdout_write` exact bytes, no newline, 1 MiB rolling limit | enforced | `FILE_READ_LIMIT_BYTES` (`file_read.rs:9`); bounded read + strict decode (`:161–172`); `OUTPUT_LIMIT_BYTES` (`run.rs:51`), `write_all` no newline (`:97–102`); unit test `run.rs:5883` (rolling limit rejects before second adapter call) | none assigned |
| `--entry` is a pure probe: rejects authority-bearing task closure even when allowed; cannot inject Path | enforced | H0620 `ENTRY_CAPABILITY_BYPASS` (`capability_root.rs:419`); three-session CI coverage (Session Y/Z/AA) asserting empty stdout, no trap (`check_all.ps1:4929`, `:4967`, `:5358`). Path injection blocked by the opaque Path boundary; no dedicated `--entry`+Path fixture found | none assigned |

### Contracts, control shape, evidence, resources, targets, diagnostics, native

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| `needs:` runs at task entry blaming the caller; `ensures:` runs after return blaming the task; `old(place)` captures entry pre-state; violations exit with blame diagnostics | enforced | `src/run.rs:1795–1835`; `needs` false → H0702 (test `run.rs:6207`, `examples/core/divide.hum`); `ensures` false → H0703 (test `:6372`, `fixtures/run/wrong_add_contract.hum`); `old(point.y)` (test `:6400`, `fixtures/run/session_t_wrong_swap_contract.hum`) | none assigned |
| Comparisons non-chainable: `1 < 2 < 3` rejected by parser-owned H0010 at the later operator | enforced | `src/parser.rs:2272`; tests `:15459` (span at second `<`, first as related site), `:15603` (sealed-corruption fail-closed) | none assigned |
| Predicate v2: one comparison per predicate; v1 arithmetic/places/`old`/`list_len`; + exact Text `==`/`!=`, ordered `List Text` `==`/`!=`, contract-only `list_count`; exclusions hold; H0701 prose-without-intent; H0704 malformed candidates | partial | Core enforced: `src/predicate.rs` grammar (`:1249`); recognition-boundary tests `:1886–1960`; H0701 warning `run.rs:1795`; H0704 `full_type_check.rs:1886/1892`. Gap (ledger #4): contract text literals neither reject invalid escapes (no H0638 in contract sections) nor decode — contract text doesn't mean the same as body text | WO28 #4 |
| Session W: only two explicit `try` forms; no implicit propagation; stable outer-to-root causal chain with root origin | enforced | `src/typed_failure.rs:932` (exact forms only; nested calls/operators rejected); `:546/:765` ("failure cannot be implicit"); causal-chain test `run.rs:6271` (`examples/probes/causal_failures.hum`) | none assigned |
| Integer overflow and division by zero trap, never wrap | enforced | `src/run.rs:2305–2318` (`checked_add/sub/mul/div` → `Trap`); tests `:6652`, `:6673`; predicate eval also checked (`:1915–1919`) | none assigned |
| M1 interpreter = the documented subset; `old(...)`/`list_count(...)` contract-only and trap from bodies | partial | Core enforced: ownership traps H0801–H0809 (`run.rs:6730–7240`); body-trap of `old` (`:2490`, test `:6589`) and `list_count` (`:2497`). Gap A (ledger #15): `hum check` accepts `list_count` in task bodies (0 errors) while `hum run` traps — check/run disagree. Gap B (ledger #16): block-scoping leak — resolver scopes `if`/`for each` blocks but the checker env is flat per task and the runtime leaks `if`-block lets (proven wrong acceptance) | WO28 #15; WO28 #16 |
| Test obligations generated from `needs:`/`ensures:`/`watch for:`/`tests:`; `covers:` linked by exact match or canonical token key | enforced | `src/syntax.rs:57–79`; `src/graph.rs:91`; `canonical_coverage_tokens` (`:245`); test `graph.rs:395` proves synonym misses fail | none assigned |
| Evidence obligations from `protects:`/`trusts:` with boundary blame; `covers:` linking; `verification_status` linked/unverified | enforced | `src/syntax.rs:80–90`; `src/evidence.rs:167–184`; test `:397` (2 obligations, 1 linked, 1 unverified) | none assigned |
| Resource honesty: `cost:`/`allocates:` lines preserved in graph facts and `hum.resource_report.v0`; `allocates: nothing` exports math obligations; optimize only with evidence | enforced (honesty machinery) | `src/resource_report.rs:189` recognizes the sections, carries `graph_node_id` (`:195/211`); `hum.resource_report.v0` (`:7`); honesty counts `src/resource_check.rs:153–184`; allocation-freedom obligation `src/math_obligations.rs:269` (test `:737`). Caveat: "optimize only with evidence" is future-facing — no optimizer exists today | none assigned |
| `targets:` lines carry `declared_not_enforced_v0`; H1201–H1205; no backend selection, profile enforcement, or portability proof in M0 | enforced | `src/check.rs:380–480` (`check_target_declarations`); `declared_not_enforced_v0` (`src/json.rs:233`, test `:979`); test `check.rs:994` | none assigned |
| Stable H#### codes; `hum explain` documents them; DIAGNOSTICS.md is the stable contract | enforced | `src/explain.rs`; `src/diagnostic_catalog.rs` is the canonical allocation authority; tests `:5308` fail on contradictory/missing codes | none assigned |
| H0634 owns native layout failures with no interpreter fallback; H0635 owns layout-valid shapes selecting ≠1 sealed native feature | enforced | H0634 `src/app_entry.rs:780`; `main.rs:1291–1360` (layout analysis first, exit 1, no interpreter path); H0635 `src/native_program.rs:225`; fixtures `fixtures/programs/hello_world/unsupported_*_fail.hum`; test `:243` asserts no fallback | none assigned |

## SPEC (repo-root `SPEC.md`, v0.0.1 design draft, 2026-07-06)

The mission, design principles 1–2 and 7–10, and the open design questions
are design intent, not checkable claims, and are not audited here. The
App/sessions Z–AD material duplicates claims already audited under decisions
0015–0017 and the LANGUAGE_REFERENCE section above; only the SPEC.md-unique
doctrine appears here.

6 claims checked: 2 enforced, 1 partial, 0 declared-only, 3 not-started,
0 unverified.

| Claim | Status | Enforcement evidence | Gap closer |
|---|---|---|---|
| Unsafe code allowed only inside visible unsafe boundaries | not started | There is no `unsafe` keyword or `unsafe task` surface anywhere in the parser/AST/lexer; `unsafe` exists only as a denied capability root (`src/capability_root.rs:33`, sandbox-bypass tier) and as the host compiler's `#![deny(unsafe_code)]` (decision 0002). The doctrine lives in `docs/UNSAFE_POLICY.md` with no language surface to attach it to | none assigned (needs a decision + Work Order) |
| Unsafe tasks must declare `why:`, `needs:`, `proves:`, and `watch for:` | not started | Same absence: nothing in the parser recognizes an unsafe task, so the section requirement cannot be declared, let alone checked. The gate the policy names is `docs/UNSAFE_POLICY.md` prose | none assigned (needs a decision + Work Order) |
| Compile-time code (`build task`) sandboxed by capabilities | not started | No `build task` surface in the parser; the only `build_task` hits are an internal call-graph helper name (`src/capability_root.rs:1136`). Same finding as decision 0006 | none assigned (decision 0006's comptime program, when scoped) |
| No macros as the first escape hatch | enforced | No Hum-language macro surface exists. The `macro_rules!` hits in `src/` are all host-Rust implementation macros (`backend_cranelift.rs`, `ast.rs`, `diagnostic_catalog.rs`); none is reachable from `.hum` source. The one reviewed escape hatch is the JIT's single `allow(unsafe_code)` (decision 0002), not a macro system. Same brittleness caveat as the other absence rows: no negative test pins it | none assigned |
| No effects hidden behind innocent-looking calls; effects inferred from contracts and body, checked against the declared interface | enforced | `src/effect_check.rs` infers reads/changes/failures from contracts and body (`inferred_reads`, `inferred_changes`, `inferred_failures` in the check summary, `:283`) and checks them against declared `uses:`/`changes:`; effect-owned diagnostics fire (e.g. H0907 missing failure declaration, `effect_check.rs:2531–2562`). The capability closure (H0617/H0618/H0619/H0621) rejects undeclared external capability use in any body. Caveat: the checkable surface is the V0 vocabulary — the effect list itself is partial (decision 0008) | none assigned |
| Blame semantics: `needs:`→caller, `ensures:`→task, and every diagnostic carries failed block, source span, blame target, repair hint | partial | Blame routing fires for contracts: `needs:` false → H0702 blames the caller, `ensures:` false → H0703 blames the task (`src/run.rs:1795–1835`; decision 0015). The catalog convention carries an explanation and a machine-readable `repair` hint per code (`src/diagnostic_catalog.rs:3119`), and review protocol demands blame-style help naming the site. But the wider map (`keeps:`/`protects:`/`trusts:`/`changes:`/`allocates:` blame targets) has no firing diagnostic, and there is no structured blame-target field — blame is prose, not a schema field | none assigned |

## Summary counts

155 claims across 30 decisions + the SPEC.

| Status | Count | Where they cluster |
|---|---|---|
| enforced | 92 | 0002 trust roots, 0004 tests-as-evidence, 0010 V0 state rule, 0011–0013, 0016 typed failure, 0017 app authority (current rule), 0019 license, 0021 text_split, 0023 dispatch health, 0025 registry + boundary-skip amendment (run 35967934714: skip on both OSes, Ubuntu 3m44s / Windows 4m39s), 0027 program-driven process; SPEC ownership/effects/contracts/diagnostics/native (34 of 40) + no-macro-escape-hatch + no-hidden-effects |
| partial | 30 | 0001/0005/0006/0007/0008/0009 evidence machinery; 0011 scope agreement (#15/#16); 0013 overflow diagnostics; 0014 linear resources; 0015 contract literals (#4); 0022 contract escapes (#4); 0024 perf-debt ledger; SPEC: BOM, predicate-v2 contract text (#4), M1 subset check/run disagreement (#15) + block scoping (#16), blame semantics |
| declared-only | 15 | 0005 receipts, 0006 comptime-adjacent, 0009 synonyms; 0015 classifier vocabulary + `needs:` policy; 0018 effect model; 0020 `may diverge:`; 0021 PD-001; 0024 north star; 0026 study; 0028 concat trigger; 0029 grant/virtio/labelled-evidence rulings |
| not-started | 16 | 0002 self-hosting; 0006 comptime/interop; 0013→(0014 repairs); 0015 release elision; 0017 audit trail + portable file read (#7); 0020 all; 0028 int_to_text/uint_to_text; 0029 property proofs + grant; SPEC: visible unsafe boundaries, unsafe-task sections, build-task sandboxing |
| unverified | 2 | 0012 test-name grammar pinning; 0013 Float-outside-core |

WO28 is the dominant gap-closer: #4 (contract text literals), #7
(portable file read), #15 (check/run builtin agreement), #16 (block
scoping), plus 0029's implementation queued after #7. Everything else
partial/declared-only/not-started without a named closer is an
unassigned obligation — the next Work Order's raw material.

## Changelog

- 2026-09-24: 0025's boundary-skip amendment → enforced. PR #33 merged;
  measured on run 35967934714 (skip confirmed on both OSes, Language
  preflight Ubuntu 3m44s / Windows 4m39s). Totals now 92 enforced / 30
  partial; "as of" SHA updated to c3ed1ad.
- 2026-09-24: added repo-root `SPEC.md` audit (6 claims; 2 enforced,
  1 partial, 3 not-started) per pre-issuance review. Totals now 155 claims.
- 2026-09-24: initial snapshot (decisions 0001–0029 accepted; SPEC as of
  `docs/LANGUAGE_REFERENCE.md` 1057 lines).

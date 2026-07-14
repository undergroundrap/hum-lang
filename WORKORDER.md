# Hum Work Order 9: Canonical Diagnostic Allocation And Cause Identity

Date: 2026-07-12
Status: issued by the BDFL. The independently accepted and BDFL-accepted bytes
are commit `45796dd688f9f28bb0c3290e8029e33ee2d20802`, published by successful
workflow `29212987679` (Ubuntu job `86703742601`, 1m 28s; Windows job
`86703742589`, 2m 30s). Historical proposal and pre-issuance language below is
preserved as reviewed issuance history. Session AN is accepted and committed
as `bea73fcf3dd82abcf25633d33d0b152667566612`; workflow `29215676504`
passed (Ubuntu job `86710967945`, 1m 32s; Windows job `86710967915`, 2m
42s). Session AO is accepted and committed as
`d750a57ed5168d0d00375972aacc148a5d37e63a`; workflow `29219105868` passed
(Ubuntu job `86720630963`, 1m 58s; Windows job `86720630971`, 3m 16s).
Session AP was authorized and implemented, then independently rejected and
corrected under the reviewed amendments recorded below. The corrective
amendment was independently accepted and BDFL-accepted,
committed as `9aedcb0ba6893d51a2cd1b2e519d332d3cc5e6f4`, and published by successful
workflow `29225093549` (Ubuntu job `86737472812`, 1m 40s; Windows job
`86737472893`, 3m 04s). The bounded typed-failure scope amendment was
independently accepted and BDFL-accepted, committed as
`407c8065e341319b6f260b33418cd9c6b8e80a83`, and published by successful
workflow `29236756896`, attempt 1 (Ubuntu job `86773108930`, 1m 35s; Windows
job `86773108925`, 4m 49s). The bounded writable-alias scope amendment was
independently accepted and BDFL-accepted, committed as
`c56f5f06e908f0ff4e38707d3f8d4ede849b1d3d`, and published by successful
workflow `29280356264`, attempt 1 (Ubuntu job `86919744432`, 1m 49s; Windows
job `86919744403`, 2m 53s). The completed Session AP correction was
independently accepted and committed as
`58ad265bd3d9e974f1d53c2accceb50175edc2d7`; workflow `29300894802` passed
(Ubuntu job `86984248993`, 6m 26s; Windows job `86984249019`, 9m 21s). Session
AP closure was recorded as `aa69cf4ee3813883e3b01ef195ac81a40080898d`;
workflow `29301747997` passed (Ubuntu job `86986758575`, 6m 25s; Windows job
`86986758577`, 11m 21s). The bounded pre-AQ CI evidence-efficiency increment
was independently accepted and BDFL-accepted, committed as
`04ea7435f5e12d0bbc82befe4384227a79d1ccd3`, and published by successful
workflow `29305696822`, attempt 1 (Ubuntu job `86998454703`, 6m 14s, including
5m 47s in `Run Hum preflight`; Windows job `86998454696`, 9m 34s, including
8m 50s in `Run Hum preflight`). That publication run is evidence for the
reviewed document only; it used the existing full workflow and is not the
future operational fast-lane trust anchor. The GitHub Actions workflow-path
correction was independently accepted and BDFL-accepted, committed as
`0f15354e5fa4028bcc37680505326321bf147352`, and published by successful
workflow `29309391970` (Ubuntu job `87009717248`, 5m 42s, including 5m 11s in
`Run Hum preflight`; Windows job `87009717246`, 10m 40s, including 9m 30s in
`Run Hum preflight`). That correction publication run used the existing full
workflow and is document/API-shape evidence only; it is not the future
operational fast-lane trust anchor. The pre-AQ CI evidence-efficiency increment
was independently accepted and committed/published as
`7630befb97055fb099b54e56c186fe3bc202f67d`. Required full CI passed in
workflow `29318848522`, attempt 1. Ubuntu job `87039029980` succeeded in 6m
32s, including 6m 07s in `Run Hum preflight`; Windows job `87039030001`
succeeded in 13m 29s, including 12m 40s in `Run Hum preflight`. Both platforms
selected `mode=full` with `reason=no_status_transition`. This exact commit and
run are the first operational full-CI trust anchor. The first successful
production fast-lane publication was committed and published as
`5da709b7456a1b71fb6ebe8c859224da71e7ce05`. Workflow `29321095009`, attempt
1, succeeded for that exact commit. Ubuntu job `87046343669` succeeded in 1m
07s, including 54s in `Run status-only evidence`; Windows job `87046343749`
succeeded in 3m 29s, including 3m 04s in `Run status-only evidence`. Both
platforms selected `mode=fast` with `reason=eligible_status_chain`, used full-
CI anchor `7630befb97055fb099b54e56c186fe3bc202f67d` and workflow
`29318848522`, attempt 1, and traced the exact transition
`7630befb97055fb099b54e56c186fe3bc202f67d` ->
`5da709b7456a1b71fb6ebe8c859224da71e7ce05`. Both succeeded in
`Run status-only evidence` and skipped Cargo caching, Rust toolchain
preparation, and `Run Hum preflight`. This publication proved the first fast
transition. The required consecutive-status proof was committed and published
as `b9658e26bbc0021a4ca8701bcaa317e0e2b3ee19`. Workflow `29352133751`,
attempt 1, succeeded for that exact commit. Ubuntu job `87150713653` succeeded
in 1m 11s, including 57s in `Run status-only evidence`; Windows job
`87150713741` succeeded in 3m 21s, including 2m 57s in
`Run status-only evidence`. Both platforms selected `mode=fast` with
`reason=eligible_status_chain`, used the same full-CI anchor and workflow, and
traced the ordered transitions
`7630befb97055fb099b54e56c186fe3bc202f67d` ->
`5da709b7456a1b71fb6ebe8c859224da71e7ce05` ->
`b9658e26bbc0021a4ca8701bcaa317e0e2b3ee19`. Both succeeded in
`Run status-only evidence` and skipped Cargo caching, Rust toolchain
preparation, and `Run Hum preflight`. The two production fast-lane
publications durably close the pre-AQ CI evidence-efficiency increment. The
permanent operating-policy amendment for exact routine Work Order status-only
closures was independently accepted, committed, and published as
`65ad0d5f5db2bcd90fdbb8a8afc1208f3d46b659`. Required full CI passed in
workflow `29357247935`, attempt 1. Ubuntu job `87167872491` on
`ubuntu-latest` succeeded in 6m 38s: Cargo caching and Rust toolchain
preparation succeeded, `Run Hum preflight` succeeded in 6m 09s, and
`Run status-only evidence` was skipped. Windows job `87167872510` on
`windows-latest` succeeded in 12m 43s: Cargo caching and Rust toolchain
preparation succeeded, `Run Hum preflight` succeeded in 11m 52s, and
`Run status-only evidence` was skipped. Both platforms reported `mode=full`
with `reason=no_status_transition`. The amendment is durably complete. Future
eligible routine status-only closures use the permanent `AGENTS.md` exception
and its proportional local evidence set. The BDFL has authorized planning only
for the proposed `PRE-AQ-INTEGRITY` permanent adversarial-evidence amendment.
The initial planning draft was independently rejected because its nine-build
implementation plan could not honestly fit the unchanged 15-minute CI limit.
The BDFL authorized only the bounded planning correction below: the later
implementation may change that job timeout from 15 to 30 minutes as its sole
workflow-byte change. The corrected planning bytes are uncommitted and await
fresh independent pre-issuance re-review. Integrity implementation, Session
AQ, GitHub Issue #1 mutation, repair work, and all later work remain
unauthorized.
Owner: BDFL (Ocean).
Work-order author: architect-reviewer acting only under the bounded Work Order
9 planning authorization.
Independent pre-issuance reviewer: a fresh architect-reviewer that did not
author, edit, generate, or directly direct this deliverable.
Future implementer: an implementer agent only after every applicable gate.
Predecessor: Work Order 8, Sessions AL-AM, closed at `047ad02`.

## Authority, evidence, and issuance gate

This document promotes confirmed GitHub Issue #1, "Architecture inbox:
centralize diagnostic namespace and cause identity." The issue is an
architecture-inbox record, not implementation authority. The BDFL has
authorized only this architecture/documentation planning pass.

Repository ground truth at authoring time is:

- clean `main` synchronized with `origin/main` at `047ad02`;
- Work Order 8 issued at `956b51f`;
- Session AL accepted at `b881f2a`;
- Session AM accepted at `7075c71`;
- Work Order 8 closed at `047ad02`; and
- workflow `29211168677` successful for Ubuntu and Windows on that closure.

Work Order 8 remains closed in git history. This document does not reopen its
callable syntax, row semantics, H1401-H1402 allocation, decisions, fixtures, or
acceptance evidence.

This proposal does not:

- issue Work Order 9;
- authorize Session AN or any implementation;
- accept, amend, or create a decision record;
- allocate or renumber a diagnostic code;
- change diagnostic meaning, severity, message, help, span, precedence, or
  exit behavior;
- add a public Hum command, report, schema identifier, JSON field, or pipeline
  gate;
- comment on or close GitHub Issue #1; or
- authorize a commit, push, release, or other publication.

Any authoritative scope, family, cause, precedence, fixture, or gate mutation
after the independent pre-issuance verdict invalidates that verdict and
requires fresh independent review of the complete changed document. Acceptance
of this document does not authorize implementation. BDFL acceptance, durable
publication with green CI, and a separate BDFL go signal for Session AN remain
distinct gates.

## Promotion evidence and defect statement

The architecture triage confirmed both halves of Issue #1.

### Allocation drift

The implemented catalog has 87 active exact codes. The 87 `DiagnosticCode`
values, 87 catalog entries, and 87 current-code rows in
`docs/DIAGNOSTICS.md` are individually unique and agree today. That exact-code
check is necessary but insufficient.

Other repository documents still claim incompatible ownership:

- `docs/DIAGNOSTICS.md` assigns H080 to ownership, H090 to explicit typed
  failure, H100 to future unsafe/FFI/ABI/provenance, H110 to future runtime
  profile/certification, H120 to backend/target/debug metadata, and H130 to
  future concurrency/memory ordering.
- `docs/DIAGNOSTICS_SCHEMA_0_1.md`,
  `docs/EFFECT_REPORT_SCHEMA_0_1.md`, `docs/RUNTIME_PROFILES.md`, and
  `docs/LANGUAGE_SUBSET_0_1.md` still use H100/H1001 for profile denial.
- `docs/SECURITY_MODEL.md` proposes H080 for package/supply-chain and H090 for
  unsafe/FFI, including exact H0803 and H0904 examples that now mean ownership
  and typed failure.
- `docs/UNSAFE_POLICY.md` proposes H0901-H0908 for unsafe diagnostics even
  though H0901-H0907 are active typed-failure codes.
- active callable codes H1401-H1402 are documented and implemented, but the
  range list omits the H140 family.

The standing preflight passes while those contradictions remain. Existing
checks prove exact active-code uniqueness and selected catalog presence; they
do not prove range ownership, reserved-family integrity, retired-code
non-reuse, or checked-document agreement.

### Cause and stage drift

`DiagnosticCode`, `Diagnostic`, and `DiagnosticInfo` do not carry a stable
fundamental-cause identity, semantic owner, owning stage, or exact prior-blocker
reference. Existing analyzers often retain reason strings, but those strings
are not one closed repository-wide identity domain.

Current precedence and propagation therefore rely on local conventions:

- callable analysis suppresses by same-line matching plus a hard-coded code
  list;
- full type and effect checking recognize H0907 ownership through exact-code
  comparisons;
- effect, ownership, resource, and profile reports replace exact earlier
  causes with generic prior-error status and counts; and
- runtime reconstructs some ownership and typed-failure diagnostics rather
  than consuming one canonical emitted occurrence.

A representative H0901 probe produced the specific diagnostic at full type,
while downstream stages retained only anonymous prior-error status/counts.
The current fixtures do not show a user-visible duplicate in that case, but the
system cannot prove that two stage-created diagnostics are the same occurrence
or reject an accidental replacement.

The accepted Session AL-AM callable facts are useful evidence, not a general
solution: their diagnostic IDs and reasons are local to `src/callable.rs`, and
their `prior_owns` filter still depends on code and line rather than a canonical
cause occurrence.

## Accepted-decision and architecture locks

Every Session AN-AQ implementation and review must preserve the following.

### Evidence-native and resolver ordering

- Diagnostic registry facts are internal compiler truth, not another prose
  ledger.
- Parser-owned source nodes and resolver-owned semantic identities precede
  type, effect, ownership, resource, runtime, and graph claims.
- A span is evidence about an occurrence. A span, message, help string, code,
  or display name alone is never semantic cause identity.
- Existing human and JSON diagnostics remain the public projections. Internal
  identity does not imply a new public surface.

### Decision 0014: ownership

- H0801-H0809 keep their accepted ownership meanings and precedence.
- Runtime and ownership checking may share an analyzer, but may not
  independently own the same fundamental occurrence.
- Cause migration grants no new ownership, borrow, alias, lifetime, linearity,
  memory-safety, or resource proof.
- No H080 code is reassigned to supply-chain diagnostics.

### Decision 0015: executable contracts

- Every recognized executable predicate continues to run.
- H0701-H0704 meanings and recognition/runtime precedence remain unchanged.
- No proof/trust classification, elision, global contract toggle, enforcement
  profile, or unreachable-guard conclusion is added.
- Cause identity is not proof classification.

### Decision 0016: nominal causal typed failure

- H0901-H0907 retain exact nominal roots, explicit `try`/wrapping, causal
  sites, meaningful `fails when:`, and current stage ownership.
- H0907 remains fundamentally owned by effect checking; full type may defer to
  that cause but may not emit or replace it.
- No implicit propagation, erased any-error, exception, unwind, recovery,
  catch, ambient backtrace, or first-class Result widening is introduced.
- No H090 code is reassigned to unsafe/FFI diagnostics.

### Decision 0017: authority

- Source authority, operator consent, and operation exercise remain separate.
- H0617-H0633 and H1204 preserve exact authority, path, target, deny-wins, and
  adapter-precedence behavior.
- Cause identity cannot manufacture authority, consent, a route, locality, or
  adapter exercise.
- Zero-adapter blocked-path evidence remains mandatory.

### Decision 0018 and Work Order 8: callable effects

- H1401-H1402 remain the accepted callable diagnostics with their current
  reasons, spans, precedence, runtime preflight, Core blockers, and graph
  relationships.
- H1400-H1499 is recorded as the already-active callable/latent-row family; this
  records the accepted H1401-H1402 allocation and does not allocate another
  code.
- Open rows remain type/effect evidence only. Cause migration grants no
  handling, capture, callable environment, allocation, authority, ownership,
  or runtime semantics.
- Session AM's exact occurrence, alias, row, tail, substitution, application,
  Core, and graph identities remain unchanged.

## Canonical diagnostic model

The final Work Order 9 result has one internal authority in
`src/diagnostic_catalog.rs`. Existing documentation and output become checked
projections. No generated or manually maintained second registry is allowed.

### Exact family intervals

The registry uses inclusive numeric intervals. This removes the ambiguity of
shorthand such as `H060x`, which currently contains H0633. The intervals below
record existing active or reserved doctrine; they do not allocate any new
exact code.

| Inclusive interval | Status | Stable family owner | Existing semantic domain |
| --- | --- | --- | --- |
| H0000-H0099 | active | `source_shape` | parser and source shape |
| H0100-H0199 | active | `intent_shape` | item shape and intent discipline |
| H0200-H0299 | active | `declared_state_effects` | effects, mutation, and declared state changes |
| H0300-H0399 | active | `cost_contracts` | cost and performance contracts |
| H0400-H0499 | active | `security_trust` | security and trust boundaries |
| H0500-H0599 | active | `test_evidence` | tests and regression obligations |
| H0600-H0699 | active | `front_end_semantics` | checked names, types, structural app/authority, and Path boundaries |
| H0700-H0799 | active | `executable_contracts` | executable contract diagnostics |
| H0800-H0899 | active | `ownership_borrowing` | ownership and borrowing |
| H0900-H0999 | active | `nominal_typed_failure` | explicit nominal typed failure |
| H1000-H1099 | reserved | `unsafe_ffi_provenance` | unsafe, FFI, ABI, and provenance |
| H1100-H1199 | reserved | `runtime_profile_policy` | runtime profile and certification policy |
| H1200-H1299 | active | `target_backend_metadata` | backend, target, portability, and debug metadata |
| H1300-H1399 | reserved | `concurrency_memory_ordering` | concurrency and memory ordering |
| H1400-H1499 | active | `callable_effect_rows` | callable and latent-row diagnostics |

Intervals absent from this table are unallocated, not implicitly free. A later
work order must update the registry under independent review before any such
family is used.

At baseline:

- exactly 87 exact codes are active;
- no exact code is retired;
- no exact code is reserved inside a reserved family;
- H1000-H1099, H1100-H1199, and H1300-H1399 reserve families only; and
- no placeholder such as H1001 or H1101 is an allocation.

### Family and code records

The canonical registry must eventually own closed records equivalent to:

```text
DiagnosticFamilySpec
  exact inclusive start and end
  stable family key
  semantic domain owner
  status: active | reserved | retired
  governing doctrine/decision references

DiagnosticCodeSpec
  opaque internal key
  exact Hdddd spelling
  family key
  title
  default severity
  semantic owner
  owning stage or shared analyzer
  status: active | reserved | retired
  explanation
  repair
  governing doctrine/decision references
```

Exact code/title/severity/explanation/repair literals live only in the
registry. `src/diagnostic.rs` may retain carrier types, spans, rendering, and
registry-backed accessors, but it may not remain an independent code/title
allocation table. Emitters use opaque registered keys rather than constructing
code strings.

Retirement is append-only. A retired code remains in the registry with its
last semantic owner and may never be reused. The initial retired set is
explicitly empty. Reserved family space is not evidence that an exact code is
reserved.

### Fundamental cause and occurrence identity

A diagnostic code groups user-facing repairs; it is not necessarily one
fundamental cause. H0704 and H1401 already have multiple closed reasons.

The final cause model must distinguish:

```text
DiagnosticCauseSpec
  opaque cause key
  stable reason projection
  diagnostic code key
  semantic owner
  one owning stage or shared analyzer
  permitted precedence relationships
  required semantic origin/route kinds

DiagnosticOccurrenceId
  cause key
  parser- or resolver-owned source/semantic node identity
  exact relationship/route identity when the cause is relational
```

The ID is never derived only from code, message, help, title, display name,
line, column, or rendered text. Parser-owned diagnostics use parser source-node
identity. Later diagnostics use resolver/analyzer relationship identities.
Spans remain structured blame evidence and may participate in the underlying
source-node identity, but a bare span is not sufficient.

Two distinct occurrences may share a code and cause key. They must retain
distinct occurrence IDs. The same occurrence emitted twice is an internal
invariant violation, not a condition to deduplicate silently. A corrupted or
duplicate cause must fail closed through the existing internal invariant path;
it receives no new H-code and must not become an ordinary user error.

Static registry corruption is rejected by unit tests and the existing
preflight before publication; AN adds no runtime registry command. Once the
collector exists, a dynamically detected duplicate/owner/substitution
invariant exits the affected CLI invocation with status 2, writes no stdout,
executes no body or adapter, and reports an internal diagnostic-invariant error
on stderr without an H-code. This path is a compiler-bug channel, not a source
diagnostic or typed operational failure. Its exact text is private and receives
no compatibility promise, but tests must distinguish it from a generic source
runtime trap.

### Owning stage, prior blockers, and precedence

Each cause has exactly one fundamental owner. Other stages may:

- consume it;
- preserve an exact internal `PriorBlockerRef` to its occurrence ID;
- project their existing blocked status/count; or
- omit a stage-local check because the exact prior cause owns the case.

They may not recreate the diagnostic, substitute a generic cause, change its
code, or claim ownership.

Precedence is relational, not one global numeric ranking. A closed rule names:

- the dominant cause key;
- the suppressed cause key;
- the semantic relationship/site on which they compete; and
- the stage that applies the rule.

Same-line matching, code-prefix membership, message matching, and hard-coded
unregistered code lists are not precedence. Two independent causes on the same
line remain two causes unless a registered semantic precedence rule says
otherwise.

The existing externally visible precedence is frozen. If migration reveals
that current outputs disagree about which cause fundamentally owns a case, the
session stops for an explicit architecture/decision gate. Work Order 9 does not
choose new user-visible precedence.

Identity-set normalization is insertion-independent. Public ordering is a
separate compatibility fact. The collector must use a closed order key
equivalent to:

```text
existing public surface
existing stage-projection ordinal for that surface
normalized source-path identity
primary line and column evidence
cause key
semantic occurrence/route identity
```

Before replacing any collector, the implementer pins the current complete
surface ordering and proves the closed ordinals reproduce it. If no such key
can reproduce current output without changing semantics, the session stops;
it may not sort diagnostics into a newly preferred order. Related spans retain
their existing analyzer-owned order.

### Public compatibility boundary

Work Order 9 adds internal truth only.

- `hum diagnostics`, `hum explain`, `hum check`, stage human/JSON reports,
  graph output, Core output, and runtime output keep their existing schema
  identifiers and fields.
- Exact active codes, titles, default severities, explanations, repairs,
  messages, help, primary/related spans, diagnostic counts, exit statuses, and
  zero-adapter behavior remain unchanged.
- No cause ID, owner, stage, family status, or prior-blocker ID is added to a
  public JSON surface in this work order.
- Internal tests may inspect those facts directly.
- Corrected documentation must not claim that an unimplemented profile,
  unsafe, supply-chain, or concurrency diagnostic has an exact allocated code.

## Mandatory sequence and stopping point

Work Order 9 contains exactly four compiler/runtime implementation sessions.
The separately gated pre-AQ CI increment below changes evidence execution only
and does not consume or rename a compiler session:

```text
AN  canonical family/code/status registry and checked documentation projections
AO  cause/occurrence identity for typed failure and callable vertical slices
AP  remaining static compiler emitters and exact prior-blocker propagation
PRE-AQ-CI  exact status-only CI fast lane; separately reviewed and authorized
PRE-AQ-INTEGRITY  permanent adversarial-evidence amendment; separately gated
AQ  runtime/top-level composition closure and repository-wide audit
STOP  close Issue #1 only after separate authorization; author the next work order separately
```

AN is mandatory first. No later Work Order 9 session and no unrelated language,
stdlib, IO, ownership, effect, or adoption work may begin before AN is
independently accepted, committed, published, green on Ubuntu and Windows, and
recorded in this file. Each following session requires the same cycle plus a
separate BDFL go signal. The two pre-AQ entries do not authorize one another:
the CI increment must complete its own review/publication/evidence cycle first,
the proposed integrity amendment must complete the separate gates defined in
its section below, and AQ still requires its own later BDFL go signal.

The compiler-ready stdlib remains the next adoption direction supported by the
2026-07-12 research triage. It is intentionally not part of Work Order 9.
Diagnostic centralization goes first because new compiler/stdlib work would
otherwise allocate and propagate causes through a contradictory namespace.

Session sizing is semantic, not just a path count. AN changes allocation truth
only. AO proves two vertical slices. AP has the widest file envelope, but every
permitted change is the same carrier migration: produce, consume, validate, or
preserve one exact prior-cause set without changing stage logic or public
output. Registering an already-observed precedence relationship is part of that
migration; inventing or changing which source cause wins is a new diagnostic
rule. If any AP stage needs a new user-visible diagnostic or precedence rule,
schema field, or source behavior, AP is no longer review-sized and must stop
for a separately reviewed Work Order amendment. AQ changes only final
composition/runtime ownership after the static migration is accepted.

## Session AN: canonical allocation registry and checked projections

Purpose: make allocation mechanically single-source before changing diagnostic
transport.

### AN exact integration map

Production/internal Rust scope:

- `src/diagnostic_catalog.rs`: sole family/code/status allocation registry,
  exhaustive validator, and registry tests;
- `src/diagnostic.rs`: carrier types, spans, rendering, and registry-backed
  code access only;
- `src/diagnostics.rs`: unchanged public catalog/check projections plus
  semantic-equivalence tests; and
- `tools/check_all.ps1`: one proportional registry/projection invocation,
  replacing selected-code string checks where the registry makes them
  redundant.

Checked documentation projections:

- `docs/DIAGNOSTICS.md`;
- `docs/DIAGNOSTICS_SCHEMA_0_1.md`;
- `docs/EFFECT_REPORT_SCHEMA_0_1.md`;
- `docs/SECURITY_MODEL.md`;
- `docs/UNSAFE_POLICY.md`;
- `docs/RUNTIME_PROFILES.md`;
- `docs/LANGUAGE_SUBSET_0_1.md`; and
- `docs/PORTABILITY_BOUNDARY_MODEL.md`.

No other production, fixture, schema, governance, decision, research,
architecture, Work Order, Cargo manifest, editor, example, or runtime file is
authorized. Malformed-registry evidence belongs in focused Rust unit tests, not
in a new public schema or second data file. If this exact envelope cannot make
the catalog authoritative, preserve the worktree and stop for architecture
review.

### AN required behavior and permanent evidence

The registry validator must reject independently:

1. overlapping inclusive family intervals;
2. the same interval with two semantic owners;
3. duplicate exact code spellings;
4. duplicate opaque code keys;
5. an active exact code outside its family;
6. an exact active/retired code inside a differently owned family;
7. reuse or semantic mutation of a retired code in an in-test frozen prior
   record comparison;
8. a reserved family treated as an exact-code allocation;
9. malformed code spelling, inverted interval, or interval outside
   H0000-H9999; status is a closed Rust domain with no string/unknown
   construction path;
10. a code/title/severity/explanation/repair mismatch between registry and
    public catalog projection;
11. a checked document claiming an unknown or contradictory exact code;
12. a checked document assigning a family to a different semantic domain; and
13. omission of H1400-H1499 or any of the 87 active codes from the checked
    projection.

The positive baseline proves exactly 87 active codes, zero retired exact codes,
the three reserved family intervals, nonoverlapping ownership, deterministic
ordering by numeric code, and repeat-identical validation.

Documentation corrections must:

- preserve active H080, H090, H120, and H140 meanings;
- remove the unallocated H0403-H0405 examples and the stale H0803/H0904
  security examples as exact allocations, preserving their ideas only as
  family-level future prose;
- remove H0901-H0908 unsafe allocation claims;
- replace stale exact H1001 profile examples with one explicit
  `<unallocated-profile-diagnostic>` placeholder and prose saying that no exact
  profile code is allocated;
- name H1100-H1199 as the reserved profile family without reserving H1101 or
  another exact code;
- state that illustrative portability examples do not allocate exact codes;
  and
- link every checked projection back to `src/diagnostic_catalog.rs` as the
  internal authority and `docs/DIAGNOSTICS.md` as the human projection.

The placeholder is not a valid H-code and the projection validator must treat
only that exact closed token as an allowed unallocated example. No other
wildcard or escape syntax is allowed.

### AN human/JSON/runtime compatibility

- `hum diagnostics` and `hum explain` human/JSON semantics are unchanged for
  all 87 active codes.
- `hum check` and every stage/runtime diagnostic render exactly as before.
- No runtime path changes and no adapter is exercised by registry validation.
- The implementer records before/after hashes of the current human and JSON
  catalog outputs in the session report; those hashes are evidence, not a new
  committed snapshot ledger.

### AN bans

No cause/occurrence carrier, emitter migration, precedence change, diagnostic
renumbering, new exact reservation, public field, schema identifier, command,
report, pipeline stage, build script, dependency, code generation step, runtime
change, Issue mutation, decision record, or Session AO hook beyond the
registry-backed opaque code key needed by AN.

Do not mechanically generate repository docs at build or runtime. Checked
projections are validated inputs; the compiler does not write the worktree.

### AN acceptance criteria and hard stop

- One registry owns every exact code allocation fact; checked documents may
  project registered literals and later sessions remove production classifiers.
- All 87 active code meanings and public outputs are preserved.
- All listed stale documents are corrected and mechanically checked.
- Every required malformed registry/projection independently fails.
- Removing the collision test or reintroducing H1001/H090 unsafe drift makes
  the standing check fail.
- Root tests, preflight, and platform checks pass.
- Stop. Session AO remains unauthorized pending independent review, scoped
  commit, BDFL-authorized publication, successful Ubuntu/Windows CI, recorded
  handoff, and a separate BDFL go signal.

## Session AO: cause identity for typed failure and callable diagnostics

Purpose: prove the internal cause model on the two accepted cross-stage systems
that exposed the defect most clearly, without migrating every emitter at once.

### AO exact integration map

Authorized files are limited to:

- `src/diagnostic_catalog.rs`;
- `src/diagnostic.rs`;
- `src/typed_failure.rs`;
- `src/callable.rs`;
- `src/full_type_check.rs`;
- `src/effect_check.rs`;
- `src/main.rs` only at the existing diagnostic collection insertion point;
- `src/run.rs` only for typed-failure/callable preflight consumption;
- focused new fixtures under `fixtures/diagnostics/session_ao_*`; and
- proportional assertions in `tools/check_all.ps1`.

No Core, graph, parser, resolver, ownership, authority, documentation, schema,
decision, Cargo, editor, or unrelated fixture file is authorized. Existing
Session W and AL-AM fixtures remain the primary corpus and may not be rewritten.

### AO cause ownership

AO must register and consume every currently reachable H0901-H0907 and
H1401-H1402 fundamental reason. It must preserve these owners:

- H0901-H0906 nominal/form compatibility: shared typed-failure analysis with
  full type as the fundamental diagnostic stage;
- H0907 meaningful failure declaration: shared typed-failure analysis with
  effect check as the fundamental diagnostic stage; and
- H1401-H1402 callable shape/signature reasons: shared callable analysis as a
  shared preflight owner, projected consistently into the existing stages and
  runtime preflight.

Full type may carry an exact H0907 prior-blocker reference but may not emit
H0907. Effect/runtime may consume H0901-H0906 or H1401-H1402 but may not
reconstruct their occurrences.

The existing callable `detail_reason` values become registered cause keys or
closed projections of them. The broad H1401/H1402 reason remains user-facing;
each detail cause keeps its existing stable reason and repair.

### AO permanent evidence

Add focused fixtures/tests proving:

1. one H0901 source occurrence has one owner and the same occurrence reference
   through full type, downstream blocking, and runtime preflight;
2. H0907 is owned once by effect checking while full type defers exactly;
3. one H1401 and one H1402 occurrence retain callable application/definition
   relationship identity through every existing consumer;
4. injecting the same typed-failure occurrence from two stages fails closed;
5. replacing an exact prior blocker with a generic or different-code cause
   fails closed;
6. two distinct occurrences with the same code and cause are both preserved;
7. code-only, span-only, line-only, message-only, help-only, or display-name-
   only identities collide in the adversarial fixture and are rejected;
8. two independent causes on one source line remain distinct unless a
   registered semantic precedence rule suppresses one;
9. H0605/H0630/H090 precedence over callable diagnostics remains exactly as
   accepted, using semantic sites rather than the old hard-coded same-line
   list; and
10. mutating cause key, owner, owning stage, semantic origin, relationship
    route, or occurrence ID independently fails.

At least these new permanent fixtures are required:

- `fixtures/diagnostics/session_ao_typed_failure_prior_blocker_fail.hum`;
- `fixtures/diagnostics/session_ao_callable_prior_blocker_fail.hum`;
- `fixtures/diagnostics/session_ao_adjacent_distinct_causes_fail.hum`; and
- `fixtures/diagnostics/session_ao_same_code_distinct_occurrences_fail.hum`.

If an existing fixture can carry one case without mutation, the new fixture may
be omitted only when the permanent Rust/preflight assertion names and proves
the exact relationship above.

### AO human/JSON/runtime compatibility

- Existing Session W and AL-AM human/JSON codes, messages, help, spans, counts,
  blocker statuses, Core/graph projections, runtime exits, and outputs are
  unchanged.
- H090/H140 blocked runtime paths execute no body or adapter and never fall to
  a generic trap.
- No internal cause/occurrence field is serialized publicly.
- Two fresh runs produce the same existing public bytes and the same internal
  occurrence IDs.

### AO bans

No new H-code, family, public field, schema, stage, command, row/callable
semantic, typed-failure semantic, handler, recovery, authority, ownership,
runtime operation, allocation, generated docs, or migration of unrelated
diagnostic families.

No silent deduplication. No global numeric precedence rank. No source-text
reparse or line-string reconstruction may create cause identity.

### AO acceptance criteria and hard stop

- Every H090/H140 cause has one registered owner and stable occurrence identity.
- Typed-failure and callable runtime paths consume, rather than recreate, the
  canonical occurrence.
- H0907 and callable precedence remain exact.
- All duplicate/replacement/identity mutations fail independently.
- Every public surface is compatibility-clean.
- All session and standing checks pass.
- Stop. Session AP remains unauthorized pending the complete review/commit/
  publication/CI/status cycle and a separate BDFL go signal.

## Session AP: remaining static emitters and prior-blocker propagation

Purpose: migrate the rest of the static compiler to registered causes and exact
prior-blocker references while preserving all current diagnostic behavior.

### AP corrective-review status and architectural gap

Session AP was authorized and implemented on committed base
`22b5e1e23bb1d9c3e137bb4b5e4ed6e9eba521a7`. The complete implementation is
preserved uncommitted after an independent `REJECT` verdict. The rejected bytes
are evidence, not accepted implementation.

The review established three architectural gaps:

1. migrated static occurrence identity can be reconstructed from a public
   diagnostic code and spans instead of being established by the parser,
   resolver, or analyzer that owns the semantic fact;
2. the five proposed AP precedence records name relationships and applying
   owners, but lookup accepts only the cause pair and the production stages do
   not consume the complete rule; and
3. Core and graph checks can validate references regenerated from the same
   downstream set instead of comparing that projection with one authoritative
   upstream occurrence set. The real `hum graph` command does not yet perform
   that comparison.

This amendment narrows the correction. It changes no diagnostic allocation,
cause meaning, precedence outcome, public projection, source behavior, or AQ
runtime/top-level ownership mandate.

### AP exact integration map

Authorized source files are:

- `src/parser.rs`;
- `src/check.rs`;
- `src/resolve.rs`;
- `src/type_env.rs`;
- `src/type_check.rs`;
- `src/full_type_check.rs` and `src/effect_check.rs` only to generalize AO's
  exact blocker carrier to registered AP causes;
- `src/typed_failure.rs` only to establish, bind, and transport producer-owned
  opaque typed-failure cause and occurrence identities required by AP's
  independently checked precedence and downstream projections. It may not
  reconstruct or select causes from public diagnostic codes, reason strings,
  rendered text, display names, filenames, spans, catalog lookups, or
  default-cause fallbacks. Typed-failure behavior, diagnostic semantics, public
  human/JSON/runtime bytes, schemas, ordering, exits, and accepted AO behavior
  remain unchanged;
- `src/app_entry.rs`;
- `src/capability_root.rs`;
- `src/path_boundary.rs`;
- `src/predicate.rs`;
- `src/ownership_check.rs`;
- `src/writable_field_alias.rs` only to replace string-carried alias diagnostic
  identity with one closed producer-owned typed alias-cause representation.
  `AliasIssue` or its equivalent producer artifact must carry that typed cause
  from the exact structural alias-analysis branch that discovers the misuse.
  The existing public reason string is derived from the typed cause only after
  semantic cause selection, and `src/ownership_check.rs` consumes the typed
  producer-owned cause directly. Neither ownership check nor another consumer
  may reconstruct or select alias cause identity from `AliasIssue.reason`, a
  diagnostic code, rendered message or help, display name, source text,
  filename, span, catalog search, or default-emitter fallback. This authority
  permits no writable-alias semantic change, new alias form, ownership
  widening, new diagnostic, or refactor beyond the typed identity carrier;
  existing alias reasons, messages, help, spans, precedence, human/JSON/runtime
  bytes, schemas, ordering, and exits remain unchanged;
- `src/resource_check.rs`;
- `src/profile_check.rs`;
- `src/core_preview.rs`, `src/core_lower.rs`, and `src/core_verify.rs` only to
  consume, validate, or preserve the exact prior cause set without adding a
  public field;
- `src/graph.rs` only to consume the same cause set and preserve existing graph
  output;
- `src/ir_readiness.rs` only to validate the exact prior cause set behind its
  existing readiness status;
- `src/main.rs` only to preserve producer-owned source occurrences through the
  existing private `load_program` boundary and, in the existing `"graph"`
  command branch, require canonical graph-projection validation before the
  unchanged serializer runs. It may not change command selection, public
  rendering, diagnostic filtering/composition, exits, app/runtime preflight,
  adapters, or the AO collector insertion point;
- `src/diagnostic_catalog.rs` only for registered cause/precedence entries;
- `src/diagnostic.rs` only for shared carrier validation;
- focused fixtures under `fixtures/diagnostics/session_ap_*`; and
- proportional `tools/check_all.ps1` assertions.

This amendment, including the bounded `src/writable_field_alias.rs` scope
addition, does not accept the current AP implementation and does not authorize
implementation to resume. Session AQ and all later work remain unauthorized.

`src/run.rs`, `src/json.rs`, and `src/diagnostics.rs` remain outside AP. The
graph serializer already accepts the existing public diagnostics, so AP must
validate the canonical occurrence/projection relationship before calling it;
changing its signature or output is unnecessary. `src/ast.rs`, `src/node_id.rs`,
and `src/callable.rs` are also unnecessary: current parser nodes, resolver IDs,
analyzer facts, and AO callable carriers are sufficient. AQ retains final
collector composition, diagnostic filtering, runtime consumption, public
compatibility closure, and removal of superseded top-level/runtime classifiers.
No Core/graph schema, documentation other than this amendment, decision, Cargo,
editor, example, or unrelated fixture change is authorized. If the correction
cannot satisfy the gates below without another file, preserve the worktree and
request a fresh reviewed amendment rather than expanding locally.

### AP static ownership rules

- Parser/source-shape causes use parser-owned source-node identity.
- Resolver/type causes use resolver definitions, references, scopes, type
  relationships, and exact source nodes.
- App/authority/path causes use their existing app, task, call, declaration,
  policy, and Path relationship IDs. Cause identity does not invent authority
  or routes.
- Predicate causes use the accepted memoized Predicate v2 recognition/place
  facts. Prose warnings and malformed/typed predicate causes remain distinct.
- Ownership causes use existing place, move, alias, view, resource, path, and
  last-use identities.
- Resource/profile stages preserve exact upstream occurrence references rather
  than anonymous replacement causes.
- Type environment, Core preview/lower/verify, graph, and IR readiness do not
  become diagnostic owners merely because they validate or project a blocked
  state. Each consumes or validates the exact originating occurrence set and
  keeps its existing public status/count/output.

The registry must contain every fundamental reason reachable from these
emitters. A cause key may map to an existing reason projection, but no emitter
may provide an unregistered arbitrary reason string as identity.

### AP canonical production and transport rules

- Every migrated producer constructs an opaque occurrence at the boundary
  where its semantic fact is known. Parser causes use parser-owned source-node
  identity; resolver/type causes use resolver definitions, references, scopes,
  and relationships; app/authority/path/predicate/ownership causes use their
  existing analyzer-owned fact identities.
- The occurrence is bound to its exact cause key, semantic owner, owning stage,
  semantic origin, route, source node, required relationship sites, and sealed
  public diagnostic projection before it leaves the producer.
- Public code, title, severity, message, help, display name, line, column, or
  spans may be projection evidence, but no combination of those public fields
  may reconstruct or select cause/occurrence identity.
- Production occurrence construction has no `code -> default cause` fallback.
  An emitter must select one exact registered cause from its semantic reason;
  an absent, unknown, wrong-family, or ambiguous reason is an invariant failure.
- The parser/check/app source boundary must transport those producer-owned
  occurrences with its diagnostics. `src/main.rs` may carry that opaque set
  through `load_program`, but may not regenerate it, apply precedence, silently
  deduplicate it, or become its semantic owner.
- Every downstream set is derived by preserving exact upstream occurrences and
  adding only occurrences constructed by the current owning analyzer. No
  migrated path may call a helper equivalent to `from_diagnostics` or
  `validate_owned_diagnostics` when that helper selects cause or identity from
  public diagnostic fields.
- Two occurrences with byte-identical public diagnostics but different
  semantic origins or routes are different facts. Substituting either origin
  or route into the other must fail against the producer-owned authority even
  if code, spans, message, help, and rendered bytes remain unchanged.

### AP exact precedence application

Every AP precedence application is a closed internal relationship containing:

```text
registered precedence rule identity
exact dominant occurrence and cause
exact suppressed occurrence and cause
registered applying owner/stage
registered semantic relationship identity
canonical competing source/semantic sites and route
```

Lookup by cause pair alone is insufficient. Applying a rule must validate every
field above against the registry and the two producer-owned occurrences. The
stage named by the rule must call that validation on the exact competing
relationship; merely retaining current output through stage order or an
unregistered local filter does not consume the rule.

The five AP relationships record existing behavior only:

- parser over resolver on the same blocked parser-owned semantic node;
- resolver over type on the same unresolved resolver relationship;
- authority over ownership on the same authority/ownership call route;
- Path over Predicate v2 on the same opaque-Path inspection relationship; and
- H0907 effect ownership over H080 ownership on the same blocked task route.

Independent causes remain independent when their nodes or routes differ, even
when their codes and spans match. A cause with the same code but another
registered reason, the correct code pair on another node, the correct cause
pair with another applying owner/relationship/route, or a fabricated public
diagnostic cannot suppress.

### AP authoritative Core and graph transport

- Each producing/static stage yields one authoritative occurrence set plus the
  exact prior references it consumed. A downstream stage validates its incoming
  references against that separately supplied upstream set before preserving or
  extending it.
- A set may generate references for transport, but it may not prove its own
  projection by regenerating expected references from itself. Validation always
  names distinct authoritative-upstream and projected-downstream inputs.
- Core preview consumes the authoritative type/static set. Core lower preserves
  that exact set. Core verify compares the lower projection with the separately
  supplied authoritative preview/static set; it does not merely return the
  lower copy.
- Profile is the final authoritative static set for AP. IR readiness validates
  its separately carried projection against that profile set.
- `hum graph` must actually validate the graph diagnostic projection against
  the authoritative profile/static set before emitting the unchanged graph
  JSON. The narrowly authorized `src/main.rs` graph branch may invoke that
  validation; `src/json.rs` remains unchanged and receives only the same public
  arguments and produces the same bytes.
- Core/graph projection validation checks exact occurrence IDs, cause keys,
  owners, stages, semantic origins, routes, required sites, sealed diagnostics,
  order, and membership. It exposes none of those internal fields publicly.

### AP permanent evidence

The static migration must independently prove:

1. every active code emitted by AP has at least one registered cause and one
   exact owner;
2. every registered AP cause has a real positive or misuse path, or is marked
   unreachable and rejected as dead registry data;
3. parser versus resolver, resolver versus type, authority versus ownership,
   Path versus predicate, effect versus ownership, and ownership versus
   resource/profile combined causes keep existing precedence;
4. two same-line independent diagnostics remain independent;
5. the same prior blocker carried through full type/effect/ownership/resource/
   profile retains one occurrence identity and one owner;
6. missing, duplicate, substituted, reordered, or extra blocker references
   fail closed;
7. mutating code key, cause key, owner, stage, semantic node, relationship
   route, primary span evidence, or required related-site set fails
   independently; and
8. every stage's existing blocker status and issue count remain unchanged.

The corrective review additionally requires independent permanent mutations
proving:

9. a public-byte-identical occurrence with a substituted semantic origin or
   route fails against the producer-owned occurrence;
10. another registered cause under the same code, another semantic node at the
    same span, and the correct cause pair with the wrong applying owner,
    relationship, route, or canonical sites cannot suppress;
11. every one of the five AP precedence records is exercised by its named
    production stage, while a same-code/same-line independent occurrence
    remains visible;
12. every stage and Core/graph projection independently rejects missing,
    duplicate, reordered, extra, substituted, and cross-occurrence prior
    references relative to a separately supplied authoritative upstream set;
13. Core verification rejects a lower set corrupted without changing its
    public blockers, and the real graph command rejects an internally corrupted
    occurrence/projection relationship before serialization; and
14. removing the legacy public-diagnostic reconstruction path or restoring it
    changes a sabotage result, proving it is non-authoritative;
15. changing an alias issue's rendered reason after typed cause selection does
    not change its producer-owned alias cause identity; and
16. a missing, substituted, unknown, or contradictory typed alias cause fails
    closed before ownership check or another consumer can use it.

Required new evidence names are:

- `fixtures/diagnostics/session_ap_parser_resolver_precedence_fail.hum`;
- `fixtures/diagnostics/session_ap_path_predicate_precedence_fail.hum`;
- `fixtures/diagnostics/session_ap_authority_ownership_precedence_fail.hum`;
- `fixtures/diagnostics/session_ap_same_line_independent_causes_fail.hum`; and
- `fixtures/diagnostics/session_ap_prior_blocker_chain_fail.hum`.

An existing permanent fixture may substitute for one named file only when
preflight runs it through the complete relevant human/JSON stage matrix and
asserts the exact occurrence/owner relationship through internal tests.

### AP human/JSON/runtime compatibility

AP does not change runtime. Static human/JSON codes, messages, help, primary
and related spans, counts, status values, reason projections, Core blockers,
and graph facts remain exactly compatible. Existing runtime behavior must stay
green through standing tests even though runtime migration waits for AQ.

### AP bans

No runtime/top-level collector change, public cause field, new code/family,
diagnostic semantic change, profile enforcement, contract classification,
authority grant, ownership widening, new resource proof, Core/graph field,
schema, command, pipeline gate, dependency, or unrelated refactor.

No code-prefix, same-line, message, or display-name classifier may replace a
registered cause/precedence rule in migrated static paths.

No code-only default-emitter lookup, span-derived occurrence reconstruction,
self-derived expected projection, cause-pair-only precedence application, or
public-byte equality may satisfy the corrective gates. The narrow
`src/main.rs` allowance is transport/graph validation only and may not perform
AQ's final cross-command collection, filtering, runtime consumption, or
classifier retirement.

### AP acceptance criteria and hard stop

- Every static emitter in scope uses registered cause identity.
- Every migrated occurrence is created from its producer-owned semantic fact;
  no production static path reconstructs identity from public diagnostics.
- Every downstream static blocker keeps the exact origin occurrence internally.
- Every registered AP precedence relationship is consumed and validated by its
  named production owner using exact occurrences, relationship, route, and
  sites.
- Core, IR readiness, and the real graph command validate downstream
  projections against a separately supplied authoritative upstream set.
- Existing precedence and every public byte/output behavior are unchanged.
- All field, combined-cause, semantic-substitution, and projection mutations
  fail closed.
- The previously reported totals of 177 registered causes, 74 default static
  emitter causes, 27 H0704 Predicate v2 reasons, and 37 reachable H080 ownership
  reasons are implementation claims to reproduce. They are not acceptance
  evidence and do not excuse dead, duplicate, unreachable, or fallback causes.
- Existing H010/H060/H070/H080/H120 and authority/path matrices remain green.
- All session and standing checks pass.
- Stop if satisfying this amendment requires `src/json.rs`, `src/run.rs`,
  `src/diagnostics.rs`, a new public field, changed output, runtime/top-level
  composition behavior, or another unauthorized file. Such pressure belongs
  to a fresh reviewed amendment or Session AQ; it is not implicit AP scope.
- Stop. Session AQ remains unauthorized pending the complete review/commit/
  publication/CI/status cycle and a separate BDFL go signal.

## Pre-AQ CI evidence-efficiency increment: exact status-only fast lane

Planning state: proposed under the BDFL's bounded planning authorization. The
first independent pre-issuance review returned `REJECT` because the proposed
status-only range proof had no independently verifiable successful-full-CI
anchor. This bounded correction does not authorize implementation. It requires
fresh independent pre-issuance review, BDFL acceptance of the exact reviewed
bytes, a scoped documentation commit, durable publication with green Ubuntu
and Windows CI, and a separate BDFL implementation go signal.

Purpose: retain the existing required CI workflow and platform conclusions
while avoiding the complete Rust/fixture preflight only when repository history
proves that every transition after one successfully completed full-CI trust
anchor changes nothing except the two bounded Work Order status regions.

### Evidence baseline and authority boundary

The accepted Session AP status-only closure commit is
`aa69cf4ee3813883e3b01ef195ac81a40080898d`. Workflow `29301747997`, attempt 1,
passed for that exact commit:

- Ubuntu job `86986758575` succeeded in 6m 25s; its `Run Hum preflight` step
  consumed 5m 53s; and
- Windows job `86986758577` succeeded in 11m 21s; its `Run Hum preflight` step
  consumed 10m 40s.

Checkout, toolchain preparation, and caching together consumed less than 40
seconds on each host. This increment therefore targets evidence selection, not
Cargo cache tuning. These observations justify the experiment but do not
authorize broad documentation skipping, a weaker required check, or a claim
that full preflight is redundant.

The accepted `aa69cf4` workflow is timing and planning evidence only. The
implementation may not hard-code that commit, workflow, or job set as its
operational trust anchor. The workflow/tool implementation commit must run the
new workflow's full lane successfully on both platforms and thereby establish
the first anchor that the production fast lane may consume.

The fast lane is allowed only for exact `WORKORDER.md` status maintenance. A
documentation label, commit-message prefix, final-tree equality, or path-only
filter is not evidence. The complete evidence unit is one exact successful
full-CI anchor plus every first-parent transition from that anchor through the
proposed head. Any uncertainty in either half selects full CI.

### Exact implementation map

The later implementation is limited to exactly:

- `.github/workflows/ci.yml`: preserve the `ci` workflow and the existing
  `preflight (windows-latest)` and `preflight (ubuntu-latest)` matrix job names,
  add only `actions: read` beside existing `contents: read`, add complete-
  history checkout, pass the exact repository/workflow/event/base/head facts
  and the process-local `GITHUB_TOKEN` to the classifier before Cargo setup,
  and condition the existing full or new status-only evidence steps;
- new `tools/check_workorder_status_boundary.ps1`: dependency-free,
  cross-platform PowerShell classifier/validator over explicit repository,
  workflow path, event kind, base commit, head commit, and read-only Actions
  evidence. Its production entry point fetches anchor evidence itself; it has
  no parameter that can assert or inject a trusted anchor;
- new `tools/test_workorder_status_boundary.ps1`: isolated temporary-repository
  adversarial tests for the complete closed classifier contract; and
- `tools/check_all.ps1`: one proportional invocation of the classifier test
  script so full local and CI preflight permanently guard the fast-lane logic.

No other workflow, tool, Cargo file, Rust source, fixture, example, generated
output, documentation file, decision, governance file, or GitHub setting is
authorized. Existing text, public-readiness, and release-readiness scripts are
invoked unchanged. If this four-path envelope is insufficient, preserve the
worktree and request a fresh reviewed amendment.

### Successful-full-CI trust anchor

Fast eligibility requires one closed internal trust envelope:

```text
FullCiTrustEnvelope
  anchor commit SHA
  ci workflow identity and exact workflow-run ID/attempt
  exact Ubuntu and Windows job IDs/conclusions
  exact full-preflight and skipped-fast-step conclusions
  ordered first-parent status-only transitions from anchor to head
```

The anchor lookup checks two independent authorities: immutable Git commit/blob
objects for repository history and current read-only GitHub Actions control-
plane evidence for completed workflow execution. The workflow grants
`GITHUB_TOKEN` only
`contents: read` and `actions: read`. The classifier uses the documented
`application/vnd.github+json` media type and API version `2026-03-10`; it may
not mutate a run, rerun, artifact, check, repository, setting, branch, or
permission, and it may not print or persist the token or response headers.

The exact lookup algorithm is:

1. Validate the push event, explicit event base, and proposed head using the
   Git-object rules below. Starting at head, walk first parents backward while
   each parent-to-child transition independently satisfies the complete exact
   status-only contract. Require at least one accepted transition. The commit
   at the beginning of that accepted suffix is the sole anchor candidate. The
   event base must lie on that same first-parent suffix or equal the candidate.
2. Query every page of
   `GET /repos/{owner}/{repo}/actions/workflows/ci.yml/runs` filtered by exact
   candidate `head_sha`, `branch=main`, and `event=push`. After locally
   rechecking every returned field, require exactly one distinct run ID for
   the file-scoped endpoint, with workflow name `ci`, returned `path` exactly
   `.github/workflows/ci.yml`, returned `head_branch` independently exactly
   `main`, exact candidate `head_sha`, `event=push`, `status=completed`,
   `conclusion=success`, and one positive exact `run_attempt`. Zero runs,
   multiple run IDs, incomplete pagination, or a mismatched field selects
   `full`. If `.github/workflows/ci.yml@main` is used internally, it is only a
   locally derived composite identity constructed after those two returned
   fields pass their independent checks. It is not a returned API field and
   may not be caller-supplied or substituted for either field check.
3. Query that exact attempt through
   `GET /repos/{owner}/{repo}/actions/runs/{run_id}/attempts/{run_attempt}/jobs`
   and consume every page. Require exactly one job named
   `preflight (ubuntu-latest)` and exactly one job named
   `preflight (windows-latest)`, with distinct stable job IDs, the same run ID
   and exact candidate `head_sha`, `status=completed`, and
   `conclusion=success`. Attempt identity is supplied by and bound to the
   attempt-specific endpoint; no job from a generic/latest or different-
   attempt response may be substituted. A missing, duplicate, extra, skipped,
   canceled, pending, neutral, stale, or mismatched platform job selects
   `full`.
4. In each exact platform job, require exactly one step named
   `Run Hum preflight` with `status=completed` and `conclusion=success`, and
   exactly one step named `Run status-only evidence` with
   `conclusion=skipped`. Missing, duplicate, renamed, wrong-attempt, or
   contradictory step evidence selects `full`. A successful fast-lane run is
   never a full-CI anchor.
5. Repeat the complete file-scoped run lookup and the exact-attempt jobs
   lookup after step validation, without trusting a cached response. Require
   the second snapshot to select the same sole run ID/attempt, the same two
   job IDs, and the same workflow, commit, branch, event, status, conclusion,
   job, and step facts. A run that starts rerunning, an attempt/status change,
   a job or step change, pagination disagreement, or any other control-plane
   mutation between snapshots selects `full`.
6. Bind the validated anchor SHA, run ID, run attempt, both job IDs, and the
   ordered anchor-to-head transition identities into the classifier result.
   Each Ubuntu/Windows job independently repeats the same lookup before lane
   selection and may emit `fast` only when its complete envelope agrees with
   its local immutable Git objects. The non-secret IDs and stable reason may be
   logged. Any platform disagreement stops closure of the increment even if
   both required jobs terminate successfully.

One distinct workflow run whose current exact attempt passed is not ambiguous
merely because an earlier attempt failed; only attempt-specific jobs from the
reported successful attempt count. Multiple run IDs for one anchor SHA, an
absent or changing attempt number, jobs from another attempt, or inability to
consume the complete response is ambiguous and selects `full`.

`concurrency.cancel-in-progress` remains `true`. It is safe only because a
pending, failed, or canceled executable commit cannot satisfy the completed-
success anchor. A status push that cancels such a run therefore selects `full`:
the last earlier successful anchor is separated from the new head by the
executable transition. A rapid status push may remain `fast` after canceling a
prior fast status run only when the same earlier full-CI anchor already proves
both platforms and every intervening transition is status-only. The permanent
matrix must prove both paths.

The production entry point obtains and validates the API evidence itself. It
has no command-line, environment, file, cache, commit-message, or workflow-
output escape that lets a caller assert that an anchor passed. The offline
test script may invoke internal pure validators with harness-owned synthetic
REST objects, but those test seams are not production classifier inputs.

### Closed classifier contract

The classifier begins in `full` mode. It may return `fast` only when the exact
Actions trust envelope above and all of the following are proven from Git
objects, not the mutable checkout:

1. one successful full-CI anchor is validated exactly as above and is a strict
   ancestor of head;
2. the event is a push to `main`, not a tag, workflow dispatch, pull request,
   schedule, first/zero-base push, or another event;
3. event base and head are present, valid commit objects, distinct, and available
   locally after complete-history checkout;
4. event base is an ancestor of head and the pushed range is one nonempty linear
   chain with no merge commit, missing parent, replaced ancestry, or ambiguous
   comparison edge;
5. event base lies on the accepted first-parent suffix from anchor through
   head, and every transition from anchor through head, not merely the
   aggregate event-base-to-head diff, modifies exactly `WORKORDER.md` as an
   ordinary file;
6. every transition rejects additions, deletions, copies, renames, type or mode
   changes, symlinks, submodules, multiple paths, and changes to source,
   fixtures, Cargo, tools, workflows, generated outputs, or any other path;
7. every `WORKORDER.md` blob from anchor through head is strict UTF-8 without
   BOM and contains exactly one header status interval and one current-gate
   interval;
8. the header interval begins immediately after the unique column-one
   `Status:` prefix and ends immediately before the unique exact unchanged
   `Owner: BDFL (Ocean).` line; the current-gate interval begins immediately
   after the unique exact heading `## Current authorization gate` and ends
   immediately before the unique exact final marker
   `<!-- workorder-current-authorization-gate:end -->`;
9. the `Status:` prefix, `Owner:` line, current-gate heading, final marker,
   their order, and every byte outside the two interval bodies are identical
   for each transition; moving, duplicating, deleting, or widening an anchor,
   or adding content after the final marker, therefore selects full CI;
10. at least one byte changes inside an allowed interval in the event's pushed
    range; and
11. anchor-to-head and event-base-to-head diff hygiene pass without conflict
    markers or whitespace errors.

The validator must read exact blobs by commit identity and compare the
immutable remainder after replacing only the two recognized interval bodies
with fixed internal sentinels. It may not infer permission from current line
numbers, hunk headers, commit messages, author identity, a `docs` prefix,
`git diff` path output alone, or the final checked-out file.

Every commit transition from the successful full-CI anchor is checked
independently. A range containing an earlier source/tool/workflow/fixture
change remains `full` even if a later commit reverts it and the aggregate final
tree looks status-only. A range with two or more individually valid status
commits may be `fast` only when all transitions trace to the same exact
successful full-CI anchor and independently satisfy the complete contract.

The result domain is exactly `fast` or `full`. Internal reason keys may explain
why `full` was selected, but they are not a public Hum schema or report. An
exception, unset output, unexpected value, missing comparison fact, Git error,
or validator disagreement remains `full`. Workflow glue must initialize the
job output to `full` and replace it with `fast` only after one successful exact
validation. A missing token, permission denial, transport error, rate limit,
API schema mismatch, incomplete pagination, lookup exception, unset output,
or classifier failure must select `full`; it must not skip both lanes or leave
a required job pending.

### CI execution contract

The `ci` workflow remains triggered and both current matrix jobs remain present
for every existing event. Do not use `paths-ignore`, skip the workflow, rename
the jobs, or create a replacement check whose absence can leave branch
protection pending.

After complete-history checkout, each platform job independently classifies
the same explicit repository/workflow/event/base/head tuple, queries the exact
same read-only Actions evidence, and runs exactly one lane:

- `full`: preserve the existing Cargo cache, stable Rust/rustfmt/Clippy setup,
  and complete `./tools/check_all.ps1` invocation; or
- `fast`: run the classifier's permanent adversarial test script, revalidate
  the actual anchor/base/head trust envelope, run `git diff --check` for both
  anchor-to-head and event-base-to-head ranges, and invoke unchanged
  `tools/check_text_hygiene.ps1`,
  `tools/check_public_readiness.ps1`, and
  `tools/check_release_readiness.ps1`.

The fast lane must not install Rust, restore/build Cargo artifacts, build Hum,
or run fixture/runtime probes. Its permission rests on proof that executable,
test, tool, workflow, contract, and requirement bytes are identical throughout
the pushed range. Cache hits are performance only and never classification
authority.

The implementation commit necessarily changes workflow/tool bytes and must
therefore select full CI. That exact run must complete successfully in both
unchanged platform jobs, with `Run Hum preflight` successful and
`Run status-only evidence` skipped, before it can become the first production
anchor. A status push while that run is pending, failed, or canceled must run
full CI instead.

The first later independently reviewed `WORKORDER.md`-only status update is the
production fast-lane proof. It must bind the implementation commit's exact
run/attempt/jobs as its anchor, produce successful jobs with the unchanged
names on Ubuntu and Windows, log that the fast lane ran and full preflight did
not, and record total and status-lane step durations. The later status update
that records those workflow/job IDs is a required consecutive-status proof: it
must trace to the same implementation anchor and also run fast on both
platforms before this increment is considered durably closed.

### Permanent adversarial evidence

`tools/test_workorder_status_boundary.ps1` must create isolated temporary Git
repositories, construct the required histories itself, own all synthetic
workflow-run/job/step API responses, run without network or repository
mutation, and remove every temporary artifact. The production entry point may
not consume the test injection seam. Each case must assert the exact lane,
anchor identity when applicable, and a stable internal reason. At minimum it
proves:

- one exact successful full-CI anchor followed by a valid header/current-gate
  status update selects `fast`;
- valid updates to either permitted interval, and two or more valid linear
  status commits, retain the same full-CI anchor and remain eligible;
- one exact successful rerun attempt is eligible only when the run record and
  attempt-specific jobs agree completely;
- an edit to an AP/AQ mandate, acceptance criterion, fixture list, decision
  lock, or other Work Order requirement selects `full` even though the only
  path is `WORKORDER.md`;
- Rust/source, fixture, Cargo, tool, workflow, generated-output, and multiple-
  path changes each select `full`;
- an earlier executable change followed by a status update, and an earlier
  executable change followed by a revert, both select `full`;
- Work Order addition, deletion, rename, copy, mode/type change, and symlink
  replacement select `full`;
- missing, zero, invalid, non-commit, unavailable, non-ancestor, and reversed
  comparison bases or anchor candidates select `full`;
- a merge commit, missing parent, empty range, tag, and workflow dispatch
  select `full`;
- a status push immediately after a pending, failed, canceled, skipped, or
  platform-incomplete executable run selects `full`;
- a rapid status push that cancels an unproven executable run selects `full`,
  while rapid consecutive status pushes after one proven full anchor remain
  `fast` even when an intermediate fast run is canceled;
- a prior successful fast-lane run offered as the anchor selects `full`;
- zero or multiple matching workflow run IDs, incomplete pagination,
  unavailable/rate-limited/unauthorized Actions evidence, wrong workflow name,
  returned `path`, returned `head_branch`, event, head SHA, status, conclusion,
  or attempt selects `full`; wrong `path` with the right `head_branch`, the
  right `path` with the wrong `head_branch`, and a caller-derived
  `.github/workflows/ci.yml@main` composite substituted for either returned
  field each select `full`;
- any run, attempt, job, step, pagination, status, or conclusion change between
  the two required control-plane snapshots selects `full`;
- missing, duplicate, extra, pending, skipped, canceled, failed, wrong-SHA, or
  wrong-attempt Ubuntu/Windows jobs select `full`;
- missing, duplicate, renamed, wrong-attempt, unsuccessful, or contradictory
  `Run Hum preflight`/`Run status-only evidence` steps select `full`;
- a failed earlier attempt followed by one exact successful current attempt is
  accepted, while a missing/changing attempt or jobs from another attempt
  selects `full`;
- missing, duplicated, reordered, moved, or altered interval anchors and
  content after the final current-gate marker select `full`;
- malformed UTF-8, BOM insertion, conflict markers, and whitespace errors do
  not receive fast-lane acceptance; and
- two fresh runs over each history return byte-identical mode/reason evidence.

The test matrix must include a case where the final aggregate diff is
status-only but an intermediate commit touched executable bytes, plus a case
where a status push would otherwise inherit a canceled or failed executable
run. Removing either case, weakening per-transition inspection, treating fast
CI as a full anchor, or dropping one platform/step check must fail full
preflight.

### Compatibility, bans, and honesty locks

This increment changes CI evidence execution only. It adds no Hum command,
schema identifier, JSON field, diagnostic, pipeline stage, language behavior,
runtime behavior, dependency, generated document system, branch-protection
mutation, GitHub API mutation, or public report. It does not change what
`tools/check_all.ps1` checks on the full path.

The exact read-only GitHub Actions queries above are the sole external evidence
lookup authorized by this increment. No Actions mutation, rerun, cancellation,
artifact write, check update, repository write, broader token permission, or
other GitHub API use is authorized.

The one internal status-boundary classifier is the sole narrow planning
exception to Work Order 9's historical no-new-pipeline-gate language. It is not
a Hum compiler stage, public report gate, or precedent for another CI bypass;
no other exception is implied.

No broad `docs/**`, Markdown-only, commit-message, author, extension, or
path-prefix exemption is allowed. No final-tree-only classifier, unbounded
line-range allowlist, mutable-checkout comparison, permissive fallback,
`continue-on-error` skip, or cache-derived trust may select `fast`.

No checked-in commit allowlist, hard-coded workflow/run/job ID, mutable cache,
workflow output from a prior fast run, or caller-supplied success claim may act
as a full-CI anchor. The implementation commit's IDs are discovered and bound
through the same generic read-only algorithm used by every later status chain.

The status-boundary validator proves location only. It does not prove that a
recorded commit hash, workflow ID, duration, acceptance claim, BDFL ruling, or
authorization statement is true. At implementation issuance, every status
update still required the normal independent review, scoped commit, separate
BDFL push authorization, and terminal CI inspection. That sentence is
preserved as historical implementation-time policy. After the independently
verified full/fast/fast production chain, the BDFL-directed 2026-07-14
`AGENTS.md` amendment supersedes it only for future exact routine status-only
closures satisfying every permanent eligibility condition: independent
review and full local preflight are omitted, while exact read-only fact
verification, explicit commit and push authority, fail-closed classifier
selection, and terminal both-platform CI remain. Every other change keeps the
normal review and full-preflight gates. This operating-policy amendment is
itself a governance/process change, is not status-only eligible, and requires
one fresh independent review plus full CI. Faster CI does not merge the
remaining governance gates.

The separately queued permanent adversarial-evidence integrity amendment is
neither rewritten nor partially implemented here. It remains the next distinct
planning/review boundary after this increment closes and before Session AQ can
be authorized.

### Independent pre-issuance review gate

The author of this amendment is disqualified from its verdict. A fresh
architect-reviewer must cold-start from repository ground truth and verify the
complete `WORKORDER.md` diff, including:

- the exact `aa69cf4` baseline and workflow/job timing evidence;
- the four-path implementation envelope and unchanged required CI job names;
- the exact read-only Actions permission, REST endpoints, run/attempt/job/step
  validation, independent returned `path` and `head_branch` validation, and ban
  on caller-supplied anchor evidence or composite-field substitution;
- one successful full-CI anchor and complete anchor-to-head, per-transition,
  Git-object-based classification;
- pending/failed/canceled predecessor behavior, retained
  `cancel-in-progress: true`, and consecutive-status tracing to one full
  anchor;
- exact status-region anchors and byte identity outside their bodies;
- default-full behavior for every ambiguity and classifier failure;
- the complete adversarial matrix, unchanged full preflight, and both-platform
  timing/closure plan;
- the sole narrow CI-gate exception and absence of a Hum/public pipeline gate;
  and
- continued separation of the queued integrity amendment and unauthorized AQ.

The reviewer runs `git diff --check` and `./tools/check_all.ps1`, reports
P0/P1/P2 findings with exact lines, and gives exactly one verdict: ACCEPT,
ACCEPT WITH REQUIRED FIX, or REJECT. It must not edit, commit, push, implement
the CI change, authorize the integrity amendment, mutate Issue #1, or begin AQ.
Any authoritative mutation after that verdict requires a fresh complete
review.

### Acceptance criteria and hard stop

Before implementation handoff:

- focused classifier tests pass on the host;
- the anchor matrix proves complete successful-run/attempt/job/step evidence,
  canceled/failed/pending predecessor fallback, consecutive status chains,
  and production/test evidence-input separation;
- `cargo fmt --check`, `cargo test`, warnings-denied Clippy,
  `git diff --check`, and full `tools/check_all.ps1` remain green;
- the worktree contains only the exact four implementation paths;
- the implementer enumerates Windows and Ubuntu behavior and any unexercised
  configuration; and
- the complete worktree remains uncommitted for fresh independent review.

After independent acceptance and a separately authorized commit/push:

- the workflow/tool implementation commit selects and passes full CI on both
  Ubuntu and Windows, with both exact full-preflight steps successful and both
  exact status-only steps skipped;
- the first eligible status-only push discovers that implementation run as its
  exact full-CI anchor and selects only the fast lane in both unchanged
  required jobs;
- the next status-only evidence-recording push traces across the prior fast
  commit to that same full-CI anchor and again selects only the fast lane in
  both required jobs;
- the report records baseline, full-path, and fast-path job/step timings without
  claiming a general CI speedup beyond those observations; and
- any false fast classification, missing required job, platform disagreement,
  unproven/canceled/failed anchor, or inability to prove the complete trust
  envelope stops the increment for correction.

Stop after recording the accepted fast-lane evidence. Do not begin the queued
integrity amendment, authorize Session AQ, mutate Issue #1, or combine another
CI optimization. Session AQ remains unauthorized pending the separate
integrity-amendment boundary and its own BDFL go signal.

### Later private batching advisory (not authorized)

The full preflight launches the built executable repeatedly across many
fixture/stage probes, and the accepted baseline shows a large Windows process-
startup cost. A later separately reviewed optimization may batch most probes
through one private in-process or manifest-driven harness while retaining a
small end-to-end executable matrix for stdout, stderr, exit status, human/JSON,
and runtime compatibility. That harness rewrite, sharding, and any broader CI
optimization are outside this increment and receive no implementation credit
or authorization here.

## PRE-AQ-INTEGRITY: permanent adversarial-evidence integrity

Planning state: proposed under the BDFL's bounded planning authorization. These
planning bytes were independently rejected once because the unchanged
15-minute CI limit contradicted the nine-build implementation envelope. The
BDFL authorized only the exact timeout correction below. The corrected bytes
are uncommitted and do not authorize implementation. The author of this section
is disqualified from its verdict. Implementation requires a fresh independent
pre-issuance re-review, BDFL acceptance of the exact reviewed bytes, a scoped
documentation commit, durable publication, terminal Ubuntu and Windows CI, a
status record, and a separate BDFL implementation go signal.

Purpose: close the evidence-authenticity defect class exposed during Session AP
before any Session AQ work. A green test, source marker, snapshot, or public
output is evidence only for what it actually exercises. A load-bearing claim
earns acceptance only when a real production entry point traverses the claimed
producer, identity, transport, precedence relationship, and consumer, and a
controlled break at that real boundary changes the observed outcome in the
required fail-closed way.

This amendment does not declare all existing tests sound, prove the absence of
all shortcuts, or retroactively certify every prior Hum feature. Its initial
closed claim inventory covers the accepted Session AP diagnostic-identity and
prior-blocker spine named below and installs a reusable private evidence rule.
Future work that wants to claim another load-bearing gate is causally exercised
must separately authorize and add a production probe and nondegenerate sabotage
for that gate; a passing supporting unit test alone never inherits that claim.

### Planning baseline and authority boundary

The preceding exact routine status closure is
`80a0b93e81233f6a836449a04aea2f88707c9440`. Workflow `29363822737`, attempt 1,
completed successfully for that exact commit:

- Ubuntu job `87190299329` on `ubuntu-latest` succeeded in 1m 15s;
- Windows job `87190299278` on `windows-latest` succeeded in 3m 38s;
- both selected `mode=fast` with `reason=eligible_status_chain`;
- both used full-CI anchor
  `65ad0d5f5db2bcd90fdbb8a8afc1208f3d46b659`, workflow `29357247935`,
  attempt 1, Ubuntu anchor job `87167872491`, and Windows anchor job
  `87167872510`;
- both proved the exact transition
  `65ad0d5f5db2bcd90fdbb8a8afc1208f3d46b659` ->
  `80a0b93e81233f6a836449a04aea2f88707c9440`; and
- both succeeded in `Run status-only evidence` while Cargo caching, Rust
  toolchain preparation, and `Run Hum preflight` were skipped.

Those facts close only the preceding status record. They neither prove this
integrity design nor authorize its implementation. GitHub Issue #1 remains
open and non-authoritative. Session AP remains accepted exactly as published;
the amendment adds evidence gates without changing its semantics or public
outputs.

### Concrete threat model

The integrity harness assumes an implementation or test can be accidentally or
opportunistically shaped to satisfy its own grader. It must detect at least:

- a fabricated or hand-asserted occurrence, cause, route, relationship,
  expected result, or authoritative set that never came from the real
  producer;
- circular validation in which actual and expected projections are produced by
  the same call, object, collection, or reconstruction path;
- self-proving transport in which a downstream stage validates facts derived
  from its own output instead of separately retained upstream authority;
- identity reconstructed or selected from public code, reason, message, help,
  rendering, display name, filename, line, column, span, or source-text scan;
- a Cartesian-product, same-region, same-code, or compatible-span precedence
  relationship that was not produced for the exact shared semantic occurrence;
- stale evidence from another source revision, fixture revision, binary,
  command, platform, run, or expected-output inventory;
- synthetic-only success through a mock, test constructor, injected provider,
  `cfg(test)` path, fake carrier, or copied answer object that production cannot
  traverse;
- substitution of a valid fact from another owner, target, call, stage, node,
  route, relationship, occurrence, or platform;
- missing, ambiguous, conflicting, reordered, duplicated, or injected evidence
  that a permissive collector silently accepts;
- a source audit, symbol-name check, snapshot, output hash, or public rendering
  that stays green after the real semantic path is broken;
- a mutation that fails to compile, crashes for an unrelated reason, skips the
  probe, edits its expected answer, or edits the test instead of breaking the
  claimed production boundary; and
- nondeterministic, shared-state, cache-derived, platform-specific, or
  order-dependent evidence presented as repeatable production behavior.

The threat model is bounded. It is not a proof against a malicious operating
system, compiler toolchain, Git implementation, or reviewer. It does require
the repository harness to reject every known evaluator-visible shortcut in its
declared evidence unit and requires a fresh human-directed independent review
before that unit can authorize later work.

### Production evidence versus supporting evidence

Evidence is classified internally as exactly one of:

1. **Production-path evidence.** A fresh process invokes the checked-in Hum
   executable through an existing public CLI command and checked-in fixture.
   It traverses the ordinary parser, resolver/analyzer, diagnostic transport,
   precedence, Core/graph/IR or runtime consumer used outside tests. No mock,
   direct Rust function call, test constructor, injected provider, hidden flag,
   synthetic occurrence, or alternate expected-answer carrier participates.
2. **Supporting evidence.** Unit tests, property tests, pure validator tests,
   synthetic objects, source audits, symbol inventories, snapshots, public
   output comparisons, hashes, and fixture-text assertions. These may explain,
   localize, or protect a production claim, but none can create or replace one.

Every load-bearing claim in the private inventory requires both a production
probe and at least one compiling production-source sabotage whose detection is
observable through that probe. A source audit may prove that a forbidden name
is absent; it cannot prove what code executed. A snapshot or hash may prove byte
stability; it cannot prove semantic causality. Expected output is frozen before
the sabotaged run and is never reconstructed from that run. Public human/JSON
rendering remains compatibility evidence, not internal occurrence authority.

Supporting tests may use synthetic inputs only to test the private manifest,
raw-byte runner, patch validation, and failure handling. The production runner
has no provider parameter, environment override, alternate manifest path,
caller-supplied binary, caller-supplied source root, expected-answer input, or
success switch. Production Rust gains no feature, `cfg`, environment variable,
hidden command, or test hook for this amendment. A synthetic seam that becomes
reachable by the production entry point is a hard failure, not a convenience.

### Exact later implementation envelope

The later implementation may change exactly five checked-in paths:

- `.github/workflows/ci.yml`: change only the existing `preflight` job scalar
  `timeout-minutes: 15` to `timeout-minutes: 30`; every other workflow byte
  remains unchanged;

- new `tools/adversarial_evidence_integrity.psd1`: one private, code-free,
  closed data inventory for exact claim IDs, command vectors, input
  fingerprints, accepted raw-channel fingerprints, semantic assertions, and
  compiling sabotage specifications;
- new `tools/check_adversarial_evidence_integrity.ps1`: the dependency-free,
  parameterless production runner that resolves the repository, Git, Cargo,
  accepted source commit, manifest, executable, and fixtures itself; captures
  raw channels; creates isolated local clones; applies exact sabotages; builds
  them offline; runs the real probes; and removes all temporary artifacts;
- new `tools/test_adversarial_evidence_integrity.ps1`: supporting synthetic
  corruption tests for the closed manifest, fingerprinting, raw process
  capture, exact patch application, fail-closed result handling, and the
  production/synthetic seam boundary; and
- `tools/check_all.ps1`: one invocation of the supporting validator followed,
  after the ordinary Hum build and Session AP matrix, by one invocation of the
  production runner.

The timeout scalar is the sole authorized workflow change. Workflow and
required job names, the Ubuntu/Windows matrix, permissions, concurrency and
`cancel-in-progress: true`, checkout and cache behavior, classifier invocation
and default-full handling, full/fast lane conditions, every existing step and
its order, status-only trust-anchor lookup, complete full-preflight contents,
and required-check continuity remain byte-unchanged. No command, environment
override, or runtime parameter may extend the timeout.

The independently verified Windows full job `87167872510` in workflow
`29357247935`, attempt 1, used 12m 43s of the existing 15-minute limit, including
11m 52s in `Run Hum preflight`. Its log recorded one cached `cargo build` taking
27.45s. Nine such builds alone require approximately 4m 07s before local clone,
production-probe, supporting-suite, evidence-processing, and cleanup overhead,
while the old ceiling leaves only 2m 17s of total job headroom. The observed
evidence therefore contradicts the former implementation envelope. Thirty
minutes is a bounded execution ceiling, not a performance or capacity claim and
not permission to add work.

The implementation changes workflow and tool paths and therefore must select
the existing full lane on both platforms. If either job cannot complete within
30 minutes, or these five paths cannot provide the required evidence, stop and
request a fresh reviewed amendment; do not weaken, shard, skip, parallelize
across jobs, or move the proof implicitly.

The implementation may create only temporary, ignored local clones and build
directories beneath the operating system's temporary directory while running.
They are not checked-in scope. Each clone is made locally from the exact
accepted commit with `git clone --no-hardlinks --no-checkout`, is detached at
that exact commit, has no alternates/promisor dependency, receives one exact
manifest-owned source mutation, builds with `cargo build --locked --offline`,
and is deleted in success and failure paths. It may mutate only temporary
copies of these accepted AP production paths:

- `src/resolve.rs` for producer-owned call occurrence construction;
- `src/ownership_check.rs` for exact resolver-call identity binding;
- `src/profile_check.rs` for prior-blocker transport;
- `src/effect_check.rs` for exact H0907-over-H0801 precedence consumption;
- `src/core_verify.rs` for independent upstream Core projection validation;
  and
- `src/graph.rs` for the final graph consumer's independent projection check.

No checked-in Rust source is changed by the implementation. A needed mutation
outside that temporary allowlist, a noncompiling mutation, or a mutation that
can pass only by changing tests, fixtures, snapshots, expected evidence, or
production code is scope pressure and stops the session.

### Closed private inventory and identity rules

`adversarial_evidence_integrity.psd1` is internal tool data, not a Hum schema,
report, command, or extension point. `Import-PowerShellDataFile` loads it as
data; no script block, expression, command, provider, environment lookup, or
executable path is permitted in it. The root and every nested record have an
exact key set. IDs use a closed ASCII grammar, are unique, and are sorted by
their declared stable ordinal. Duplicate complete records are still errors.

Each production claim freezes:

- one stable claim ID and one of the closed boundary kinds `producer`,
  `identity`, `transport`, `precedence`, or `consumer`;
- the exact logical executable `hum`, ordered argument vector, repository
  working directory, and empty environment-delta set;
- every checked-in input path and SHA-256 of its raw bytes;
- expected exit code, stdout SHA-256, stderr SHA-256, exact required and
  forbidden public facts, and whether JSON parsing is required;
- the exact accepted producer/owner/stage/relationship route the claim means;
- at least one sabotage ID that breaks that boundary; and
- the exact fail-closed detection class expected from the compiling mutated
  production binary.

Each sabotage freezes a unique ID, boundary kind, one allowlisted source path,
the full accepted source-file SHA-256, one exact old raw UTF-8 byte sequence and
its SHA-256, one exact replacement raw UTF-8 byte sequence and its SHA-256, the
requirement that the old sequence occur exactly once, the resulting full-file
SHA-256, affected claim IDs, and the one expected detection class. Patch text is
stored as base64 data so newline and escaping behavior cannot reinterpret it.
The replacement must differ, remain strict UTF-8 without BOM, compile
successfully, and break the real boundary rather than a test or assertion.

An unknown key, enum, path, command, argument, environment delta, algorithm,
hash form, claim, mutation, detection class, or relationship fails before any
probe. Missing, duplicate, extra, reordered, cross-linked, or orphaned records
fail. Absolute paths, `..`, symlinks/reparse points, submodules, paths outside
the repository, and mutation targets outside the six-path temporary allowlist
fail. The inventory cannot bless its own changed hash or compute expected
channels from the candidate run.

### Exact production probe set

The logical executable is resolved only as
`<repo>/target/debug/hum.exe` on Windows and
`<repo>/target/debug/hum` elsewhere after the ordinary root `cargo build`.
The production runner invokes it directly with `UseShellExecute = false`, no
shell command string, the repository as working directory, and exactly these
ordered argument vectors:

1. `resolve --format json fixtures/diagnostics/session_ap_parser_resolver_precedence_fail.hum`;
2. `type-check --format json fixtures/diagnostics/session_ap_same_line_independent_causes_fail.hum`;
3. `profile-check --format json fixtures/diagnostics/session_ap_authority_ownership_precedence_fail.hum`;
4. `effect-check --format json fixtures/diagnostics/session_ap_effect_ownership_precedence_fail.hum`;
5. `profile-check --format json fixtures/diagnostics/session_ap_ownership_resource_profile_chain_fail.hum`;
6. `core-verify --format json fixtures/diagnostics/session_ap_effect_ownership_precedence_fail.hum`;
7. `graph fixtures/diagnostics/session_ap_effect_ownership_precedence_fail.hum`;
8. `ir-readiness --format json fixtures/diagnostics/session_ap_effect_ownership_precedence_fail.hum`;
9. `run fixtures/diagnostics/session_ap_effect_ownership_precedence_fail.hum --entry caller`; and
10. `run fixtures/diagnostics/session_ap_ownership_resource_profile_chain_fail.hum --entry caller`.

The manifest pins the already accepted exact raw channels and exit for each
command before sabotage. It also pins the semantic observations already owned
by Session AP: parser/resolver coexistence; two distinct same-line H0605 facts;
H0618 over only its exact H0801 competitor; H0907 over only its exact H0801
competitor; unchanged H0801 transport through profile; successful Core, graph,
and IR projection validation without private-field leakage; runtime exit 2,
zero stdout, the exact owning diagnostic on stderr, and no generic trap. The
runner does not infer those expectations from current output, test names,
source text, or the mutated binary.

Every baseline command runs twice in two fresh processes. Every sabotage runs
in a separately created clone and fresh process after a successful offline
build. Process reuse, output reuse, inherited test-only environment, a prior
result cache, or accepting one run for two claims is forbidden. A build failure
or unrelated crash earns no sabotage credit. The mutated probe must reach the
named production boundary and produce its exact registered fail-closed
detection; merely producing different bytes is insufficient.

### Fingerprints and reproducible byte evidence

Every fingerprint uses NIST SHA-256 over the stated byte sequence and is
rendered as exactly 64 lowercase hexadecimal characters. Git object IDs are
identities, not substitutes for these fingerprints. No newline normalization,
JSON reserialization, path normalization, text decoding, or shell capture may
occur before hashing.

The exact byte producers are:

- checked-in input, manifest, script, source preimage, source postimage, and
  temporary patched-file fingerprints: the bytes returned by
  `[System.IO.File]::ReadAllBytes(<resolved exact path>)`;
- stdout and stderr fingerprints: the separate bytes copied directly from
  `System.Diagnostics.Process.StandardOutput.BaseStream` and
  `StandardError.BaseStream` for the exact executable and ordered arguments
  above, with `UseShellExecute = false`; exit status is recorded separately as
  a signed decimal integer;
- command-identity fingerprints: UTF-8 without BOM for the logical executable
  name followed by one `0x00`, then each ordered argument followed by one
  `0x00`; and
- the tracked implementation-diff fingerprint reported at handoff: raw stdout
  bytes, captured through the same process base-stream runner, from the exact
  argument vector
  `git -c safe.directory=<resolved-repo> diff --binary --no-ext-diff -- .github/workflows/ci.yml tools/check_all.ps1`.

The three new untracked implementation files are fingerprinted individually by
raw file bytes; concatenating filenames, text output, or PowerShell pipeline
strings is forbidden. The handoff reports every fingerprint with its algorithm
and exact producer. The production runner emits one canonical LF-terminated,
UTF-8-without-BOM private log line per baseline run and sabotage containing the
claim/mutation ID, command-identity hash, input/source hashes, stdout/stderr
hashes, exit, and detection class in manifest ordinal order. This log is CI
evidence only, not a public report or stable Hum schema.

Two fresh complete runner invocations on one platform must produce byte-
identical canonical evidence after excluding only the absolute temporary-root
display field, which is never part of an identity or hash. The implementation
reviewer independently reproduces the hashes from raw bytes rather than copying
the implementer's report. Ubuntu and Windows must execute the identical claim
and mutation inventory. The same platform-neutral command, fixture, channel,
semantic, and detection fingerprints are required on both; a difference may
not be normalized or explained away inside the implementation. If the accepted
base genuinely differs, stop for a reviewed amendment.

### Permanent sabotage and adversarial matrix

The permanent matrix must prove every row independently; one mutation may not
stand in for another category:

| Boundary | Required compiling production sabotage | Required observation |
| --- | --- | --- |
| producer | break resolver construction from the parser-owned call node while leaving display data and source-audit markers intact | the exact resolve/downstream production probe fails closed; a hand-built occurrence cannot rescue it |
| identity | bind an H080 use to a valid sibling call or target while preserving code, line, span, and public-looking bytes | zero precedence for the substituted occurrence and the registered invariant/detection result, never heuristic selection |
| transport | drop, duplicate, reorder, and valid-cross-occurrence-substitute the carried blocker before profile consumption | each independent mutation is rejected; current output or a rebuilt expected set cannot become authority |
| precedence | weaken H0907-over-H0801 consumption to matching codes/region or the wrong shared call | the unrelated competitor survives or the relationship is rejected exactly; no Cartesian product is accepted |
| Core consumer | validate a Core projection against itself or omit one upstream occurrence | `core-verify` rejects the independent-authority corruption before public projection |
| graph consumer | validate graph facts against graph-derived authority or substitute a valid foreign occurrence | `graph` rejects before serialization even when public JSON would otherwise look unchanged |

The minimum is nine separately built sabotages: one producer, one identity,
four independent transport mutations (missing, duplicate, reordered, and valid
cross-occurrence substitution), one precedence, one Core consumer, and one
graph consumer. A combined patch, one compiled binary reused for multiple
rows, or one observed failure credited to several mutations does not count.

In addition, the supporting corruption suite must independently reject:

- missing, duplicate, extra, reordered, renamed, invalid-kind, and orphaned
  claim or sabotage records;
- every missing, extra, duplicated, malformed, or substituted field and every
  unknown nested key in the private inventory;
- wrong command order, executable, fixture, input hash, source hash, old/new
  patch hash, detection class, expected exit, stdout hash, or stderr hash;
- an old patch sequence occurring zero times or more than once, a postimage
  hash mismatch, no-op patch, BOM, invalid UTF-8, path escape, symlink/reparse
  target, submodule, or non-allowlisted source path;
- a fabricated expected object, actual-as-expected alias, same collection on
  both sides, copied public diagnostic, caller-supplied success, and synthetic
  provider offered to the production entry point;
- source audit and snapshot remaining green while each real production
  sabotage is active;
- editing a unit test, fixture expectation, snapshot, manifest copy, or
  temporary expected output instead of the named production boundary;
- stale evidence from another HEAD, source file, fixture, command, binary,
  platform, run ordinal, or earlier temporary clone;
- missing, conflicting, nondeterministic, reordered, duplicated, injected, or
  platform-disagreeing evidence lines;
- a noncompiling mutation, skipped probe, generic unrelated crash, zero-command
  run, reused process, reused temp directory, leaked temp artifact, or networked
  Cargo build; and
- removal of any one of the nine required production sabotages, ten baseline
  commands, two fresh runs, raw-channel hashes, or Windows/Ubuntu requirements.

The test script owns its synthetic objects and may call pure closed validators,
but it cannot satisfy a production row. The production runner must explicitly
reject a synthetic-provider parameter or environment variable if one is
introduced by mutation. A source audit ensures the seam stays absent, but the
production-entry sabotage is the causal evidence.

### Checks, configuration, and reviewer reproduction

The focused implementation commands are exactly:

```powershell
pwsh -NoLogo -NoProfile -File .\tools\test_adversarial_evidence_integrity.ps1
pwsh -NoLogo -NoProfile -File .\tools\check_adversarial_evidence_integrity.ps1
```

Then the implementer and independent reviewer run:

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
git diff --check
.\tools\check_all.ps1
```

`check_all.ps1` must invoke the focused supporting suite and production runner
exactly once each; recursively invoking full preflight, silently skipping a
mutation, or treating a source audit as the production result is forbidden.
The existing Windows-locality and effect-bakeoff checks, text/public/release
readiness, and status-boundary tests remain green and unchanged.

The implementer reports the exact five-path envelope; the workflow diff proving
that only the timeout scalar changed; the two-tracked-path diff fingerprint and
three new-file raw hashes; manifest, source, fixture, command, channel, and
evidence-log hashes; ten baseline outcomes; nine compiling sabotages; cleanup;
and all locally exercised configurations. It reports total job duration,
complete-preflight duration, supporting-suite duration, production-runner
duration, timeout headroom, and baseline, build, probe, and cleanup timing where
the runner emits them for each platform. These are observations for the exact
run, not generalized speed or capacity claims. Only the installed host is
required before review; an unavailable non-host configuration is disclosed,
not invented or downloaded.

The fresh independent reviewer must not have authored or edited the planning
or implementation bytes. It cold-starts from the accepted implementation base,
recomputes every fingerprint, executes both focused scripts twice, inspects the
temporary clone/build isolation, and creates at least two additional temporary
nondegenerate sabotages: one at a producer/identity boundary and one at a
transport/precedence/consumer boundary. They must exercise real production
commands and are removed afterward. Repeating a manifest mutation or trusting
the implementer's canonical log is insufficient.

After a separately authorized implementation push, the implementation commit
must select `mode=full` on both unchanged required jobs. `Run Hum preflight`
must succeed, `Run status-only evidence` must be skipped, and the two platform
logs must contain the same complete claim/mutation inventory and matching
platform-neutral evidence hashes. The implementation review must prove that the
workflow diff is exactly `timeout-minutes: 15` to `timeout-minutes: 30`, with no
other workflow byte changed, and both jobs must finish terminally without
cancellation or timeout. Missing evidence, timeout, fast selection, platform
disagreement, or one green platform is failure. No command may extend the job
timeout, and no other workflow behavior may change under this scope.

### Required gated sequence

The only valid sequence is:

1. this planning author leaves only `WORKORDER.md` modified and uncommitted;
2. one fresh independent architect-reviewer reviews the complete planning diff;
3. the BDFL accepts the exact reviewed bytes and separately authorizes a
   `WORKORDER.md`-only commit;
4. the BDFL separately authorizes its push; Ubuntu and Windows terminate green;
5. an exact routine status-only record may use the permanent `AGENTS.md`
   exception only if it satisfies every eligibility condition and its fast
   evidence terminates green on both platforms;
6. the BDFL issues a separate implementation go signal for the exact five-path
   envelope;
7. the implementer leaves the complete implementation uncommitted for one
   fresh independent full review;
8. after ACCEPT, the BDFL separately authorizes the exact scoped commit and
   later the push; the implementation commit runs full CI on both platforms;
9. a final exact status record captures commit, workflow, jobs, full-lane
   selection, complete evidence hashes, platform agreement, and durations; and
10. only after that status record is durably published and terminally green may
    the BDFL consider a separate Session AQ go signal.

An ACCEPT WITH REQUIRED FIX or REJECT returns only to the appropriate author or
implementer. The reviewer never edits the deliverable. Commit authority does
not imply push authority; publication does not imply later-session authority.
Any substantive change after a verdict requires a fresh complete review.

### Compatibility, exclusions, and hard stop

This amendment adds no public Hum command, schema identifier, JSON field,
report, diagnostic, compiler stage, language behavior, runtime behavior,
decision ruling, dependency, generated-document system, GitHub setting, branch
protection change, Actions permission, or unrelated CI optimization. Existing
human/JSON/runtime bytes, codes, messages, help, spans, counts, ordering, exit
behavior, adapter behavior, Session AP semantics, decisions 0014-0018, and the
three error channels remain unchanged.

The private manifest and canonical CI log are internal test inputs/evidence,
not public product surfaces. The runner may not call the network. `cargo
--offline` and local Git clones are mandatory. It may not alter the main
worktree, index, Git config, `.git` metadata, Cargo lockfiles, fixtures,
snapshots, tests, any workflow byte beyond the exact timeout scalar, cache
policy, or expected production outputs.

The accepted status-boundary classifier and fast-lane contract remain the sole
CI evidence-selection authority. Exact routine status-only commits may inherit
the last proven full-CI anchor because every executable and integrity-tool byte
is unchanged; they do not rerun the sabotage matrix. Any integrity-tool,
manifest, source, fixture, workflow, or requirement change selects full CI.

Stop if the five-path implementation cannot produce compiling causal sabotage,
cannot finish within 30 minutes on either platform, requires a production seam,
requires platform-specific expected answers, or discovers a genuine semantic
or public-output defect. Also stop if success would require sharding, parallel-
job expansion, skipped evidence, changed lane logic, changed job contents, or
any workflow byte beyond the exact timeout scalar. Record the pressure and
request a separately reviewed scope or decision gate without repair or scope
expansion. Do not repair Session AP semantics under an evidence amendment.

Stop after the final accepted status record. Session AQ remains unauthorized
until this complete plan/review/commit/publication/implementation/review/full-
CI/status sequence is durably complete and the BDFL gives a separate go signal.
GitHub Issue #1 remains untouched. No dashboard, broader retrospective test
audit, later private batching, or other optimization is included.

## Session AQ: runtime and top-level composition closure

Purpose: remove the final independent reconstruction paths, make the collector
enforce one owner per occurrence, and prove repository-wide closure.

### AQ exact integration map

Authorized files are limited to:

- `src/main.rs`;
- `src/run.rs`;
- `src/diagnostics.rs`;
- `src/json.rs` only for compatibility tests, not new fields;
- `src/app_entry.rs` and `src/capability_root.rs` only to remove superseded
  hard-coded diagnostic classifiers;
- `src/diagnostic.rs` and `src/diagnostic_catalog.rs` only for final collector
  enforcement and audit closure;
- focused fixtures under `fixtures/diagnostics/session_aq_*`; and
- proportional `tools/check_all.ps1` assertions.

No other compiler stage, docs/schema, decision, Work Order, Cargo, editor,
example, adapter, grant, target, experiment, or research file is authorized.

### AQ composition and runtime rules

- Top-level collection accepts one canonical occurrence from its owning
  analyzer/stage.
- Recomputing app/capability applicability may select a different canonical
  analysis scope, but may not delete/recreate diagnostics by hard-coded code
  membership.
- Runtime preflight consumes the same static/shared cause occurrence for
  parser, resolver, callable, predicate, authority, Path, typed-failure,
  ownership, and resource blockers.
- Runtime-execution-owned H0702/H0703 and true runtime invariant/trap paths keep
  their current distinct channels.
- A duplicate occurrence, owner mismatch, or prior-blocker replacement is an
  internal invariant failure. It is never silently deduplicated or rendered as
  a second user diagnostic.
- Multiple real occurrences sharing one code remain visible in deterministic
  semantic/source order.
- Collection order changes may not alter cause identity, precedence, or public
  output ordering.

### AQ permanent evidence and repository audit

Add permanent evidence proving:

1. one shared static/runtime typed-failure cause and one shared ownership cause
   are emitted once and consumed by runtime preflight;
2. duplicate injection from static and runtime paths fails closed;
3. two real same-code occurrences remain two diagnostics;
4. app/capability recomputation preserves occurrence identity and exact route;
5. blocked authority, Path, predicate, callable, typed-failure, ownership, and
   resource paths call zero adapters and preserve existing exits;
6. runtime contract violations remain runtime-owned and do not collide with
   preflight causes;
7. different collector insertion orders normalize identically and two fresh
   runs are byte-identical;
8. deleting or restoring a superseded hard-coded classifier changes a sabotage
   result and is caught;
9. no production Rust source outside `src/diagnostic_catalog.rs` contains a raw
   exact H-code allocation or an unregistered cause-identity literal; and
10. every active code, registered cause, owner, stage, and precedence rule has
    at least one validation path.

Required new fixtures/tests include:

- `fixtures/diagnostics/session_aq_static_runtime_shared_cause_fail.hum`;
- `fixtures/diagnostics/session_aq_same_code_distinct_occurrences_fail.hum`;
- `fixtures/diagnostics/session_aq_app_scope_reanalysis_fail.hum`; and
- an in-test collector-order/duplicate-emitter mutation matrix.

The source audit may allow raw H-code strings only in the canonical registry,
checked documentation projections, test expectations, and fixture snapshots.
It must reject allocations or production classifiers elsewhere. Test strings
are evidence, not authority.

### AQ human/JSON/runtime compatibility

- Existing public commands and schemas are unchanged.
- All active diagnostic output remains exact in code, title, severity, message,
  help, spans, count, ordering, and exit behavior.
- Runtime typed failures remain exit 1; source/preflight misuse retains current
  exit 1 or 2 ownership; invariant traps remain a separate compiler-bug
  channel.
- No adapter is called on a blocked path.
- Graph/Core/status outputs preserve current facts and do not expose internal
  registry metadata.

### AQ bans

No new diagnostic, public registry command, public cause field, schema, report,
stage, runtime operation, handler, recovery, IO, authority, ownership,
resource, callable, row, standard-library, migration, GitHub mutation, or
decision ruling.

No message cleanup, severity adjustment, help rewrite, span improvement, or
precedence correction may ride along. Record such findings for a later
explicitly authorized session.

### AQ acceptance criteria and hard stop

- One registry owns allocation and cause identity repository-wide.
- One fundamental occurrence has one owner and one emitted diagnostic.
- Downstream stages and runtime preserve exact internal blocker references.
- Every superseded code/line/message classifier is deleted or proven
  nonauthoritative by sabotage.
- All public outputs and accepted semantics remain unchanged.
- Complete negative mutation, fixture, and projection matrices pass.
- All standing checks and Ubuntu/Windows CI pass after separate authorization.
- Stop. Work Order 9 ends. No issue closure, stdlib work, Session AR, or later
  implementation is authorized by AQ completion.

## Cross-session evidence matrix

Every session must independently run its affected positive and misuse evidence
through every applicable existing surface:

- `hum check` human and JSON;
- `hum resolve` human and JSON;
- `hum type-env` human and JSON;
- `hum type-check` human and JSON;
- `hum full-type-check` human and JSON;
- `hum effect-check` human and JSON;
- `hum ownership-check` human and JSON;
- `hum resource-check` human and JSON;
- `hum profile-check` human and JSON;
- `hum core-preview`, `core-lower`, and `core-verify` human and JSON;
- `hum graph` using the existing surface;
- `hum run` where runtime/preflight is applicable; and
- `hum diagnostics` and `hum explain` human and JSON.

A stage may expose its existing blocked status instead of the originating
diagnostic, but its internal blocker reference must point to the exact
occurrence. Public JSON is not extended to prove that fact; focused Rust tests
inspect it.

Every session must verify:

- exact active-code count and deterministic registry order;
- no new or renumbered code;
- exact public diagnostic compatibility;
- same-code distinct-occurrence preservation;
- duplicate-cause and replacement rejection;
- combined-cause precedence;
- zero adapters on blocked paths;
- no generic trap for source misuse;
- repeat-stable IDs and outputs; and
- absence of unrelated work.

## Standing commands and configuration coverage

Before every implementation handoff and independent review, run targeted
fixtures/tests and then:

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
git diff --check
.\tools\check_all.ps1
```

The implementer and reviewer must enumerate all affected configurations:

- ordinary Windows production build;
- unit and fixture tests;
- doctests if present;
- all-target warnings-denied Clippy;
- the existing effect-bakeoff and Windows-locality dependency checks run by
  preflight;
- text, public, and release readiness; and
- Ubuntu and Windows CI after an authorized push.

No new `cfg`, feature, optional dependency, build dependency, generated source,
target-specific branch, or platform API is authorized. If a non-host target is
not installed locally, state the gap and inspect any affected platform-neutral
code manually. Do not download a target or modify global Git configuration.

## Decision gate

No new decision record is required for:

- centralizing existing allocations;
- correcting stale aspirational examples to match accepted active codes;
- adding internal cause/occurrence/stage identity;
- preserving exact prior blockers internally; or
- deleting redundant classifiers while public behavior remains identical.

Stop and request an explicit architecture/decision gate before:

- renumbering, retiring, or materially redefining an active code;
- changing an accepted semantic owner or fundamental emitting stage;
- changing which cause wins a user-visible precedence case;
- changing severity, message, help, primary/related blame, count, ordering, or
  exit behavior;
- allocating a new family or exact code; or
- exposing registry/cause identity through a public command, report, schema, or
  JSON field.

The decision gate is not permission to continue the current session. Preserve
the worktree and wait for a separately reviewed ruling/amendment.

## Explicit exclusions and honesty locks

Work Order 9 implements no language feature. In particular, it adds or implies
none of:

- new syntax, type, effect, ownership, borrow, resource, profile, capability,
  path, contract, failure, callable, handler, recovery, or runtime semantics;
- proof that diagnostics are complete, minimal, optimal, or globally
  principal;
- proof that every compiler bug is converted into a diagnostic;
- automatic issue closure or future allocation authority;
- public stability beyond existing pre-alpha commitments;
- a generated documentation system or public registry API;
- source-diagnostic accumulation as a first-class Hum value;
- standard-library diagnostics, line maps, Bytes, Text, compiler support,
  filesystem APIs, or self-hosting machinery; or
- Session AR or another Work Order.

The 2026-07-12 stdlib research distinguishes operational typed failure, source
diagnostics as data, and compiler invariant failure. Work Order 9 preserves
that distinction. It centralizes compiler diagnostic identity; it does not turn
typed operational failure or invariant traps into source diagnostics.

## Completion and Issue #1 disposition

GitHub Issue #1 remains open throughout AN-AQ. Its implementation sessions do
not comment on, narrow, label, or close it.

After AQ is independently accepted, committed, published, green on Ubuntu and
Windows, and recorded in this Work Order, the BDFL may separately authorize a
read-only closure audit and then separately authorize a GitHub comment/close
operation. The issue closes only when evidence proves:

- one allocation registry;
- checked documentation projections;
- no active overlap or out-of-family code;
- append-only retirement protection;
- registered cause and owner identity;
- exact prior-blocker propagation;
- duplicate/replacement failure evidence; and
- no public or semantic scope expansion.

## Independent pre-issuance review requirements

The fresh reviewer must cold-start from repository ground truth and review the
complete proposed Work Order 9. The author is disqualified from issuing or
advocating the verdict.

The review must verify:

- clean synchronized baseline `047ad02` and Work Order 8 closure;
- Issue #1's exact current body and non-authoritative status;
- the 87-code baseline and every conflicting allocation document;
- exact inclusive family intervals, especially H060, H100/H110, H120, and
  H140;
- that recording H140 preserves, rather than reallocates, H1401-H1402;
- the distinction between code, cause, occurrence, owner, stage, span, route,
  and precedence;
- that a code may have multiple causes and multiple occurrences without
  permitting duplicate emission;
- AN-AQ session sizing, file envelopes, tests, compatibility locks, and hard
  stops;
- decisions 0014-0018 and resolver/evidence-native ordering;
- absence of a new public command/schema/report/pipeline gate;
- the explicit decision gate for any semantic change;
- preservation of the three error channels;
- diagnostic work as the first mandatory Work Order 9 sequence;
- continued deferral of stdlib and unrelated adoption work; and
- that only `WORKORDER.md` changed.

The reviewer must run:

```powershell
git diff --check
.\tools\check_all.ps1
```

It reports P0/P1/P2 findings with exact lines and exactly one verdict: ACCEPT,
ACCEPT WITH REQUIRED FIX, or REJECT. It must not edit, commit, push, accept a
decision, issue Work Order 9, authorize Session AN, or begin implementation.

## Current authorization gate

Work Order 8 remains closed at `047ad02`. GitHub Issue #1 is confirmed and open.
Work Order 9 was independently accepted, accepted by the BDFL, committed as
`45796dd688f9f28bb0c3290e8029e33ee2d20802`, and published by successful
workflow `29212987679`: Ubuntu job `86703742601` succeeded in 1m 28s and
Windows job `86703742589` succeeded in 2m 30s. Work Order 9 is issued.

Session AN is accepted and committed as
`bea73fcf3dd82abcf25633d33d0b152667566612`. Workflow `29215676504` passed:
Ubuntu job `86710967945` succeeded in 1m 32s and Windows job `86710967915`
succeeded in 2m 42s.

Session AO is accepted and committed as
`d750a57ed5168d0d00375972aacc148a5d37e63a`. Workflow `29219105868` passed:
Ubuntu job `86720630963` succeeded in 1m 58s and Windows job `86720630971`
succeeded in 3m 16s.

Session AP was authorized and implemented on base
`22b5e1e23bb1d9c3e137bb4b5e4ed6e9eba521a7`, independently rejected, and
corrected under the reviewed amendments below. The corrective amendment was
independently accepted and BDFL-accepted,
committed as `9aedcb0ba6893d51a2cd1b2e519d332d3cc5e6f4`, and published by successful
workflow `29225093549`: Ubuntu job `86737472812` succeeded in 1m 40s and Windows
job `86737472893` succeeded in 3m 04s. The bounded typed-failure scope amendment
was independently accepted and BDFL-accepted, committed as
`407c8065e341319b6f260b33418cd9c6b8e80a83`, and published by successful
workflow `29236756896`, attempt 1: Ubuntu job `86773108930` succeeded in 1m 35s
and Windows job `86773108925` succeeded in 4m 49s. The bounded writable-alias
scope amendment was independently accepted and BDFL-accepted, committed as
`c56f5f06e908f0ff4e38707d3f8d4ede849b1d3d`, and published by successful
workflow `29280356264`, attempt 1: Ubuntu job `86919744432` succeeded in 1m 49s
and Windows job `86919744403` succeeded in 2m 53s. The completed Session AP
correction was independently accepted and committed as
`58ad265bd3d9e974f1d53c2accceb50175edc2d7`. Workflow `29300894802` passed:
Ubuntu job `86984248993` succeeded in 6m 26s and Windows job `86984249019`
succeeded in 9m 21s. Session AP closure was recorded as
`aa69cf4ee3813883e3b01ef195ac81a40080898d`. Workflow `29301747997`, attempt 1,
passed for that exact commit: Ubuntu job `86986758575` succeeded in 6m 25s and
Windows job `86986758577` succeeded in 11m 21s.

The planning pass recorded in "Pre-AQ CI evidence-efficiency increment: exact
status-only fast lane" was independently accepted and BDFL-accepted, committed
as `04ea7435f5e12d0bbc82befe4384227a79d1ccd3`, and published by successful
workflow `29305696822`, attempt 1. Ubuntu job `86998454703` succeeded in 6m 14s,
including 5m 47s in `Run Hum preflight`; Windows job `86998454696` succeeded in
9m 34s, including 8m 50s in `Run Hum preflight`. This publication run is
evidence for the reviewed document only. It used the existing full workflow
and is not the future operational fast-lane trust anchor. CI implementation
remains unauthorized pending a separate BDFL go signal.

The GitHub Actions workflow-path correction was independently accepted and
BDFL-accepted, committed as `0f15354e5fa4028bcc37680505326321bf147352`, and
published by successful workflow `29309391970`. Ubuntu job `87009717248`
succeeded in 5m 42s, including 5m 11s in `Run Hum preflight`; Windows job
`87009717246` succeeded in 10m 40s, including 9m 30s in `Run Hum preflight`.
This correction publication run used the existing full workflow and is
document/API-shape evidence only. It is not the future operational fast-lane
trust anchor.

The pre-AQ CI evidence-efficiency increment was independently accepted and
committed/published as `7630befb97055fb099b54e56c186fe3bc202f67d`.
Required full CI passed in workflow `29318848522`, attempt 1. Ubuntu job
`87039029980` succeeded in 6m 32s, including 6m 07s in `Run Hum preflight`;
Windows job `87039030001` succeeded in 13m 29s, including 12m 40s in `Run Hum
preflight`. Both platforms selected `mode=full` with
`reason=no_status_transition`. This exact commit and run are the first
operational full-CI trust anchor.

The first successful production fast-lane publication was committed and
published as `5da709b7456a1b71fb6ebe8c859224da71e7ce05`. Workflow
`29321095009`, attempt 1, succeeded for that exact commit. Ubuntu job
`87046343669` succeeded in 1m 07s, including 54s in
`Run status-only evidence`; Windows job `87046343749` succeeded in 3m 29s,
including 3m 04s in `Run status-only evidence`. Both platforms selected
`mode=fast` with `reason=eligible_status_chain`, used full-CI anchor
`7630befb97055fb099b54e56c186fe3bc202f67d` and workflow `29318848522`,
attempt 1, and traced the exact transition
`7630befb97055fb099b54e56c186fe3bc202f67d` ->
`5da709b7456a1b71fb6ebe8c859224da71e7ce05`. Both succeeded in
`Run status-only evidence` and skipped Cargo caching, Rust toolchain
preparation, and `Run Hum preflight`. This publication proved the first fast
transition.

The required consecutive fast-lane evidence record was committed and
published as `b9658e26bbc0021a4ca8701bcaa317e0e2b3ee19`. Workflow
`29352133751`, attempt 1, succeeded for that exact commit. Ubuntu job
`87150713653` succeeded in 1m 11s, including 57s in
`Run status-only evidence`; Windows job `87150713741` succeeded in 3m 21s,
including 2m 57s in `Run status-only evidence`. Both platforms selected
`mode=fast` with `reason=eligible_status_chain`, used the same full-CI anchor
and workflow, and proved the ordered transitions
`7630befb97055fb099b54e56c186fe3bc202f67d` ->
`5da709b7456a1b71fb6ebe8c859224da71e7ce05` ->
`b9658e26bbc0021a4ca8701bcaa317e0e2b3ee19`. Both succeeded in
`Run status-only evidence` and skipped Cargo caching, Rust toolchain
preparation, and `Run Hum preflight`. The two successful production fast-lane
publications durably close the pre-AQ CI evidence-efficiency increment.

The permanent operating-policy amendment for exact routine Work Order status-
only closures was independently accepted, committed, and published as
`65ad0d5f5db2bcd90fdbb8a8afc1208f3d46b659`. Required full CI passed in
workflow `29357247935`, attempt 1. Ubuntu job `87167872491` on
`ubuntu-latest` succeeded in 6m 38s: Cargo caching and Rust toolchain
preparation succeeded, `Run Hum preflight` succeeded in 6m 09s, and
`Run status-only evidence` was skipped. Windows job `87167872510` on
`windows-latest` succeeded in 12m 43s: Cargo caching and Rust toolchain
preparation succeeded, `Run Hum preflight` succeeded in 11m 52s, and
`Run status-only evidence` was skipped. Both platforms reported `mode=full`
with `reason=no_status_transition`. The amendment is durably complete. Future
eligible routine status-only closures use the permanent `AGENTS.md` exception
and its proportional local evidence set.

The BDFL authorized only the planning pass for the proposed
`PRE-AQ-INTEGRITY` permanent adversarial-evidence amendment. The proposed bytes
were independently rejected once because the nine-build plan could not fit the
unchanged 15-minute job limit. The BDFL authorized only the bounded correction
that permits the later implementation to change `timeout-minutes: 15` to
`timeout-minutes: 30` as the sole workflow-byte change. The corrected planning
bytes are uncommitted and await fresh independent pre-issuance re-review. The
preceding routine status closure remains
`80a0b93e81233f6a836449a04aea2f88707c9440`;
workflow `29363822737`, attempt 1, passed with Ubuntu job `87190299329` in
1m 15s and Windows job `87190299278` in 3m 38s. Both used `mode=fast`,
`reason=eligible_status_chain`, and full-CI anchor
`65ad0d5f5db2bcd90fdbb8a8afc1208f3d46b659`, and both skipped Cargo,
toolchain, and full-preflight work while succeeding in status-only evidence.

Integrity implementation remains unauthorized pending independent acceptance,
BDFL acceptance, scoped commit and publication, a durable status record, and a
separate BDFL implementation go signal. Session AQ, GitHub Issue #1 mutation,
repair work, dashboard work, and all later work remain unauthorized. No
integrity mechanism, emitter migration, precedence change, diagnostic
allocation, decision ruling, commit, push, or scope expansion is implicitly
authorized.

<!-- workorder-current-authorization-gate:end -->

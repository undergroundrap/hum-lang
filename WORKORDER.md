# Hum Work Order 9: Canonical Diagnostic Allocation And Cause Identity

Date: 2026-07-12
Status: issued by the BDFL. The independently accepted and BDFL-accepted bytes
are commit `45796dd688f9f28bb0c3290e8029e33ee2d20802`, published by successful
workflow `29212987679` (Ubuntu job `86703742601`, 1m 28s; Windows job
`86703742589`, 2m 30s). Historical proposal and pre-issuance language below is
preserved as reviewed issuance history. Session AN is accepted and committed
as `bea73fcf3dd82abcf25633d33d0b152667566612`; workflow `29215676504`
passed (Ubuntu job `86710967945`, 1m 32s; Windows job `86710967915`, 2m
42s). Session AO is next but remains unauthorized pending a separate BDFL go
signal.
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

Work Order 9 contains exactly four implementation sessions:

```text
AN  canonical family/code/status registry and checked documentation projections
AO  cause/occurrence identity for typed failure and callable vertical slices
AP  remaining static compiler emitters and exact prior-blocker propagation
AQ  runtime/top-level composition closure and repository-wide audit
STOP  close Issue #1 only after separate authorization; author the next work order separately
```

AN is mandatory first. No later Work Order 9 session and no unrelated language,
stdlib, IO, ownership, effect, or adoption work may begin before AN is
independently accepted, committed, published, green on Ubuntu and Windows, and
recorded in this file. Each following session requires the same cycle plus a
separate BDFL go signal.

The compiler-ready stdlib remains the next adoption direction supported by the
2026-07-12 research triage. It is intentionally not part of Work Order 9.
Diagnostic centralization goes first because new compiler/stdlib work would
otherwise allocate and propagate causes through a contradictory namespace.

Session sizing is semantic, not just a path count. AN changes allocation truth
only. AO proves two vertical slices. AP has the widest file envelope, but every
permitted change is the same carrier migration: consume, validate, or preserve
one exact prior-cause set without changing stage logic or public output. If any
AP stage needs a new diagnostic rule, precedence rule, schema field, or source
behavior, AP is no longer review-sized and must stop for a separately reviewed
Work Order amendment. AQ changes only composition/runtime ownership after the
static migration is accepted.

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

### AP exact integration map

Authorized source files are:

- `src/parser.rs`;
- `src/check.rs`;
- `src/resolve.rs`;
- `src/type_env.rs`;
- `src/type_check.rs`;
- `src/full_type_check.rs` and `src/effect_check.rs` only to generalize AO's
  exact blocker carrier to registered AP causes;
- `src/app_entry.rs`;
- `src/capability_root.rs`;
- `src/path_boundary.rs`;
- `src/predicate.rs`;
- `src/ownership_check.rs`;
- `src/resource_check.rs`;
- `src/profile_check.rs`;
- `src/core_preview.rs`, `src/core_lower.rs`, and `src/core_verify.rs` only to
  consume, validate, or preserve the exact prior cause set without adding a
  public field;
- `src/graph.rs` only to consume the same cause set and preserve existing graph
  output;
- `src/ir_readiness.rs` only to validate the exact prior cause set behind its
  existing readiness status;
- `src/diagnostic_catalog.rs` only for registered cause/precedence entries;
- `src/diagnostic.rs` only for shared carrier validation;
- focused fixtures under `fixtures/diagnostics/session_ap_*`; and
- proportional `tools/check_all.ps1` assertions.

`src/main.rs`, `src/run.rs`, `src/json.rs`, and `src/diagnostics.rs` remain
outside AP; AQ owns final composition, runtime, and public compatibility
closure. No Core/graph schema, documentation, decision, Cargo, editor, example,
or unrelated fixture change is authorized.

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

### AP acceptance criteria and hard stop

- Every static emitter in scope uses registered cause identity.
- Every downstream static blocker keeps the exact origin occurrence internally.
- Existing precedence and public output are unchanged.
- All field and combined-cause mutations fail closed.
- Existing H010/H060/H070/H080/H120 and authority/path matrices remain green.
- All session and standing checks pass.
- Stop. Session AQ remains unauthorized pending the complete review/commit/
  publication/CI/status cycle and a separate BDFL go signal.

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

Session AO is next but remains unauthorized pending a separate BDFL go signal.
Sessions AP-AQ and all later work remain unauthorized. No cause-carrier
migration, emitter migration, precedence change, diagnostic allocation,
decision ruling, implementation session, commit, push, GitHub Issue #1
mutation, or scope expansion is implicitly authorized by this status update.

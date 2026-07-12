# Hum Work Order 8: Canonical Callable Semantic Spine

Date: 2026-07-12
Status: issued by the BDFL. The independently accepted bytes are commit
`956b51f`, published by successful workflow `29183983775` (Ubuntu job
`86626699948`; Windows job `86626699928`). No implementation session is
authorized. A separate BDFL go signal remains required before Session AL.
Owner: BDFL (Ocean). Work-order author: architect-reviewer acting under the
bounded Work Order 8 planning authorization. Independent pre-issuance reviewer:
a fresh architect-reviewer that did not author or edit this deliverable.
Future implementer: agent sessions only after every applicable gate.
Predecessor: Work Order 7, Sessions AF-AK, closed and committed at `e7dbadb`.
Session AK and accepted decision 0018 are commit `1b324fb`. The closed Work
Order 7 text remains authoritative in git history and is not reopened here.

## Authority, evidence, and issuance gate

This document records a BDFL-directed architecture plan. It does not transfer
governance authority, accept or amend a decision, or authorize implementation.
The predecessor evidence is:

- Work Order 7 closure: `e7dbadb`;
- Session AK plus accepted decision 0018: `1b324fb`;
- successful workflow: `29178915990`;
- successful Ubuntu job: `86613010219`; and
- successful Windows job: `86613010224`.

The BDFL ruling, author incorporation, independent `ACCEPT`, BDFL acceptance,
scoped commit, publication, and CI gates are complete. The durable issuance
evidence is:

- issued Work Order 8 commit: `956b51f`;
- successful workflow: `29183983775`;
- successful Ubuntu job: `86626699948`; and
- successful Windows job: `86626699928`.

The only remaining gate before implementation is a separate BDFL go signal for
Session AL.

Any authoritative source-form, scope, diagnostic, fixture, or gate mutation
after the independent `ACCEPT` invalidates that verdict and requires fresh
independent review of the complete changed document. A BDFL ruling has
authority, but it is not
silently treated as already reviewed. An exact scoped commit of the reviewed
bytes does not change the deliverable and does not require a second reviewer.

Acceptance or commitment of this document alone does not authorize Session AL.
Each accepted session still requires its own scoped commit, successful CI,
independent review, recorded handoff, and a separate BDFL go signal before the
next session. No review, commit, push, CI result, or decision status silently
authorizes a following session.

Research is input, not authority. The accepted Session AG corpus, the accepted
Session AH-AJ prototypes, the Session AK scorecard, and decisions 0014-0018
control whenever research prose disagrees with repository evidence. In
particular, none of these research snapshots creates production syntax or
implementation credit:

- `docs/research/2026-07-10-overnight-research-triage.md`;
- `docs/research/deep/2026-07-10-effect-polymorphism-corpus.txt`; and
- `docs/research/deep/2026-07-10-flagship-wedge.txt`.

## Accepted-decision locks

Every Session AL-AM implementation and review must preserve these locks.

### Decision 0014: ownership

- `borrow`, `change`, and `consume` remain distinct permissions.
- Aliasing, move, mutation, resource, and lifetime safety are not implied by an
  effect row or callable type.
- The first slice accepts no permission-bearing callable parameter, callable
  argument, callable result, or indirect call argument.
- No copying, owner table, monomorphic substitution, hidden box, or runtime
  reference may stand in for ownership proof.
- Internal references remain deferred and receive no ownership credit here.

### Decision 0015: executable contracts

- Every recognized executable predicate continues to run.
- Callable analysis adds no proof/trust classifier, enforcement profile,
  predicate elision, build-mode distinction, global contract toggle, or
  unreachable-guard conclusion.
- Callable fixtures may contain ordinary checked predicates, but callable
  support does not alter Predicate v2 recognition or H0701-H0704 ownership.

### Decision 0016: nominal causal typed failure

- Nominal roots and variants, explicit `try`, explicit wrapping, causal origin,
  propagation, and wrapping sites remain exact.
- No implicit fallible call, erased any-error propagation, exception, unwind,
  catch, recovery, or ambient backtrace is introduced.
- The canonical callable representation reserves an exact optional nominal
  failure-root field. Session AL accepts only `none`; it does not make indirect
  fallible calls legal. Later work may use that field only under a separately
  reviewed slice that preserves H0901-H0907.

### Decision 0017: authority

- Source authority, operator consent, and exercised operation remain separate.
- A latent row or captured authority never grants authority, manufactures
  consent, expands an app maximum, or proves that an adapter ran.
- Existing exact authorities, deny-wins behavior, app/task/caller closure,
  forensic route identity, and H0617-H0633/H1204 precedence remain unchanged.
- Sessions AL-AM contain no authority-bearing callable or indirect authority-
  bearing operation. A future row slice may represent an exact latent authority
  requirement only as evidence and under a new accepted work order; this order
  does not modify grant policy.

### Decision 0018: selected effect model

- First-class callable types carry an open multiset row of exact latent labels
  and at most one structural tail variable.
- Application propagates the latent row. Duplicate labels remain duplicates;
  aliases and stable identities are preserved.
- Exact one-occurrence handling is type-level evidence only and remains outside
  Work Order 8.
- Rows do not prove captures, ownership, resource lifetime, allocation,
  authority, consent, or operation exercise.
- Stored and returned callables require explicit environment and companion
  facts. Work Order 8 does not claim those facts are implemented.
- Formula and capture prototype mechanisms remain research evidence, not
  parallel production effect cores.

## Incorporated BDFL source-form ruling

The BDFL approved this exact bounded source-form ruling on 2026-07-12. It
selects only the AL-AM spellings and boundaries below. It does not issue this
Work Order, authorize Session AL, select later callable syntax, or amend
decision 0018.

### Complete AL positive program

The exact UTF-8 source is:

```hum
module examples.probes.passed_pure_callable

task increment(value: UInt) -> UInt {
  why:
    provide the first pure named task value

  ensures:
    result == value + 1

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    return value + 1
}

task double(value: UInt) -> UInt {
  why:
    provide an independent sabotage callable

  ensures:
    result == value * 2

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    return value * 2
}

task apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {
  why:
    apply exactly one passed pure callable

  needs:
    value < 1000000

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    return transform(value)
}

task run_passed_callable -> UInt {
  why:
    observe the passed callable result through runtime execution

  ensures:
    result == 42

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    return apply_once(increment, 41)
}
```

The positive invocation is:

```powershell
hum run examples/probes/passed_pure_callable.hum --entry run_passed_callable
```

The semantic result is `UInt 42`; stdout is UTF-8 bytes `34 32 0A`; exit status
is zero. No H1401-H1402 diagnostic is permitted.

The receiver has no honest general postcondition expressible in Predicate v2
because its result depends on an opaque callable. Positive evidence therefore
requires zero errors and zero H1401-H1402 diagnostics while pinning any existing
nonblocking H0107/H0109 warning. No tautological or filler contract may be added
to manufacture a warning-free fixture.

### Callable type and parameter

The exact callable type spelling is:

```hum
task(UInt) -> UInt
```

The grammar is:

```text
hws0          ::= *(SP / HTAB)
hws1          ::= 1*(SP / HTAB)
ordinary-type ::= type-ident
callable-type ::= "task" "(" hws0 ordinary-type hws0 ")" hws1 "->" hws1 ordinary-type
callable-param ::= value-ident ":" hws1 callable-type
```

For AL-AM, the only accepted ordinary type on both sides is `UInt`. There is no
whitespace between `task` and `(`. Horizontal whitespace never consumes a
newline. Comments are not accepted inside `callable-type`, and callable types
cannot nest.

The source spelling is row-elided; it does not assert a Boolean `pure`
property. AL accepts it only when the resolved task value's inferred latent row
is exactly closed and empty. AM permits one internal file-local structural tail
during analysis. No public, recursive, stored, or returned generalization
follows from the omitted row.

The exact callable parameter is:

```hum
transform: task(UInt) -> UInt
```

The parameter name is `transform`. At least one SP/HTAB is mandatory after the
colon. No permission keyword, newline, or comment is permitted inside the
parameter. The receiver has exactly one callable parameter and exactly one
ordinary `UInt` parameter. Its canonical header is:

```hum
task apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {
```

The comma is followed by `hws1`.

### Direct task value and indirect application

The exact task-value occurrence is `increment` in:

```hum
apply_once(increment, 41)
```

Its grammar is:

```text
task-value ::= value-ident
```

Only an unqualified identifier resolving to a task definition in the same file
and permitted lexical task/app subtree is a task value. Dotted names, fields,
module paths, locals, parameters, literals, parenthesized expressions,
conditionals, call results, and anonymous forms are not task values.

Normal lexical resolution occurs first. A same-named local or parameter wins
and must not fall back to the hidden task. Its task-value use then receives
H1401 with the shadowing definition as related evidence.

The exact indirect application is `transform(value)` in:

```hum
return transform(value)
```

Its grammar is:

```text
indirect-application ::= callable-param-ident "(" hws0 ordinary-value-ident hws0 ")"
```

The callee resolves to the one callable parameter. The argument resolves to the
receiver's one ordinary `UInt` parameter. No task call, indirect call,
arithmetic expression, permission wrapper, field, element, or literal is
accepted as the indirect argument in AL. The application is the complete return
expression; AL permits no intermediate binding. Exactly one indirect
application is required. Chaining and nesting are unsupported.

### Closed lexical envelope

- `hws0` is zero or more ASCII SP/HTAB; `hws1` is one or more.
- Horizontal whitespace never includes CR or LF.
- The accepted type is `"task" "(" hws0 "UInt" hws0 ")" hws1 "->" hws1 "UInt"`.
- A task value is one `value-ident` with identifier boundaries on both sides.
- The accepted application is `value-ident "(" hws0 value-ident hws0 ")"`.
- The task value occurs only as the first argument of the resolved receiving-
  task call; the second argument is the ordinary `UInt` value.
- Value identifiers remain `[a-z_][a-z0-9_]*`; type identifiers remain
  `[A-Z][A-Za-z0-9]*` under decision 0012.
- Comments may occur before or after complete lines, never inside a callable
  type, callable parameter, task value, or indirect application.
- Callable-type and indirect-application nesting are zero.
- One receiver has at most one task-value argument, one callable parameter, and
  one indirect application.

### Name and scope ownership

- An unresolved task value is H0601 at the identifier, with help to declare or
  name a visible same-file task.
- Duplicate task definitions remain H0602 with both spans.
- A local/parameter shadow wins lexical resolution; its task-value use is H1401
  with the non-task definition as related evidence.
- Any other non-task task-value is H1401 at the value use with its resolved
  definition related.
- A local shadowing the callable parameter is H0602 with both definitions.
- Application outside the callable parameter's scope is H0601.
- A cross-file task value or callable relationship is H1401 at the task-value or
  receiver call, with the external definition related when available.

### Malformed and unsupported corpus

H1401 is `invalid or unsupported callable form`, with reason
`invalid_or_unsupported_callable_form_v0`. It owns canonical-looking malformed
callable types/applications and recognized forms outside the slice. Existing
parser diagnostics retain malformed outer item/header ownership.

| Case | Exact source | Owner |
| --- | --- | --- |
| missing outer task-parameter close | `task apply_once(transform: task(UInt) -> UInt, value: UInt -> UInt {` | H0007 |
| missing indirect close | `return transform(value` | H1401 |
| extra close | `return transform(value))` | H1401 |
| mismatched delimiters | `return transform[value)` | H1401 |
| zero indirect arguments | `return transform()` | H1402 |
| two indirect arguments | `return transform(value, value)` | H1402 |
| trailing prose | `return transform(value) later` | H1401 |
| chained application | `return transform(value)(value)` | H1401 |
| nested application | `return transform(increment(value))` | H1401 |
| anonymous callable | `return apply_once(task(value) { return value + 1 }, 41)` | H1401 |
| stored callable | `let saved = transform` | H1401 |
| returned callable | `return transform` | H1401 |
| permission-bearing callable type | `transform: task(change UInt) -> UInt` | H1401 |
| permission-bearing argument | `return transform(change value)` | H1401 |
| known fallible task passed as pure | `return apply_once(fallible_increment, 41)` | H1402 |
| zero application | `return value` | H1401 |
| second application | `return transform(first)` after `let first = transform(value)` | H1401 |

For the known fallible task, H1402 reports expected `failure_root = none` and
the exact actual nominal root, with the task definition related. Zero
application uses reason `required_exactly_one_callable_application_v0` at the
callable parameter/receiver relationship. A second application blames the
second occurrence and relates the first without a duplicate competing error.

Unknown ordinary types remain H0605; invalid identifiers remain H0009;
malformed outer task headers retain H0003/H0006/H0007/H0008; wrong resolved
callable input, result, or failure root is H1402.

### AL observation and sabotage

AL input and result are `UInt`.

| Form | Expected result |
| --- | ---: |
| `apply_once(increment, 41)` | 42 |
| `apply_once(increment, 40)` | 41 |
| `apply_once(double, 41)` | 82 |

Permanent evidence must fail if `apply_once` ignores either input, directly
calls `increment`, returns a constant, resolves by display name, or accepts
poisoned expected-result metadata as semantic input.

### AM row presentation and nonempty observation

There is no AM row source syntax. The callable type remains
`task(UInt) -> UInt`, and its latent row is inferred. AL accepts only an
inferred closed-empty result. AM may generate one internal structural tail for
the single file-local nonrecursive callable-parameter relationship. The tail is
semantic evidence, not source syntax or a public-signature rule. No user-written
row variable/alias, effect annotation, handler annotation, effect exclusion,
formula, or associated effect is introduced.

AM uses the existing non-authority-bearing Core `change` effect. Its exact
positive source is:

```hum
module examples.probes.passed_callable_row

task bump_with_local(value: UInt) -> UInt {
  why:
    expose an existing non-authority-bearing local mutation effect

  ensures:
    result == value + 1

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    change current: UInt = value
    set current = current + 1
    return current
}

task apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {
  why:
    propagate one inferred callable row

  needs:
    value < 1000000

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    return transform(value)
}

task run_passed_callable_row -> UInt {
  why:
    observe the effectful callable result without external authority

  ensures:
    result == 42

  cost:
    time: O(1)
    space: O(1)
    check: warn

  does:
    return apply_once(bump_with_local, 41)
}
```

Runtime returns 42, writes UTF-8 bytes `34 32 0A`, exits zero, and calls no
adapter. Source authority, operator consent, and exercised external operation
are all absent.

The expected row has distinct `change` occurrences at the mutable declaration
and `set` origins. Application substitutes the complete multiset for the one
internal tail and propagates every occurrence without deduplication. If existing
canonical Core classification proves the mutable declaration permission-only,
the row contains only the `set` occurrence; the implementation may not invent a
duplicate. A separate two-`set` fixture then proves duplicate-label behavior.

### Visibility and generalization boundary

AL-AM accept only one nonrecursive same-file callable-parameter relationship
and one indirect application.

- Cross-file callable use is H1401 at the use with receiving-task/call-site
  evidence; there is no public row inference.
- A recursive relationship is H1401 at the recursive edge with caller, callee,
  and route spans; there is no recursive generalization or monomorphic fallback.
- Stored callable use is H1401 with no environment allocation or retained-
  capture fact.
- Returned callable use is H1401 with no returned environment or lifetime fact.

A future public, recursive, stored, or returned callable boundary must expose a
stable row variable or named alias under a later accepted work order. This
ruling selects no spelling for that future boundary.

The pinned corpus reuses existing identifier, task-header, call-argument, and
body boundaries. It introduces no anonymous function, closure literal, generic
type parameter, permission-bearing callable, storage, callable return, handler,
recovery, or higher-order library API. The independent reviewer must verify the
complete incorporated ruling before any AL go signal.

## Mandatory sequence and stopping point

Work Order 8 contains only two production sessions:

```text
AL  canonical callable facts plus one passed pure callable vertical slice
AM  latent open-row propagation through that callable application
STOP  retrospective and a new work order before handling or environments
```

This is the smallest safe production sequence. AL proves that one semantic
spine can survive every existing stage before effect polymorphism is added. AM
then adds the selected row mechanism while callable identity, invocation, and
runtime behavior are already independently accepted.

The remaining accepted architectural order is mandatory but deferred:

1. exact one-occurrence type-level handling;
2. returned callable environments;
3. stored callable environments and registration lifetime;
4. immutable-value and exact-authority capture bridging;
5. owned, borrowed, and linear-resource capture/lifetime bridging; and
6. higher-order standard-library APIs last.

Handling is not included because no source spelling or recovery boundary has
been pinned. Environments are not included because representation, allocation,
cleanup, and lifetime facts deserve independent evidence after application is
stable. The capture/ownership/resource/authority bridge is split because each
domain has different existing diagnostic ownership and a row cannot launder
any of them. Library APIs would multiply unsupported shapes before the
foundation is proven. A post-AM retrospective must recommend the next bounded
work order; it may not silently start the deferred list.

## Canonical semantic spine

Sessions AL and AM use one shared, immutable, memoized production analysis in
`src/callable.rs`. It must be constructed from typed syntax nodes plus resolver
scope/definition/reference identities. A source slice may be retained for
display and blame, but no semantic branch may be selected by rescanning that
slice. Formatted AST output, debug text, rendered diagnostics, JSON, names, and
display strings are never semantic inputs. The analysis is built once per
parsed program/command after resolver identity exists and is passed to every
consumer.

The shared analysis owns these closed facts:

- `CallableDefinitionFact`: stable callable ID, source path/span, lexical scope
  ID, resolver definition ID, ordered ordinary input definitions and types,
  result type, exact optional nominal failure root, and definition status;
- `CallableTypeFact`: stable type ID, ordered ordinary input types, result type,
  exact optional nominal failure root, and latent-row ID;
- `LatentEffectRowFact`: stable row ID, exact ordered-normalized multiset of
  label identities, optional structural tail-variable identity, closed/open
  status, aliases, and source/inference origins;
- `CallableValueFact`: stable value-use ID, source span, referring lexical scope,
  resolver reference ID, resolved task definition ID, expected callable-type
  ID, and recognition/resolution/type status;
- `CallableApplicationFact`: stable application ID, caller definition ID,
  callable-parameter definition ID, callable-value/definition IDs, invocation
  span, ordered ordinary argument definition/value facts, result type, nominal
  failure root, input and output row IDs, and status/reason/blame sites; and
- `CallableDiagnosticFact`: stable diagnostic ID, H-code, fundamental reason,
  primary span, ordered related spans, and one actionable repair direction.

Stable IDs derive from normalized source identity, lexical definition identity,
and exact occurrence spans. Display spans retain original source spelling.
Paths normalize separators only for identity comparison, matching existing
forensic practice. Row normalization is deterministic, preserves duplicate
labels, preserves aliases as diagnostic evidence, and alpha-renames a single
tail by first semantic occurrence. It never concatenates tail names or treats a
set as a multiset.

The following identities are distinct closed semantic domains even when two
display names match: source-node ID, lexical-scope ID, definition ID, reference
ID, callable-definition ID, callable-type ID, callable-value/use ID, callable-
parameter ID, ordinary parameter/result type ID, nominal failure-root ID,
application-occurrence ID, row ID, row-variable ID, effect-label-occurrence ID,
alias ID, substitution ID, route ID, and diagnostic ID. Names, aliases, and
spans may explain an identity but may not replace it. Tests must construct
same-named definitions and renamed aliases/tails that would fail under name-
based identity.

Facts are authoritative only within their proven status. Parser facts do not
claim resolution; resolver facts do not claim typing; type facts do not claim
ownership, authority, allocation, Core lowering, or execution. Unsupported or
blocked facts remain visible with an honest blocker and never become a more
complete downstream fact.

The AL core identity model must survive later stored/returned callable work
without semantic replacement. A later environment is a companion fact keyed by
the accepted callable-value/use identity; it may add allocation, retained
values, authority/resources, cleanup owner, and lifetime without changing the
meaning of the callable definition, type, application, failure root, latent row,
or source relationships accepted here. AL does not predeclare empty environment
fields, enum variants, ABI promises, boxes, or extension hooks. Unsupported
storage, return, or capture is represented by the closed recognition/diagnostic
boundary and exercised H1401 fixtures, not by unused placeholder metadata.

The AL runtime task handle is a private nonescaping projection from the same
callable-value/use and resolved-definition identities. It is neither the
canonical semantic identity nor a promise that future callable environments use
the same physical representation. Replacing that private projection later is
permitted; replacing the accepted semantic identity/relationship model is not.

### Competing-recognizer retirement gate

Before code edits, the integration map must inventory every existing call,
expression, task-header, and type recognizer that could observe the pinned
callable forms, including source-string helpers in resolver, Core, ownership,
and runtime modules. For each recognizer the implementer must state exactly one
outcome:

- deleted because the canonical structured analysis supersedes it;
- narrowed and permanently tested as disjoint from callable syntax; or
- retained only as a projection consuming canonical facts, with no source
  recognition authority.

No old helper may remain as a fallback when canonical analysis is blocked.
Permanent sabotage tests must make a legacy recognizer disagree with a
canonical fact and prove every stage and runtime follow the canonical fact. The
parser is the only source recognizer. `src/callable.rs` accepts only structured
AST nodes plus resolver scope/definition/reference identities and must expose no
source-text or line-string recognition entry point. The preflight must reject a
new callable/row source scan anywhere outside `src/parser.rs`. If a recognizer
cannot be retired or made provably disjoint within the session, stop rather
than accept two semantic spines.

### Required pre-edit integration map

Before editing code in either session, the implementer must report an exact map
from the pinned source corpus to:

- syntax nodes and parser entry points in `src/ast.rs` and `src/parser.rs`;
- lexical scopes, definitions, references, and call links in
  `src/resolve.rs::build_report`, `Resolver::resolve_callable`, and
  `Resolver::resolve_statement_references`;
- canonical type references in `src/type_env.rs::build_report`;
- declaration/return checking in `src/type_check.rs::build_report`;
- statement and application checking in `src/full_type_check.rs::build_report`;
- effect propagation in `src/effect_check.rs::build_report` and
  `check_statement_effect`;
- ownership and resource blockers in `src/ownership_check.rs::build_report` and
  `src/resource_check.rs::build_report`;
- Core representation in `src/core_preview.rs::build_report`,
  `src/core_lower.rs::build_report`/`lower_expression`, and
  `src/core_verify.rs::build_report`/`verify_operation`;
- definition/use and row relations in `src/graph.rs`;
- runtime preflight and `Interpreter::execute_task`, `eval_expr`, and the
  indirect-application insertion point in `src/run.rs`; and
- human and JSON rendering in the existing command surfaces.

The map must name every new or changed structure and function, its input fact,
its output claim, and its exact blocker. For every stage it must classify the
stage action as exactly one or more of: consumes a canonical fact, validates an
invariant over it, projects it into a typed downstream representation, or
exposes an exact honest blocker. It must also include the competing-recognizer
inventory and retirement outcome. If implementation evidence shows this map
cannot stay shared and review-sized, stop for architecture review instead of
adding a second analyzer or downstream string pattern.

## Diagnostics and precedence

H100x, H110x, and H130x remain reserved by existing diagnostic doctrine for
unsafe/FFI/ABI/provenance, runtime-profile/certification, and concurrency/memory-
ordering diagnostics. Work Order 8 uses the otherwise unreserved H140x family
and introduces only two production source diagnostics:

- H1401 `invalid_or_unsupported_callable_form_v0`: a canonical-looking
  callable type/value/application is malformed or outside the authorized direct
  passed-callable slice; and
- H1402 `callable_signature_mismatch_v0`: resolved callable input, result, or
  exact failure-root identity does not match the expected callable type.

No production H-code is reserved for latent-row mismatch or ambiguous row
inference in Work Order 8. The authorized AM slice has one internal tail, one
relationship, and one application, so its structural substitution is unique.
Ambiguity is not a producible source case and is deferred with public/recursive/
stored/returned generalization.

Missing, substituted, deduplicated, erased, prematurely closed, or foreign-tail
row facts are deliberately corrupted in-test semantic/Core artifacts. They must
fail Core verification with stable verifier reason IDs and relationship sites;
they are not source diagnostics and never enter runtime preflight as H-codes.

Precedence follows the real pipeline. Existing parser diagnostics own malformed
outer headers. H1401 owns canonical callable candidates after parser
recognition, except unresolved names remain H0601 and duplicates remain H0602.
H1402 is emitted only after resolution, ordinary typing, and nominal failure-
root comparison. The effect stage consumes the canonical valid or blocked fact
and may expose the same earlier blocker; it does not wait for ownership or
resource checking. Ownership and resource stages then consume the earlier
status and retain H0801-H0809 ownership of their own independent causes.
Core/lowering never upgrades a blocked fact.

Unknown ordinary types remain H0605. Ordinary statement mismatches retain their
existing full-type diagnostic. H0901-H0907 own typed-failure causes.
H0617-H0633 and H1204 own authority causes. No post-resource collector or
pipeline inversion is introduced.

Each fundamental cause produces exactly one primary diagnostic in human and
JSON surfaces with compatible spans, facts, and repair direction. Runtime
preflight consumes the same collected blockers before task arguments, bodies,
callable environments, or adapters. An unsupported callable form never falls
through to a generic resolver/type error or runtime trap. Independent errors on
independent statements remain independently visible; precedence must not mask
them into one synthetic callable error.

## Session AL: passed pure callable vertical slice

Purpose: prove the smallest real first-class task value through one canonical
semantic spine without effects, captures, storage, or allocation.

### AL authorized language and semantics

Only the exact BDFL-pinned spellings are recognized. Their semantic envelope is:

- one named task declaration used as a callable value;
- one receiving task with exactly one callable parameter;
- the callable has exactly one ordinary, non-permission-bearing parameter and
  one ordinary result;
- the callable has no nominal failure root; its row-elided type is accepted in
  AL only when analysis infers an exact closed empty latent row;
- the caller passes the direct task name in the pinned argument position;
- the receiver invokes that parameter exactly once with one already-typed
  ordinary value; and
- the real returned value is observed through existing runtime behavior.

The positive fixture must be hand-authored Hum source, produce a nonconstant
result that depends on both the supplied callable and its ordinary argument,
and fail if the receiver ignores the callable, substitutes a direct named call,
or returns a canned value. The fixture must run twice from fresh state with
byte-identical stdout, stderr, exit status, callable IDs, call route, Core facts,
and graph relations. Human/JSON reports must contain zero errors and zero
H1401-H1402 rows; only exact pinned existing H0107/H0109 doctrine warnings are
allowed. No filler contract may silence them.

### AL module and stage scope

AL may add `src/callable.rs`, the minimum AST/parser nodes selected by the BDFL
ruling, renderer/schema fields required by existing commands, and permanent
fixtures. The shared analyzer must be consumed by:

- resolver: callable parameter/value definitions, direct task-value reference,
  and indirect application link;
- type environment: canonical callable type and exact ordinary component types;
- type/full-type: exact input/result/failure-root compatibility and application
  result type;
- effect: an explicitly inferred closed empty latent row, never a `pure` Boolean
  or a claim inferred from missing analysis;
- ownership/resource: explicit `not_applicable_to_al_ordinary_value_v0` facts,
  not ownership or allocation proof;
- Core preview/lower/verify: a callable type/value/application node with stable
  IDs and the inferred closed empty row, preserved without string reparsing;
- graph: definition, value-use, passed-as-argument, parameter-bind, and
  application edges with exact source spans;
- runtime: one nonallocating direct task handle bound to the callable parameter
  and one indirect dispatch through the resolved definition ID; and
- human/JSON: the same status, IDs, row, result, reasons, spans, and repairs.

The runtime representation is a nonescaping handle to an existing task
definition, not a heap closure or first-class runtime `Result`. AL must report
zero callable-environment allocation and no retained values/resources/authority.
That report is slice-specific evidence, not a general allocation-free claim.

The exact AL file envelope is limited to:

- syntax and analysis: `src/ast.rs`, `src/parser.rs`, `src/syntax.rs`, the new
  `src/callable.rs`, and `src/resolve.rs`;
- existing semantic consumers: `src/type_env.rs`, `src/type_check.rs`,
  `src/full_type_check.rs`, `src/effect_check.rs`, `src/ownership_check.rs`,
  `src/resource_check.rs`, `src/core_expr.rs`, `src/core_preview.rs`,
  `src/core_lower.rs`, `src/core_verify.rs`, `src/graph.rs`, and `src/run.rs`;
- existing render/catalog wiring only as necessary: `src/diagnostic.rs`,
  `src/diagnostic_catalog.rs`, `src/diagnostics.rs`, `src/json.rs`,
  `src/main.rs`, and the crate module declaration;
- existing authoritative documentation/schema files that directly describe
  those changed surfaces: `SPEC.md`, `docs/ARCHITECTURE.md`,
  `docs/DIAGNOSTICS.md`, `docs/FORMAL_CORE.md`,
  `docs/LANGUAGE_REFERENCE.md`, `docs/MILESTONE_0_GRAMMAR.md`,
  `docs/SYNTAX_SURFACE_SCHEMA.md`, the existing resolve/type/effect/ownership/
  resource/Core/graph schema documents, and the TextMate grammar; and
- new Session AL fixtures under `fixtures/`, one positive probe under
  `examples/probes/`, focused Rust unit tests beside the changed modules, and
  proportional assertions in `tools/check_all.ps1`.

No README, governance, decision, research, bake-off corpus, scorecard,
experiment, target-facts, grant, capability, Path, file, clock, output, runtime-
profile, release, or unrelated example file belongs to AL. If a required fact
cannot be represented inside this envelope, preserve the worktree and stop for
architecture review.

### AL positive and misuse evidence

Permanent evidence must include at least:

1. the nondegenerate positive described above;
2. wrong callable input type;
3. wrong callable result type;
4. a fallible task supplied where the exact failure root is `none`;
5. an unresolved task value;
6. a non-task value in the callable argument position;
7. an anonymous/nested/chained/returned/stored callable attempt;
8. a callable with zero or multiple parameters;
9. a permission-bearing callable parameter or call argument;
10. a callable invoked zero times or more than once where the pinned slice
    requires exactly one application;
11. malformed and near-miss forms from the pinned syntax corpus; and
12. a mixed fixture proving an independent body-type error is not masked;
13. two same-named task definitions in distinct lexical scopes and a renamed
    callable parameter proving semantic identity does not follow display names;
14. poisoned expected result/reason data proving the implementation follows the
    parsed/resolved relationship rather than fixture polarity;
15. a deliberately corrupted callable/Core identity, application link, type,
    or closed-row fact rejected by Core verification; and
16. a legacy-recognizer sabotage proving a source-string helper cannot override
    or rescue a blocked canonical fact.

Every misuse must pin human and JSON code, message, help, primary/related spans,
stable reason, and runtime preflight result. H1401-H1402 cases must exit before
body execution with zero adapter calls and no generic trap. Resolver through
Core verify and graph must either consume the shared blocked fact or state the
same exact honest blocker.

### AL bans

No open row, row variable, nonempty latent label, handler, recovery, catch,
anonymous closure, callable return, callable storage, multiple application,
partial application, currying, generic callable type, overloading, dynamic
dispatch, function pointer ABI, permission-bearing callable, indirect fallible
call, implicit propagation, retained environment, allocation, capture,
authority-bearing operation, IO, higher-order library API, production migration
tool, new command, new pipeline gate, runtime JSON surface, or Session AM hook.

### AL acceptance criteria and hard stop

- The exact pinned source corpus is the only recognized slice and fails closed
  at every boundary.
- The positive depends on actual indirect application and produces the same
  value and evidence twice.
- All existing stages share one callable analysis; no consumer reparses source.
- Every competing recognizer is deleted, provably disjoint, or a nonauthoritative
  projection, with permanent sabotage evidence.
- Exact ordinary types and `failure_root = none` agree from resolver through
  runtime.
- The accepted semantic identity model can acquire later companion environment
  facts without changing callable/type/application/row meaning or reserving
  speculative fields now.
- Core and graph preserve identity and call relationships without claiming
  captures or allocation.
- Existing direct calls, Predicate v2, typed failures, ownership, authority,
  app entry, Core, and runtime fixtures remain unchanged.
- All session and standing checks pass. Stop. Session AM remains unauthorized
  pending independent review, scoped commit, successful CI, recorded handoff,
  and a separate BDFL go signal.

## Session AM: latent open-row propagation through application

Purpose: extend the accepted AL application with the smallest production form
of decision 0018's open multiset row, while leaving handling and callable
environments absent.

### AM authorized semantics

AM reuses the exact accepted AL syntax and identities. It adds no new callable
shape. The shared analyzer may infer:

- exact latent label identities from the already checked callable body;
- a deterministic normalized multiset that preserves duplicate occurrences;
- at most one structural tail variable for the receiving callable;
- stable alias evidence without treating aliases as distinct effects; and
- one structural substitution at the accepted application, propagating all
  labels and the open tail into the caller.

AM inference is intentionally local and nonrecursive. It operates only over the
single pinned file-local callable-parameter relationship and its one application
site. It does not generalize a recursive strongly connected component, export a
callable-accepting signature across a file/app boundary, infer a stored or
returned callable signature, or choose a value restriction. Those shapes fail
at the pinned H1401 boundary rather than receiving a monomorphic default.

Decision 0018's future boundary remains binding: a public, recursive, stored,
or returned callable signature must expose a stable row variable or named alias
under a later accepted work order. AM's local inference may not be reused as an
implicit public-signature rule. No result from AM claims global principal
inference, recursive generalization, effect exclusion, associated effects, or
handler-polymorphic inference.

The BDFL-pinned AM corpus must select an already accepted, non-authority-bearing
operation or checker-visible requirement for the nonempty-row observation. It
must not invent a new effect operation merely to test rows. If repository
evidence cannot provide such an observation without adding authority,
ownership, failure, handler, or IO scope, AM must stop for a bounded architecture
ruling rather than use a synthetic production effect.

Rows are type/effect evidence. AM does not execute a handler or change runtime
behavior for the underlying operation. The runtime positive must still execute
the accepted callable application and observe its real result; effect-check,
Core, and graph output must separately expose the propagated row. A negative
fixture must prove that dropping, renaming, deduplicating, closing, or replacing
the inferred tail fails closed.

### AM module and stage scope

`src/callable.rs` remains the only row analyzer. AM may extend its facts and the
existing stage schemas/renderers, but may not add a second effect-polymorphism
engine. Required consumers are:

- resolver: unchanged callable identities plus stable row-alias references;
- type environment/type/full-type: callable row variable, substitution, and
  application result without caller annotation in the ordinary positive;
- effect: exact input/output rows and one application-propagation relationship;
- ownership/resource: unchanged honest nonclaims and blockers;
- Core preview/lower/verify: exact label multiset, tail ID, substitution, and
  application propagation relationship;
- graph: row-variable, label-occurrence, alias, substitution, and propagation
  nodes/edges linked to callable definitions and the application span;
- runtime preflight: shared H1401/H1402 source blockers before execution, with
  no row strings interpreted by the evaluator; structurally corrupted row facts
  exist only in verifier unit tests and are not runtime inputs; and
- human/JSON: deterministic identical facts, reasons, spans, aliases, and repair.

No stage may call an accepted row closed, pure, handled, authority-safe,
ownership-safe, allocation-free, or principal beyond the facts it actually
consumes. Near-principal evidence is bounded to the accepted fixture: the
ordinary positive requires no caller-supplied row annotation and two fresh
analyses yield alpha-equivalent then byte-identical normalized output.

### AM positive, misuse, and stress evidence

Permanent evidence must include:

1. the accepted AL empty-row positive unchanged;
2. one nonempty-row callable application whose real result and static row are
   both observed;
3. two callables with order-permuted source requirements that normalize
   identically;
4. renamed tail variables that normalize identically by first semantic use;
5. duplicate labels that remain two occurrences;
6. a nested, chained, or second indirect application producing H1401 because
   AM retains AL's exactly-one-application boundary;
7. in-test corrupted semantic/Core facts for a missing label, substituted
   label, deduplicated label, erased row, prematurely closed tail, and foreign
   tail, each rejected by Core verification with one stable verifier reason and
   the exact relationship sites;
8. a bounded 256-label internal analyzer stress test preserving multiset and
   tail identity without adding a production source surface; and
9. poisoned expected output/reason fixtures proving analysis follows callable
    structure rather than test polarity.

The evidence must also include public/cross-file, recursive, stored, and
returned boundary misuses proving that AM refuses to silently monomorphize or
generalize them. A same-named row variable in two lexical scopes and two
distinct tail identities with identical display spelling must remain distinct.

The stress test proves only deterministic bounded behavior, not production
throughput, global principal inference, or an accepted row-size limit. Every
structured fact and relationship field needs an independent mutation test.
Missing, duplicate, contradictory, extra, reordered, or unknown identities fail
closed. Human/JSON diagnostics contain domain language and source aliases, not
raw substitutions or solver dumps.

### AM bans

No handler syntax or runtime, exact-label removal, recovery, catch, exception,
unwind, callable storage/return, runtime environment, capture set, allocation,
cleanup, ownership/lifetime bridge, permission-bearing callable, indirect
authority-bearing operation, consent inference, capability grant, new IO,
effect exclusion, Boolean formula core, capture-only effect core, associated
effect, generic higher-order API, callback registry, retry, timeout, parallel
execution, scheduler, migration tool, command, gate, or later-session hook.

AM may change only `src/callable.rs`, the AL semantic consumers and renderers
that must expose the additional row fact, their directly corresponding existing
schema/doctrine files, focused fixtures/tests, and proportional
`tools/check_all.ps1` assertions. It may not revisit AL syntax, change the
accepted callable shape, edit governance/decisions/research/bake-off artifacts,
or touch authority/adapter modules. Any need outside that envelope is a hard
stop for architecture review.

### AM acceptance criteria and hard stop

- Exact label multisets, aliases, and the single structural tail survive
  resolution, typing, effect checking, Core, graph, diagnostics, and runtime
  preflight through shared facts.
- Application propagates the complete row without loss, deduplication,
  substitution by spelling, or authority/ownership laundering.
- The ordinary positive has bounded near-principal inference with no caller row
  annotation and repeat-stable output.
- Local inference remains confined to the pinned nonrecursive relationship;
  public, recursive, stored, and returned boundaries fail closed without a
  temporary monomorphic or implicit-generalization rule.
- H1401/H1402 follow parser/resolver/full-type precedence and are consumed as
  prior blockers by effect, ownership, and resource stages. Row-corruption
  mutations are verifier failures, not user H-codes or runtime diagnostics.
- AL and every standing fixture remain green.
- All session and standing checks pass. Stop. Work Order 8 ends. No handling,
  environment, bridge, library, or later session is authorized.

## Cross-stage command and evidence matrix

Each session must independently run every positive and misuse fixture through:

- `hum resolve` human and JSON;
- `hum type-env` human and JSON;
- `hum type-check` human and JSON;
- `hum full-type-check` human and JSON;
- `hum effect-check` human and JSON;
- `hum ownership-check` human and JSON;
- `hum resource-check` human and JSON;
- `hum core-preview` human and JSON;
- `hum core-lower` human and JSON;
- `hum core-verify` human and JSON;
- `hum graph` using its existing surface;
- `hum run` for runtime behavior and preflight; and
- `hum diagnostics`/`hum explain` human and JSON for every new H-code.

The permanent preflight must assert the complete matrix, exact diagnostic
counts, zero generic traps, zero adapter calls for blocked forms, stable IDs,
stage agreement, Core verification, graph relationships, repeatability, and
scope hygiene. No new report, schema identifier, command, runtime JSON surface,
or pipeline gate is authorized; proportional fields extend only the existing
surfaces named above.

Before handoff, run:

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
git diff --check
.\tools\check_all.ps1
```

Also run every targeted fixture command explicitly rather than trusting the
preflight summary. The reviewer must rerun them independently.

## Compilation and platform coverage

The implementer and reviewer must enumerate every affected production and test
configuration: ordinary binary/library compilation, unit tests, integration
fixtures, doctests if present, all-target Clippy, and the preflight's release and
public-readiness paths. Any new `cfg`, feature, optional dependency, or target-
specific branch is out of scope.

The host Windows configuration must be executed locally. After a separately
authorized push, the existing Ubuntu and Windows CI jobs must both pass before
the session handoff can be closed. No work order may mandate an unproven cross-
target command merely to claim coverage. If another target is not installed,
record the gap, inspect all relevant non-host branches manually, and state why
the platform-neutral facts remain valid. Do not download a target, alter global
Git configuration, or add platform scaffolding to hide the gap.

## Deliberate deferrals and concrete costs

### Type-level handling

Deferral means effect rows can describe and propagate requirements but cannot
yet subtract one handled occurrence. This blocks typed retry/timeout wrapper
shapes and any handler relationship in Core or graph. It avoids prematurely
choosing source syntax or implying recovery; decision 0016 remains exact.

### Returned and stored callable environments

Deferral means event-handler factories, memoizing wrappers, logging middleware,
registries, and resource-bearing returned/stored callables remain unsupported.
No closure allocation, retained environment, cleanup owner, registration
lifetime, or runtime callable environment exists after Work Order 8.

### Capture, ownership, resource, and authority bridge

Deferral means even a sound row does not authorize higher-order captured state.
Immutable retention, moved/borrowed ownership, linear resources, exact authority
identity, escape, transfer, and lifetime remain unsupported. Authority cannot be
laundered through a callable, and operator consent cannot be captured or
inferred.

### Higher-order standard library

No `map`, effectful map, retain/filter, fold, retry, timeout, parallel map,
callback registry, wrapper, or middleware API is reserved or prebuilt. Their
cost is continued lack of ergonomic higher-order programs. Adding them earlier
would make library shape conceal missing callable environments and bridges.

### Internal references

Internal references remain the next ownership repair when ownership work
resumes. Program 5 still cannot naturally store a checked view into a parser's
owned buffer and receives no credit from callable rows.

### Air-gapped update validator

The validator remains the next adoption destination after the required
foundation. No Bytes, directory input, hashing, manifest, canonical JSON IO,
file writing, evidence directory, signature/provenance, or sandbox claim may be
prebuilt here. Deferral continues to cost the flagship real-tool proof, but
forcing it into callable foundations would fake both IO and security evidence.

## Independent pre-issuance review requirements

The fresh reviewer must cold-start from repository ground truth and verify:

- clean synchronized `main` at `e7dbadb` before this documentation change;
- Work Order 7 closure and the exact Session AK/decision/CI evidence above;
- governance authority, one-pen discipline, independent review, and separate go
  signals;
- decisions 0014, 0015, 0016, 0017, and 0018 in full;
- the complete Session AG corpus/result contract, Session AH-AJ candidates, and
  Session AK scorecard;
- current AST/parser/type/runtime reality, especially the absence of accepted
  callable syntax and runtime environments;
- the complete incorporated BDFL source corpus, lexical envelope, diagnostic-
  ownership table, and nonempty-row ruling;
- the shared semantic spine, diagnostic precedence, integration map, fixtures,
  allocation/authority nonclaims, and hard stops;
- explicit stable semantic identities rather than name/display-string identity;
- construction from parsed/resolved nodes rather than shared line-string scans;
- retirement or proven disjointness of every competing recognizer;
- durability of the callable identity model when later companion environment
  facts arrive, without speculative placeholder fields;
- the local inference boundary and explicit deferral of public/recursive/
  stored/returned generalization;
- whether AL and AM are each independently review-sized; and
- whether stopping before handlers/environments/bridges is the smallest safe
  boundary.

Only `WORKORDER.md` may differ. Research recommendations must remain hypotheses.
The reviewer must run `git diff --check` and `.\tools\check_all.ps1`, report
P0/P1/P2 findings with exact lines, and issue exactly one pre-issuance verdict.
The author of this document is disqualified from that verdict.

## Current authorization gate

Work Order 7 remains closed at `e7dbadb`. Session AK and accepted decision 0018
remain `1b324fb`, with workflow `29178915990`, Ubuntu job `86613010219`, and
Windows job `86613010224` successful. Work Order 8 is issued at `956b51f`, with
workflow `29183983775`, Ubuntu job `86626699948`, and Windows job `86626699928`
successful.

The current next action belongs only to the BDFL: give a separate go signal for
Session AL or leave implementation stopped.

Session AL remains unauthorized until the BDFL gives that separate go signal.
Session AM and every deferred phase remain unauthorized. No compiler/runtime
implementation, decision ruling, further commit, push, or publication is
authorized by this status update. Publishing remains a BDFL-reserved action
under `docs/GOVERNANCE.md`.

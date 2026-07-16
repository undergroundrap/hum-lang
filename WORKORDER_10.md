# Hum Work Order 10: Pre-AR Semantic Foundation Repair

Date: 2026-07-15
Status: proposed and unauthorized. These exact bytes require one fresh
independent pre-issuance architect-reviewer verdict, BDFL acceptance, a scoped
documentation commit, durable publication with successful required Ubuntu and
Windows CI, and a separate BDFL go signal for each prerequisite increment.
Nothing in this document authorizes implementation, a commit, a push, Session
AR, Hum IR, a standard library, a backend, or later work.

Owner: BDFL (Ocean).
Author: architect-reviewer acting only under the bounded Work Order 10
authoring authorization and therefore disqualified from this document's
independent verdict.
Baseline: clean `main` synchronized with `origin/main` at
`38704accfe839238cfe304ddec10106ea9f80e0b`.
Predecessor: Work Order 9, including Sessions AN-AQ and its final post-AQ
evidence correction, is closed and frozen at this baseline. `WORKORDER.md`
must not be amended.

## Authority and purpose

Decision 0008 makes the current interpreter the executable semantic reference
that a future Hum IR must preserve. A bounded independent foundation audit
found silent-wrong and self-disagreeing behavior in that reference. The BDFL
authorized this planning pass to close one seam before Session AR:

> one parser-owned expression and block-scoping semantics, consumed
> consistently by body execution, Predicate v2, Core, static gates, and every
> runnable entry, with correct place mutation and honest ownership claims.

This is foundation remediation, not a feature batch. It admits no new general
indexing, collection API, callable model, failure model, handler, recovery,
effect, capture, allocation, IO, concurrency, IR, backend, or standard-library
surface. It replaces divergent or unchecked behavior with one correct meaning
or an honest refusal.

The audit attachment available to this author is a summarized report rather
than the auditor's full scratchpad. This Work Order records only findings whose
live behavior was independently reproduced by the root audit on the clean
baseline. It does not infer unreported examples, causes, or tastes.

## Verified audit evidence

The root audit built fresh temporary `.hum` probes, ran
`target/debug/hum.exe`, recorded the exact results below, and removed every
temporary artifact. These observations are authorization evidence, not
implementer claims:

| ID | Live source shape | Reproduced result at `38704ac` | Required result |
| --- | --- | --- | --- |
| P0-1 | a caller passes a mutable value to a `change` parameter, the callee writes it, and the caller then observes its own place | exit 0, stdout `41\n`; the callee's mutation was silently dropped | caller-visible write-through; the corresponding positive fixture observes `42\n` |
| P0-2 | `set items[0] = "after"` followed by reading the list | exit 0, stdout `after\n`; the list binding was replaced by a scalar | update only element 0, preserve list identity/length/other elements |
| P1-1a | `return 8 * 6 / 4` | exit 0, stdout `8\n` | ordinary left-associative multiplicative evaluation: `12\n` |
| P1-1b | byte-identical `ensures: result == 8 * 6 / 4` and body return | exit 1, no stdout, false H0703 task blame | body and contract agree on 12; exit 0, no H0703 |
| P1-4 | Text literals containing `}` or `{` inside an item body | item recognition is corrupted and produces H0001/H0004 | literal braces remain Text contents and cannot open or close an item block |

The code inspection supporting the integration map also establishes:

- `src/run.rs::split_top_level_operator` chooses the rightmost operator and
  recursively makes equal-precedence body arithmetic right-associative;
- `src/predicate.rs::Parser::{parse_additive, parse_multiplicative}` builds a
  left-associative Predicate v2 tree;
- `src/core_expr.rs` reports multiple operators as
  `operator_precedence_not_resolved` instead of preserving their semantic
  tree;
- `src/parser.rs::find_matching_close` counts every brace character without a
  Text-literal state;
- `src/main.rs::execute_run_command` applies resolve/type/full-type blocking
  only to structural app mode, so legacy single-task selection and explicit
  `--entry` can bypass the same gate;
- direct calls in `src/run.rs` strip `borrow`/`change`, pass cloned values into
  `execute_task`, and return without copying a changed formal back to its
  caller place; and
- `src/run.rs::write_place` handles a direct record field but treats
  `items[0]` as a whole-root assignment.

No other audit finding is promoted into implementation by this Work Order.

## Decision locks

Every increment and review preserves the following accepted decisions.

### Decision 0008: interpreter before backend

- The corrected interpreter remains the semantic reference for later IR and
  backend comparison.
- No Hum IR node, bytecode, native lowering, backend bridge, optimizer, code
  generator, ABI, target contract, or differential backend claim is added.
- Session AR may rely only on the final accepted behavior from this Work Order,
  never the reproduced buggy behavior or an intermediate increment.

### Decision 0014: place-based ownership

- Values remain owned by default; `borrow`, `change`, and `consume` retain
  their accepted meanings.
- A `change` formal is mutation authority over its exact resolved caller
  place, not a copied input whose writes may disappear.
- A direct numeric list element is a place. Element assignment may update only
  that element; it may not replace the root list, widen to general indexing,
  manufacture mutation authority, or hide allocation.
- Existing H0801-H0809 ownership, move, view, alias, and precedence meanings
  remain owned by the ownership subsystem. Any new cause or code requires the
  explicit diagnostic gate below.
- The existing locks remain: no complete ownership, borrow, linear-resource,
  internal-reference, disjoint-projection, or memory-safety claim.

### Decision 0015: executable contracts

- Every recognized executable predicate continues to run; no proof/trust
  classification, elision, profile, global toggle, or unreachable-guard
  conclusion is introduced.
- Predicate recognition/type ownership and H0701-H0704 meanings remain
  unchanged. Sharing expression syntax does not turn prose into a predicate.
- H0702/H0703 continue to report actual caller/task contract violations only.
  Parser or evaluator disagreement may never manufacture contract blame.

### Decision 0016: nominal causal typed failure

- Nominal roots/variants, explicit `try` or wrapping, causal sites, and
  H0901-H0907 ownership remain unchanged.
- Expression convergence must preserve the distinction between a value, a
  typed operational failure, a source diagnostic, and an internal invariant
  failure. It adds no implicit propagation, erased error, exception, catch,
  recovery, or unwind.

### Decision 0017: authority

- Source authority, operator consent, and exercised operations remain
  separate exact facts.
- No expression, mutation, scoping, or run-gate repair may create a capability,
  grant, route, locality claim, adapter call, or ambient authority.
- Every blocked runnable path reaches zero output, replay, locality, and file
  adapter calls in injected-adapter evidence.

### Decision 0018: effects and capture companions

- Open rows, captures, ownership, resources, allocation, authority, and
  operation exercise remain distinct.
- No handler, callable environment, effect inference expansion, capture
  bridge, or higher-order library is implemented.
- Decision 0018 explicitly withholds ownership soundness and general
  linear-resource checking. Together with decision 0014's partial-checker
  locks, that already discloses the audit's actual linear-resource leak through
  a `try` path as a known limitation. It is recorded in the backlog below and
  is not implemented here.

## Exact semantic contract

### One expression grammar and tree

The parser owns one immutable, span-preserving expression tree per expression
occurrence. Body execution, Predicate v2 eligibility/type analysis, Core
preview/lower/verify, resolver/type/effect/ownership/resource/profile gates,
graph projection, and runtime must consume or project that tree. No consumer
may rescan the expression string to rediscover operators, calls, places,
parentheses, or literal boundaries.

The shared grammar is limited to already recognized executable forms plus the
direct element-assignment place explicitly mandated here. It must preserve:

1. grouping with parentheses;
2. literals and current list/record literal shapes;
3. exact identifier, field-place, and direct numeric element-place nodes;
4. current call and permission-wrapper nodes;
5. multiplicative `*` and `/`, left associative;
6. additive `+` and `-`, left associative;
7. the current comparison operators, non-chainable unless already explicitly
   supported;
8. current `and` then `or` short-circuit precedence; and
9. exact source spans and stable parser/resolver identities for every node.

For example:

```text
8 * 6 / 4       == (8 * 6) / 4 == 12
20 - 6 - 4      == (20 - 6) - 4 == 10
8 + 6 * 4       == 8 + (6 * 4) == 32
48 / (6 / 2)    == 16
```

This table is semantic, not a formatting preference. Whitespace and redundant
parentheses may change spans/text but not meaning. Overflow and division by
zero keep their existing fail-closed ownership; this order does not define a
new numeric mode.

Predicate v2 remains a semantic restriction over the shared tree. Its closed
operand/type/call/eligibility envelope remains in `src/predicate.rs`, but it
must inspect the parser-owned nodes rather than parse the line again. A shared
tree accepted in both a body and an executable predicate has the same operator
tree and literal values. Contract-only functions such as `old(...)` and
`list_count(...)` remain contract-only.

The authoritative parsed expression tree and every per-node span remain
private compiler facts in this Work Order. Core's private Rust lowering and
verification representations must consume and validate the complete tree,
node identities, operator structure, associativity, and exact spans. An
admitted multi-operator expression may no longer be internally represented as
`operator_precedence_not_resolved`. Missing, substituted, reordered, extra,
rotated, deduplicated, or reparented nodes, wrong operators or associativity,
and wrong identities or spans must fail internal verification before public
serialization or runtime.

Existing Core preview/lower/verify human and JSON schemas remain byte-
compatible. They expose only projections representable by their current
fields, such as existing root summaries and IDs; this Work Order adds no child-
tree or node-span field. Human/JSON agreement applies to those existing public
projections only. Private-tree correctness must be proved by independently
supplied internal mutation evidence and may never be inferred merely because
two public root summaries or serialized projections are identical.

The repository currently repeats mechanical `header_body` and
`strip_keyword` helpers across several stages. This Work Order does not turn
their wholesale cleanup into unrelated refactoring. A copy may remain during a
bounded increment only when tests prove it is a disjoint projection of an
already parser-owned statement/node and cannot recognize, choose, or override
expression or block meaning. Any copy that still rediscovers authoritative
expression, place, call, predicate, or block structure must be retired or made
non-authoritative in the increment that migrates that structure. The final 10F
source audit records every surviving copy and its proven non-authoritative
purpose; an unexplained competing recognizer keeps the Work Order open.

### String-aware scoping

One parser-owned lexical delimiter state recognizes item and executable block
braces. Braces inside a valid Text literal are literal contents and never
change block depth. The state must respect the repository's current quote and
backslash behavior; it must not invent a new escape syntax.

The parser owns item boundaries before sections, body statements, expression
nodes, resolver identities, or Core facts are built. Downstream stages may not
recount raw source braces. Runtime block matching consumes parser-owned block
relationships; it may not use a competing `ends_with('{')`/`text == "}"`
recognizer as authority.

Real malformed source retains source-shape ownership: H0003 owns a true item
header missing `{`, H0004 owns a truly unclosed item block, and H0001 remains a
genuine unexpected top-level line. Quoted `{` or `}` must not create any of
those diagnostics or steal their blame span.

### Universal checked execution

Every production `hum run` selection uses the same execution-eligibility fact:

```text
load/parse/check
-> selected entry and reachable task closure
-> resolve clean
-> declaration/type clean
-> full-type accepted, with zero rejected, unchecked, or unsupported reachable statements
-> existing effect/ownership/resource/profile/runtime blockers
-> arguments and body
-> adapters
```

The structural app path, legacy no-app single-task selection, and explicit
`--entry` path differ only in how they select the root. They do not differ in
which source can execute. A selected or reachable statement classified by the
static pipeline as blocked, rejected, unchecked, or unsupported cannot run.
An unreachable unrelated task does not become executable evidence and must not
be used to mask a healthy selected closure.

One internal sealed eligibility value must bind the exact program analysis,
selected resolver definition, and reachable closure. The runner consumes that
fact before raw argument conversion, task bodies, mutation, or adapters. No
boolean, display name, source text, diagnostic code list, or test-only
reconstruction may stand in for it. All production runner entry points either
require the fact or are private adapters beneath the single gate. Superseded
legacy runnable paths are removed or made mechanically incapable of bypassing
the gate.

Existing source/resolver/type/full-type diagnostics and precedence remain the
public refusal. This Work Order does not allocate a generic "execution gate"
diagnostic merely to hide the originating cause. Human/JSON facts must agree,
runtime emits no generic trap for a statically owned rejection, and blocked
paths call zero adapters.

### `change` parameter write-through

At a direct task call, every formal marked `change` must correspond one-to-one
to an argument spelled `change <place>` whose exact parser/resolver place has
writable authority. The runtime copies the value into the callee environment
and then follows this closed outcome table:

| Outcome | Callee body | Applicable `ensures:` | Caller-visible copy-out |
| --- | --- | --- | --- |
| static or runtime preflight rejection | not entered | not evaluated | none |
| false `needs:` | not entered | not evaluated | none |
| successful explicit `return` | completed | evaluated against the final callee local environment with the result binding; completion continues only when it passes | each `change` formal exactly once in formal/source declaration order, after `ensures:` passes and before the caller continues |
| ordinary fallthrough | completed | evaluated against the final callee local environment with the current Unit result; completion continues only when it passes | each `change` formal exactly once in formal/source declaration order, after `ensures:` passes and before the caller continues |
| false `ensures:` after mutation | completed | fails with the existing task-contract failure | none |
| explicit typed `fail` after mutation | completed typed-failure outcome | not evaluated: the accepted runtime currently routes `Flow::Fail` directly to `TaskResult::Failed`, so `ensures:` remains success-only | each `change` formal exactly once in formal/source declaration order before the typed failure propagates |
| internal invariant trap | not a language completion | not evaluated | none |

False `ensures:` therefore cannot expose the callee's private changed locals to
the caller, but this is only the consequence of the bootstrap copy-in/copy-out
boundary. It is not a general transaction, rollback, exception, catch, unwind,
alias, or multi-place atomicity claim. Explicit typed failure remains the
nominal causal outcome defined by decision 0016; this rule does not convert it
to an exception or evaluate a success-only contract on it. Its completed body
outcome retains the final callee local environment solely so the exact changed
formals can be copied out before the typed failure propagates. An internal
invariant remains an internal invariant rather than a source diagnostic.

This copy-in/copy-out implementation is the bootstrap representation of
write-through, not a claim about a future ABI.

Copy-out must use semantic definition/place identity, never the argument text,
display name, span alone, parameter position alone, or a late source rescan.
Already accepted multiple `change` formals preserve formal/source declaration
order when transported to their corresponding arguments. This ordering rule
does not add a general multiple-place or atomic transaction model. Aliasing or
overlap among writable arguments must be rejected by the existing ownership
owner if its meaning is exact; the runtime may not make result depend on copy-
out order. If no existing diagnostic honestly owns a required overlap, the
increment stops at the diagnostic decision gate rather than reusing a nearby
H-code.

At minimum, the repair covers an authorized mutable local and transitive
write-through when a `change` parameter is passed onward. Direct field and
direct numeric element caller places join only when their existing resolver
and ownership facts prove the same exact place; no general lvalue or reference
surface is inferred.

### Direct numeric list-element assignment

`set items[uint-literal] = value` uses the already authorized direct numeric
element-place shape. It requires writable authority over `items`, the value
must match the list element type, and runtime replaces exactly that in-bounds
element. The operation preserves the root list binding identity, length, all
other elements, and allocation facts. It does not claim a new public object-
identity model and is not structural growth, append, general indexing, an
element alias, or a retained reference.

The parser, resolver, full type, effect, ownership, Core, graph, and runtime
must agree on the exact root and literal index. Substituting the root/index,
dropping the element relationship, or projecting a whole-root assignment must
fail closed. An out-of-range access must leave the list unchanged and follow
the existing bounds/runtime-error ownership; the implementation may not clamp,
grow, append, wrap, or overwrite the root. If that existing error channel
cannot represent the refusal without a generic or misleading trap, stop at the
diagnostic decision gate.

An existing direct element view of the exact written element is invalidated
under H0807 ownership. A provably distinct direct literal element remains
distinct. This assignment does not silently broaden H0806's structural-growth
during iteration rule; any required policy change beyond exact element-place
identity stops for review.

## Global integration map by intent

The maps below are exhaustive by intent and list the current likely files and
functions after production inspection. A named file is authorized only in the
increment that names it and only for the stated intent. If honest
implementation needs another file, the implementer stops with the tree
preserved and requests a bounded independently reviewed amendment. It may not
guess around the envelope.

Current likely production spine:

- syntax/identity: `src/ast.rs` (`ParsedExpression*`, body syntax),
  `src/parser.rs` (`find_matching_close`, `parse_task_body_syntax`,
  `parse_expression_syntax`, call nodes), and one new internal
  `src/expression.rs` if needed to keep the grammar/evaluator shared;
- body facts: `src/core_body.rs`;
- contract facts: `src/predicate.rs` (recognition, eligibility, typing, place
  resolution; not a second syntax parser);
- Core projection: `src/core_expr.rs`, `src/core_preview.rs`,
  `src/core_lower.rs`, and `src/core_verify.rs`;
- static consumers: `src/resolve.rs`, `src/type_env.rs`, `src/type_check.rs`,
  `src/full_type_check.rs`, `src/effect_check.rs`,
  `src/ownership_check.rs`, `src/resource_check.rs`,
  `src/profile_check.rs`, and `src/ir_readiness.rs` only where their existing
  expression/place/blocker projections must consume the canonical fact;
- graph/public composition: `src/graph.rs`, `src/json.rs`, and `src/main.rs`;
- runtime: `src/run.rs` (`execute_task`, `execute_task_body`, `eval_block`,
  `eval_expr`, contract evaluation, direct calls, `write_place`, and runner
  preflight); and
- permanent evidence: `tools/check_all.ps1` plus hand-authored fixtures under
  `fixtures/foundation/`.

Current likely documentation projections, changed only when shipped behavior
requires honesty:

- `docs/MILESTONE_0_GRAMMAR.md`;
- `docs/LANGUAGE_REFERENCE.md`;
- `docs/FORMAL_CORE.md`;
- `docs/CORE_LANGUAGE_SHAPE.md`;
- `docs/HUM_CORE_PREVIEW_SCHEMA.md`;
- `docs/HUM_CORE_LOWER_SCHEMA.md`;
- `docs/HUM_CORE_VERIFY_SCHEMA.md`;
- `docs/HUM_FULL_TYPE_CHECK_SCHEMA.md`;
- `docs/HUM_EFFECT_CHECK_SCHEMA.md`;
- `docs/HUM_OWNERSHIP_CHECK_SCHEMA.md`; and
- `docs/DIAGNOSTICS.md` only for the linear-resource wording correction.

No new public command, schema identifier, JSON field, H-code, pipeline stage,
dependency, feature, `cfg`, generated source, or target-specific branch is
authorized. Existing schemas may document corrected meanings; adding a field
or identifier requires a reviewed amendment.

## Work-Order-local increment labels and Session AR

The global session odometer has completed Session AQ; its next label is AR.
The BDFL has already reserved the name **Session AR** for the later Hum IR plus
minimal compiler-ready standard-library milestone and expressly forbade this
planning pass from beginning or silently renaming it.

Therefore the repairs below are named **Prerequisite Increment 10A** through
**10F**, not sessions. They are Work-Order-local, review-gated implementation
units created solely to reconcile the odometer with the reserved future name.
They do not change the global odometer policy, create a parallel lane, or imply
that Session AR has been reviewed. Each increment still follows the normal
one-pen implementation, independent review, scoped commit, separately
authorized push/CI, status record, and next-go-signal lifecycle. Session AR
remains the next lettered session and stays unauthorized after 10F until a
fresh Work Order for AR is separately authored and issued after the required
foundation audit.

Mandatory order:

```text
10A canonical expression and string-aware scope facts
-> 10B body / Predicate v2 / Core semantic convergence
-> 10C universal checked execution
-> 10D change-parameter write-through
-> 10E direct list-element assignment
-> 10F linear-help honesty and foundation closure
-> stop; Session AR remains unauthorized
```

No increment may begin before the previous one is independently accepted,
committed, published, green on Ubuntu and Windows, and recorded in this fresh
Work Order, followed by its own separate BDFL go signal.

## Prerequisite Increment 10A: canonical syntax and string-aware scope

### Scope and likely files

Intent: make parser-owned block and expression identity authoritative without
changing evaluation yet.

Likely production files:

- `src/ast.rs`: canonical expression/block node types and stable identities;
- `src/parser.rs`: quote-aware `find_matching_close`, full current expression
  tree construction, exact spans, and parser mutation tests;
- `src/core_body.rs`: retain parser-owned statement/block relationships rather
  than reconstructing them from line endings;
- `src/main.rs`: module registration only if a new internal
  `src/expression.rs` is used;
- `src/expression.rs` (optional new internal module): closed token/operator
  definitions and parser support, with no public surface; and
- `tools/check_all.ps1` plus `fixtures/foundation/pre_ar_text_braces_pass.hum`
  and `fixtures/foundation/pre_ar_real_unclosed_block_fail.hum`.

### Required evidence

- A real source fixture containing both `"}"` and `"{"` in Text values parses
  through `load_program`, resolves, and reaches every existing applicable
  human/JSON stage without H0001/H0003/H0004. Runtime observes the exact Text
  bytes, not merely parse success.
- A genuine unclosed item produces exactly H0004 at the real item header;
  changing a literal brace cannot repair it.
- A genuine malformed header retains H0003, and an actual stray top-level line
  retains H0001.
- Parser tests mutate quote state, escaped-quote behavior already supported by
  the repository, brace direction, node order, node span, and source-node ID;
  each independent corruption fails.
- The canonical expression tree for `8 * 6 / 4`, `20 - 6 - 4`, mixed
  precedence, and grouping is structurally left-associated before any runtime
  consumer exists.
- Retained section text is sabotaged after parsing; block and expression facts
  remain unchanged. Conversely, mutating the parser-owned node changes the
  fact. This proves the parser, not a source rescan, owns semantics.

### Diagnostics and cross-stage boundary

Parser/source-shape diagnostics retain exact human/JSON messages, severity,
ordering, causes, spans, and precedence. Later stages may consume or block on
the new fact but cannot yet claim corrected execution. Existing callable,
Predicate v2, typed-failure, authority, and ownership identities remain stable.

### Bans and acceptance

- No body or contract evaluator change, mutation repair, run-gate change, new
  expression form, public field, or schema identifier.
- No downstream source rescan added to ease migration.
- `cargo fmt --check`, focused parser/fixture tests, `cargo test`, warnings-
  denied Clippy, `git diff --check`, and `tools/check_all.ps1` pass.
- Host configurations are enumerated; platform-neutral code is manually
  inspected and later proven by Ubuntu/Windows CI.
- Hard stop after an uncommitted 10A worktree for fresh independent review.

## Prerequisite Increment 10B: semantic convergence

### Scope and likely files

Intent: make body execution, Predicate v2, and Core consume the one 10A tree.

Likely production files:

- `src/expression.rs` or the 10A canonical expression module;
- `src/predicate.rs`: remove syntax reparsing; preserve recognition,
  eligibility, typing, exact place identities, H0701/H0704 ownership, and
  accepted Predicate v2 facts;
- `src/core_expr.rs`: build the private canonical Core tree and eliminate
  unresolved precedence for admitted expressions while preserving only the
  existing public projection;
- `src/run.rs`: evaluate the canonical tree for bodies and contracts through
  one operator implementation and shared message builders;
- `src/core_preview.rs`, `src/core_lower.rs`, `src/core_verify.rs`: consume and
  validate the exact private tree, then serialize only existing public fields;
- `src/resolve.rs`, `src/type_env.rs`, `src/type_check.rs`,
  `src/full_type_check.rs`, `src/effect_check.rs`,
  `src/ownership_check.rs`, `src/resource_check.rs`,
  `src/profile_check.rs`, `src/ir_readiness.rs`, `src/graph.rs`, and
  `src/json.rs` only where their current expression facts or blocker
  projections require the canonical node; and
- the directly dependent Core/language schema documents named in the global
  map, `tools/check_all.ps1`,
  `fixtures/foundation/pre_ar_left_associative_arithmetic_pass.hum`, and
  `fixtures/foundation/pre_ar_body_contract_expression_agreement_pass.hum`.

### Required evidence

- The real CLI returns exact stdout `12\n` for `8 * 6 / 4`, `10\n` for
  `20 - 6 - 4`, `32\n` for `8 + 6 * 4`, and `16\n` for `48 / (6 / 2)`.
- Byte-identical `ensures:` and body arithmetic produce structurally identical
  canonical subtrees with distinct occurrence identities, value 12, exit 0,
  empty stderr, and no H0703.
- False `needs:` and false `ensures:` controls still produce their existing
  exact caller/task blame; the fix does not suppress genuine H0702/H0703.
- Text, list, calls, permissions, places, short-circuiting, overflow, division
  by zero, typed failure, and contract-only forms retain their admitted
  behavior or exact existing rejection.
- Private resolver/type/checker/Core/graph/runtime consumers agree on the
  authoritative node identities and operator order. Existing human/JSON
  outputs agree only on projections representable by their current fields and
  remain byte-compatible; no private child tree, identity, or span leaks
  through an unauthorized public field.
- Before public serialization or runtime, independently supplied Core mutation
  tests remove a node, rotate operands, replace `*` with `/`, change
  associativity, remove grouping, reorder children, add or duplicate a node,
  substitute a node from another same-text occurrence, and corrupt spans/IDs.
  Every mutation fails. Identical public root summaries are held constant in a
  control while private children are corrupted, proving the public summary is
  not treated as private-tree validation.
- Source audits prove `run.rs`, `predicate.rs`, and `core_expr.rs` contain no
  competing operator splitter/parser used by production semantics.
- Two fresh processes produce byte-identical human/JSON/runtime results.

### Diagnostics and acceptance

Malformed or ill-typed Predicate v2 candidates remain H0704 before H0702/
H0703. Static type/ownership/authority failures retain precedence over runtime
evaluation. Parser failures never become contract blame. No new H-code or
error channel is authorized.

The full targeted matrix and all standing checks pass. Hard stop after an
uncommitted 10B worktree for independent review. A required new public field,
general expression form, or semantic diagnostic triggers a reviewed amendment,
not local invention.

## Prerequisite Increment 10C: universal checked execution

### Scope and likely files

Intent: eliminate the legacy runnable bypass without changing which task is
selected.

Likely production files:

- `src/main.rs`: `execute_run_command` and the real command orchestration;
- `src/run.rs`: require/validate the sealed eligibility fact before argument
  conversion, body execution, mutation, or adapters;
- one new internal `src/execution_gate.rs` plus module registration if that is
  the smallest way to keep the fact shared and sealed;
- `src/app_entry.rs`, `src/resolve.rs`, `src/type_check.rs`, and
  `src/full_type_check.rs` only for exact selected-root/reachable-closure facts;
- downstream checker modules only if their existing blocker carrier must be
  consumed, never to create a second eligibility computation;
- `docs/LANGUAGE_REFERENCE.md`, `docs/ARCHITECTURE.md`, and directly affected
  existing schemas only to state the now-universal behavior; and
- `tools/check_all.ps1` with three real fixtures:
  `pre_ar_app_unchecked_execution_fail.hum`,
  `pre_ar_single_task_unchecked_execution_fail.hum`, and
  `pre_ar_entry_unchecked_execution_fail.hum`.

### Required evidence

- The same selected task body containing one static unsupported statement is
  rejected through structural app, legacy single-task, and explicit
  `--entry`; no form reaches the statement.
- Each path runs through real `load_program` and command composition, produces
  the existing owning human/JSON diagnostic or stage report, exits with its
  pinned nonzero status, emits zero stdout, emits no generic runtime trap, and
  calls each injected adapter exactly zero times.
- Selected resolver, type, and full-type failures are independently pinned in
  all three entry modes. Combined-cause precedence remains source -> resolver
  -> type -> full type -> later checker/runtime ownership.
- A healthy selected task beside an unreachable invalid task retains the
  current reachability policy and runs only if every fact used to justify that
  selectivity is parser/resolver-owned. The test must observe the healthy
  output and prove the invalid task was not reachable, not hide a global error.
- Sabotage that removes the single-task gate, removes the `--entry` gate,
  changes the selected definition, drops one reachable task, turns unsupported
  into accepted, reconstructs a fact by name/text, or calls an adapter before
  validation makes a permanent test fail.
- All test helpers enter through the production command extraction. A unit
  test that manually fabricates an eligibility fact is corruption evidence
  only and cannot satisfy positive execution evidence.

### Diagnostics and acceptance

No generic gate code is added. The originating parser/resolver/type/full-type
owner renders exactly once; downstream reports retain the exact blocker. Any
case that cannot refuse without changing public precedence stops at a decision
gate. Full standing checks pass. Hard stop after an uncommitted 10C worktree.

## Prerequisite Increment 10D: `change` parameter write-through

### Scope and likely files

Intent: implement P0-1 correctly over exact resolved places.

Likely production files:

- `src/ast.rs`/canonical expression facts for `change <place>` identity;
- `src/resolve.rs` for formal-to-argument place relationships;
- `src/full_type_check.rs`, `src/effect_check.rs`, and
  `src/ownership_check.rs` for permission, type, overlap, and write-through
  checking;
- `src/core_expr.rs`, `src/core_preview.rs`, `src/core_lower.rs`,
  `src/core_verify.rs`, and `src/graph.rs` for the same formal/caller-place
  relationship without a new public schema field;
- `src/run.rs` for callee completion plus ordered exact-place copy-out;
- directly affected existing grammar/Core/ownership documentation; and
- `tools/check_all.ps1`,
  `fixtures/foundation/pre_ar_change_parameter_write_through_pass.hum`,
  `pre_ar_change_parameter_transitive_pass.hum`,
  `pre_ar_change_parameter_fallthrough_pass.hum`,
  `pre_ar_change_parameter_typed_failure_fail.hum`,
  `pre_ar_change_parameter_false_needs_fail.hum`,
  `pre_ar_change_parameter_false_ensures_fail.hum`,
  `pre_ar_change_parameter_preflight_fail.hum`, and exact misuse fixtures for
  missing `change`, immutable/borrowed source, wrong type, and overlapping
  arguments.

### Required evidence

- The reproduced real program observes caller value 42, not merely the
  callee's returned value.
- A transitive caller -> middle -> leaf `change` chain observes every accepted
  write-through in the outer caller.
- Multiple distinct `change` arguments copy out to their own resolver places
  in formal/source declaration order, independent of display names and
  statement text. This is ordering evidence for already accepted formals, not
  a general multiple-place or atomicity claim.
- Shadowing, same spelling in another scope, reordered parameters, wrong
  definition ID, wrong place root/field/index, missing copy-out, duplicate
  copy-out, and a same-span/same-text foreign call each fail independently.
- Missing permission, immutable or borrowed arguments, wrong operand type, and
  overlapping writable arguments fail before the callee body and before all
  adapters. A recognized existing H-code is used only when its registered
  cause exactly owns the misuse.
- Real-source explicit-return and fallthrough fixtures traverse the production
  call path, inspect the caller-visible final value, and prove exactly one
  ordered copy-out per accepted `change` formal only after success-only
  `ensures:` passes.
- A real-source explicit typed-failure fixture mutates first and then executes
  `fail`; the production path proves the final changed value is copied out
  exactly once before the existing typed failure propagates, while success-
  only `ensures:` is not evaluated.
- Real-source false-`needs:`, false-`ensures:`-after-mutation, and static/runtime
  preflight-rejection fixtures prove zero copy-out, the exact existing
  diagnostic or typed outcome, the caller value unchanged at the private call
  boundary, and zero forbidden output/replay/locality/file adapter calls.
- All seven outcome paths (explicit return, fallthrough, typed failure, false
  `needs:`, false `ensures:`, preflight rejection, and invariant) run with
  injected counting output/replay/locality/file adapters and assert exact zero
  calls because these fixtures contain no authorized operation.
- Where the public caller cannot continue after contract failure or typed
  failure, a narrow `#[cfg(test)]` observer may expose the actual production
  call-boundary result and caller environment to the test; it may not
  reconstruct execution or provide expected state. The fixture still enters
  through real parsing, resolution, preflight, and the production call path.
- An internal-invariant test may synthesize corruption only through a test-only
  seam that traverses the real defensive production branch. It proves zero
  copy-out, zero adapters, and internal-invariant ownership rather than a
  source diagnostic. Synthetic success, contract, or typed-failure outcomes do
  not count.
- Independent sabotage separately performs zero copy-outs, duplicate copy-out,
  reordered copy-out, copy-out before `ensures:`, copy-out after false
  `ensures:`, copy-out after false `needs:`, dropped typed-failure copy-out, and
  copy-out after an invariant; each corresponding permanent test must fail.
- Core and graph corruption tests reject substituted formal, caller place,
  target, ordering, or route facts.
- Runtime uses the checked relationship rather than stripping `change` and
  rediscovering argument text.

### Default-correct decision and emergency stop

The BDFL decision is to implement this behavior correctly in 10D. The
implementer may not replace it with a permanent rejection. Only if real code
inspection during 10D proves that exact write-through requires a genuinely
multi-session ownership/ABI redesign may the implementer stop without editing
outside the envelope. The stopped report must provide the concrete production
dependency, why no review-sized correct slice exists, the exact currently
silent case, and a designated diagnostic proposal that would make that case
fail closed. The BDFL then decides whether to amend or defer. No diagnostic is
allocated and no silent behavior remains by implementer discretion.

Full standing checks pass. Hard stop after an uncommitted 10D worktree.

## Prerequisite Increment 10E: direct list-element assignment

### Scope and likely files

Intent: implement P0-2 correctly as the exact direct numeric element place.

Likely production files:

- canonical AST/parser/place modules, including `src/element_place.rs`;
- `src/resolve.rs`, `src/full_type_check.rs`, `src/effect_check.rs`, and
  `src/ownership_check.rs` for exact root/index/type/authority and H0807 view
  invalidation;
- Core preview/lower/verify and graph modules for exact element-write facts;
- `src/run.rs::{eval_set, write_place}` for in-bounds element replacement;
- `docs/MILESTONE_0_GRAMMAR.md`, `docs/LANGUAGE_REFERENCE.md`,
  `docs/FORMAL_CORE.md`, `docs/CORE_LANGUAGE_SHAPE.md`, and directly affected
  existing checker/Core schemas; and
- `tools/check_all.ps1`,
  `fixtures/foundation/pre_ar_list_element_set_pass.hum`, plus misuse fixtures
  for immutable root, wrong element type, out-of-range index, stale same-
  element view, and a distinct-element control.

### Required evidence

- The real positive starts with at least two Text elements, executes
  `set items[0] = "after"`, and observes the complete list with the first
  element changed, the second unchanged, and the same length. Observing only
  `items[0]` is insufficient because the reproduced bug also prints `after`.
- Mutation of index 0 versus 1 produces different expected complete lists.
- Whole-root replacement, append, list growth, length change, other-element
  change, allocation claim, or copy of a separate list fails the positive
  assertions.
- Static human/JSON stages, Core, graph, and runtime agree on exact root/index.
- Wrong type and immutable/borrowed root fail before runtime; out-of-range
  leaves the complete list unchanged and follows its exact existing error
  owner.
- Same-element live-view misuse produces H0807 with binding/write sites;
  distinct literal element control remains accepted where current ownership
  evidence can prove disjointness.
- Missing, extra, duplicate, reordered, root-substituted, index-substituted,
  whole-root, and foreign-occurrence projections independently fail.

### Default-correct decision and emergency stop

The BDFL decision is to implement exact element assignment correctly in 10E.
The same exceptional stop rule as 10D applies only upon concrete proof of a
genuine multi-session blow-up. The stopped report must name the dependency and
propose a designated diagnostic for the exact silent case; it may not silently
defer, clamp, append, replace the root, or broaden indexing.

Full standing checks pass. Hard stop after an uncommitted 10E worktree.

## Prerequisite Increment 10F: linear-help honesty and closure

### Scope and likely files

Intent: remove the overclaim from shipped H0803/H0804 evidence and prove the
foundation corpus end to end.

Likely files:

- `src/ownership_check.rs` help builders;
- `src/run.rs` only if its corresponding runtime help/message makes the same
  overclaim;
- `src/diagnostic_catalog.rs` only where the checked explanation/repair or
  cause wording itself asserts coverage not proved by the checker;
- `docs/DIAGNOSTICS.md`, `docs/HUM_OWNERSHIP_CHECK_SCHEMA.md`,
  `docs/CORE_LANGUAGE_SHAPE.md`, and existing doctrine text only to align
  claims with current proof;
- existing H0803/H0804 fixtures plus one real
  `fixtures/foundation/pre_ar_linear_help_honesty_fail.hum`; and
- `tools/check_all.ps1` for the final complete pre-AR foundation matrix.

### Required behavior and evidence

- Help may say the reported resource is unconsumed or consumed twice on the
  exact checked path named by the diagnostic. It may not claim the checker has
  proved all possible paths, all `try` propagation, general linear resources,
  ownership soundness, or memory safety.
- Existing H0803/H0804 codes, severity, primary/related blame, fundamental
  owner, and precedence remain unchanged. If a stable reason identifier is
  retained for compatibility despite historical wording, prose and schema
  must explicitly identify it as an identifier rather than a proof claim. If
  changing the public reason is necessary for honesty, stop for a bounded
  diagnostic-compatibility amendment; do not drift it incidentally.
- The real positive and misuse fixtures from 10A-10E run through their actual
  CLI entry points. No synthetic-only test counts as ownership, mutation,
  expression, scoping, or gate evidence.
- Independent sabotage for right-association, Predicate/body divergence, Core
  tree corruption, quoted-brace depth, app/single/entry gate bypass, dropped
  change copy-out, and whole-root element replacement makes the corresponding
  permanent test fail.
- Two fresh complete runs produce byte-identical stdout, stderr, exits, human/
  JSON facts, Core, graph, and audit evidence for identical inputs.
- Documentation claims match the shipped proof, including the exact remaining
  backlog below.

Full standing checks pass. Hard stop after an uncommitted 10F worktree for
independent review. Acceptance and publication of 10F close Work Order 10; they
do not authorize Session AR.

## Anti-ghost and discriminating evidence rules

These rules bind every increment:

1. Positive fixtures are hand-authored `.hum` programs and enter through
   `load_program` and the real applicable CLI orchestration. They observe the
   corrected value, complete list, exact Text, diagnostic, or zero-adapter
   result. Parse-only or declaration-only success is insufficient.
2. Focused unit tests may mutate an immutable fact or artifact, but cannot
   replace real-path positive evidence. Synthetic state is permitted only for
   an explicitly labeled unreachable corruption invariant and must traverse
   the real defensive branch.
3. Each positive test must fail under its named sabotage. The implementer
   demonstrates the sabotage locally, removes it, and reports the failure; the
   reviewer independently repeats representative nondegenerate sabotage.
4. Tests may not reconstruct expected identities from the same text, code,
   span, helper, or projection they validate. Upstream authority and downstream
   projection are independently supplied.
5. Existing public Core root summaries and IDs are projection evidence only.
   A test that proves only byte-identical public summaries cannot prove private
   child-tree structure, associativity, node identity, or span correctness.
6. No fixture may rely on an unused declaration, hard-coded task name, fixed
   line number, answer lookup, or output that the reproduced bug also emits.
7. Masking tests combine parser, resolver, type, Predicate, ownership, and
   authority failures to prove the fundamental owner retains precedence.
8. Every adapter-bearing blocked case injects counting output, replay,
   locality, and file adapters and asserts exact zero calls.
9. Temporary reviewer/implementer probes are removed and the final path/hash
   inventory is verified before handoff.

## Cross-stage evidence matrix

For every applicable positive and misuse fixture, each increment runs:

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
- `hum graph` with structural assertions, not substring-only checks;
- `hum ir-readiness` human and JSON without claiming IR exists; and
- `hum run` in every applicable app, legacy single-task, and explicit-entry
  mode, with counting adapters where blocked execution is at issue.

The matrix pins existing public schemas and representable projections,
diagnostic ordering and public spans, exit status, stdout/stderr bytes, exact
blocker identity, and repeatability. Private expression-tree node/route/span
identity is carried and checked separately inside the compiler and by mutation
tests before serialization/runtime; it is not required to appear in public
human or JSON. A stage may retain its established blocked status, but it must
reference the exact upstream fact and cannot reconstruct meaning. Matching
public root summaries alone never establish private-tree agreement.

## Diagnostics and decision gate

No active H-code is renumbered, repurposed, or allocated by default. Existing
messages, severity, help, blame, ordering, and exits change only where this
Work Order expressly authorizes corrected H0803/H0804 honesty wording or the
removal of false H0703 blame.

Stop for a separate architecture/decision amendment before:

- allocating a diagnostic code or family;
- reusing an H-code for a cause outside its registered meaning;
- changing which cause wins a combined case;
- changing typed-failure, bounds-failure, or invariant-channel semantics;
- adding a public expression/Core/graph/schema field; or
- making either P0 behavior a permanent rejection instead of implementing it.

The stopped report must preserve the worktree and name the exact unrepresentable
case, current wrong behavior, proposed diagnostic meaning, primary/related
sites, precedence, human/JSON/runtime behavior, and repair. The current
increment does not continue until that proposal is independently reviewed and
BDFL-authorized.

## Deferred known limitations and later routing

These records are backlog only. They authorize no implementation or claim:

- **Linear-resource leak through `try` (audit P1-2 behavior):** decision 0014
  already withholds full ownership/linear-path soundness, and decision 0018
  explicitly withholds general linear-resource checking. Preserve this as a
  known limitation for a future ownership/control-flow Work Order. 10F fixes
  only overclaiming help.
- **P2-1, invariant `Result` inconsistency:** backlog for a later invariant-
  channel cleanup after the expression/run foundation is stable.
- **P2-3, mixed error channels:** backlog and required input to Session AR's
  later IR-outcome typing. AR must distinguish ordinary values, typed
  operational failures, source-diagnostic blocking, and compiler invariant
  failure rather than flattening the current mixture.

The audit supplied no taste findings; none are invented or discarded.

The next ownership repair after this order remains whatever a later fresh
Work Order derives from accepted decision 0014 and live evidence. This order
does not reorder the accepted effect/capture, internal-reference, stdlib, or
adoption strategy.

## Standing bans

No increment may add or begin:

- Session AR, Hum IR, code generation, a backend, ABI, native execution, or
  self-hosting;
- the minimal compiler-ready standard library, Bytes/Path expansion,
  collections, allocation APIs, compiler-support packages, or adoption work;
- general indexing, slices, element aliases, retained references, internal
  references, broad disjoint-place inference, general lvalues, or reference
  types;
- exception, catch, recovery, unwind, implicit failure propagation, erased
  errors, or a global Result redesign;
- handlers, first-class callable environments, effect-model expansion,
  capture/ownership bridge, concurrency, scheduling, timeout, or parallelism;
- new IO, ambient authority, capability widening, sandbox claim, network,
  process, directory, manifest, hashing, signing, or file writing;
- speculative CI/evidence-integrity work, `PRE-AQ-INTEGRITY`, dashboard work,
  GitHub Issue #1 mutation, governance changes, or unrelated refactoring;
- a second expression parser, evaluator, brace counter, or runnable legacy
  bypass retained for compatibility; or
- claims of complete parsing, typing, effects, ownership, linear resources,
  memory safety, IR readiness, backend readiness, or standard-library
  readiness.

## Checks and configuration coverage

Each implementer handoff and each independent implementation review runs the
increment's focused real fixtures, mutation tests, and then:

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
git diff --check
.\tools\check_all.ps1
```

The report enumerates:

- ordinary Windows production and test builds;
- doctests if present;
- all-target warnings-denied Clippy;
- every fixture/human/JSON/runtime configuration affected;
- the Windows-locality and effect-bakeoff checks owned by preflight;
- text, public, and release readiness;
- every added or changed `cfg`, feature, optional/build dependency, generated
  source, or target-specific branch (none are authorized); and
- any locally unavailable configuration.

Only installed, already proven targets may run locally. Do not download a
target or mutate global configuration. Required post-push Ubuntu and Windows
CI remain the cross-platform authority.

## Work Order 10 acceptance criteria

Work Order 10 closes only when all six increments are independently accepted,
committed, published, green on required CI, and recorded, and evidence proves:

1. one parser-owned expression tree and one string-aware block-scoping fact;
2. body, Predicate v2, private Core/graph/static/runtime facts agree on the
   admitted expression semantics, while existing public Core human/JSON fields
   remain byte-compatible projections rather than full-tree proof;
3. `8 * 6 / 4` is 12 everywhere and byte-identical body/contract text cannot
   falsely blame the task;
4. quoted braces cannot alter item/block identity or diagnostic blame;
5. app, legacy single-task, and explicit `--entry` execution all consume the
   same sealed static eligibility fact;
6. no selected/reachable blocked, rejected, unchecked, or unsupported
   statement executes and every blocked path calls zero adapters;
7. `change` formals visibly write back to the exact caller place, including a
   transitive control, with the closed success/fallthrough/typed-failure/
   contract/preflight/invariant outcome table and exact zero-or-one ordered
   copy-out evidence;
8. direct numeric element assignment updates only the exact element and
   preserves the root list binding identity, length, other elements, and
   allocation honesty without claiming a public object-identity model;
9. H0803/H0804 wording claims only the exact checked path and all decisions'
   honesty locks remain intact;
10. every positive has real-path discriminating evidence and every named
    corruption fails closed;
11. public schemas, diagnostic meanings except the expressly corrected words,
    authority, ownership, typed-failure, callable, and IO behavior remain
    compatible; and
12. all standing checks pass with configuration gaps disclosed.

If any item is not proved, Work Order 10 remains open. Nearness, green narrow
tests, or absence of an observed failure is not completion evidence.

## Independent pre-issuance review

The fresh reviewer did not author, edit, generate, or directly direct this
file. The reviewer cold-starts from repository ground truth and checks:

- authority validity and Work Order 9 closure/freeze at `38704ac`;
- exact linkage from each promoted mandate to the five reproduced results and
  inspected production paths, with no invented audit detail;
- the one-seam boundary and exclusion of taste/backlog work;
- decision 0008 and decisions 0014-0018, including the linear-resource
  honesty locks;
- satisfiability and review sizing of 10A-10F;
- the global AR odometer reconciliation without renaming or preauthorizing AR;
- exact intent envelopes, stop-and-amend clauses, real fixtures, positive-
  evidence and anti-ghost gates, diagnostic precedence, configuration
  coverage, and hard stops;
- the closed `change` completion table, existing success-only `ensures:` rule,
  and discriminating zero/one ordered copy-out tests for every outcome;
- private authoritative expression/Core trees and per-node spans being checked
  before serialization/runtime while existing public Core schemas remain byte-
  compatible, with no inference from root summaries;
- default-correct implementation of both P0 behaviors and the narrow emergency
  deferral gate;
- P2-1/P2-3 and the actual linear leak remaining backlog only; and
- that only untracked `WORKORDER_10.md` exists, `WORKORDER.md` is unchanged,
  and the index is empty.

It runs:

```powershell
git diff --check
.\tools\check_all.ps1
```

It reports P0/P1/P2 findings with exact `WORKORDER_10.md` lines and exactly
one verdict: ACCEPT, ACCEPT WITH REQUIRED FIX, or REJECT. It does not edit,
commit, push, accept a decision, issue Work Order 10, authorize 10A, author
Session AR, or begin implementation.

An ACCEPT verdict authorizes only a BDFL-scoped documentation commit after
explicit BDFL acceptance. Publication and implementation remain separately
gated.

## Current authorization gate

Work Order 9 is closed and `WORKORDER.md` is frozen at baseline `38704ac`.
This Work Order 10 file is proposed and unissued. Its author is disqualified
from its verdict.

No prerequisite increment is authorized. The required next actor is one fresh
independent pre-issuance architect-reviewer. After ACCEPT, BDFL acceptance, a
scoped commit, separately authorized push, successful Ubuntu/Windows CI, and a
status record, Increment 10A still requires its own separate BDFL go signal.

Session AR remains the next globally lettered session but is reserved for a
future fresh Hum IR/minimal compiler-ready standard-library Work Order. It has
not been authored, reviewed, issued, or authorized here. 10F completion must
stop and return to the BDFL for the broad independent foundation audit before
any AR planning.

No commit, push, implementation, decision ruling, GitHub mutation,
`PRE-AQ-INTEGRITY`, dashboard, Session AR, or later work is authorized.

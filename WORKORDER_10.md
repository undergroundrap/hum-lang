# Hum Work Order 10: Pre-AR Semantic Foundation Repair

Date: 2026-07-15
Status: issued. The initial Work Order and local-train amendment remain
published as `49e6534a6cd3e4d567f924b69336c72563b1c95f` and
`334a7416e1014232d1e47e7be49ceb730fca33b3`. Increment 10A's accepted
implementation and local status commits,
`935550a4f40bcf425ddbc22f235b0011893219ae` and
`89c18ed363b78e725aa1a2736a24f21b08d29636`, are now published in the ordered
first-parent chain.

The bounded 10B rejection amendment was independently `ACCEPT`ed and
BDFL-accepted, including the exact H0010 allocation, meaning, sites, and
precedence. It was committed and published as
`098d5d3f2fa616c8faa3b6f4e4d8312f95f23ce7`. Workflow `29530510693`, attempt
1, succeeded for that exact commit. Ubuntu job `87729422199` succeeded in
10m 02s, including 9m 36s in `Run Hum preflight`; Windows job `87729422210`
succeeded in 17m 10s, including 16m 02s in `Run Hum preflight`. Both selected
`mode=full` with `reason=no_status_transition`; Cargo caching and Rust
toolchain preparation succeeded, and `Run status-only evidence` was skipped.

The bounded H0010 allocation-ripple amendment was independently `ACCEPT`ed and
BDFL-accepted. It was committed and published as
`ebc59fba003fb16540f2f8e37f8a5c4a5810d544`. Workflow `29539840435`, attempt
1, succeeded for that exact commit. Ubuntu job `87759532113` on
`ubuntu-latest` succeeded in 11m 09s, including 10m 34s in
`Run Hum preflight`; Windows job `87759532117` on `windows-latest` succeeded
in 13m 27s, including 12m 28s in `Run Hum preflight`. Both selected
`mode=full` with `reason=no_status_transition`, succeeded in Cargo caching and
Rust toolchain preparation, completed the full Hum preflight, skipped
`Run status-only evidence`, and concluded success. `HEAD`, local
`origin/main`, and live remote `main` now name that exact commit.

Increment 10B's monolithic implementation and bounded correction were each
independently `REJECT`ed with the same architectural finding shape. The current
correction tree remains preserved, uncommitted, and unaccepted as exactly 31
modified tracked implementation, documentation, and tool files plus six
untracked foundation fixtures, with an empty index. Its reproducible 31-path
Git snapshot tree OID is `0fac92602f632dbc145d641d73e74bd9ac15c545`.
The former shell-piped diff value remains historical only. The tree's 5,465
insertions and 4,143 deletions confirm that the former one-pass 10B unit is not
review-sized.

The repeated-rejection rule terminates that correction cycle; no third attempt
against the monolithic scope is authorized. The bounded re-scope amendment
below received final independent `ACCEPT` with no P0, P1, or P2 findings and
was BDFL-accepted, committed, and published as
`1dcccf6e1285ceee6d78ac7166bb166bed3126a1`. Workflow `29556009214`, attempt
1, succeeded for that exact commit. Ubuntu job `87808081590` on
`ubuntu-latest` succeeded in 10m 58s, including 10m 25s in
`Run Hum preflight`; Windows job `87808081602` on `windows-latest` succeeded
in 18m 08s, including 17m 24s in `Run Hum preflight`. Both selected
`mode=full` with `reason=no_status_transition`, succeeded in Cargo caching and
Rust toolchain preparation, completed the full Hum preflight, skipped
`Run status-only evidence`, and concluded success. The re-scope amendment is
durably published.

The preserved rejected 10B tree remains unchanged, uncommitted, and
unaccepted. Its ordinary green checks are implementation evidence only. 10B.0
is the next planning target but remains unauthorized. Disposition or deletion
of the preserved tree requires separate explicit BDFL authority, and 10B.0
requires a separate BDFL go signal after this status record is accepted and
durably published. No 10B.1, 10C, Session AR, Hum IR, standard-library,
backend, or later work is authorized.

The bounded rejected-tree archival/disposal amendment below was independently
`ACCEPT`ed and BDFL-accepted. It was committed and published as
`58ed878312338f5d056f30e1d00846f91a7cc953`. Workflow `29563872980`, attempt
1, succeeded for that exact commit. Ubuntu job `87832013964` on
`ubuntu-latest` succeeded in 10m 00s, including 9m 32s in
`Run Hum preflight`; Windows job `87832013993` on `windows-latest` succeeded
in 18m 42s, including 17m 52s in `Run Hum preflight`. Both selected
`mode=full` with `reason=no_status_transition`, succeeded in Cargo caching and
Rust toolchain preparation, completed the full Hum preflight, skipped
`Run status-only evidence`, and concluded success. The amendment is durably
published. Its publication status record was independently accepted,
BDFL-accepted, committed, and published as
`74828ae5f26b0d2eea069452b0a0c9080cd5581a`.

Under the subsequent explicit BDFL execution signal, the rejected tree was
archived from `$ArchiveBase`
`74828ae5f26b0d2eea069452b0a0c9080cd5581a` as the single commit
`3fdf236b0076534766ef89b592b3358f67a6315d` on the write-once local and live
remote branch
`archive/workorder-10b-rejected-monolith-2026-07-17`. The archive commit's
parent is exactly `$ArchiveBase`; its exact 37-path inventory is recorded in
the Current authorization gate below. Its 31 non-fixture paths reproduce Git
snapshot tree OID `0fac92602f632dbc145d641d73e74bd9ac15c545`, and all six
fixture blobs reproduce their frozen SHA-256 hashes. The recorded retrieval
command recovered
`fixtures/foundation/pre_ar_condition_chained_comparison_fail.hum` with exact
SHA-256 `c49bc27b53c2fbbfa8012525c25e756eb8da4871fe83ea2b6caec94466bc9d41`
before and after clearing. `git switch main` alone restored clean `main` at
`$ArchiveBase`; the index and untracked set were empty, the live archive ref
remained exact, and no workflow ran for the archive commit. No rejected byte
was merged into `main`.

The archive-execution status record was independently accepted, committed, and
durably published as `2e492e9e830a50dfd5e16bd9c7e22bd02043da3c`.
Required workflow `29604936061`, attempt 1, completed successfully for that
exact commit. Ubuntu job `87965789896` on `ubuntu-latest` succeeded in 11m11s;
its Cargo cache, Rust-toolchain preparation, and `Run Hum preflight` steps
succeeded, with preflight completing in 10m41s, while `Run status-only
evidence` was skipped. Windows job `87965789902` on `windows-latest` succeeded
in 20m22s; its Cargo cache, Rust-toolchain preparation, and `Run Hum preflight`
steps succeeded, with preflight completing in 19m32s, while `Run status-only
evidence` was skipped. Both jobs selected `mode=full` with
`reason=no_status_transition`.

The rejected monolith remains archived and recoverable at
`archive/workorder-10b-rejected-monolith-2026-07-17`, exact commit
`3fdf236b0076534766ef89b592b3358f67a6315d`; `main` remains cleared. The
rejected-tree archival lifecycle is complete.

Increment 10B.0's exact two-path dead-selector repair received final
independent `ACCEPT` after zero correction cycles and was committed and
published as `bc8140e668483ef2cb4042a5b8eb9a66caa820b9`, with first parent
`4b1030b79c9cbeab1afccc9e75d953062ad48f3b`. Its exact committed envelope is
`tools/check_all.ps1` and `tools/test_exact_rust_selector.ps1`. Required
workflow `29615977602`, attempt 1, succeeded for that exact commit. Ubuntu job
`88001070186` on `ubuntu-latest` succeeded in 10m 16s; Cargo caching and Rust
toolchain preparation succeeded, `Run Hum preflight` succeeded in 9m 48s, and
`Run status-only evidence` was skipped. Windows job `88001070180` on
`windows-latest` succeeded in 16m 10s; Cargo caching and Rust toolchain
preparation succeeded, `Run Hum preflight` succeeded in 15m 21s, and
`Run status-only evidence` was skipped. Both jobs selected `mode=full` with
`reason=no_status_transition`. The repair is complete and durably published;
the independently audited first-parent history confirms that no published
commit passed while the selector was dead. Increment 10B.1 is the next target
but remains unauthorized pending independent acceptance and durable
publication of this status record plus a separate explicit BDFL go signal.
10B.2, 10C, Session AR, and every later item remain unauthorized.

Increment 10B.1 was subsequently activated, implemented, independently
`REJECT`ed, corrected once, and independently `REJECT`ed again. Its single
correction cycle is exhausted. The second verdict confirmed one foundational
seal gap shared by operator/delimiter/call/depth and loop-binder facts, one
small discriminating-entry evidence gap, and one overbroad static source-audit
requirement that no text scanner can satisfy. The rejected work remains
uncommitted and unaccepted as exactly eight modified tracked paths plus five
untracked fixtures, with an empty index. Its exact 13-path Git subtree OID is
`af756a7fea21353794de585869a7d2df487fe663`, reproduced independently through
fresh empty temporary indexes in PowerShell and Git Bash.

The 10B.1 foundation re-scope amendment below received independent `ACCEPT`
with no P0, P1, or P2 findings and was BDFL-accepted, committed, and durably
published as `4fcd67473777751994c989363034e794a8624e5f`. Required workflow
`29705697959`, attempt 1, succeeded for that exact commit. Ubuntu job
`88242175358` on `ubuntu-latest` succeeded in 8m 15s, including 7m 35s in
`Run Hum preflight`; Windows job `88242175363` on `windows-latest` succeeded
in 16m 49s, including 16m 07s in `Run Hum preflight`. Both selected
`mode=full` with `reason=no_status_transition`; Cargo caching and Rust
toolchain preparation succeeded, and `Run status-only evidence` was skipped.

Under the subsequent explicit BDFL execution signal, the rejected 10B.1 tree
was archived from `$ArchiveBase`
`1584783763ac6eec3afd9b9850bde895d1b37365` as the single commit
`6f1caf857908c03769e5126eeba0df8af7d01b34` on the write-once local and live
remote branch `archive/workorder-10b1-rejected-2026-07-19`. Its parent is
exactly `$ArchiveBase`, its exact 13-path inventory reproduces subtree OID
`af756a7fea21353794de585869a7d2df487fe663` independently in PowerShell and
Git Bash, and all five fixture blobs retain their frozen SHA-256 hashes.
Byte-exact `git show "${ArchiveCommit}:<path>"` retrieval proved both
`src/parser.rs` and the condition-chain fixture before `git switch main`
performed the sole clearing operation. Local and live remote `main` remain at
`$ArchiveBase`; the worktree, real index, and untracked set are empty; the
archive ref remains exact; and no workflow ran for the archive branch. The
rejected-tree preservation lifecycle is complete.

Increment 10B.1a.1 subsequently received its separate BDFL go signal and
stopped at the mandatory size gate before review. The preserved uncommitted
tree changes only `src/ast.rs` by 92 insertions and `src/parser.rs` by 323
insertions and 6 deletions, with an empty index and no untracked paths. Those
421 changed lines already intermingle all fourteen planned identity facts
while omitting the independent catalogue, pair matrix, cross-occurrence
matrix, exact selector, and named sabotage evidence. The stopped work is not
accepted implementation evidence. Its exact two-path tree OID remains
`5dc0d187645fb9c84f0cddbb81eb344efde51a09`.

The bounded size-stop decomposition amendment below received independent
`ACCEPT` with no P0, P1, or P2 findings and was BDFL-accepted, committed, and
durably published as `c9284b4125127339e1a8ab56c456d71bec2c7aab`.
Workflow `29714772931`, attempt 1, succeeded for that exact commit. Ubuntu job
`88265749122` on `ubuntu-latest` succeeded in 10m 15s, including 9m 45s in
`Run Hum preflight`; Windows job `88265749118` on `windows-latest` succeeded
in 16m 17s, including 15m 32s in `Run Hum preflight`. Both selected
`mode=full` with `reason=no_status_transition`; Cargo caching and Rust
toolchain preparation succeeded, and `Run status-only evidence` was skipped.

The stopped two-path implementation remains preserved, uncommitted, and
unaccepted at the exact tree OID above. Narrowing remains unauthorized pending
independent acceptance and durable publication of this status record plus a
separate explicit BDFL signal. Increment 10B.1a.1.1 is the next target but
remains unauthorized. Increment 10B.1a.1.2, 10B.1a.2, 10B.1b, 10B.2, 10C,
Session AR, and every later item remain unauthorized.

Increment 10B.1a.1.1 subsequently received its separate BDFL go signal. Its
initial 499-line implementation was independently `REJECT`ed for two P1
findings: the retained authority constructed its own projection and therefore
preserved self-validation, and the `ProducerArm`, `ValidatorArm`, and
`EqualLengthEvidence` sabotages changed test bookkeeping rather than production
behavior. The single authorized correction cycle removed those shortcuts, but
Rustfmt produced an exact 507-line tree, crossing the published 500-line hard
ceiling. The implementer stopped before acceptance checks, fresh review,
commit, or push.

That exact corrected tree remains preserved, uncommitted, and unaccepted as
502 insertions and 2 deletions in `src/parser.rs` plus 2 insertions and 1
deletion in `tools/check_all.ps1`, with an empty index and no untracked path.
Fresh temporary Git indexes in PowerShell and Git Bash independently reproduce
its exact two-path tree OID
`70d248f77d4b851520b3a5960060b4c2d085a85b`. The proposed frozen-tree
size-stop amendment below grants no acceptance or implementation authority.
Increment 10B.1a.1.2 and every later item remain unauthorized.

The frozen-tree size-stop amendment received independent `ACCEPT` with no P0,
P1, or P2 findings and was BDFL-accepted, committed, and durably published as
`862f4a09f527b12e3ebf66059b7cef6be7c5d66c`. Workflow `29721006041`,
attempt 1, succeeded for that exact commit. Ubuntu job `88283875461` on
`ubuntu-latest` succeeded in 10m 14s, including 9m 51s in
`Run Hum preflight`; Windows job `88283875494` on `windows-latest` succeeded
in 16m 58s, including 16m 16s in `Run Hum preflight`. Both selected
`mode=full` with `reason=no_status_transition`; Cargo caching and Rust
toolchain preparation succeeded, and `Run status-only evidence` was skipped.

The corrected 507-line implementation remains frozen, uncommitted,
unaccepted, and byte-identical at two-path tree OID
`70d248f77d4b851520b3a5960060b4c2d085a85b`. No implementation acceptance
check has run against it. The only next eligible action is to reproduce that
exact OID, run the complete Increment 10B.1a.1.1 acceptance checks, and submit
those exact bytes for fresh independent implementation review. That action
remains unauthorized pending independent acceptance, commit, publication,
and terminal required CI for this status record plus a separate explicit BDFL
go signal. No implementation edit or further correction cycle is authorized.
Increment 10B.1a.1.2, 10B.1a.2, 10B.1b, 10B.2, 10C, Session AR, and every
later item remain unauthorized.

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
-> 10B.0 exact-test-selector integrity
-> 10B.1 canonical expression-occurrence and recursive H0010 closure
-> 10B.2 resolver and callable convergence
-> 10B.3 typed-failure / Path / return-dependency convergence
-> 10B.4 mutation / place / writable-alias convergence
-> 10B.5 Predicate v2 semantic-overlay convergence
-> 10B.6 type and full-type convergence
-> 10B.7 effect / ownership / resource convergence
-> 10B.8 Core construction and lowering convergence
-> 10B.9 Core verification and projection-transport convergence
-> 10B.10 runtime body-expression convergence
-> 10B.11 runtime contract-expression convergence and body/contract agreement
-> 10B.12 legacy expression/call authority retirement and 10B closure
-> 10C universal checked execution
-> 10D change-parameter write-through
-> 10E direct list-element assignment
-> 10F linear-help honesty and foundation closure
-> stop; Session AR remains unauthorized
```

No increment may begin before the previous one is independently accepted,
committed, published, green on Ubuntu and Windows, and recorded in this fresh
Work Order, followed by its own separate BDFL go signal.

## BDFL local-train amendment (2026-07-16)

### Purpose, activation, and exact authority

The initial Work Order 10 lifecycle above was independently accepted and fully
satisfied through publication of
`49e6534a6cd3e4d567f924b69336c72563b1c95f` and successful workflow
`29472827923`, attempt 1. The exact job and lane evidence is recorded in the
header and was independently reproduced through read-only GitHub inspection.

The BDFL now issues one Work-Order-local standing authorization to remove
repeated human prompt relay and six intervening remote-CI waits while retaining
the semantic and review gates of every increment. This amendment is a
substantive authorization change, not an exact routine status-only closure. It
is inactive until all of these activation facts exist:

1. a fresh independent architect-reviewer that did not author, edit, generate,
   or directly direct these amendment bytes returns `ACCEPT`;
2. the BDFL accepts those exact bytes and authorizes one scoped
   `WORKORDER_10.md` commit;
3. the BDFL separately authorizes that exact commit to be pushed;
4. the commit is the published `origin/main` head and the required Ubuntu and
   Windows jobs both select `mode=full`, complete `Run Hum preflight`
   successfully, skip `Run status-only evidence`, and terminate successfully
   for that exact commit; and
5. the local worktree and index are clean, with no untracked files, before 10A
   begins.

Pending, failed, canceled, skipped, missing, ambiguous, platform-incomplete,
or wrong-commit publication evidence leaves the amendment inactive. No agent
may infer activation from this proposal, from local tests, from the initial
`49e6534` publication, or from one green platform.

Once those five facts are proven, the BDFL's standing authorization activates
10A and, conditionally, 10B through 10F in the mandatory order. It replaces
only the repeated per-increment BDFL go signal, per-increment push, per-
increment remote CI, and human relay requirements. It does not let an agent
originate authority: each later activation is the direct consequence of this
exact BDFL instruction plus the closed conditions below. No stage silently
authorizes another.

### Closed local-train state machine

For each increment `10X`, in order, the complete cycle is exactly:

1. **Precondition.** The preceding train record is a committed first-parent
   ancestor, `HEAD` descends only from the published amendment commit, the
   index and worktree are clean, no untracked path exists, and both the local
   `origin/main` reference and a read-only live lookup of remote `main` still
   name the published amendment commit. 10A uses the activated amendment
   itself as its preceding record.
2. **One implementer pen.** One implementer cold-starts from repository ground
   truth, reports the exact integration map before editing, changes only the
   current increment's intent envelope, runs its focused real-path and
   sabotage evidence plus every standing local check, and leaves the complete
   deliverable uncommitted with an empty index for review.
3. **Fresh independent review.** One architect-reviewer that did not author,
   edit, generate, or directly direct that increment cold-starts independently,
   reviews only that complete deliverable against this Work Order, reproduces
   nondegenerate positive, misuse, masking, and sabotage evidence, runs every
   mandated local check, and returns exactly `ACCEPT`,
   `ACCEPT WITH REQUIRED FIX`, or `REJECT`. Green tests alone are never
   acceptance or continuation authority.
4. **At most one bounded correction.** On the first non-accept verdict, only
   the original implementer may apply the reviewer's bounded corrections. The
   independent reviewer then reviews the complete corrected deliverable once.
   A second verdict other than `ACCEPT` stops the train and returns the
   preserved worktree to the BDFL; no third cycle or local workaround is
   authorized.
5. **Exact implementation commit.** Only after final `ACCEPT`, the implementer
   makes the reviewer's exact scoped Conventional Commit. The commit contains
   the accepted increment paths only, has the expected first parent, and is
   not pushed. Commit authority here grants no remote, repair, decision,
   governance, or later-session authority.
6. **Exact local status record.** After that implementation commit, a bounded
   status-author may modify only the `Status:` body and
   `## Current authorization gate` body of `WORKORDER_10.md`. The first body
   starts after the unique column-one `Status:` prefix and ends immediately
   before the unique unchanged `Owner: BDFL (Ocean).` line. The second starts
   after the unique exact current-gate heading and runs to end of file. Every
   byte outside those bodies must match the implementation commit. The record
   may state only facts already proven: increment label, final verdict,
   correction-cycle count, implementation commit and parent, exact committed
   path envelope, standing check results and affected configurations,
   disclosed local coverage gaps, the clean worktree/index/untracked state
   observed after the implementation commit and before the status edit,
   unchanged published train anchor, and the next conditionally active
   increment or final stop. It may make no mandate, acceptance, scope,
   evidence, architecture, diagnostic, decision, governance, or
   implementation-contract change.
7. **Exact status commit.** The status author runs `git diff --check`,
   `./tools/check_text_hygiene.ps1`, `./tools/check_public_readiness.ps1`, and
   `./tools/check_release_readiness.ps1`, verifies that only
   `WORKORDER_10.md` changed, then commits only that file as
   `docs(workorder): record increment 10X local acceptance`. This temporary
   Work-Order-local authority is not the repository's general exact routine
   `WORKORDER.md` status exception and creates no precedent outside this
   train. The immediately preceding accepted implementation already ran the
   complete preflight; the factual record does not claim to rerun it.
8. **Automatic bounded advancement.** Only after both exact local commits are
   present in order and the worktree, index, and untracked set are clean does
   this standing BDFL authorization activate the next named increment. No
   acceptance, commit, or continuation is inferred merely from passing tests,
   an implementer report, an uncommitted status edit, or a reviewer finding
   without final `ACCEPT`. The next implementer and reviewer each verify that
   the status commit changed only the two exact bodies and that its recorded
   implementation parent, paths, verdict, and checks match repository facts.

The implementer and reviewer remain different pens for each increment. The
status author writes only after the verdict and cannot change or advocate that
verdict. An agent that authored an implementation remains forever disqualified
from its independent verdict even if another agent commits or records it.

### Ordered local history and remote prohibition

The published amendment commit is the train anchor. Every implementation and
local status commit must form one ordered, merge-free first-parent chain on
local `main` above that anchor. During 10A through accepted 10F:

- no push, pull, merge, rebase, reset, cherry-pick, force update, remote branch
  mutation, tag, release, pull request, or GitHub Issue mutation is authorized;
- no local increment may claim Ubuntu, Windows, remote CI, publication, or
  `origin/main` coverage;
- no fast-lane or full-lane result from an unpushed local commit exists or may
  be invented;
- `origin/main` must remain at the published amendment anchor; an unexpected
  local or remote history movement stops the train for the BDFL; and
- a commit with unexpected parents, paths, artifacts, staged content, merge
  ancestry, or untracked files stops before review, commit, or advancement.

This prohibition intentionally batches cross-platform evidence. It does not
turn local Windows evidence into Ubuntu evidence and does not weaken any
increment's complete host checks or manual inspection of platform-neutral and
non-host surfaces.

### Mandatory stops

The train stops immediately with all evidence preserved and returns to the
BDFL upon any of:

- a P0 finding;
- a genuine newly exposed production defect outside the current increment;
- material intent-envelope or file-scope expansion;
- a new diagnostic allocation/meaning/precedence requirement;
- a new decision, governance, authority, schema, public surface, dependency,
  feature, `cfg`, generated-source, or target-specific requirement;
- unexpected worktree, index, untracked, parent, branch, or remote history;
- inability to satisfy the current increment honestly within its reviewed
  scope or complete local evidence;
- a requested permanent rejection of either default-correct P0 repair;
- an unavailable or ambiguous required fact; or
- the second failed independent review after the one bounded correction.

The stop grants no repair, amendment, diagnostic, decision, push, rollback, or
later-increment authority. The BDFL chooses whether to amend, backlog, or
continue.

### Final cumulative publication and closure

After accepted 10F, its exact implementation commit, and its exact local status
commit, the train stops. Local `main` must be clean, have an empty index and no
untracked files, and be ahead of the unchanged `origin/main` amendment anchor
by the complete ordered train. No cumulative push is authorized by this
amendment.

The BDFL must separately authorize the exact cumulative push. The resulting
required Ubuntu and Windows CI jobs must both run the full lane for the exact
published head, exercise the cumulative 10A-10F implementation and standing
evidence, and terminate successfully. Any red, canceled, skipped, missing,
ambiguous, wrong-mode, wrong-commit, or platform-incomplete result stops for a
separately authorized bounded response. It is not routine cleanup.

After successful cumulative CI, one durable final status/closure record must
capture the train anchor, ordered implementation/status commits, pushed head,
workflow attempt, Ubuntu and Windows jobs, full-lane evidence, exact outcomes,
and disclosed configuration coverage. Work Order 10 closes only after that
record is independently or otherwise validly reviewed under then-applicable
repository rules, committed, published, and terminally green as required.

Session AR remains unauthorized throughout the train and cumulative CI.
Closing Work Order 10 still returns to the BDFL for the already mandated broad
independent foundation audit. Only after that audit may a fresh Work Order
author Session AR planning; no AR implementation follows implicitly.

### Exact supersession and preserved requirements

This dated amendment supersedes, for 10A-10F sequencing only:

- the header's original `proposed and unauthorized` issuance gate, which was
  satisfied by the independently accepted `49e6534` publication;
- the paragraph above requiring a separate BDFL go signal, push, Ubuntu/
  Windows CI, and status publication between every increment;
- each increment's implied per-increment remote publication/CI prerequisite;
- the 10F sentence that publication of 10F itself closes Work Order 10;
- the checks section's per-increment reading of required post-push CI;
- the acceptance criterion requiring each individual increment to be remotely
  published and green before the next begins; and
- the initial independent pre-issuance section's proposal/untracked-file
  assumptions, which are satisfied issuance history and are replaced for this
  amendment by its dedicated independent review gate; and
- the former Current authorization gate that required a new human go signal
  for 10A and every later increment.

Those statements remain historical issuance text where retained. Their
semantic replacement is the closed local-train state machine plus one final
cumulative publication and closure cycle above. Every increment's order,
intent, file envelope, bans, positive/misuse/mutation evidence, independent
review, complete local checks, configuration disclosure, exact local commit,
anti-ghost rules, diagnostic/decision stops, and uncommitted-review hard stop
remain fully binding. No accepted decision, `WORKORDER.md` closure, global
session odometer, Session AR reservation, or standing ban is changed.

This exception applies only to Work Order 10 increments 10A-10F after this
amendment activates. It does not amend `AGENTS.md` or governance, authorize a
second work order, generalize local trains, permit speculative CI/integrity
work, or grant future agents the power to create similar standing authority.

### Independent review gate for this amendment

The author of these amendment bytes is disqualified from their verdict. A
fresh independent architect-reviewer must verify the exact published
`49e6534` baseline and workflow evidence, authority validity, the activation
gate, every state transition, one-pen/reviewer separation, one-correction
limit, factual-status boundary, ordered first-parent history, mandatory stops,
final cumulative CI/closure, explicit supersession, unchanged increment
semantics, and absence of a global governance exception.

The reviewer confirms only `WORKORDER_10.md` changed, runs
`git diff --check` and `./tools/check_all.ps1`, reports P0/P1/P2 findings with
exact lines, and issues exactly one verdict: `ACCEPT`,
`ACCEPT WITH REQUIRED FIX`, or `REJECT`. It does not edit, commit, push,
activate 10A, implement an increment, accept a decision, or authorize Session
AR. Any semantic amendment after its verdict requires fresh complete review.

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

## Increment 10B rejection amendment (2026-07-16; proposed)

### Authority, rejection, and preserved implementation

This amendment is the bounded response to the final independent 10B verdict:
`REJECT`, with P0 none, three P1 findings, and P2 none. It neither accepts nor
modifies that implementation. Increment 10B's one correction cycle has not
begun. Increment 10C and every later increment remain unauthorized.

The preserved local history is:

```text
334a7416e1014232d1e47e7be49ceb730fca33b3  published local-train anchor
-> 935550a4f40bcf425ddbc22f235b0011893219ae  accepted 10A implementation
-> 89c18ed363b78e725aa1a2736a24f21b08d29636  accepted 10A local status record
-> uncommitted rejected 10B implementation
```

The rejected implementation remains exactly 18 modified tracked files, with
1,896 insertions and 932 deletions, plus these two untracked fixtures:

- `fixtures/foundation/pre_ar_left_associative_arithmetic_pass.hum`, SHA-256
  `22b2e9c09c9a5ed8f3984ccc08c318ff56922c5ebd89369092bf375f398ff3e9`; and
- `fixtures/foundation/pre_ar_body_contract_expression_agreement_pass.hum`,
  SHA-256
  `c40056a83eff8580e757ea6955892b98ce57d4325707a978f0df0603f4329381`.

The BDFL-pinned preservation fingerprint is the PowerShell pipeline:

```powershell
git diff --binary --no-ext-diff -- . ':(exclude)WORKORDER_10.md' |
  git hash-object --stdin
```

It returns `52f1ab82fe987678d6e2ef5d87c675fe99fde3cf` and remains the authoritative
preservation gate for this amendment and its review. A Git Bash raw-byte pipe
over the same diff returns the supplemental value
`01dc70da1d5e6573914a25686b12f56bb0914309`; that distinct shell transport must
not be substituted for or used to invalidate the pinned PowerShell value. The
exact path inventory, numstat, fixture hashes, and empty index also match the
rejected handoff.

### Complete pre-authoring production-rescan inventory

The repeated-rejection discipline requires one complete envelope audit before
authoring. The audit found the following authoritative rescans that must be
retired or made parser-owned for 10B to close.

Existing 10B-authorized files still requiring correction:

- `src/parser.rs`: `parser_expression_kind*`, `parse_expression_syntax`, and
  `parse_canonical_expression` independently rediscover expression shape;
  `top_level_binary_operator` currently admits a comparison chain.
- `src/predicate.rs`: `reachable_diagnostics`, `parse_syntax_for_parser`,
  `PredicateSyntaxBuilder`, and its arithmetic/place/call/literal helpers build
  and consume a second semantic grammar and call graph.
- `src/core_preview.rs`: Predicate lowering consumes `PredicateAst`; statement
  name/call/block helpers reconstruct private facts from display text.
- `src/core_lower.rs`: checked-return joins and brace blockers use text rather
  than exact parser node/block identity.
- `src/resolve.rs`: binding/loop/set/save/reference helpers reconstruct
  definitions, calls, and places from statement or atom text.
- `src/type_check.rs`: binding facts are text-derived, and comparison typing
  accepts a nested comparison as Bool without validating non-chainability.
- `src/full_type_check.rs`: builtin-call discovery, argument splitting,
  binding/place parsing, `infer_expression_type_text`, and comparison typing
  remain competing semantic recognizers.
- `src/effect_check.rs`: binding/set/save/place/resource helpers reparse
  statement text for effect ownership.
- `src/ownership_check.rs`: block depth, returned views, moves, calls, places,
  bindings, and set/save targets remain text-derived.
- `src/resource_check.rs`: allocation risk is inferred from raw `[`/`{`
  characters rather than canonical List/Record nodes.
- `src/run.rs`: reachable calls, Predicate evaluation, loop/binding/set/place
  structure, returned-view sources, and typed-failure wrapper facts still
  depend on text or the duplicate Predicate AST.

The complete additional production envelope, beyond the original 10B map, is
exactly:

- `src/typed_failure.rs`: replace `parse_try_expression`,
  `parse_direct_call`, `calls_in_expression`, call-span reconstruction,
  statement-expression extraction, argument splitting, and related production
  scans with parser canonical nodes, structured Try/wrapper facts, resolver
  call occurrences, and parser node ranges.
- `src/path_boundary.rs`: replace exact-file-read, Path-use, identifier-count,
  call-argument, and call-span text scans with canonical calls and resolver
  definition identities while preserving H0630.
- `src/return_dependency.rs`: retain result-annotation grammar, but replace
  returned-view call/group/argument/place parsing with canonical
  Call/Group/Permission/place traversal.
- `src/writable_field_alias.rs`: replace binding, permission, field-place,
  call, reference, set-target, and reconstructed block-depth scans with
  parser-owned binding/block facts, canonical nodes, and resolver place IDs,
  preserving H0808/H0809.
- `src/check.rs`: replace mutation/save/set target and loop-body structure
  rediscovery used by source diagnostics with parser-owned statement, target,
  binding, condition, and block facts. This grants no 10D/10E mutation
  behavior.
- `src/field_place.rs`: retain field-type declaration lookup, but confine or
  retire `split_field_place` as downstream expression authority; semantic
  consumers use structured place identity.

The audit explicitly excludes additional authority for:

- `src/element_place.rs`: direct-element raw parsing may remain parser-only;
  downstream semantic call sites must disappear, while type-annotation helpers
  remain non-expression grammar. 10E behavior remains unauthorized.
- `src/callable.rs`: it already consumes parser nodes and resolver IDs.
- `src/capability_root.rs`: it consumes resolver call facts and declaration
  grammar.
- `src/type_env.rs`: its remaining parsing is declaration/type syntax.
- `src/profile_check.rs`, `src/ir_readiness.rs`, `src/graph.rs`, and
  `src/json.rs`: the audit found projection/composition only, not an
  authoritative expression/place/call/block rescan.
- declaration/type helpers such as typed-failure result-root parsing,
  return-dependency result-annotation parsing, field-type lookup,
  `list_element_type`, Path type detection, public rendering, and native CLI
  argument parsing.

Any second request to add another production file for the same rescan class is
a repeated envelope failure and stops the correction for a new BDFL
backlog-versus-redesign decision. It is not another local amendment cycle.

### Predicate v2 root cause and required design resolution

Predicate v2 is structurally blocked by the current private representation,
not merely by an unfinished consumer swap.

Every accepted Predicate v2 value form is representable by the canonical tree:
places and fields, literals, lists, grouping, arithmetic, comparisons, and the
current `old`, `list_len`, and `list_count` calls. A permanent Predicate
`Expr`/`PredicateAst` is therefore unjustified duplicate semantic authority.
However, the current canonical tree alone cannot preserve these required
lexical and diagnostic facts:

- executable intent-signal presence and exact span, including quoted
  shielding;
- exact comparison-operator token span;
- call opening/closing delimiter, adjacency, separator, trailing-token, and
  empty-argument facts;
- precise malformed cause, offending range, expected token/text, and actual
  token/text;
- maximum delimiter depth for malformed input;
- missing or mismatched delimiters, unterminated Text, missing operands,
  invalid operator spelling or operand starter, malformed field place, list
  separator/trailing-comma/non-Text element, and out-of-range Int distinctions;
  and
- UTF-8-correct source-relative range conversion.

The correction must create one private parser-owned expression-occurrence
fact. It contains:

1. the canonical expression tree and stable parser node IDs;
2. exact node, token, operator, call-delimiter, argument-separator, and
   trailing-status ranges;
3. intent-signal and maximum-delimiter-depth facts; and
4. one structured lexical status carrying the exact malformed cause,
   offending range, expected evidence, and actual evidence.

Predicate v2 becomes a semantic restriction and overlay over that fact. It may
retain recognition status, task/section/line identity, Path/H0630 owner,
place-resolution and eligibility facts, resolver scope/root/field definition
IDs, operand types, and accepted comparison status, all keyed by canonical
node IDs. It may not retain a second successful value tree.

The correction deletes production semantic authority from
`PredicateSyntaxBuilder`, `PredicateAst`, Predicate `Expr` and `Place`, and
their duplicate arithmetic/comparison/call parsers. Runtime evaluates the
canonical tree under an explicit contract context; ordinary body context
cannot execute `old` or `list_count`. Core preview/lower keep their established
public Predicate projection, but derive it from canonical nodes plus the
Predicate overlay. Core verify independently joins and validates every
lowered Predicate operation against its exact parser-owned occurrence; it may
not verify only zipped `does:` expressions.

H0701 prose ownership, every non-chain H0704 cause and boundary fixture,
H0630 precedence, place rows, human/JSON bytes, and existing public Core/graph
schemas remain unchanged.

### Chained-comparison diagnostic decision gate

The body expression `1 < 2 < 3` currently passes check/type/full-type/Core/IR
and then reaches the generic runtime trap `expected Int value, got Bool`.
This violates the already issued non-chainable grammar and fail-closed rule.
The earliest owner is the shared expression parser.

The registry audit found no existing exact owner:

- H0704 and its comparison causes are Predicate-only;
- H0606 is a task return-type mismatch and does not own a Bool-return or
  condition chain; and
- no active H000x or H060x cause means a shared body/contract comparison chain.

Therefore this amendment reaches the diagnostic decision gate. It proposes,
but does not allocate before BDFL acceptance:

- proposed code: `H0010`, the next free code in the active
  `H0000-H0099` `source_shape` family;
- title/message: `comparison chaining is not supported`;
- proposed stable cause: `chained_comparison_not_supported_v0`;
- severity, semantic owner, and owning stage: error, shared expression syntax,
  parser;
- cardinality: one occurrence per chained expression;
- primary site: the later comparison operator that would turn an existing
  comparison into a chain;
- related site: the first comparison operator, labeled as the comparison
  already being chained;
- help: repeat the middle operand and join the independent comparisons, for
  example `1 < 2 and 2 < 3`;
- human/JSON: identical code, severity, message, help, primary span, and
  structured related span;
- runtime: source rejection before selection, argument conversion, bodies, or
  adapters; zero stdout, no generic trap, normal source-error exit; and
- precedence: item/block failures that prevent a trustworthy expression remain
  earlier; once the occurrence exists, H0010 dominates resolver/type,
  Predicate H0704, Path H0630, ownership, authority, and runtime consequences
  for that same occurrence, while unrelated occurrences remain independent.

`a < b and b < c` and comparison-looking Text remain accepted controls. This
is the only authorized Predicate diagnostic compatibility change: a chained
Predicate candidate moves from H0704 to the shared proposed H0010 owner. Every
other H0704 meaning and byte remains frozen.

If the BDFL does not explicitly accept this exact proposed allocation,
meaning, sites, and precedence, 10B correction remains stopped. The
implementer may not reuse H0704, H0606, H0009, or another nearby code.

Upon explicit BDFL acceptance, the diagnostic portion adds exactly:

- `src/diagnostic_catalog.rs` for one allocation/cause and checked registry
  evidence; and
- `docs/DIAGNOSTICS.md` as the checked public projection.

No new diagnostic family, JSON field, schema identifier, global precedence
rank, or independent document ledger is authorized.

### Single correction-cycle scope and evidence

The original 10B envelope remains authorized only for its stated intent. This
amendment adds only the six production files above and, conditionally upon the
exact diagnostic ruling, the two diagnostic projection files. It adds these
exact real fixtures:

- `fixtures/foundation/pre_ar_return_chained_comparison_fail.hum`;
- `fixtures/foundation/pre_ar_condition_chained_comparison_fail.hum`;
- `fixtures/foundation/pre_ar_predicate_chained_comparison_fail.hum`; and
- `fixtures/foundation/pre_ar_comparison_conjunction_pass.hum`.

The correction must preserve the current 10B arithmetic/body-contract
positives and both existing fixture hashes. It must additionally prove:

- body execution, Predicate typing/evaluation, runtime, private Core, and every
  static consumer use one exact canonical occurrence;
- body and contract chain fixtures produce exactly one proposed H0010 at the
  later operator with the first operator as related evidence, in human and
  JSON, and never reach runtime or any adapter;
- the conjunction control runs successfully, and comparison-looking Text does
  not create a chain;
- all existing Predicate v2 positive/misuse fixtures remain exact except the
  expressly changed chained-comparison owner;
- unsupported `try` remains exactly nine H0906 rows, and typed-failure causal
  sites remain exact;
- H0630, H0701, H0702, H0703, all non-chain H0704 causes, H0808/H0809, callable
  facts, authority facts, returned-view facts, and public Core type/status
  projections remain unchanged;
- every lowered Predicate operation is joined to and verified against its
  parser-owned canonical occurrence before serialization/runtime; and
- two fresh complete human/JSON/Core/graph/runtime runs are byte-identical.

Permanent source audits strip `#[cfg(test)]` bodies and fail on behavior, not
renamable symbol spellings. They require:

- no production Predicate successful AST/parser/evaluator besides the
  canonical occurrence;
- no production use of `typed_failure::calls_in_expression`,
  `statement_expression`, `parse_try_expression`, `call_span_in_statement`, or
  `call_span_for_identifier_use`;
- no `infer_expression_type_text`, builtin expression argument splitter, or
  semantic call graph derived from expression text;
- no semantic `canonical_text(...)` fed into typed-failure,
  return-dependency, alias, place, call, or block parsing;
- no semantic `field_place::split_field_place` consumer;
- `element_place::split_element_place` is parser-only;
- no `statement.text.ends_with('{')` block authority;
- no allocation decision from raw `[` or `{`;
- no raw set/save/binding/loop target helper in check, resolver, effect,
  ownership, Core, or runtime; and
- an explicit allowlist limited to parser lexical helpers, declaration/type
  grammar, public display projection, and native CLI argument parsing.

Named sabotage must independently turn the relevant permanent test red:

- poison retained body or contract text while holding canonical nodes fixed;
- mutate canonical Predicate child/operator/group/node ID while holding text
  and public root summary fixed;
- substitute a same-text foreign Predicate occurrence;
- corrupt operator/token span, call-gap, delimiter, separator, trailing-comma,
  lexical-issue, or place-definition join;
- re-enable a second Predicate AST or raw call scanner;
- poison quoted `source()` text, same-text duplicate calls, nested calls,
  grouped `slice_until`, field spelling, and braces/brackets in Text while
  keeping canonical/resolver facts fixed;
- route alias/place/block/allocation checks back through display text;
- disable chain detection, blame the first operator, omit the related site, or
  treat `and`-separated comparisons/quoted operators as a chain; and
- corrupt an accepted private tree into a chain while keeping the public root
  projection unchanged.

Tests may not reconstruct expected identities from the artifact being
validated. Existing anti-ghost, positive-evidence, masking, cross-stage,
configuration, compatibility, and hard-stop rules remain unchanged.

### Amendment and correction gates

This amendment is proposed and unauthorized. Its author is disqualified from
its verdict. One fresh independent architect-reviewer must verify the complete
inventory, Predicate satisfiability resolution, proposed H0010 allocation and
precedence, exact envelope, preservation fingerprint correction, evidence,
bans, and unchanged 10C-10F mandates. The reviewer runs:

```powershell
git diff --check
.\tools\check_all.ps1
```

and returns exactly `ACCEPT`, `ACCEPT WITH REQUIRED FIX`, or `REJECT` without
editing.

Final `ACCEPT` authorizes no implementation. Before correction may resume:

1. the BDFL explicitly accepts these exact amendment bytes and the proposed
   H0010 allocation/meaning/sites/precedence;
2. `WORKORDER_10.md` alone is committed with the rejected 10B worktree
   preserved;
3. the BDFL separately authorizes publication; that push necessarily publishes
   the already accepted 10A implementation/status commits plus this amendment,
   but no uncommitted 10B implementation;
4. required Ubuntu and Windows full CI succeeds for that exact amendment head,
   and the publication evidence is durably recorded; and
5. the BDFL gives one separate explicit corrective go signal.

That go signal resumes only 10B's single correction cycle. It does not reset
the cycle count, authorize a second envelope amendment, or authorize 10C.
After correction, one fresh independent reviewer examines the complete
corrected 10B tree. Any further non-`ACCEPT`, another same-class file request,
or another diagnostic/public/schema requirement stops the train for a BDFL
backlog-versus-redesign decision.

## Increment 10B H0010 allocation-ripple amendment (2026-07-16; proposed)

### Authority and current hard stop

This amendment responds only to the new test-only envelope blocker discovered
after the accepted H0010 allocation entered the preserved 10B correction tree.
The implementer confirmed the three known `src/diagnostics.rs` literal changes,
reverted them, and stopped. This document does not accept the incomplete 10B
implementation or resume its correction cycle. Increment 10C and every later
item remain unauthorized.

The verified planning baseline is `main` with `HEAD == origin/main ==
812a3766e041f4275f6d770e753a37c17c7cc250`, an empty index, clean
`WORKORDER_10.md` and `src/diagnostics.rs`, and the preserved incomplete 10B
tree named in the handoff. The current tree contains 25 modified tracked 10B
paths plus the six authorized foundation fixtures; no 10C or unrelated work is
present.

### Complete H0010 allocation-ripple inventory

The complete repository sweep separates numeric identities from catalog-size
expectations and finds exactly these four allocation-ripple surfaces:

1. `src/diagnostic_catalog.rs`, already authorized for H0010, contains the
   independent registry assertions. `summary.active_codes` already expects
   88, while `DIAGNOSTIC_CAUSES.len()` still expects 178 and must expect 179
   after `chained_comparison_not_supported_v0` is registered. The allocation,
   family, public ordinal, cause identity, and dynamic registry machinery are
   not otherwise changed by this amendment.

   The 179 count is semantic, not inferred from fixture cardinality. Condition,
   Predicate, and return chains each express the same fundamental parser-owned
   cause: a later comparison operator applies to the result of an earlier
   comparison. All three enter the same producer with cause key 179,
   `parser_expression_node` relationship, and `parser_expression_route` route
   class. Their exact parser semantic node, producer event, ordered route, and
   source sites create three distinct occurrences of that one registered
   cause. Section context changes neither the cause meaning nor its owner. If
   production evidence instead requires a second cause, the count may not be
   guessed upward locally; implementation stops for a fresh diagnostic
   amendment.
2. `docs/DIAGNOSTICS.md`, already authorized as H0010's checked public
   projection, contains the catalog JSON example whose literal `count` must
   change from 87 to 88. Its H0010 row remains the only new diagnostic row.
3. `tools/check_all.ps1`, already authorized for 10B evidence, contains one
   three-part catalog assertion: reported count, row count, and unique row
   count must each change from 87 to 88, including the corresponding failure
   text.
4. `src/diagnostics.rs` is the sole additional file. Its
   `registry_catalog_and_check_projections_are_semantically_equivalent` test
   independently pins `catalog.len()`, the human heading, and the JSON count;
   those three literals must change from 87 to 88.

No schema document, generated artifact, readiness script, enumeration test,
or other production/test surface contains another live catalog-size
expectation. The 87 references in frozen `WORKORDER.md` are historical Work
Order 9 baseline evidence and remain unchanged. Numeric 87 values used as
opaque diagnostic-code keys, cause keys, or historical public ordinals are
identities rather than catalog counts and remain unchanged unless the already
accepted H0010 registry design expressly owns them.

### Exact current-red accounting

An independent `cargo test` run at the planning baseline executed 430 tests:
419 passed, 11 failed, zero were ignored. Every failure is accounted for:

- H0010 allocation-count ripple:
  - `diagnostic_catalog::tests::canonical_registry_and_checked_projections_are_valid`
    observes 179 registered causes while its independent literal still says
    178; and
  - `diagnostics::tests::registry_catalog_and_check_projections_are_semantically_equivalent`
    observes 88 codes while its three independent literals still say 87.
- Intentionally incomplete canonical-expression/path/Predicate convergence:
  - `path_boundary::tests::accepts_only_exact_hardened_file_read_consumption_of_path`;
  - `predicate::tests::retained_contract_text_cannot_override_parser_owned_predicate_syntax`;
  - `run::tests::exact_file_read_writes_checked_utf8_and_joins_forensic_events`;
  - `run::tests::file_authority_precedence_rejects_before_locality_or_candidate_adapter`;
  - `run::tests::locality_and_every_file_adapter_failure_are_typed_and_causal`;
  - `run::tests::integrated_local_app_exact_denies_precede_their_adapters`;
  - `run::tests::integrated_local_app_missing_file_keeps_outer_to_root_cause`;
  - `run::tests::integrated_local_app_is_repeatable_for_complete_inputs`; and
  - `run::tests::predicate_preflight_aggregates_all_independent_h0704_rows`.

The path/runtime failures stop at the unfinished canonical Path/call migration,
the Predicate retained-text test exposes unfinished canonical range transport,
and the aggregate test still expects the superseded H0704 ownership for the
one chain now correctly surfaced as H0010. None is absorbed as a generic
"expected red," and none identifies a defect outside the already accepted 10B
correction. Any later failure that cannot be traced to these exact unfinished
relationships or the four allocation-ripple surfaces is a newly exposed
defect and stops the cycle for BDFL triage.

### Bounded authorization and anti-ghost lock

After independent review, BDFL acceptance, a `WORKORDER_10.md`-only commit,
publication, terminal required CI, a durable status record, and a separate
corrective go signal, `src/diagnostics.rs` joins 10B's envelope only for the
three literal 87-to-88 assertions in
`registry_catalog_and_check_projections_are_semantically_equivalent`.

The test literals are independently supplied expectations. They may not be
derived from the canonical registry or the public projection being validated.
Production `diagnostics_text()` and `diagnostics_json()` must continue deriving
their count from the canonical registry; no production rendering path may gain
a hard-coded count. This preserves the anti-ghost rule that the validator does
not reconstruct its expected answer from the artifact under validation.

The already authorized edits above are limited to the one cause-count literal
in `src/diagnostic_catalog.rs`, the one JSON example count in
`docs/DIAGNOSTICS.md`, and the exact catalog-count assertion and failure text in
`tools/check_all.ps1`. No production rendering, schema, ordering, message,
help, severity, diagnostic meaning, registry behavior, family, public ordinal,
or other test may change under this amendment. A further H0010 allocation-
ripple file request is a repeated envelope failure and stops for a BDFL
backlog-versus-redesign decision; it is not another local amendment.

### Review and resume gates

This amendment is proposed and unauthorized. Its author is disqualified from
its verdict. A fresh independent architect-reviewer must verify the complete
ripple inventory, exact 11-failure accounting, literal-expectation anti-ghost
lock, one-cause/three-occurrence H0010 identity, one-file envelope addition,
preserved dirty tree, empty index, bans, and unchanged 10C-10F mandates. The
reviewer runs `git diff --check`; complete local preflight is transparently
expected to remain red while the authorized 10B correction is intentionally
incomplete and must not be weakened or repaired during document review.

Final `ACCEPT` authorizes no implementation. Resumption still requires BDFL
acceptance of these exact bytes, a `WORKORDER_10.md`-only commit, separately
authorized publication with terminal Ubuntu and Windows CI, a durable status
record, and a separate BDFL corrective go signal. No commit, push, 10B source
edit, 10C work, Session AR work, or later action is authorized by this text.

## Increment 10B repeated-rejection re-scope amendment (2026-07-16; proposed)

### Authority, repeated-rejection diagnosis, and hard stop

The BDFL authorized this architecture/documentation pass only after the
monolithic 10B implementation and its bounded correction received two
independent `REJECT` verdicts with the same architectural shape:

| First verdict | Second verdict | Same underlying failure |
| --- | --- | --- |
| Predicate v2 retained an independent semantic AST | a duplicate raw-text expression/call graph remained authoritative | canonical parser authority was claimed while a parallel recognizer still supplied semantics |
| top-level chained comparisons reached a generic runtime trap | nested chained comparisons still reached the generic trap | H0010 recognition was not recursive over the admitted expression tree |
| typed-failure analysis rescanned statement text | runtime, effect, and ownership rendered canonical expressions and reparsed them | downstream consumers reconstructed meaning instead of consuming producer-owned structure |

This is the repository's repeated-rejection loop condition. The canonical
parser-owned expression tree remains the accepted architecture; 10A already
proved its operator order, identity, string-aware scope, and corruption
boundary. The failed unit was the attempted all-consumer migration, not that
architecture. The former single correction cycle is terminated. No third
implementation attempt against the original `Prerequisite Increment 10B`,
`Increment 10B rejection amendment`, or `Increment 10B H0010
allocation-ripple amendment` scope is authorized; those sections remain only
historical evidence of the rejected approach.

This section prospectively supersedes the original single-unit 10B scope, the
first rejection amendment's one-correction-cycle/resume language, the H0010
ripple amendment's resumption gate, and the local-train correction rules only
as they apply to monolithic 10B. Their accepted history remains intact. The
H0010 allocation and public meaning survive unchanged. The 10C-10F mandates,
all accepted decisions, and every standing ban remain unchanged.

The current uncommitted tree is preserved exactly for this planning pass:

- `HEAD == origin/main ==
  5691e17b1a7d01b036007cdce39108471df94641`;
- the index is empty;
- 31 tracked files are modified, with 5,465 insertions and 4,143 deletions;
- the exact tracked implementation diff excluding this amendment, produced by
  `git diff --binary --no-ext-diff -- . ':(exclude)WORKORDER_10.md' |
  git hash-object --stdin`, is
  `539d384ad5b0220095e1845f45a5d6ea6e050394`; and
- the six untracked fixtures are:
  - `fixtures/foundation/pre_ar_body_contract_expression_agreement_pass.hum`,
    SHA-256
    `c40056a83eff8580e757ea6955892b98ce57d4325707a978f0df0603f4329381`;
  - `fixtures/foundation/pre_ar_comparison_conjunction_pass.hum`, SHA-256
    `3081b3ba84045cb64bb8c049fde683cbfc64c91dae956e9a71d012b193951433`;
  - `fixtures/foundation/pre_ar_condition_chained_comparison_fail.hum`,
    SHA-256
    `c49bc27b53c2fbbfa8012525c25e756eb8da4871fe83ea2b6caec94466bc9d41`;
  - `fixtures/foundation/pre_ar_left_associative_arithmetic_pass.hum`,
    SHA-256
    `22b2e9c09c9a5ed8f3984ccc08c318ff56922c5ebd89369092bf375f398ff3e9`;
  - `fixtures/foundation/pre_ar_predicate_chained_comparison_fail.hum`,
    SHA-256
    `7376de1f01f018943174876886ea37da02e5a4458b032d24214f5ff4116e8d30`;
    and
  - `fixtures/foundation/pre_ar_return_chained_comparison_fail.hum`, SHA-256
    `6096390130a62ddc5a2128b936b188d05a0a63aa9036cad4ffd84c7b16207fb8`.

This amendment neither edits nor accepts that tree. Before any re-scoped
subincrement begins, the BDFL must separately authorize its exact preservation
or disposal and restoration to a clean descendant of the published 10A
baseline. No implementation go signal implicitly authorizes a reset, checkout,
stash, deletion, or other destructive cleanup.

### Consumer diagnosis: representation gaps versus migration work

The code audit distinguishes facts the 10A representation does not yet carry
from consumers that merely have not been migrated:

| Consumer boundary | Why text or parallel structure remains authoritative | Classification and required resolution |
| --- | --- | --- |
| parser expression occurrence | `ParsedCallSyntax` synthesizes delimiters and separators from child endpoints, carries no exact call-gap/adjacency fact, `Try` retains a wrapper string, and chain detection checks only the root | representation gap; 10B.1 must record actual tokens/ranges, structured failure wrapper identity, complete statement facts, and recursive chain ownership |
| resolver and callable analysis | both still traverse `ParsedExpressionKind`; callable analysis therefore uses the parallel call graph even though resolver IDs exist | migration effort after 10B.1; consume canonical call nodes plus exact parser positions and resolver definitions |
| typed failure | canonical traversal exists, but `Try` wrapper identity is reparsed from its string and older statement/call scans remain reachable | one parser representation gap plus migration effort; structured wrapper fact lands in 10B.1, consumer migration in 10B.3 |
| Path and return dependency | accepted call, place, group, permission, and return facts are representable by canonical nodes; result and Path type text are declaration grammar | migration effort only; keep declaration parsing, remove expression rescans in 10B.3 |
| check, field place, writable alias | set/save/loop/binding/block ownership needs parser-owned statement facts that clean 10A does not yet expose completely | representation gap closed in 10B.1, then migration effort in 10B.4 |
| Predicate v2 | accepted values fit the canonical tree, but exact malformed cause, intent signal, token ranges, delimiter depth, call adjacency, and resolver place joins require an overlay | representation gap closed by the parser occurrence fact in 10B.1; Predicate becomes a restriction/overlay, never a second value tree, in 10B.5 |
| type and full type | the canonical tree already represents every admitted expression fact these stages own | migration effort only in 10B.6; display text may remain output but cannot feed inference |
| effect, ownership, resource | set/place/resource roots are currently obtained by `canonical_text` followed by `first_resource` or equivalent splitting | migration effort only in 10B.7; use structured place roots, resolver uses, and List/Record nodes |
| Core construction/lowering/verification | current work contains useful private-tree and corruption logic, but some joins still use rendered text or regenerated projections | migration and independent-verification work in 10B.8-10B.9; public text remains projection only |
| runtime body evaluator | the operator evaluator is substantively salvageable, but fail/wrapper/place joins still render and reparse | migration effort in 10B.10 after static consumers converge |
| runtime contract evaluator | it still depends on Predicate's parallel representation and must share the body evaluator without reconstructing source | migration effort in 10B.11 after 10B.5 and 10B.10 |
| graph, JSON, profile, IR readiness | the audit found projection, composition, and blocker transport rather than an independent expression grammar | no semantic parser migration; 10B.9 must prove they validate or project separately supplied upstream facts without becoming another authority |
| legacy expression/call graph | `ParsedExpressionKind`, `ParsedCall`, and `parser_owned_top_level_call_ranges` remain usable by production consumers | intentional temporary compatibility only; they are deleted or made test-inaccessible in 10B.12 after the last consumer migrates |

Public rendering, declaration/type grammar, native CLI argument parsing, and
diagnostic message construction are not expression authority. They may remain
only where a dataflow audit proves that their output cannot return to semantic
recognition, selection, inference, evaluation, identity, precedence, or
validation.

### Selective salvage and clean-baseline rule

The preserved 9,608-line patch is not salvageable as one implementation unit.
Its shims, genuine migrations, documentation, and tests are interleaved across
the same files, so continuing it would make every later review depend on
unaccepted earlier bytes. Every 10B subincrement therefore starts from a clean
committed descendant of the accepted 10A implementation, not from the dirty
tree and not from a bulk-applied patch.

This is selective rather than blanket rejection:

- the H0010 public allocation, one-cause/multiple-occurrence model, exact sites,
  fixtures, and catalog-ripple expectations are retained as requirements;
- the parser lexical-cause model, parser-owned statement-fact idea, Predicate
  semantic-overlay design, canonical traversal helpers, private Core mutation
  catalog, and shared arithmetic evaluator are acceptable design evidence and
  may be reimplemented after re-derivation against the clean boundary;
- direct canonical traversals that already avoid text reconstruction may be
  reintroduced in their owning subincrement after independent source audit;
- no hunk containing a raw scanner, rendered-value semantic parse, synthesized
  delimiter/call fact, parallel successful AST, public-summary self-validation,
  spelling-only audit, or zero-test selector is salvageable; and
- tests from the rejected tree are claims to reproduce, not evidence to copy.
  Each must be rebuilt around the real production entry point and must fail
  under the named sabotage before it can be credited.

No actor may mechanically cherry-pick, apply, or copy the rejected patch as an
increment. Reuse is by independently justified behavior and structure, with
the subincrement's exact envelope and review gate.

### Re-scoped dependency sequence and exact envelopes

The real dependency graph creates the following bounded units. Each is a
Work-Order-local subincrement, not a global session. Each receives a separate
BDFL go signal only after every predecessor is independently accepted,
committed, published, terminal-green on Ubuntu and Windows, and durably
recorded.

#### 10B.0: exact-test-selector integrity

Exact envelope:

- `tools/check_all.ps1`; and
- new `tools/test_exact_rust_selector.ps1`.

This is harness integrity, not semantic 10B implementation. It must make every
exact Rust selector in `check_all.ps1` prove that its requested fully qualified
test exists and that exactly one test ran. Zero, duplicate, ambiguous, renamed,
filtered, malformed, or unavailable selection fails before the selected test is
credited. The helper and its tests remain dependency-free and platform-neutral.

The specific dead selector
`typed_failure::tests::exact_call_spans_and_identifier_ownership_fail_closed`
was introduced with a real test in commit `58ad265` on 2026-07-13 and remains
live on published `HEAD`. It became dead only inside the current uncommitted
10B rewrite when that test was removed while its selector remained. Therefore
no accepted commit or remote CI passed while this selector was dead, but local
10B preflights incorrectly reported success for `running 0 tests`.

The deleted test was intended to prove all of the following old-boundary
properties: two same-line calls have different exact positions; an identifier
used by only the second call binds to that call; an identifier used by two
sibling calls fails closed as ambiguous; repeated same-text calls remain
distinct; and an identifier with no call owner fails closed. 10B.0 preserves
that intended coverage as a named inventory. Later canonical resolver tests may
replace the obsolete scanner-specific form, but the selector may not silently
disappear. Permanent sabotage must rename or delete a selected test, add an
ambiguous matching test, and select a nonexistent test; each makes the harness
red. No compiler source, diagnostic, fixture, or semantic behavior may change.

#### 10B.1: canonical expression occurrence and recursive H0010

Exact envelope:

- `src/ast.rs`;
- `src/parser.rs`;
- `src/core_body.rs`;
- `src/diagnostic_catalog.rs`;
- `src/diagnostics.rs`;
- `docs/DIAGNOSTICS.md`;
- `docs/LANGUAGE_REFERENCE.md`;
- `tools/check_all.ps1`; and
- the five exact fixtures
  `pre_ar_comparison_conjunction_pass.hum`,
  `pre_ar_condition_chained_comparison_fail.hum`,
  `pre_ar_nested_chained_comparison_fail.hum`,
  `pre_ar_predicate_chained_comparison_fail.hum`, and
  `pre_ar_return_chained_comparison_fail.hum` under `fixtures/foundation/`.

The parser must produce one expression-occurrence fact containing the canonical
tree, stable parser node/child positions, actual operator and delimiter token
ranges, actual call-open/call-close/separator/trailing/gap facts, intent signal,
delimiter depth, structured lexical status, exact statement binding/set/save/
loop/condition facts, and a structured typed-failure wrapper root/variant. No
call syntax may be inferred from child endpoints, and no wrapper may be stored
only as a string that a consumer must parse.

H0010 remains exactly one registered parser cause and one public code. The
parser recursively visits every admitted canonical child, including groups,
Boolean operands, calls, lists, records, permissions, and Try nodes. It rejects
`(1 < 2 < 3) and true`, chains nested on either side of another operator, and
the three context fixtures before resolver or runtime. It does not reject
`1 < 2 and 2 < 3`, comparison-looking Text, or independent grouped
comparisons. The later comparison token is primary, the first comparison token
is related, and the existing model-neutral repair is preserved. Nested cases
exit through H0010 with zero stdout and no generic trap. Catalog totals become
exactly 88 active codes and 179 causes; one fixture never creates another
registered cause. The dedicated nested fixture must run through the real parser
and permanent human, JSON, and runtime matrices. An independent recursive
tree-corruption sabotage must move or remove the nested comparison child while
holding source and retained text fixed and must change the owned H0010
occurrence or fail closed before resolver/runtime; a root-only scan cannot pass.

#### 10B.2: resolver and callable convergence

Exact envelope: `src/resolve.rs`, `src/callable.rs`, and
`tools/check_all.ps1`.

Resolver definitions, references, call occurrences, targets, arguments, and
callable-value uses must traverse canonical nodes and exact parser child
positions. `ParsedExpressionKind`, `ParsedCall`, line text, display names,
spans, and a separately scanned call graph cannot select or mint semantic
identity. Callable behavior and H1401/H1402 public bytes remain unchanged.
Repeated same-text, same-line, nested, shadowed, and sibling calls remain
distinct through resolver-owned identity.

#### 10B.3: typed-failure, Path, and return-dependency convergence

Exact envelope: `src/typed_failure.rs`, `src/path_boundary.rs`,
`src/return_dependency.rs`, and `tools/check_all.ps1`.

Typed-failure call/wrapper ownership must use structured canonical nodes and
resolver call occurrences. Path consumption and return-view dependencies must
use exact call, permission, group, and place identities. Result annotations,
Path type detection, and nominal failure declarations may retain declaration
grammar, but expression text cannot be scanned or rendered and reparsed. H0630
and H0901-H0907 meanings, ownership, precedence, and public behavior remain
unchanged.

#### 10B.4: mutation, place, and writable-alias convergence

Exact envelope: `src/check.rs`, `src/field_place.rs`,
`src/writable_field_alias.rs`, and `tools/check_all.ps1`.

Source diagnostics, set/save/loop structure, binding identity, field place,
permission, alias origin, and block ownership must consume parser statement
facts, canonical place nodes, and resolver definitions. Field type declaration
lookup may remain textual. No 10D write-through or 10E element-assignment
behavior enters scope; H0808/H0809 and existing source diagnostics remain
byte-compatible.

#### 10B.5: Predicate v2 semantic-overlay convergence

Exact envelope: `src/predicate.rs` and `tools/check_all.ps1`.

Predicate v2 becomes only a closed restriction and semantic overlay keyed by
10B.1 canonical node IDs and resolver place definitions. It may retain
recognition, eligibility, operand types, exact place joins, Path/H0630 owner,
and diagnostic evidence. It may not retain or reconstruct another successful
value AST, call graph, operator tree, literal parser, or place parser. This
subincrement changes static Predicate analysis only; runtime contract
evaluation remains unchanged until 10B.11.

#### 10B.6: type and full-type convergence

Exact envelope: `src/type_check.rs`, `src/full_type_check.rs`, and
`tools/check_all.ps1`.

Expression type inference, builtin call shape, binding type, comparison
typing, place typing, and prior-blocker joins consume canonical and resolver
facts. Type/declaration grammar and public display text may remain. H060x,
H070x, H090x, H140x, ordering, and public human/JSON bytes remain unchanged.

#### 10B.7: effect, ownership, and resource convergence

Exact envelope: `src/effect_check.rs`, `src/ownership_check.rs`,
`src/resource_check.rs`, and `tools/check_all.ps1`.

Effect, ownership, and resource roots must come from structured place nodes,
resolver identifier uses, parser statement facts, and List/Record nodes. A
rendered canonical expression may not flow to `first_resource`, `split`, or an
equivalent recognizer. Existing H0801 ownership, exact AP/AQ precedence,
allocation visibility, view invalidation, source authority, and resource
behavior remain unchanged.

#### 10B.8: Core construction and lowering convergence

Exact envelope:

- `src/core_expr.rs`;
- `src/core_preview.rs`;
- `src/core_lower.rs`;
- `docs/FORMAL_CORE.md`;
- `docs/HUM_CORE_PREVIEW_SCHEMA.md`;
- `docs/HUM_CORE_LOWER_SCHEMA.md`; and
- `tools/check_all.ps1`.

Private Core construction, preview, and lowering consume exact canonical
children, operator order, grouping, call/place IDs, and occurrence routes.
Rendered text is public projection only and cannot feed lowering. Public schema
fields stay unchanged.

#### 10B.9: Core verification and projection transport

Exact envelope: `src/core_verify.rs`, `src/main.rs`,
`docs/HUM_CORE_VERIFY_SCHEMA.md`, and `tools/check_all.ps1`.

Core verification receives separately supplied upstream canonical authority;
it may not regenerate the expected tree or projection from the observed tree.
Top-level composition validates the exact verified result before graph, JSON,
profile, and IR-readiness projection. `src/graph.rs`, `src/json.rs`,
`src/profile_check.rs`, and `src/ir_readiness.rs` remain unchanged unless this
increment stops for a new reviewed envelope; their real commands are required
as read-only consumers in the evidence matrix.

#### 10B.10: runtime body-expression convergence

Exact envelope: `src/run.rs`,
`fixtures/foundation/pre_ar_left_associative_arithmetic_pass.hum`, and
`tools/check_all.ps1`.

Body evaluation uses the canonical tree and one operator implementation for
arithmetic, calls, places, permissions, lists, records, typed failure, short
circuiting, overflow, and division by zero. It produces the required 12, 10,
32, and 16 results through the real CLI. Contract evaluation is still frozen;
this subincrement must not claim body/contract agreement.

#### 10B.11: runtime contract convergence and agreement

Exact envelope: `src/run.rs`,
`fixtures/foundation/pre_ar_body_contract_expression_agreement_pass.hum`, and
`tools/check_all.ps1`.

Needs/ensures evaluation consumes the 10B.5 Predicate overlay and invokes the
same canonical evaluator/operator implementation as body execution with an
explicit contract context. Contract-only `old`, `list_len`, and `list_count`
remain unavailable to ordinary bodies. Only after this increment passes may
the Work Order claim that byte-identical body and contract arithmetic produce
structurally equal trees with distinct occurrence IDs and the same value 12.
False needs/ensures retain exact H0702/H0703 ownership and adapter blocking.

#### 10B.12: legacy authority retirement and 10B closure

Exact envelope:

- `src/ast.rs`;
- `src/parser.rs`;
- `src/resolve.rs`;
- `src/callable.rs`;
- `docs/ARCHITECTURE.md`;
- `docs/FORMAL_CORE.md`;
- `docs/LANGUAGE_REFERENCE.md`; and
- `tools/check_all.ps1`.

After all consumers have migrated, production `ParsedExpressionKind`,
`ParsedCall`, `parser_owned_top_level_call_ranges`, the duplicate expression
constructor, raw expression/call scanners, and any adapter that makes them
authoritative are deleted. If a legacy structure must remain solely for a
non-semantic public projection, it must be unreachable from production
semantic selection and justified by a named source audit; a second successful
tree is forbidden. This increment runs the complete cross-stage matrix and is
the only point at which 10B may close. Survival of a competing expression or
call authority keeps 10B open and blocks 10C.

### Un-shimmable evidence gate for every consumer

Every semantic consumer subincrement 10B.1-10B.12 must satisfy all three
independent proofs below. Passing ordinary tests without all three is a
`REJECT`.

1. **Mechanical source/dataflow audit.** The converged production path contains
   no canonical-render-to-semantic-parse flow, raw-text expression scanner,
   expression text splitter, parallel call graph, or fallback recognizer.
   Declaration/type grammar and public rendering are allowed only when a named
   audit proves their values cannot flow back into expression semantics. The
   audit itself is sabotaged by inserting a forbidden flow and must turn red.
2. **Real-path behavior.** A parser-loaded Hum program reaches the actual
   resolver/analyzer/checker/Core/CLI consumer, not a direct helper or fabricated
   answer carrier, and produces its exact owned facts, diagnostics, output,
   ordering, spans, exit, or fail-closed blocker. Existing positive and misuse
   behavior remains no worse than the published baseline.
3. **Canonical-tree corruption sensitivity.** Starting from the same parsed
   source, hold source text, retained line text, public root summaries, and any
   still-present legacy projection fixed; then independently mutate the exact
   canonical node, child order, operator, grouping, call target/argument,
   place, wrapper, range, or node identity owned by that consumer. The real
   consumer's structured output must change or its verifier must reject before
   public serialization/runtime. If it stays green, the consumer has not proved
   dependence on the canonical authority. The expected result may not be
   reconstructed from the mutated tree, and a test-only helper that bypasses
   the production entry point earns no credit.

Each subincrement names at least one positive control where two fresh parses
produce byte-identical consumer evidence and at least one wrong-tree control
where retained text is unchanged. Projection-only stages in 10B.9 satisfy the
same physical property by rejecting or changing their projection when their
separately supplied upstream canonical authority is corrupted; they do not
gain permission to parse expressions themselves.

Tests selected by `tools/check_all.ps1` receive no credit unless the 10B.0
selector guard proves exactly one test ran. Source audits, behavior tests, and
tree-corruption tests are three separate test identities; one self-validating
test cannot stand in for all three.

### Compatibility, correction, and hard-stop rules

Every subincrement preserves all accepted public human/JSON/runtime schemas,
messages, help, severity, blame, ordering, exit behavior, Core/graph fields,
diagnostic causes, ownership, authority, effects, resources, and runtime
semantics except the already accepted H0010 rejection and the explicitly
scheduled arithmetic/body-contract fixes. No new command, schema identifier,
dependency, feature, `cfg`, handler, capture model, callable environment,
backend, standard-library surface, 10C behavior, or Session AR work enters
scope.

Each subincrement receives one implementation pass and one fresh independent
implementation review. A bounded correction may occur only after an explicit
BDFL signal and only inside that subincrement's exact envelope. A second
same-shaped finding, a new file requirement, a missing parser fact, a public or
diagnostic semantic change, a fake/zero-test selector, or an unexercised
load-bearing path stops for a new BDFL backlog-versus-rescope decision. No
subincrement automatically authorizes its successor.

The original anti-ghost, positive-evidence, masking-analysis, configuration,
repeatability, non-host inspection, and standing-check requirements remain in
force. Each accepted subincrement must be committed and published separately,
reach terminal required Ubuntu and Windows success, receive a truthful status
record, and stop for the next BDFL go signal.

### Amendment review and issuance gate

This re-scope amendment is proposed and unauthorized. Its author is
disqualified from its verdict. One fresh independent architect-reviewer must
verify:

- the two verdicts are the same repeated-rejection shape;
- the per-consumer representation-gap/migration diagnosis against code;
- the selective-salvage decision and clean-baseline rule;
- the exact dependency order and each review-sized envelope;
- the three-part un-shimmable evidence gate;
- recursive H0010 ownership and the one-cause model;
- the final duplicate-authority retirement boundary;
- the dead selector's history, intended coverage, and separate urgency;
- preservation of the current dirty tree, fingerprint, fixtures, and empty
  index; and
- unchanged 10C-10F, Session AR, decisions, and standing bans.

The reviewer runs:

```powershell
git diff --check
```

Complete local preflight is not an issuance prerequisite for this document
pass because the preserved rejected 10B tree is not an accepted executable
baseline; if run, its results are diagnostic only and may not be used to accept
the amendment. The reviewer must not edit the document or implementation and
returns exactly `ACCEPT`, `ACCEPT WITH REQUIRED FIX`, or `REJECT`.

Final document `ACCEPT` authorizes no commit, push, dirty-tree cleanup,
subincrement, or later work. BDFL acceptance of the exact bytes, a
`WORKORDER_10.md`-only commit, separately authorized publication with terminal
required CI, a durable status record, and an explicit disposition/go signal for
10B.0 remain required. Increment 10C and every later item remain unauthorized.

## Increment 10B rejected-tree archival and clean-baseline amendment (2026-07-17; proposed)

### Authority, evidence, and non-execution boundary

The BDFL authorized only this architecture/documentation planning pass. The
rejected monolithic 10B tree remains the selective-salvage source required by
the accepted re-scope, while every 10B.0-10B.12 unit must start from a clean
accepted baseline. Clearing the tree without durable recovery would destroy
evidence on which the re-scope depends; continuing from it would violate the
clean-baseline rule. The resolution is archive first, verify the archive, then
return to clean `main` without merging any rejected byte.

The planning baseline is exact:

- `HEAD == origin/main ==
  ce3a6b640f4623b42185afaf7ddf183a926842ff`;
- the index is empty and `WORKORDER_10.md` is clean before this amendment;
- 31 tracked implementation, documentation, and tool paths are modified;
- their exact Git snapshot tree OID is
  `0fac92602f632dbc145d641d73e74bd9ac15c545`; and
- the six untracked fixture paths and SHA-256 hashes remain exactly those
  frozen at lines 1387-1404 above.

The earlier
`539d384ad5b0220095e1845f45a5d6ea6e050394` value remains historical evidence
of the shell-piped binary-diff command used during the re-scope. Independent
review proved that byte stream changes across PowerShell, Git Bash, encoding,
and line-ending configuration, so it is superseded as an active preservation
or archive precondition. It must not accept or reject disposal execution.

The replacement identity is a Git-computed tree containing only the exact 31
rejected tracked paths. It is independent of the rest of `HEAD`, including
later `WORKORDER_10.md` planning/status commits, and excludes all six untracked
fixtures, whose separately frozen SHA-256 hashes remain authoritative. The
identity is constructed by creating an empty temporary index through
`GIT_INDEX_FILE`, adding the exact 31 paths, and asking `git write-tree` for the
content-addressed tree OID. The real index, working files, branch, and refs are
not used or changed; the temporary index is removed after computation.

The author demonstrated the primitive independently in Windows PowerShell and
Git Bash. In both shells the path count was exactly 31 and `git write-tree`
returned exactly:

`0fac92602f632dbc145d641d73e74bd9ac15c545`.

The PowerShell proof used:

```powershell
$SnapshotPaths = @(
  git diff --name-only -- . ':(exclude)WORKORDER_10.md'
)
if ($SnapshotPaths.Count -ne 31) { throw 'snapshot path count changed' }
$SnapshotIndex = Join-Path $env:TEMP `
  ("hum-10b-subtree-" + [guid]::NewGuid().ToString('N') + ".index")
$PriorIndex = $env:GIT_INDEX_FILE
try {
  $env:GIT_INDEX_FILE = $SnapshotIndex
  git read-tree --empty
  git add -- $SnapshotPaths
  if (@(git ls-files).Count -ne 31) { throw 'snapshot index changed' }
  $SnapshotTree = (git write-tree).Trim()
} finally {
  if ($null -eq $PriorIndex) {
    Remove-Item Env:GIT_INDEX_FILE -ErrorAction SilentlyContinue
  } else {
    $env:GIT_INDEX_FILE = $PriorIndex
  }
  if (Test-Path -LiteralPath $SnapshotIndex) {
    Remove-Item -LiteralPath $SnapshotIndex -Force
  }
}
if ($SnapshotTree -cne '0fac92602f632dbc145d641d73e74bd9ac15c545') {
  throw 'snapshot tree identity changed'
}
```

The Git Bash proof used the same Git operations:

```bash
set -euo pipefail
snapshot_index="/tmp/hum-10b-subtree-${BASHPID}.index"
cleanup() { /usr/bin/rm.exe -f -- "$snapshot_index"; }
trap cleanup EXIT
mapfile -t snapshot_paths < <(
  git diff --name-only -- . ':(exclude)WORKORDER_10.md'
)
[ "${#snapshot_paths[@]}" = 31 ]
export GIT_INDEX_FILE="$snapshot_index"
git read-tree --empty
git add -- "${snapshot_paths[@]}"
mapfile -t captured_paths < <(git ls-files)
[ "${#captured_paths[@]}" = 31 ]
snapshot_tree="$(git write-tree)"
[ "$snapshot_tree" = 0fac92602f632dbc145d641d73e74bd9ac15c545 ]
```

Both proofs removed their temporary index and a final real-index check remained
empty. A shell-piped patch hash is no longer part of this plan.

This amendment does not execute or authorize archive creation, staging,
committing, pushing, switching branches, restoring files, deleting files, or
starting 10B.0. Its acceptance authorizes only a scoped documentation commit.
Archive execution requires separate BDFL authority after these exact bytes are
independently accepted, committed, published, terminal-green in required CI,
and durably recorded.

### One durable archive, not two drifting artifacts

The sole authoritative recovery artifact will be the write-once remote branch

`origin/archive/workorder-10b-rejected-monolith-2026-07-17`.

Its first and only commit will have summary

`chore(archive): preserve rejected increment 10B monolith`

and will contain exactly the 31 frozen tracked changes plus the six frozen
fixture blobs. A separate patch file, bundle, stash, tag, or duplicate archive
is forbidden: the archive commit is already content-addressed, can reproduce a
patch with `git diff`, and avoids two purported authorities drifting apart.
The branch is write-once. It may not be force-updated, appended to, merged,
rebased, deleted, or repurposed during Work Order 10. Later deletion requires a
separate BDFL ruling after Work Order 10 closes.

The archive commit's parent is the then-current published `main` head after
this disposal amendment and its publication status are durably recorded. That
parent must descend from the planning baseline above and may differ from it
only through independently accepted and published `WORKORDER_10.md` planning
or status commits. If any implementation, fixture, tool, workflow, schema, or
other path changed on `main`, archive execution stops for fresh review.

Before creating the branch, the executor must re-prove all of the following:

- clean synchronized `main` as the archive parent;
- `WORKORDER_10.md` clean and the index empty;
- exactly the same 31 tracked paths, six untracked fixtures, tracked-diff
  snapshot tree OID, and fixture hashes;
- no additional modified, staged, untracked, ignored-in-scope, replacement,
  mode, symlink, submodule, or unrelated path; and
- absence of both the local and remote archive branch.

Any mismatch stops without archive, cleanup, or repair.

### Exact archive and clearing sequence

After separate BDFL archive-and-push authority, the executor records the exact
published `main` head as `$ArchiveBase`, creates the archive branch from that
head without changing the dirty files, stages only the frozen 37 paths, and
commits once:

```powershell
$ArchiveBranch = 'archive/workorder-10b-rejected-monolith-2026-07-17'
$ArchiveBase = (git rev-parse main).Trim()
git switch -c $ArchiveBranch
git add -u --
git add -- `
  fixtures/foundation/pre_ar_body_contract_expression_agreement_pass.hum `
  fixtures/foundation/pre_ar_comparison_conjunction_pass.hum `
  fixtures/foundation/pre_ar_condition_chained_comparison_fail.hum `
  fixtures/foundation/pre_ar_left_associative_arithmetic_pass.hum `
  fixtures/foundation/pre_ar_predicate_chained_comparison_fail.hum `
  fixtures/foundation/pre_ar_return_chained_comparison_fail.hum
git commit -m "chore(archive): preserve rejected increment 10B monolith"
$ArchiveCommit = (git rev-parse HEAD).Trim()
```

`git add -u --` is permitted only after the exact 31-path precheck; it is not a
general authorization to stage whatever happens to be dirty. Before commit,
the staged set must be exactly those 31 paths plus the six named fixtures, with
no unstaged or untracked remainder. After commit, the executor must prove:

- `$ArchiveCommit^` equals `$ArchiveBase`;
- the commit changes exactly 37 paths and no path outside the frozen set;
- the exact 31 non-fixture paths from `$ArchiveBase..$ArchiveCommit`, loaded
  from the clean archive-branch worktree into the same empty temporary-index
  procedure, reproduce Git snapshot tree OID
  `0fac92602f632dbc145d641d73e74bd9ac15c545`;
- every archived fixture blob reproduces its frozen SHA-256 hash; and
- the archive branch worktree and index are clean.

Only after those checks may the separately authorized non-force push publish
the exact commit to
`refs/heads/archive/workorder-10b-rejected-monolith-2026-07-17`. The executor
must verify the live remote ref equals `$ArchiveCommit`. Any push failure,
different remote SHA, unexpected workflow, or preservation mismatch stops
before clearing `main`.

The exact clearing operation is then:

```powershell
git switch main
```

No `reset`, `clean`, stash, checkout-path overwrite, wildcard deletion, or
manual file removal is authorized. Because the archive commit and `main` share
`$ArchiveBase`, switching back to `main` reverts exactly the 31 rejected tracked
paths and removes exactly the six fixture paths that exist only on the archive
branch. `WORKORDER_10.md` is shared at `$ArchiveBase` and is not part of the
archive commit or clearing diff. The executor must then prove:

- `HEAD == origin/main ==` live remote `main == $ArchiveBase`;
- the worktree, index, and untracked set are empty;
- the live archive ref still equals `$ArchiveCommit`; and
- no implementation, fixture, status, or unrelated path remains.

If `git switch main` cannot perform exactly that transition, the executor stops
without substituting another cleanup command.

### Fixture-by-fixture reuse ruling

The six fixture files are source-program inputs, not accepted test wiring. None
contains a test-only helper, expected-answer carrier, rendered-tree projection,
private Rust identity, raw scanner, or shim-specific representation. Their
source bytes are directly reusable as follows:

| Fixture | Ruling and owning increment | Why the source is representation-neutral |
| --- | --- | --- |
| `pre_ar_comparison_conjunction_pass.hum` | reuse exact blob in 10B.1 | It expresses two independent comparisons joined by `and` plus comparison-looking Text; it distinguishes legal syntax without naming a parser implementation. |
| `pre_ar_condition_chained_comparison_fail.hum` | reuse exact blob in 10B.1 | It places one forbidden chain in a real `if` condition; H0010 must own it before resolver/runtime regardless of downstream representation. |
| `pre_ar_predicate_chained_comparison_fail.hum` | reuse exact blob in 10B.1 | It places the same parser-owned misuse in `ensures`; the program does not depend on Predicate v2's rejected parallel AST because parser rejection precedes that consumer. |
| `pre_ar_return_chained_comparison_fail.hum` | reuse exact blob in 10B.1 | It places the same misuse in a return expression and contains no implementation-specific evidence. |
| `pre_ar_left_associative_arithmetic_pass.hum` | reuse exact blob in 10B.10 | Its four tasks state observable values 12, 10, 32, and 16 while their `ensures` compare against literals; it tests body evaluation without claiming contract-expression convergence. |
| `pre_ar_body_contract_expression_agreement_pass.hum` | reuse exact blob in 10B.11 | It deliberately repeats `8 * 6 / 4` in body and contract, which is the accepted agreement pressure; the source alone does not claim or encode how the shared tree is implemented. |

"Reuse exact blob" does not allow any fixture to remain in the worktree across
the clean-baseline boundary. After archival, each fixture disappears from
`main` and may be restored from the exact archive commit only inside its named,
separately authorized envelope. The missing nested-chain fixture remains new
10B.1 work. The rejected tree's test wiring, assertions, selector entries,
expected projections, and ordinary green results remain unaccepted claims that
must be rebuilt and sabotaged under the owning subincrement's real-path gates.
Thus fixture-source reuse does not weaken the rule that rejected tests are not
evidence to copy.

### Cold-start retrieval and selective-salvage route

After archive execution, a status-only Work Order record must name
`$ArchiveBase`, the exact `$ArchiveCommit`, the live remote branch, the 37-path
inventory, the tracked snapshot tree OID, and all six fixture hashes. 10B.0 remains
unauthorized until that record is independently accepted, committed, published,
and terminal-green, followed by a separate BDFL go signal.

A cold-start 10B.x implementer retrieves evidence through the immutable commit,
never through this conversation:

```powershell
git fetch origin `
refs/heads/archive/workorder-10b-rejected-monolith-2026-07-17:`
refs/remotes/origin/archive/workorder-10b-rejected-monolith-2026-07-17
git show "${ArchiveCommit}:<path>"
git diff $ArchiveBase $ArchiveCommit -- <path>
```

For Rust, documentation, schema, and tool paths, `git show` and `git diff` are
read-only design inspection only. No cherry-pick, patch application, bulk copy,
branch merge, path restore, or mechanical hunk transfer is permitted. An
implementer must re-derive each behavior and structure against the clean
accepted boundary, its owning envelope, the source audit, the real-path proof,
and the canonical-tree corruption sabotage.

Only the six fixture blobs ruled directly reusable above may be materialized
from `$ArchiveCommit`, one owning increment at a time and only after its BDFL go
signal. Their archive hash must match before use, and their new test wiring must
be authored independently. Any other desired reuse is selective salvage and
requires fresh justification inside the existing 10B.x envelope; if that
envelope is insufficient, the increment stops for review rather than copying
around it.

### Review, execution, and hard stop

This disposal plan requires one fresh independent architect-reviewer to verify:

- exact preservation of the current snapshot tree OID, path inventory, fixture
  inventory, hashes, and empty index;
- that one immutable remote branch is sufficient durable recovery without a
  second drifting artifact;
- that the archive parent, staged set, commit, non-force publication, remote-ref
  verification, and `git switch main` clearing sequence fail closed;
- that clearing touches exactly the 31 rejected paths and six fixture paths;
- every fixture ruling against its source and owning 10B.x mandate;
- that source-fixture reuse cannot launder rejected test wiring or evidence;
- the cold-start retrieval path and write-once archive lock; and
- unchanged 10B.0-10B.12 mandates, envelopes, evidence gates, order, H0010
  meaning, clean-baseline/selective-salvage rule, 10C, and later bans.

The reviewer runs only:

```powershell
git diff --check
```

Complete local preflight remains invalid acceptance evidence while the rejected
tree contains the dead exact-test selector that 10B.0 exists to repair. The
reviewer must not edit any file, execute archive or cleanup commands, stage,
commit, push, or begin 10B.0, and returns exactly `ACCEPT`, `ACCEPT WITH
REQUIRED FIX`, or `REJECT`.

Even `ACCEPT` authorizes only a BDFL-scoped `WORKORDER_10.md` documentation
commit. Archive creation, archive commit, archive push, rejected-tree clearing,
the archival status record, and 10B.0 each remain separately gated. No 10B.0,
10B.1, 10C, Session AR, or later work may begin implicitly.

## Increment 10B.1 repeated-rejection re-scope amendment (2026-07-19; proposed)

### Authority, diagnosis, and supersession

The BDFL authorized only this architecture/documentation pass after Increment
10B.1 exhausted its one correction cycle with two independent `REJECT`
verdicts. The four final P1 findings have two different architectural weights:

- the incomplete seal in `validate_expression_occurrence` and the
  independently mutable loop binder/token projection are one foundational
  defect class: the accepted parser occurrence projects facts that its
  validator does not bind to independently retained parser events;
- the explicit-entry matrix is a small discriminating-evidence defect: a
  nonexistent entry was tested on a parser-invalid file, so parser precedence
  correctly produced the same H0010 before entry selection; and
- the demand that one source-text audit detect every alias, macro, closure,
  method, upstream scanner, or arbitrary indirection is a ghost requirement.
  A finite spelling scan cannot prove that negative. Typed production
  boundaries plus real-path behavior and authority-held corruption are the
  load-bearing proof; a bounded direct-construction audit is only a redundant
  backstop.

The architecture remains unchanged: one parser-owned canonical expression
tree and one parser-owned occurrence/relationship authority are the sole
source of expression meaning. The failure was ordering. H0010 was implemented
before the occurrence authority was fully sealed, allowing a consumer test to
discover foundation work mid-increment. This amendment therefore
prospectively supersedes only the existing `10B.1: canonical expression
occurrence and recursive H0010` unit with the ordered 10B.1a.1-10B.1a.11
foundation train and 10B.1b below. It does not alter the published H0010
meaning, 10B.2-10B.12, 10C-10F, any accepted decision, or any public behavior.

No third correction against the rejected 10B.1 patch is authorized. Both new
subincrements start from the clean accepted `8a245ede1649519d5d07a5454f65e93d0aa13049`
line, advanced only by independently accepted and published documentation,
preservation, status, and predecessor implementation commits. Rejected source
may be inspected as design evidence after separately authorized archival, but
no hunk may be cherry-picked, restored, copied mechanically, or treated as
accepted test evidence.

### Complete canonical-seal inventory

This inventory is derived from the accepted 10A canonical kinds, the current
parser grammar, the rejected occurrence representation, and the exact
statement forms parsed by `parse_task_body_syntax`. It is complete only for
expression occurrences and their owning statement relationships. Type
declarations, callable signatures, item headers, effect declarations, and
other declaration grammar remain outside 10B.1a unless a listed expression or
statement edge references their already-produced identity.

#### Independent authority domains

The seal must not validate one mutable projection against another projection
constructed from the same mutable object. The parser must retain three closed,
private, typed domains produced from the original lexing and reduction events:

1. an expression-structure authority for nodes, payloads, child roles, and
   reduction order;
2. a lexical-event authority for exact tokens, delimiters, separators,
   operator phrases, semantic whitespace/gaps, malformed events, and depth;
   and
3. a statement-relationship authority for the owning item/section/statement,
   intent, binders, targets, destinations, expression roles, and block edges.

Every exported parser fact refers to opaque source, occurrence, node, token,
reduction, and relationship identities from those domains. They are minted
from parser traversal and lexer events, not from diagnostic codes, rendered
text, display names, filenames, line/column pairs, spans alone, or a hash of a
public projection. Repeated byte-identical expressions and tokens in one
statement or file remain distinct. Validation receives the retained authority
separately and compares the complete projection against it before Core
serialization or any later semantic consumer.

#### Occurrence-wide facts

Every expression occurrence independently seals:

- semantic source/file identity and the exact source-blob revision being
  parsed;
- owning item path, section identity, statement identity, and closed
  expression role;
- one opaque occurrence identity, exact root node identity, exact root byte
  range including the zero-width Unit position, and root reduction identity;
- the complete preorder node count and order;
- expression intent and the exact statement/section event that assigned it;
- an independently typed predicate-recognition signal, including its required
  presence or absence, rather than an optional span that may disappear;
- root lexical status, maximum delimiter depth, and the complete ordered token
  interval; and
- any typed-failure wrapper identity and its exact relation to the root.

`Return`, `Binding`, `SetValue`, `SaveValue`, `Condition`, `LoopCollection`,
`LoopRangeStart`, `LoopRangeEnd`, `Failure`, `TestExpectation`,
`NeedsPredicate`, `EnsuresPredicate`, and `Other` are the exhaustive current
intent set. A new intent variant must fail compilation until its producer,
validator, projection, and corruption row are added.

#### Facts common to every canonical node

Every node independently seals:

- opaque node and occurrence identities, optional parent identity, exact
  closed child-role plus ordinal, and preorder ordinal;
- exact UTF-8 byte range, source identity, token interval, and grammar
  reduction identity;
- exact canonical kind discriminant and kind-specific payload;
- ordered child identities and exact child cardinality;
- delimiter depth before and after the reduction plus node lexical status; and
- the absence of fields that are illegal for that kind.

A free-form `Vec<usize>` child path is not sufficient by itself. Child roles
must be closed and typed (`binary.left`, `binary.right`, `call.callee`,
`call.argument[n]`, `record.field[n].value`, and so on), so reparenting,
reordering, or exchanging two same-shaped children cannot remain coherent.
No validator match may use a wildcard arm for a canonical kind.

#### Kind-specific facts

The following table is the exhaustive expression-kind seal. It is a review
failure if the implementation AST contains a kind with no row, or a row is
implemented without an exhaustive constructor and validator arm.

| Canonical kind | Independently sealed facts beyond the common facts |
| --- | --- |
| Unit | zero-width position, empty token interval, and zero children |
| Identifier | identifier token identity, exact token range and spelling, and semantic identifier value |
| Field | base child, dot token identity/range, field-token identity/range/spelling, and field value |
| Direct numeric element place | base child, open/close bracket identities and ranges, unsigned index-token identity/range/spelling/value, and element-place role |
| UInt literal | digit-token identity/range/spelling and parsed `u64` value |
| Int literal | sign and digit token identities/ranges/spelling, parsed `i64` value, and the distinction between a signed literal and binary subtraction |
| Bool literal | exact keyword-token identity/range/spelling and Boolean value |
| Text literal | opening/closing quote identities and ranges, raw content interval, escape events, decoded value, and termination state |
| List literal | bracket pair, ordered element children, comma tokens, empty-list fact, semantic gaps, and trailing-comma state |
| Record literal | optional record-name token, brace pair, ordered field-name tokens, colon and comma tokens, ordered value children, empty-record fact, and trailing state |
| Call | callee child, parenthesis pair, ordered argument children, separator tokens, every grammar-significant gap/adjacency fact, missing/mismatched close state, and trailing state |
| Permission | exact `borrow`/`change`/`consume` keyword token, permission discriminant, semantic gap, and value child |
| Typed failure (`Try`) | exact `try` token, value child, optional `or`/wrapper relation, exact failure-root and variant token identities/ranges, dot token, and wrapper kind |
| Binary | exact operator discriminant, complete one- or multi-token operator phrase and range, precedence class, associativity, left/right child identities, and reduction boundary |
| Group | parenthesis pair, one child, and depth transition |
| Unsupported | one closed unsupported/malformed cause, exact offending and consumed ranges, partial reduction event, and no invented successful payload |

Direct numeric element-place syntax is an accepted Work Order 10 grammar fact
(`items[0]`) but is absent from both the clean 10A canonical enum and the
rejected 10B.1 enum. 10B.1a must add and seal its internal canonical form now
without changing evaluation, mutation, ownership, or public Core behavior.
Deferring it while claiming a complete seal is forbidden. Its consumers remain
owned by 10B.4 and 10B.10; 10D/10E behavior does not enter 10B.1a.

#### Lexical and malformed-event facts

The lexical authority must distinguish, rather than collapse into `Other`, all
grammar-significant tokens used by the rows above: identifiers; unsigned and
signed integer components; Boolean and Text components; quotes and escapes;
parentheses, brackets, braces, commas, dots, colons, and assignment tokens;
all sixteen current binary operators (`*`, `/`, `+`, `-`, `==`, `!=`, `<`,
`<=`, `>`, `>=`, `is`, `does`, `returns`, `fails with`, `and`, `or`);
permission and typed-failure wrapper keywords; and statement/relationship
keywords and phrases listed below. Multi-token operators and phrases retain
each token plus their phrase/reduction identity. Whitespace is recorded only
where the grammar assigns it meaning, including call and multi-word phrase
gaps; arbitrary display whitespace is not semantic authority.

Delimiter evidence includes each open/close token identity, delimiter kind,
pair identity, nesting parent, depth before/after, maximum depth, missing or
mismatched close, separators, trailing token, and adjacency/gap facts. A call
open/close or separator may never be synthesized from child endpoints.

The exact current malformed-cause set is also closed:
`UnterminatedTextLiteral`, `MissingDelimiter`, `MismatchedDelimiter`,
`DelimiterDepthExceeded`, `MissingOperand`, `InvalidComparisonOperator`,
`InvalidOperandStarter`, `MalformedFieldPlace`, `ListElementSeparator`,
`ListTrailingComma`, `ListNonTextElement`, and
`IntegerLiteralOutOfRange`. Each occurrence seals cause, offending range,
expected evidence, actual token/depth/end-of-input evidence, and producing
lexical event. `Complete` means the retained authority contains no malformed
event for that occurrence; it is not a mutable enum that can be co-changed with
the projection. Every malformed cause must validate its `actual` evidence.

#### Statement and relationship facts

The statement authority independently seals exact keyword/phrase token
identities, owning statement identity, block relationship and depth, ordered
expression occurrence identities, and these form-specific facts:

- `needs:` and `ensures:` predicate owner, section/line identity, predicate
  root, and recognition event;
- `return`: keyword and returned occurrence;
- immutable `let` and mutable `change` bindings: keyword, binder token,
  optional type-annotation boundary, assignment token, and value occurrence;
- `set`: keyword, target occurrence, assignment token, and value occurrence;
- `save`: keyword, saved occurrence, `in` relationship token, and destination
  token;
- `if` and `while`: keyword, condition occurrence, and block-open token;
- `for each`: complete phrase, binder token, `in` token, collection
  occurrence, and block-open token;
- `for index`: complete phrase, binder token, `from` token and start
  occurrence, exact `until` or `through` token and end occurrence, and
  block-open token;
- unconditional `loop`: keyword, block-open token, and absence of an
  expression;
- `fail`: keyword and failure occurrence;
- `expect`: keyword and expectation occurrence;
- block close: close token, owning block identity, and absence of an
  expression; and
- other/free expression statements: exact ordered occurrence roots without a
  fabricated statement kind.

A binder is one producer-owned token/event identity shared by the statement
and relationship facts. Copying its name and span into two structures is not
identity. Swapping a same-spelled binder, relationship token, loop bound,
expression root, or block edge from another statement must fail even if line,
column, display text, and all copied values remain compatible.

#### Completeness and co-mutation construction rule

10B.1a must define one closed test-only `CanonicalSealField` catalogue that
names every occurrence-wide, common-node, kind-specific, lexical, malformed,
wrapper, and statement/relationship fact above. The catalogue is an
independently supplied expectation, not generated from the production
validator or authority ledger. Exhaustive Rust matches over the production
kind, intent, malformed-cause, statement, loop, operator, permission,
delimiter, and wrapper enums must fail compilation when a variant lacks a
producer or validator arm.

For every catalogue field, a production parser fixture or data-driven real
`parse_source` corpus must produce at least one reachable instance. A
test-only corruption seam may alter the exported projection but may not alter
the retained authority. The permanent matrix must prove:

1. mutating, removing, duplicating, substituting, reordering, or adding that
   field fails validation;
2. every unordered pair of catalogue fields, including fields in different
   domains, fails when coherently co-mutated while the authority is fixed; and
3. substituting a valid field, node, token, binder, relationship, or complete
   occurrence from another same-shaped occurrence fails.

The test reports and independently pins the expected field and pair counts. It
may not derive its expected count, identities, or answer from the validator,
the authority being checked, or a public Core/root summary. A sabotage that
deletes a producer arm, validator arm, catalogue row, single mutation, pair
mutation, or cross-occurrence substitution must turn the exact selected test
red. This is the complete-by-construction gate that prevents later consumers
from discovering another unsealed parser fact.

### Increment 10B.1a foundation train: ordered canonical seal completion

The complete inventory above is retained unchanged, but it is not one
review-sized implementation. The accepted 10B.0 repair changed about 280 lines
and received `ACCEPT` with zero correction cycles; that is the empirical
review-size target. Each foundation subincrement below therefore targets
roughly 200-350 total changed lines across production, tests, tools, and
fixtures. Five hundred total changed lines is a hard pre-review ceiling, not a
goal. If honest implementation crosses that ceiling, approaches four figures,
needs an unlisted file, or cannot be reviewed in one sitting, it stops for a
fresh decomposition amendment before review. Deletions, generated code, or a
large fixture do not exempt a change from the total.

The subincrements are independent publication units. They may not be combined
in one dirty worktree, review, commit, push, CI run, or status record. No later
subincrement may begin until its predecessor is independently accepted,
committed, published, terminal-green on Ubuntu and Windows, durably recorded,
and given a separate explicit BDFL go signal. No parser diagnostic or semantic
consumer may use the new authority until the final 10B.1a.11 complete-seal gate
has passed. Partial convergence is representation-only and may not be claimed
as canonical-consumer evidence.

#### Common evidence gate for every 10B.1a subincrement

Each subincrement inherits the complete anti-ghost rules and must carry the
same cumulative corruption gate for every fact sealed through that point:

- a real `parse_source` input produces every newly sealed fact through the
  ordinary parser path; no hand-built successful authority is credited;
- mutating, removing, duplicating, reordering, adding, or substituting only
  that fact while retaining the independent authority makes validation fail;
- every unordered pair of sealed facts, including a pair crossing two earlier
  subincrements, fails under coherent co-mutation while the authority stays
  fixed;
- a valid same-shaped fact from another occurrence, source, node, token,
  reduction, statement, or relationship cannot substitute successfully;
- the independently authored `CanonicalSealField` expectation and exact
  single/pair counts remain outside the validator and authority under test;
- deleting any producer arm, validator arm, catalogue row, mutation operator,
  cross-occurrence case, or exact nonzero selector turns the selected test red;
  and
- two fresh runs produce identical private inventories and results while all
  existing public human, JSON, Core, runtime, diagnostic, ordering, span, and
  exit bytes remain unchanged.

10B.1a.1 introduces the independently supplied test-only
`CanonicalSealField` catalogue. Each later subincrement extends it only with
the exact facts that unit owns, preserves every earlier row and mutation, and
reruns all cumulative unordered pairs. 10B.1a.11 pins the final complete
catalogue. No subincrement may regenerate an expected row, count, pair, or
identity from the production validator or retained authority.

Each subincrement runs its focused exact-selector matrix followed by
`cargo fmt --check`, `cargo test`,
`cargo clippy --all-targets -- -D warnings`, `git diff --check`, and
`.\tools\check_all.ps1`. Its report enumerates host production/test/all-target
coverage, inspects non-host `cfg` branches, discloses unavailable local
configurations, and leaves Ubuntu/Windows confirmation to the separately
authorized publication CI. A green aggregate suite without the exact
single/pair/cross-occurrence counts is not acceptance evidence.

An earlier accepted fact may not be reopened or weakened in a later
subincrement. If a later dependency proves an earlier seal incomplete, that is
a stopped-gate finding against the earlier unit, not authority to repair both
inside the later increment.

The rejected incomplete foundation already changed the same four
implementation/harness paths by 3,134 total lines (`src/ast.rs` 204/0,
`src/parser.rs` 2,659/80, `src/core_body.rs` 73/0, and
`tools/check_all.ps1` 116/2) while still lacking the independent authority and
complete seal. At the proven 280-line target, that observed lower bound is
approximately eleven review units. The eleven-unit dependency split below is
therefore evidence-derived, not ceremonial subdivision.

#### 10B.1a.1: source, owner, and occurrence identity kernel

Dependency role: establish the private occurrence boundary before node or
token payloads enter it.

Exact envelope: `src/ast.rs`, `src/parser.rs`, and `tools/check_all.ps1`.

This unit introduces closed opaque source-blob, semantic-file, occurrence,
item, section, statement, and expression-role identities; exact source
revision, root identity/range including Unit position, expression intent and
assigning event, predicate-recognition presence/absence, and the separately
retained authority handle. Repeated byte-identical expressions, same-line
expressions, and compatible public spans remain distinct. It also introduces
the independent test-only field catalogue and corruption mechanism, but no
node payload, H0010, Core use, or consumer.

#### 10B.1a.2: common node topology and event identity

Dependency role: establish the exhaustive node skeleton used by every later
kind.

Exact envelope: `src/ast.rs`, `src/parser.rs`, and `tools/check_all.ps1`.

This unit seals opaque node/token/reduction identities; exact parent and closed
child-role/ordinal; preorder identity/count/order; exact UTF-8 range, source,
and token interval; kind discriminant; grammar reduction; ordered children and
cardinality; and absence of fields illegal for the kind. No free-form child
path, display string, range alone, or public-projection hash may mint or select
identity. Exhaustive kind matches have no wildcard. Kind payload values remain
owned by later units and may not be projected as sealed early.

#### 10B.1a.3: scalar literal and identifier seal

Dependency role: add leaf payloads after occurrence and common-node identity
are authoritative.

Exact envelope: `src/ast.rs`, `src/parser.rs`, and `tools/check_all.ps1`.

This unit exhaustively seals `Unit`, `Identifier`, `UIntLiteral`, `IntLiteral`,
`BoolLiteral`, and `TextLiteral`, including token identity/range/spelling,
semantic value, signed-literal versus subtraction, quotes, raw content,
escapes, decoded Text, and successful termination. Same-spelled identifiers,
same-valued literals, signs, escapes, and compatible ranges must remain
independently corruptible under the cumulative gate.

#### 10B.1a.4: field and direct numeric element-place seal

Dependency role: establish exact place topology before aggregate/call or
statement consumers can reference it.

Exact envelope: `src/ast.rs`, `src/parser.rs`, and `tools/check_all.ps1`.

This unit seals `Field` base/edge, dot token, field token/range/spelling/value,
and the accepted direct numeric element-place base, bracket pair, unsigned
index token/range/spelling/value, and place role. The element node is private
representation only: no read, write, mutation, ownership, evaluation, public
Core, 10D, or 10E behavior enters this unit.

#### 10B.1a.5: delimiter kernel, grouping, and list seal

Dependency role: establish delimiter topology and the first recursive
aggregate over accepted leaf/place nodes.

Exact envelope: `src/ast.rs`, `src/parser.rs`, and `tools/check_all.ps1`.

This unit seals delimiter token/pair identity, kind, nesting parent, depth
before/after, maximum depth, missing/mismatched close, and semantic gap facts;
then exhaustively seals `Group` and `ListLiteral`, including ordered children,
brackets, commas, empty/trailing state, and grammar-significant whitespace.
No delimiter fact may be synthesized from child endpoints.

#### 10B.1a.6: record and call seal

Dependency role: close the two larger ordered recursive structures after the
delimiter kernel exists.

Exact envelope: `src/ast.rs`, `src/parser.rs`, and `tools/check_all.ps1`.

This unit exhaustively seals `RecordLiteral` name, braces, ordered field-name
tokens, colons, commas, value edges, empty/trailing state; and `Call` callee,
parentheses, ordered arguments, separators, every grammar-significant gap and
adjacency, close state, and trailing state. Same-shaped records/calls, foreign
separators, argument/field reorder, and valid delimiter substitutions must fail
under the cumulative matrix.

#### 10B.1a.7: binary, permission, and typed-failure wrapper seal

Dependency role: close the successful operator and wrapper kinds after their
child and delimiter facts are sealed.

Exact envelope: `src/ast.rs`, `src/parser.rs`, and `tools/check_all.ps1`.

This unit exhaustively seals `Binary`, `Permission`, and typed-failure `Try`:
all sixteen operator discriminants and one-/multi-token phrases, exact ranges,
precedence, associativity, word boundaries, reduction order, and left/right
roles; permission keyword/discriminant/gap/value; and typed-failure keyword,
value, optional wrapper relation, root/dot/variant tokens, and wrapper kind.
H0010 remains absent; this unit seals comparison facts but does not diagnose
them.

#### 10B.1a.8: lexical status, unsupported, and malformed seal

Dependency role: bind parser-error evidence only after every successful kind
and its token topology are authoritative.

Exact envelope: `src/ast.rs`, `src/parser.rs`, and `tools/check_all.ps1`.

This unit exhaustively seals `Unsupported`, `Complete`, and all twelve accepted
malformed causes with producing event, offending/consumed ranges, expected
evidence, and actual token/depth/EOF evidence. `Complete` is independently
proven absence of a malformed event. Wrong but plausible actual evidence,
cause-plus-expected/actual co-mutation, maximum-depth co-mutation, partial
reduction substitution, and valid foreign malformed facts must fail.

#### 10B.1a.9: section and linear statement relationships

Dependency role: bind sealed roots to non-control-flow owners before adding
block and loop topology.

Exact envelope: `src/ast.rs`, `src/parser.rs`, and `tools/check_all.ps1`.

This unit seals `needs:`/`ensures:` predicate ownership; return; immutable and
mutable binding; set; save; fail; expect; and other/free expression roots. It
includes exact section/line/statement identity, keywords, binder tokens, type
boundary, assignment token, target/value/destination, relationship token, and
ordered expression occurrences. Same-name binders and valid foreign roots or
tokens cannot substitute.

#### 10B.1a.10: control-flow, loop, binder, and block relationships

Dependency role: complete statement authority with shared binder/token and
block topology.

Exact envelope: `src/ast.rs`, `src/parser.rs`, and `tools/check_all.ps1`.

This unit seals if, while, `for each`, `for index` with `until` and `through`,
unconditional loop, and block close: exact phrase tokens, condition/collection/
start/end roots, block-open/close identity, block owner/depth, relation kind,
and absence of expressions where required. Each binder is one shared
producer-owned token/event identity, never two equal name/span copies.
Co-mutating relation kind with token length, binder copies, loop bounds, or
block edges must fail against retained authority.

#### 10B.1a.11: complete-seal integration and Core boundary

Dependency role: prove the complete frozen inventory as one authority and make
validation mandatory at the first private Core boundary.

Exact envelope:

- `src/parser.rs` for final exhaustive composition and the cumulative matrix;
- `src/core_body.rs` only to require the complete validated seal before
  private Core-body construction while preserving public bytes;
- `tools/check_all.ps1` for exact selectors, real parser/Core paths,
  independent complete counts, repeatability, and sabotage; and
- one new real source fixture,
  `fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum`, containing
  every successful kind and every relationship form that can coexist in one
  valid program.

`src/ast.rs` is read-only in this final unit. Any needed AST change means an
earlier unit failed to close its facts and stops 10B.1a.11. The ordinary
`parse_source` then Core-body path must reject every missing, extra,
substituted, reordered, reparented, wrong-kind, wrong-payload, wrong-token,
wrong-range, wrong-depth, wrong-intent, wrong-status, wrong-wrapper,
wrong-binder, wrong-loop, wrong-block, and cross-occurrence projection before
private Core construction or serialization. The test independently pins the
complete field count and every unordered-pair count across
10B.1a.1-10B.1a.11. No public Core schema or byte changes.

Passing 10B.1a.11 is the only event that closes the seal foundation. It does
not allocate or emit H0010 and authorizes no semantic consumer. If any unit
cannot stay under the 500-line ceiling, it stops for a finer reviewed split;
facts may not be moved into 10B.1b.

### Increment 10B.1b: recursive H0010 consumer and discriminating entry

#### Exact envelope and dependency

10B.1b may begin only after each of 10B.1a.1 through 10B.1a.11 has separately
completed its independent acceptance, commit, publication, terminal Ubuntu
and Windows CI, durable status record, and BDFL go-signal sequence. It may
modify only:

- `src/parser.rs` for the recursive canonical-tree H0010 visitor and its
  parser-owned occurrence construction;
- `src/diagnostic_catalog.rs` and `src/diagnostics.rs` for the already accepted
  one-code/one-cause allocation and independently pinned 88-code/179-cause
  projections;
- `docs/DIAGNOSTICS.md` and `docs/LANGUAGE_REFERENCE.md` for checked H0010
  projection and the accepted non-chainable comparison rule;
- `tools/check_all.ps1` for real human/JSON/runtime, entry-discrimination,
  bounded audit, and corruption matrices; and
- the five exact fixtures
  `pre_ar_comparison_conjunction_pass.hum`,
  `pre_ar_condition_chained_comparison_fail.hum`,
  `pre_ar_nested_chained_comparison_fail.hum`,
  `pre_ar_predicate_chained_comparison_fail.hum`, and
  `pre_ar_return_chained_comparison_fail.hum` under
  `fixtures/foundation/`.

`src/ast.rs` and `src/core_body.rs` are read-only accepted complete-seal
authority in 10B.1b. Changing them means 10B.1a.1-10B.1a.11 did not close the
foundation and stops the increment; it is not a local correction. No resolver
or later consumer convergence enters scope.

#### H0010 and recursive consumption

H0010 remains exactly one active parser code and exactly one registered cause,
`chained_comparison_not_supported_v0`. Condition, Predicate, return, nested,
grouped, call-argument, list, record, permission, Try, Boolean-left, and
Boolean-right sites are distinct occurrences of that one fundamental cause,
not new causes. The parser visitor accepts only the validated sealed canonical
occurrence plus its separately retained parser authority. It has no raw source,
rendered canonical text, public code lookup, default cause, fallback scanner,
parallel successful tree, or child-endpoint inference input.

The visitor exhaustively recurses through every admitted child role and rejects
a comparison whose operand contains another comparison. It rejects top-level
and nested chains, including `(1 < 2 < 3) and true` and chains nested on either
side of another operator, before resolver/runtime. It preserves independent
comparisons such as `1 < 2 and 2 < 3`, independent grouped comparisons, and
comparison-looking Text. The later comparison token is primary, the first is
related, the accepted message/help/severity/order remain exact, runtime exits
1 with zero stdout, and no generic trap appears. H0010 remains an ordinary
source diagnostic; this amendment does not change its exit semantics.

Catalog evidence pins exactly 88 active codes, three reserved families, zero
retired codes, and 179 registered causes. The test literals are independently
supplied expectations and may not be derived from the registry projection they
validate. H0010 retains one historical public ordinal and all existing public
diagnostic ordering remains unchanged.

#### Discriminating entry evidence

Parser diagnostics own malformed source before runtime entry selection. A
nonexistent entry against an H0010-invalid file therefore correctly produces
H0010 and cannot distinguish entry handling; that rejected test shape is
retired.

The permanent control instead uses the parser-clean
`pre_ar_comparison_conjunction_pass.hum` fixture with an exact nonexistent
entry. It must exit 1 with the existing exact `entry task <name> was not found`
entry-resolution error bytes, contain no H0010, and produce no source
diagnostic projection. A run command naming the exact existing task in each
chained-comparison fixture must also exit 1, but parser-first preflight must
instead produce exactly one H0010 with its sealed code identity, message, help,
primary and related sites, exact stderr bytes, and zero stdout, and must not
contain the entry-resolution error.

The evidence discriminates by diagnostic identity and exact stderr content,
never by exit status. Substituting either command's complete stderr, diagnostic
identity, message, projection, or count for the other must fail. Sabotages that
replace the clean control with an invalid file, ignore `--entry`, reuse either
result, erase the exact message distinction, or assert only the shared nonzero
exit must turn the test red. This changes no parser-first precedence,
entry-selection behavior, diagnostic semantics, or public exit behavior.

#### Honest source-audit boundary and load-bearing proof

`audit_h0010_production_dataflow` is explicitly bounded to mechanically
detecting direct production construction of public `H0010` or its registered
cause outside the one parser visitor, direct raw-text/render-to-parse helper
calls inside that typed visitor slice, and loss of the visitor's sealed typed
signature. It is not evidence for, and is not required to discover, arbitrary
aliases, macros, closures, methods, generated code, upstream scanners,
dynamic dispatch, or future indirection. Claiming otherwise is prohibited.

The load-bearing proof is:

1. the compile-time typed boundary accepts only the sealed canonical
   occurrence and separate parser authority;
2. real parser, human, JSON, and runtime paths produce the exact H0010
   occurrences; and
3. while source and retained text remain fixed, removing, moving,
   substituting, reparenting, or changing the nested comparison node, operator
   token, child role, delimiter/depth, occurrence identity, or authority route
   changes the owned H0010 occurrence or fails closed before resolver/runtime.

A text renderer/reparser or root-only scan cannot pass that corruption matrix.
The source audit is defense in depth only. Review may reject a real typed or
behavioral bypass, but may not revive the impossible claim that spelling scans
prove absence of every possible program indirection.

### Rejected 10B.1 preservation and clean-baseline gate

The rejected work is preserved, not accepted:

- baseline `HEAD == origin/main ==
  8a245ede1649519d5d07a5454f65e93d0aa13049`;
- empty real index;
- eight modified tracked paths:
  `docs/DIAGNOSTICS.md`, `docs/LANGUAGE_REFERENCE.md`, `src/ast.rs`,
  `src/core_body.rs`, `src/diagnostic_catalog.rs`, `src/diagnostics.rs`,
  `src/parser.rs`, and `tools/check_all.ps1`;
- 3,091 tracked insertions and 88 tracked deletions; and
- five untracked fixtures with exact SHA-256:
  - `pre_ar_comparison_conjunction_pass.hum`:
    `3081b3ba84045cb64bb8c049fde683cbfc64c91dae956e9a71d012b193951433`;
  - `pre_ar_condition_chained_comparison_fail.hum`:
    `c49bc27b53c2fbbfa8012525c25e756eb8da4871fe83ea2b6caec94466bc9d41`;
  - `pre_ar_nested_chained_comparison_fail.hum`:
    `15e0caed7466978b95c1867d9492b83844197b3dc59754cd7805949f8b5a5b50`;
  - `pre_ar_predicate_chained_comparison_fail.hum`:
    `7376de1f01f018943174876886ea37da02e5a4458b032d24214f5ff4116e8d30`;
  - `pre_ar_return_chained_comparison_fail.hum`:
    `6096390130a62ddc5a2128b936b188d05a0a63aa9036cad4ffd84c7b16207fb8`.

The condition fixture hash above is intentionally checked as a complete
64-hex SHA-256 value during review; any transcription or byte mismatch is a
document rejection.

In each shell a fresh empty temporary Git index received exactly the listed 13
paths, and `git write-tree` produced
`af756a7fea21353794de585869a7d2df487fe663`. The procedure was run once from
PowerShell and once from Git Bash with distinct temporary index files; both
returned that exact OID, both reported exactly 13 indexed paths, both removed
their temporary files, and the real index remained empty. This 13-path subtree
OID, not shell-piped diff bytes or a full repository tree that would drift
with later documentation commits, is the cross-shell content identity.

After this amendment is independently accepted, BDFL-accepted, committed,
published, terminal-green, and durably recorded, a separate BDFL preservation
signal may create the single write-once branch
`archive/workorder-10b1-rejected-2026-07-19`. Its one commit must have the then
published documentation-only `main` head as parent, contain exactly these 13
paths, reproduce the tree OID and fixture hashes, and be pushed only to that
archive ref. Retrieval of at least one Rust path and one fixture through
`git show "${ArchiveCommit}:<path>"` must reproduce their archived content
hashes before `git switch main` clears the tree. No patch, stash, second
artifact, force update, merge, cherry-pick, reset, clean, or manual deletion is
authorized. Archive execution, archive push, retrieval proof, clearing, and
its status record remain separately gated.

Only the five fixture blobs may later be materialized byte-for-byte in 10B.1b
after their hashes are checked. Rejected Rust, documentation, harness wiring,
tests, and assertions are inspection-only design evidence. 10B.1a.1 starts
from clean accepted main; it may not start with the interleaved rejected patch.

### Review, publication, and hard stops

This amendment is proposed and unauthorized. Its author is disqualified from
its verdict. A fresh independent architect-reviewer must cold-start from
repository ground truth and verify, rather than merely restate:

- both independent rejection verdicts and the one-cycle exhaustion;
- every inventory row against the accepted 10A AST, the current parser
  grammar, every admitted canonical kind/operator/intent/malformed cause,
  typed-failure wrapper, and every body/section statement relationship;
- the direct numeric element-place omission and its representation-only
  placement in 10B.1a;
- independent authority domains, exhaustive compile-time closure, single and
  pairwise corruption, cross-occurrence substitution, and anti-tautology
  locks;
- that every one of 10B.1a.1-10B.1a.11 and 10B.1b is separately review-sized,
  ordered, capped, and has an exact envelope, real paths, positive/misuse
  evidence, configuration coverage, compatibility locks, bans, and hard stop;
- that the explicit-entry control is genuinely discriminating without
  inverting parser precedence;
- the bounded static-audit claim and the stronger typed/behavioral/corruption
  proof;
- the exact eight tracked paths, five fixture paths and hashes, empty index,
  cross-shell 13-path subtree OID, and absence of 10B.2 or unrelated work; and
- unchanged 10B.2-10B.12, 10C-10F, Session AR, decisions, diagnostics outside
  the accepted H0010 allocation, archive history, and later-work bans.

The reviewer runs only `git diff --check`. The rejected implementation's green
tests are not document-acceptance evidence, and no implementation is resumed
during review. It reports P0/P1/P2 findings and exactly one verdict: `ACCEPT`,
`ACCEPT WITH REQUIRED FIX`, or `REJECT`. It must not edit, archive, clear,
commit, push, implement, or begin another increment.

Even `ACCEPT` authorizes only a BDFL-scoped `WORKORDER_10.md` documentation
commit. Separate BDFL authority remains required for publication, status,
archive execution, tree clearing, every 10B.1a subincrement, and each later
increment. Any missing inventory fact, new file need, public behavior change,
newly exposed defect, second rejection within a subincrement, or scope pressure
stops for BDFL triage. No actor may solve it through another shim, fallback
scanner, partial seal, locally expanded envelope, or implied authority.

## Increment 10B.1a.1 size-stop decomposition amendment (2026-07-19; proposed)

### Authority, stop evidence, and exact preserved identity

The BDFL authorized only this architecture/documentation pass after the first
10B.1a.1 implementation attempt obeyed the existing 500-line hard stop. This
amendment does not accept, correct, resume, archive, commit, or dispose of that
attempt. It prospectively supersedes only the sizing and sequence of the
published `10B.1a.1: source, owner, and occurrence identity kernel`. Every
fact, corruption obligation, compatibility lock, configuration requirement,
and later-unit prohibition in the published 10B.1 foundation re-scope remains
binding.

The stopped implementation is frozen for amendment review as exactly:

- baseline `HEAD == origin/main ==
  b6c5f35e447133cdcf0f7c39891db25c70b7bef8`;
- `src/ast.rs`: 92 insertions and zero deletions;
- `src/parser.rs`: 323 insertions and 6 deletions;
- empty real index and no untracked path; and
- no modification to `WORKORDER_10.md`, `tools/check_all.ps1`, 10B.1a.2, or
  later work before this amendment began.

Fresh empty temporary Git indexes containing exactly those two working-tree
paths produced this content-addressed identity:

- two-path tree OID:
  `5dc0d187645fb9c84f0cddbb81eb344efde51a09`;
- `src/ast.rs` blob:
  `bdbaad8568d27dcad3256c663938a114025594fb`; and
- `src/parser.rs` blob:
  `80b890ff8b3d2f4de9ca0aa089c1bfbf50baa7f4`.

The procedure was run independently in PowerShell and Git Bash. Each shell
indexed exactly two paths and returned the same tree and blob OIDs; both
temporary indexes were removed and the real index remained empty. Review must
reproduce that exact identity. A shell-piped textual diff hash, file count, or
line count cannot substitute for it.

The attempt stopped honestly because 421 changed lines left only 79 lines
under the absolute ceiling while the implementation still lacked all of the
load-bearing independent evidence: the fourteen-row test-only catalogue, 91
unordered coherent pair corruptions, the required cross-occurrence cases,
mutation operators, exact selector assertions, and named sabotage wiring.
Compressing those obligations into the remaining budget, omitting them, or
calling a green aggregate suite acceptance evidence is prohibited.

### Exact disposition of the stopped two-file tree

The stopped tree is inspection-only design evidence. It contains useful type
and traversal candidates, but its lines receive no presumption of correctness
and no prior implementation credit. After this amendment is independently
accepted, BDFL-accepted, committed, published, terminal-green, durably
recorded, and followed by a separate BDFL go signal for 10B.1a.1.1, the first
unit may narrow that same working tree by reviewed source edits. It may not use
`reset`, `clean`, `restore`, checkout-path, stash, patch application,
cherry-pick, or an archive round-trip to manufacture a clean-looking result.

The only candidates that may be selectively retained in 10B.1a.1.1 are:

- the type shells for `SourceBlob`, `SemanticFile`, `SourceRevision`, `Item`,
  `Section`, `Statement`, and `AuthorityHandle`;
- exact retained source-revision bytes captured by the ordinary
  `parse_source` entry;
- parser-owned item, section, and statement traversal facts; and
- source/owner authority construction that is separated from its exported
  projection and independently validated.

Even those candidates must be re-reviewed as part of the complete first-unit
diff. The current tuple fields and `Vec<usize>` payloads are crate-visible,
the current source-blob value uses source length as part of its identity, and
the current authority and projection are built together. Those shapes are not
accepted merely because their names match the inventory. Retention is allowed
only after the unit proves producer ownership, constructor opacity, exact
same-length/different-content separation, and an authority independent of the
projection under corruption.

The following stopped work is premature 10B.1a.1.2 material and must be
removed before 10B.1a.1.1 review:

- `Occurrence`, `ExpressionRole`, `Root`, `RootRange`, `Intent`,
  `AssigningEvent`, and `PredicateRecognition` types and fields;
- the fourteen-field `CanonicalOccurrenceFact`, occurrence authority, seal,
  and `OCCURRENCE_SEAL_FACT_COUNT` projection;
- `Section.expression_occurrences` and all role/root/event attachment;
- `occurrence_ranges`, `trimmed_range`, the current role-index occurrence
  minting, and predicate-recognition dispatch; and
- the current fourteen-way `validate_expression_occurrence` comparison.

The current `seal_section_occurrences` function interleaves both units and may
not be retained as a whole. First-unit source/owner logic must be isolated or
re-authored without carrying any second-unit claim. No current test catalogue,
single-field mutation, pair mutation, cross-occurrence mutation, selector, or
sabotage exists to retain. Deleting premature lines is narrowing, not evidence;
the surviving and newly written first-unit code must independently pass every
10B.1a.1.1 gate below.

10B.1a.1.2 starts only from the independently accepted, committed, published,
terminal-green, durably recorded 10B.1a.1.1 baseline. It may inspect the
removed stopped code as design evidence, but it may not copy it mechanically
or claim its former green compilation as proof. Every 10B.1a.1.2 line and test
is reviewed as new work against the cumulative fourteen-field contract.

### 10B.1a.1.1: source and owner authority kernel

Dependency role: establish the producer-owned source and owner boundary before
an occurrence, role, root, range, intent, or predicate-recognition fact can be
attached to it.

Exact envelope: `src/ast.rs`, `src/parser.rs`, and `tools/check_all.ps1` only.

This unit owns exactly seven independently catalogued fields:

1. `SourceBlob`;
2. `SemanticFile`;
3. `SourceRevision`;
4. `Item`;
5. `Section`;
6. `Statement`; and
7. `AuthorityHandle`.

The test-only `CanonicalSealField` catalogue is authored independently of the
production authority and validator and contains those exact seven literal
rows. Its expected field count is the independently supplied literal `7`.
Every unordered pair is exercised, with the independently supplied literal
count `21 = 7 * 6 / 2`; the test may not derive either expectation from the
catalogue, validator, authority, or projection it checks.

Real `parse_source` inputs must produce every field through ordinary source,
item, section, and statement traversal. At least two byte-distinct source
blobs of equal length, repeated same-spelled items/sections/statements in
separate valid sources, and multiple owner records in one source must prove
that content length, filename, display name, span, line/column, public text,
or source order alone cannot mint or select authority.

For each of the seven fields, independently corrupting, removing,
duplicating, reordering, adding, or substituting only that projection while
the retained producer authority stays fixed must fail validation. Coherently
co-mutating every one of the 21 unordered pairs while that authority stays
fixed must also fail. Valid source/owner facts from another real parsed record
must not substitute across semantic files, items, sections, statements, or
authority handles even when public-looking values are compatible.

The exact focused selector must select a documented nonzero set and must fail
if the source/owner producer arm, validator arm, catalogue row, any one of the
required mutation operators, any pair case, or the equal-length-source and
foreign-owner probes are removed. Two fresh executions must produce identical
private inventories and results. Existing human, JSON, Core, graph, runtime,
diagnostic, ordering, span, and exit bytes remain unchanged.

The unit targets 200-350 total changed lines relative to
`b6c5f35e447133cdcf0f7c39891db25c70b7bef8`. Five hundred total changed lines
across production, tests, tools, and fixtures is an absolute pre-review
ceiling. The stopped tree must be narrowed until the complete honest unit,
including evidence, fits. Crossing the ceiling, needing an unlisted file, or
leaving a load-bearing test outside the exact selector stops for a fresh
amendment; it does not authorize compression or omission.

10B.1a.1.1 adds no occurrence-role field, node payload, token or reduction
seal, H0010 allocation or behavior, Core use, parser diagnostic, semantic
consumer, public schema, fixture, or later-unit work.

### 10B.1a.1.2: occurrence and role kernel

Dependency role: attach one exact producer-owned occurrence and its role/root
facts to the accepted source/owner authority without reopening or weakening
the first seven fields.

Exact envelope: `src/ast.rs`, `src/parser.rs`, and `tools/check_all.ps1` only.

This unit adds exactly seven independently catalogued fields:

1. `Occurrence`;
2. `ExpressionRole`;
3. `Root`;
4. `RootRange`, including the exact zero-width `Unit` position;
5. `Intent`;
6. `AssigningEvent`; and
7. `PredicateRecognition`, including required presence and absence.

The cumulative independently authored `CanonicalSealField` catalogue now
contains exactly fourteen literal rows. It reruns all seven accepted first-unit
single-field mutations, adds all seven new single-field mutations, and pins the
literal cumulative pair count `91 = 14 * 13 / 2`: 21 old/old pairs, 49 old/new
pairs, and 21 new/new pairs. No expected row, count, pair, identity, or answer
may be generated from the production validator, retained authority, exported
projection, or public root summary.

The cumulative real-parse corpus must include exactly these six independently
named cross-occurrence relationship classes:

1. same bytes parsed as a distinct source blob and semantic file;
2. distinct owning items in one semantic file;
3. distinct sections in one item;
4. distinct statements in one section;
5. sibling expression roles in one statement, including target/value or
   loop-start/loop-end positions; and
6. repeated byte-identical same-line expressions whose public-looking text and
   compatible ranges cannot collapse their occurrence, role, root, event, or
   predicate-recognition identities.

For each class, substituting a valid complete occurrence or any valid field
from the foreign occurrence must fail against the retained authority. Missing,
duplicate, reordered, extra, coherent same-value, cross-owner, and
cross-occurrence projections must fail. `RootRange` is display and blame
evidence, never semantic identity; Unit's zero-width position remains exact
without becoming an identity mint.

The exact selector and permanent `tools/check_all.ps1` wiring must assert the
nonzero focused test inventory, literal `14` field count, literal `91` pair
count, and all six named cross-occurrence cases. Named sabotages must prove the
selector turns red when any producer or validator arm, catalogue row,
single-field mutation, pair family, relationship class, Unit-position check,
predicate presence/absence check, or independent-authority input is removed or
reconstructed from the projection. Two fresh runs must be byte-identical.

The unit targets 200-350 total changed lines relative to the accepted
10B.1a.1.1 commit and has the same absolute 500-line pre-review ceiling. It
must preserve every first-unit positive and misuse case. Any needed change to
the meaning, opacity, producer, validator, catalogue row, or corruption
operator of a first-unit field is a stopped-gate finding against 10B.1a.1.1,
not local 10B.1a.1.2 scope.

10B.1a.1.2 adds no common-node payload, token or reduction identity, H0010,
Core consumption, parser diagnostic, semantic consumer, public schema, or
10B.1a.2 work. Completing it completes the published 10B.1a.1 container only;
it does not complete the canonical seal foundation.

### Common checks, compatibility, and ordered publication gates

Each replacement unit separately runs its exact focused selector and sabotage
matrix, then `cargo fmt --check`, `cargo test`,
`cargo clippy --all-targets -- -D warnings`, `git diff --check`, and
`.\tools\check_all.ps1`. Each report enumerates the host production, host test,
and all-target configurations, inspects any non-host `cfg` branch, identifies
locally unavailable configurations, and leaves required Ubuntu and Windows
confirmation to the separately authorized publication CI.

Both units inherit every common 10B.1a anti-ghost, real-parse-source,
independent-authority, singleton mutation, coherent pair mutation,
cross-occurrence substitution, deterministic-repeatability,
public-compatibility, configuration, and hard-stop gate. A synthetic
successful authority, projection-derived expectation, zero-match selector,
public-only summary, hand-built occurrence, or corruption that mutates both
the projection and its supposedly independent authority receives no evidence
credit.

No byte of 10B.1a.1.2 may be implemented before 10B.1a.1.1 is independently
accepted, committed, published, terminal-green on Ubuntu and Windows, durably
recorded in this Work Order, and separately authorized by the BDFL. The same
full sequence must complete for 10B.1a.1.2 before 10B.1a.2 can begin. Every
prior reference to completion of `10B.1a.1` now means completion of both
replacement units in that order. Existing references to
`10B.1a.1-10B.1a.11` remain otherwise unchanged.

H0010 allocation and behavior, canonical node payloads, common node topology,
Core construction or consumption, parser diagnostics, semantic consumers,
10B.1a.2, 10B.1b, 10B.2, 10C, Session AR, and every later item remain
unauthorized. A newly exposed defect, second failed review within either
replacement unit, need for another file, public-byte change, inability to fit
the complete evidence under 500 lines, or any amendment need stops and returns
to the BDFL without a workaround.

### Independent pre-issuance review and authorization boundary

This amendment is proposed and unauthorized. Its author is disqualified from
its verdict. A fresh independent architect-reviewer must cold-start from
repository ground truth, reproduce the two-path Git-object identity in
PowerShell and Git Bash, inspect the complete stopped diff, and verify:

- the source/owner versus occurrence-role disposition is exact and leaves no
  second-unit fact or claim in the first unit;
- seven fields and 21 unordered pairs are complete for 10B.1a.1.1;
- seven additional fields, fourteen cumulative fields, 91 unordered pairs,
  and six nondegenerate cross-occurrence classes are complete for
  10B.1a.1.2;
- each unit retains the full common evidence, public-compatibility,
  configuration, 200-350-line target, 500-line stop, exact-envelope, and
  publication gates;
- the stopped tree receives no acceptance credit and remains byte-for-byte
  unchanged during this documentation pass; and
- H0010, node payloads, Core, semantic consumers, 10B.1a.2, and every later
  item remain unauthorized.

The reviewer runs only proportional documentation checks:
`git diff --check` and `.\tools\check_text_hygiene.ps1`. The incomplete stopped
implementation is not local preflight acceptance evidence. The reviewer must
not edit, narrow, implement, commit, push, begin either replacement unit, or
begin later work. It reports P0/P1/P2 findings and exactly one verdict:
`ACCEPT`, `ACCEPT WITH REQUIRED FIX`, or `REJECT`.

Even `ACCEPT` authorizes only a separately BDFL-scoped `WORKORDER_10.md`
documentation commit. Publication, status, narrowing of the stopped tree,
10B.1a.1.1 implementation, 10B.1a.1.2 implementation, and every later action
retain separate BDFL gates.

## Increment 10B.1a.1.1 frozen-tree size-stop amendment (2026-07-20; proposed)

### Authority, rejection history, and corrected stop

The BDFL authorized only this bounded documentation amendment. It does not
accept, resume, correct, test, review, commit, or publish the stopped
implementation. It supersedes only the 500-line pre-review stop for the one
exact corrected tree identified below; every semantic, evidence,
compatibility, configuration, envelope, sequencing, and later-work gate in
the published Increment 10B.1a.1.1 mandate remains binding.

The first 499-line implementation was independently `REJECT`ed with exactly
two P1 findings:

1. the retained source/owner authority constructed its own exported
   projection, so the purported independent validation remained
   self-validation; and
2. the `ProducerArm`, `ValidatorArm`, and `EqualLengthEvidence` sabotages
   changed test bookkeeping or answers rather than load-bearing production
   behavior.

The single authorized correction cycle removed only those shortcuts:

- exported projection construction now begins independently from ordinary
  parser-owned facts instead of a
  `CanonicalSourceOwnerAuthority::project` path;
- retained authority remains separate from the exported projection;
- producer sabotage exercises prohibited projection-derived authority
  reconstruction;
- validator sabotage bypasses an actual validation arm against a corrupted
  projection;
- equal-length sabotage changes the actual real-source control; and
- the existing 42 single-field rejections, 21 coherent unordered-pair
  rejections, and nine foreign-owner rejections remain present.

Rustfmt then produced an exact 507 changed lines. Because that crossed the
published 500-line hard ceiling, the implementer stopped immediately before
running the complete acceptance checks, requesting fresh implementation
review, committing, or pushing. No green or accepted result is inferred from
the corrected source.

### Exact frozen implementation identity

The sole tree eligible for the exception below is frozen against synchronized
`main` at
`9ca7a33e88d3fe3c387c6b9faf3d8b1c907a82b5` as exactly:

- `src/parser.rs`: 502 insertions and 2 deletions;
- `tools/check_all.ps1`: 2 insertions and 1 deletion;
- 507 total changed lines;
- clean `WORKORDER_10.md` and `src/ast.rs` at the pre-amendment boundary;
- empty real index and no untracked path; and
- no Increment 10B.1a.1.2 or later work.

A fresh temporary Git index containing exactly those two implementation paths
must produce tree OID
`70d248f77d4b851520b3a5960060b4c2d085a85b`. PowerShell and Git Bash
independently reproduced that same OID before this amendment was authored, and
all temporary indexes were removed. Line counts, path counts, a shell-piped
diff hash, or a test report cannot substitute for the Git-object identity.

### One-time exception and unchanged limits

The 200-350 changed-line range remains the project-wide review-size target.
The absolute 500-line pre-review ceiling remains unchanged for Increment
10B.1a.1.2 and every later unit. This amendment authorizes a one-time exception
only for the exact 507-line Increment 10B.1a.1.1 tree at OID
`70d248f77d4b851520b3a5960060b4c2d085a85b`.

The exception does not accept the implementation and authorizes no additional
implementation byte before review. It does not relax, replace, or reduce any
of the following accepted requirements:

- the exact seven-field independent catalogue and literal field count;
- all 42 single-field mutation rejections;
- all 21 coherent unordered-pair mutation rejections;
- all nine foreign-owner and cross-owner rejections;
- every producer, validator, catalogue, mutation-operator, pair,
  equal-length-source, and foreign-owner sabotage;
- ordinary real-`parse_source` production evidence and independently retained
  authority;
- deterministic repeatability and the exact nonzero selector;
- unchanged human, JSON, Core, graph, runtime, diagnostic, ordering, span, and
  exit bytes;
- host, all-target, unavailable-configuration, Ubuntu, and Windows coverage;
  or
- the exact file envelope and every 10B.1a.1.2 and later-work prohibition.

No actor may cite the seven-line excess to compress, remove, weaken, combine,
or reconstruct any evidence obligation. No other tree, future formatting
result, correction, cleanup, or near-equivalent diff inherits this exception.

### Exact next gate and fail-closed stop

This amendment must first receive fresh independent pre-issuance review. Even
an `ACCEPT` verdict authorizes only a BDFL-scoped `WORKORDER_10.md` commit.
Only after the exact amendment is BDFL-accepted, committed, published,
terminal-green on required Ubuntu and Windows CI, durably recorded, and
followed by a separate explicit BDFL go signal may an implementer do exactly
the following:

1. reproduce the frozen two-path OID before any acceptance command;
2. run the complete Increment 10B.1a.1.1 focused selector and sabotage matrix,
   `cargo fmt --check`, `cargo test`,
   `cargo clippy --all-targets -- -D warnings`, `git diff --check`, and
   `.\tools\check_all.ps1`; and
3. if and only if every check succeeds and the OID remains exact, submit those
   exact frozen implementation bytes to a fresh independent architect-reviewer.

That later go signal authorizes no implementation edit. If the OID changes,
any check fails, or fresh implementation review requires any correction, the
unit stops for BDFL re-scope. No further correction cycle, locally expanded
envelope, cleanup, compression, or workaround is implied.

For this document pass, the independent pre-issuance reviewer reproduces the
repository envelope and cross-shell OID, reads the complete frozen diff, and
verifies the rejection history, exact correction claims, single-tree
exception, unchanged future ceilings, complete retained evidence gates, and
fail-closed next sequence. The reviewer runs only `git diff --check` and
`.\tools\check_text_hygiene.ps1`; the unaccepted implementation must not be
used as preflight acceptance evidence. The reviewer must not edit, run
implementation acceptance checks, commit, push, resume implementation, begin
Increment 10B.1a.1.2, or perform later work. It reports P0/P1/P2 findings and
exactly one verdict: `ACCEPT`, `ACCEPT WITH REQUIRED FIX`, or `REJECT`.

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
independent review. Under the original issuance sequence, acceptance and
publication of 10F would have closed Work Order 10. The 2026-07-16 amendment
supersedes that remote-per-increment reading: accepted 10F is committed and
recorded locally, then the train stops before one separately authorized
cumulative push, required Ubuntu/Windows full CI, and the durable final closure
record. None of those gates authorizes Session AR.

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
target or mutate global configuration. Under the 2026-07-16 local train,
required post-push Ubuntu and Windows CI remain the cross-platform authority
for the final cumulative implementation head rather than being claimed for an
unpublished individual increment.

## Work Order 10 acceptance criteria

Work Order 10 closes only when all six increments are independently accepted,
committed and factually recorded in the ordered local train, the complete
cumulative implementation chain is separately authorized for publication and
green in required Ubuntu and Windows full CI, the durable final closure record
is published under its applicable gates, and evidence proves:

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

## Initial independent pre-issuance review (historical)

This section records the review gate that the initial `49e6534` issuance
satisfied. The 2026-07-16 amendment has its own independent review gate above;
the historical proposal and untracked-file assumptions below are not current
amendment-state assertions.

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
Work Order 10 was independently accepted, BDFL-accepted, committed, and
published as `49e6534a6cd3e4d567f924b69336c72563b1c95f`. Workflow
`29472827923`, attempt 1, succeeded for that exact commit. Ubuntu job
`87539307880` succeeded in 11m 09s, including 10m 40s in
`Run Hum preflight`; Windows job `87539307917` succeeded in 14m 10s,
including 13m 22s in `Run Hum preflight`. Both selected `mode=full` with
`reason=no_status_transition`; `Run status-only evidence` was skipped.
Work Order 10 is issued.

The 2026-07-16 BDFL local-train amendment was independently accepted,
BDFL-accepted, committed, and published as
`334a7416e1014232d1e47e7be49ceb730fca33b3`. Workflow `29475816732`, attempt
1, succeeded for that exact commit. Ubuntu job `87548425248` succeeded in 11m
22s, including 10m 56s in `Run Hum preflight`; Windows job `87548425235`
succeeded in 16m 12s, including 15m 43s in `Run Hum preflight`. Both selected
`mode=full` with `reason=no_status_transition`; `Run status-only evidence` was
skipped.

Increment 10A's complete six-path implementation received final independent
`ACCEPT` after one bounded correction cycle. It is committed locally as
`935550a4f40bcf425ddbc22f235b0011893219ae`, with first parent
`334a7416e1014232d1e47e7be49ceb730fca33b3`, and contains exactly:

- `src/ast.rs`;
- `src/parser.rs`;
- `src/core_body.rs`;
- `tools/check_all.ps1`;
- `fixtures/foundation/pre_ar_text_braces_pass.hum`; and
- `fixtures/foundation/pre_ar_real_unclosed_block_fail.hum`.

The accepted implementation passed formatting, 427 Rust tests,
warnings-denied all-target Clippy, diff hygiene, the complete Hum preflight,
105 classifier cases twice deterministically, 16 Windows-locality tests, 60
effect-bakeoff tests, and 489-file text/public/release readiness. Only
`x86_64-pc-windows-msvc` was locally installed; the change introduced no
dependency, feature, `cfg`, or platform-specific branch. The worktree, index,
and untracked set were clean immediately after the implementation commit and
before its local status edit. The implementation and status commits are now
published in the ordered chain above `334a741`.

Increment 10B was activated after the exact 10A status record, implemented,
and independently `REJECT`ed. Its 18 modified tracked files and two untracked
fixtures remain preserved with an empty index. The independently reproduced
arithmetic/contract results and 431-test/full-preflight success are
implementation evidence, not acceptance. The rejected tree is not committed.

The 2026-07-16 10B rejection amendment above records the complete rescan
inventory, resolves Predicate v2's representational blocker, allocates H0010
under the exact BDFL ruling, and freezes the one-pass correction envelope. It
was independently `ACCEPT`ed after one document-only fingerprint correction,
BDFL-accepted including H0010, committed, and published as
`098d5d3f2fa616c8faa3b6f4e4d8312f95f23ce7`, with first parent
`89c18ed363b78e725aa1a2736a24f21b08d29636`.

Workflow `29530510693`, attempt 1, succeeded for that exact amendment head.
Ubuntu job `87729422199` succeeded in 10m 02s, including 9m 36s in
`Run Hum preflight`; Windows job `87729422210` succeeded in 17m 10s,
including 16m 02s in `Run Hum preflight`. Both jobs selected `mode=full` with
`reason=no_status_transition`, succeeded in Cargo caching and Rust toolchain
preparation, completed the full Hum preflight, skipped
`Run status-only evidence`, and concluded success. Live and local
`origin/main` named `098d5d3f2fa616c8faa3b6f4e4d8312f95f23ce7` at that gate.

The bounded H0010 allocation-ripple amendment above was independently
`ACCEPT`ed and BDFL-accepted. It was committed and published as
`ebc59fba003fb16540f2f8e37f8a5c4a5810d544`, with first parent
`812a3766e041f4275f6d770e753a37c17c7cc250`.

Workflow `29539840435`, attempt 1, succeeded for that exact amendment head.
Ubuntu job `87759532113` on `ubuntu-latest` succeeded in 11m 09s, including
10m 34s in `Run Hum preflight`; Windows job `87759532117` on
`windows-latest` succeeded in 13m 27s, including 12m 28s in
`Run Hum preflight`. Both jobs selected `mode=full` with
`reason=no_status_transition`, succeeded in Cargo caching and Rust toolchain
preparation, completed the full Hum preflight, skipped
`Run status-only evidence`, and concluded success. `HEAD`, local
`origin/main`, and live remote `main` now name
`ebc59fba003fb16540f2f8e37f8a5c4a5810d544`.

The monolithic 10B implementation and its bounded correction were each
independently `REJECT`ed. The second verdict repeated the first verdict's
architecture shape: parallel expression/call authority, nonrecursive H0010,
and rendered-expression reparsing by semantic consumers. Under the repository
loop-diagnosis rule, the old correction cycle is terminated and cannot receive
a third attempt.

The current correction tree remains preserved, uncommitted, and unaccepted as
31 modified tracked implementation, documentation, and tool files plus six
untracked foundation fixtures, with an empty index. Its reproducible 31-path
Git snapshot tree OID is `0fac92602f632dbc145d641d73e74bd9ac15c545`;
the former shell-piped diff value is historical only. Its 5,465 insertions and
4,143 deletions, green ordinary checks, and reproduced positive output are
evidence about the rejected design attempt, not acceptance.

The 2026-07-16 repeated-rejection re-scope amendment above received final
independent `ACCEPT` with no P0, P1, or P2 findings and was BDFL-accepted,
committed, and published as
`1dcccf6e1285ceee6d78ac7166bb166bed3126a1`, with first parent
`5691e17b1a7d01b036007cdce39108471df94641`. That commit changes only
`WORKORDER_10.md`, with 517 insertions and 41 deletions.

Workflow `29556009214`, attempt 1, succeeded for that exact amendment head.
Ubuntu job `87808081590` on `ubuntu-latest` succeeded in 10m 58s, including
10m 25s in `Run Hum preflight`; Windows job `87808081602` on
`windows-latest` succeeded in 18m 08s, including 17m 24s in
`Run Hum preflight`. Both jobs selected `mode=full` with
`reason=no_status_transition`, succeeded in Cargo caching and Rust toolchain
preparation, completed the full Hum preflight, skipped
`Run status-only evidence`, and concluded success. `HEAD`, local
`origin/main`, and live remote `main` name that exact commit. The re-scope
amendment is durably published.

The amendment preserves the canonical parser-owned tree, records the
representation gaps and per-consumer migration graph, chooses clean-baseline
reimplementation with selective design salvage, requires the un-shimmable
source/behavior/tree-corruption gate, makes H0010 recursive, schedules final
duplicate-authority retirement, and separates the dead exact-selector repair
as urgent 10B.0 harness work.

The rejected monolithic 10B worktree remains preserved, unchanged,
uncommitted, and unaccepted. Its ordinary green checks remain implementation
evidence only. 10B.0 is the next planning target but remains unauthorized.
Disposition or deletion of the preserved tree requires separate explicit BDFL
authority. 10B.0 requires a separate BDFL go signal after this status record is
accepted and durably published. Every 10B.0-10B.12 unit retains its own go
signal and stop; none may begin implicitly. 10B.1, 10C, Session AR, and every
later item remain unauthorized.

The rejected-tree archival and clean-baseline amendment above was independently
`ACCEPT`ed and BDFL-accepted, committed, and published as
`58ed878312338f5d056f30e1d00846f91a7cc953`. Workflow `29563872980`, attempt
1, succeeded for that exact commit. Ubuntu job `87832013964` on
`ubuntu-latest` succeeded in 10m 00s, including 9m 32s in
`Run Hum preflight`; Windows job `87832013993` on `windows-latest` succeeded
in 18m 42s, including 17m 52s in `Run Hum preflight`. Both jobs selected
`mode=full` with `reason=no_status_transition`, succeeded in Cargo caching and
Rust toolchain preparation, completed the full Hum preflight, skipped
`Run status-only evidence`, and concluded success. The amendment is durably
published. Its publication status record was independently accepted,
BDFL-accepted, committed, and published as
`74828ae5f26b0d2eea069452b0a0c9080cd5581a`.

The separately authorized rejected-tree archival is complete. Its exact
identities are:

- `$ArchiveBase`:
  `74828ae5f26b0d2eea069452b0a0c9080cd5581a`;
- `$ArchiveCommit`:
  `3fdf236b0076534766ef89b592b3358f67a6315d`;
- local and live remote write-once branch:
  `archive/workorder-10b-rejected-monolith-2026-07-17`;
- `$ArchiveCommit^ == $ArchiveBase`; and
- exact 31-path non-fixture Git snapshot tree OID:
  `0fac92602f632dbc145d641d73e74bd9ac15c545`.

The archive commit contains exactly these 37 paths and no other path:

- `docs/DIAGNOSTICS.md`;
- `docs/FORMAL_CORE.md`;
- `docs/HUM_CORE_LOWER_SCHEMA.md`;
- `docs/HUM_CORE_PREVIEW_SCHEMA.md`;
- `docs/HUM_CORE_VERIFY_SCHEMA.md`;
- `docs/LANGUAGE_REFERENCE.md`;
- `src/ast.rs`;
- `src/check.rs`;
- `src/core_body.rs`;
- `src/core_expr.rs`;
- `src/core_lower.rs`;
- `src/core_preview.rs`;
- `src/core_verify.rs`;
- `src/diagnostic_catalog.rs`;
- `src/diagnostics.rs`;
- `src/effect_check.rs`;
- `src/field_place.rs`;
- `src/full_type_check.rs`;
- `src/main.rs`;
- `src/ownership_check.rs`;
- `src/parser.rs`;
- `src/path_boundary.rs`;
- `src/predicate.rs`;
- `src/resolve.rs`;
- `src/resource_check.rs`;
- `src/return_dependency.rs`;
- `src/run.rs`;
- `src/type_check.rs`;
- `src/typed_failure.rs`;
- `src/writable_field_alias.rs`;
- `tools/check_all.ps1`;
- `fixtures/foundation/pre_ar_body_contract_expression_agreement_pass.hum`;
- `fixtures/foundation/pre_ar_comparison_conjunction_pass.hum`;
- `fixtures/foundation/pre_ar_condition_chained_comparison_fail.hum`;
- `fixtures/foundation/pre_ar_left_associative_arithmetic_pass.hum`;
- `fixtures/foundation/pre_ar_predicate_chained_comparison_fail.hum`; and
- `fixtures/foundation/pre_ar_return_chained_comparison_fail.hum`.

The six fixture blobs in that inventory reproduce these exact SHA-256 hashes:

- `pre_ar_body_contract_expression_agreement_pass.hum`:
  `c40056a83eff8580e757ea6955892b98ce57d4325707a978f0df0603f4329381`;
- `pre_ar_comparison_conjunction_pass.hum`:
  `3081b3ba84045cb64bb8c049fde683cbfc64c91dae956e9a71d012b193951433`;
- `pre_ar_condition_chained_comparison_fail.hum`:
  `c49bc27b53c2fbbfa8012525c25e756eb8da4871fe83ea2b6caec94466bc9d41`;
- `pre_ar_left_associative_arithmetic_pass.hum`:
  `22b2e9c09c9a5ed8f3984ccc08c318ff56922c5ebd89369092bf375f398ff3e9`;
- `pre_ar_predicate_chained_comparison_fail.hum`:
  `7376de1f01f018943174876886ea37da02e5a4458b032d24214f5ff4116e8d30`;
  and
- `pre_ar_return_chained_comparison_fail.hum`:
  `6096390130a62ddc5a2128b936b188d05a0a63aa9036cad4ffd84c7b16207fb8`.

Before clearing, and again from clean `main`, the recorded
`git show "${ArchiveCommit}:<path>"` retrieval path recovered
`fixtures/foundation/pre_ar_condition_chained_comparison_fail.hum` with the
exact frozen hash above. The non-force archive-ref push succeeded, the live
remote ref equals `$ArchiveCommit`, and no workflow ran for the archive commit.
`git switch main` was the sole clearing operation; at execution completion,
`HEAD`, local `origin/main`, and live remote `main` all equaled `$ArchiveBase`,
and the worktree, index, and untracked set were empty. No rejected archive byte
was merged into `main`.

The archive-execution status record was independently accepted, committed, and
durably published as `2e492e9e830a50dfd5e16bd9c7e22bd02043da3c`.
Required workflow `29604936061`, attempt 1, completed successfully for that
exact commit. Ubuntu job `87965789896` on `ubuntu-latest` succeeded in 11m11s;
its Cargo cache, Rust-toolchain preparation, and `Run Hum preflight` steps
succeeded, with preflight completing in 10m41s, while `Run status-only
evidence` was skipped. Windows job `87965789902` on `windows-latest` succeeded
in 20m22s; its Cargo cache, Rust-toolchain preparation, and `Run Hum preflight`
steps succeeded, with preflight completing in 19m32s, while `Run status-only
evidence` was skipped. Both jobs selected `mode=full` with
`reason=no_status_transition`.

The BDFL's later disjoint process-documentation and public-readiness commits
through `89fbff66a846471135530a2bb40272245ee3a32e` do not change any archive
identity, retrieval fact, or cleared-tree state. The rejected monolith remains
archived and recoverable at
`archive/workorder-10b-rejected-monolith-2026-07-17`, exact commit
`3fdf236b0076534766ef89b592b3358f67a6315d`; `main` remains cleared. The
rejected-tree archival lifecycle is complete.

Increment 10B.0's exact two-path dead-selector repair received final
independent `ACCEPT` after zero correction cycles and was committed and
published as `bc8140e668483ef2cb4042a5b8eb9a66caa820b9`, with first parent
`4b1030b79c9cbeab1afccc9e75d953062ad48f3b`. Its exact committed envelope is
`tools/check_all.ps1` and `tools/test_exact_rust_selector.ps1`. Required
workflow `29615977602`, attempt 1, succeeded for that exact commit. Ubuntu job
`88001070186` on `ubuntu-latest` succeeded in 10m 16s; Cargo caching and Rust
toolchain preparation succeeded, `Run Hum preflight` succeeded in 9m 48s, and
`Run status-only evidence` was skipped. Windows job `88001070180` on
`windows-latest` succeeded in 16m 10s; Cargo caching and Rust toolchain
preparation succeeded, `Run Hum preflight` succeeded in 15m 21s, and
`Run status-only evidence` was skipped. Both jobs selected `mode=full` with
`reason=no_status_transition`. The repair is complete and durably published;
the independently audited first-parent history confirms that no published
commit passed while the selector was dead. Increment 10B.1 is the next target
but remains unauthorized pending independent acceptance and durable
publication of this status record plus a separate explicit BDFL go signal.
10B.2, 10C, Session AR, and every later item remain unauthorized.

Increment 10B.1 was then implemented, independently `REJECT`ed, corrected once,
and independently `REJECT`ed again. The correction cycle is exhausted, no
implementation is accepted, and no third correction against that scope is
authorized. The exact rejected state remains preserved with an empty index as
the eight tracked paths and five fixtures recorded in the proposed 2026-07-19
re-scope amendment; its cross-shell 13-path Git subtree OID is
`af756a7fea21353794de585869a7d2df487fe663`.

The amendment replaces 10B.1 prospectively with the ordered
10B.1a.1-10B.1a.11 canonical-seal foundation train followed by 10B.1b recursive
H0010 consumption and discriminating entry evidence. It bounds the direct
source audit honestly and requires a separately authorized write-once archive
before the rejected tree may be cleared. The amendment received independent
`ACCEPT` with no P0, P1, or P2 findings, was BDFL-accepted, and was committed
and durably published as `4fcd67473777751994c989363034e794a8624e5f`.
Workflow `29705697959`, attempt 1, succeeded for that exact commit. Ubuntu job
`88242175358` on `ubuntu-latest` succeeded in 8m 15s, including 7m 35s in
`Run Hum preflight`; Windows job `88242175363` on `windows-latest` succeeded
in 16m 49s, including 16m 07s in `Run Hum preflight`. Both selected
`mode=full` with `reason=no_status_transition`; Cargo caching and Rust
toolchain preparation succeeded, and `Run status-only evidence` was skipped.

Under the subsequent explicit BDFL execution signal, the rejected 10B.1
implementation was archived from `$ArchiveBase`
`1584783763ac6eec3afd9b9850bde895d1b37365` as `$ArchiveCommit`
`6f1caf857908c03769e5126eeba0df8af7d01b34` on the write-once local and live
remote branch `archive/workorder-10b1-rejected-2026-07-19`.
`$ArchiveCommit^ == $ArchiveBase`, and the archive commit contains exactly
these 13 paths and no other path:

- `docs/DIAGNOSTICS.md`;
- `docs/LANGUAGE_REFERENCE.md`;
- `fixtures/foundation/pre_ar_comparison_conjunction_pass.hum`;
- `fixtures/foundation/pre_ar_condition_chained_comparison_fail.hum`;
- `fixtures/foundation/pre_ar_nested_chained_comparison_fail.hum`;
- `fixtures/foundation/pre_ar_predicate_chained_comparison_fail.hum`;
- `fixtures/foundation/pre_ar_return_chained_comparison_fail.hum`;
- `src/ast.rs`;
- `src/core_body.rs`;
- `src/diagnostic_catalog.rs`;
- `src/diagnostics.rs`;
- `src/parser.rs`; and
- `tools/check_all.ps1`.

Fresh empty temporary indexes in PowerShell and Git Bash each loaded exactly
those 13 archived paths and independently reproduced subtree OID
`af756a7fea21353794de585869a7d2df487fe663`. The five archived fixture blobs
reproduce these exact SHA-256 hashes:

- `pre_ar_comparison_conjunction_pass.hum`:
  `3081b3ba84045cb64bb8c049fde683cbfc64c91dae956e9a71d012b193951433`;
- `pre_ar_condition_chained_comparison_fail.hum`:
  `c49bc27b53c2fbbfa8012525c25e756eb8da4871fe83ea2b6caec94466bc9d41`;
- `pre_ar_nested_chained_comparison_fail.hum`:
  `15e0caed7466978b95c1867d9492b83844197b3dc59754cd7805949f8b5a5b50`;
- `pre_ar_predicate_chained_comparison_fail.hum`:
  `7376de1f01f018943174876886ea37da02e5a4458b032d24214f5ff4116e8d30`;
  and
- `pre_ar_return_chained_comparison_fail.hum`:
  `6096390130a62ddc5a2128b936b188d05a0a63aa9036cad4ffd84c7b16207fb8`.

Before archival, `src/parser.rs` had SHA-256
`6b5466ad403286ddcb322cd6a0eabe1025ddafbeb9d9434e6063cbf8ed74298b`,
and the condition-chain fixture had SHA-256
`c49bc27b53c2fbbfa8012525c25e756eb8da4871fe83ea2b6caec94466bc9d41`.
Raw byte-stream retrieval through the recorded
`git show "${ArchiveCommit}:<path>"` route reproduced both exact hashes, and
all temporary retrieval and index files were removed.

Only after archive publication and both retrieval proofs succeeded did
`git switch main` perform the sole clearing operation. `HEAD`, local
`origin/main`, and live remote `main` all remain at `$ArchiveBase`; the main
worktree, real index, and untracked set are empty; `WORKORDER_10.md` is
unchanged outside this status record; local and live archive refs both equal
`$ArchiveCommit`; and no workflow ran for the archive branch. No rejected
archive byte was merged into `main`. The rejected Increment 10B.1 preservation
lifecycle is complete.

Increment 10B.1a.1 received its separate BDFL go signal and stopped before
review at the mandatory size ceiling with exactly the two modified source
paths and Git-object identity recorded in the proposed size-stop decomposition
amendment. The implementation remains uncommitted, unaccepted, and preserved
with an empty index and no untracked path at exact two-path tree OID
`5dc0d187645fb9c84f0cddbb81eb344efde51a09`.

The decomposition amendment received independent `ACCEPT` with no P0, P1, or
P2 findings and was BDFL-accepted, committed, and durably published as
`c9284b4125127339e1a8ab56c456d71bec2c7aab`. Workflow `29714772931`, attempt
1, succeeded for that exact commit. Ubuntu job `88265749122` on
`ubuntu-latest` succeeded in 10m 15s, including 9m 45s in
`Run Hum preflight`; Windows job `88265749118` on `windows-latest` succeeded
in 16m 17s, including 15m 32s in `Run Hum preflight`. Both selected
`mode=full` with `reason=no_status_transition`; Cargo caching and Rust
toolchain preparation succeeded, and `Run status-only evidence` was skipped.

The stopped two-path implementation remains preserved and unaccepted.
Narrowing remains unauthorized pending independent acceptance and durable
publication of this status record plus a separate explicit BDFL signal.
Increment 10B.1a.1.1 is the next target but remains unauthorized. Increment
10B.1a.1.2, 10B.1a.2, 10B.1b, 10B.2, 10C, Session AR, and every later item
remain unauthorized.

Increment 10B.1a.1.1 then received its separate BDFL go signal. Its initial
499-line implementation was independently `REJECT`ed for self-validating
authority/projection construction and non-load-bearing producer, validator,
and equal-length sabotages. The single authorized correction removed those
shortcuts while retaining the complete 42 single-field, 21 pair, and nine
foreign-owner cases. Rustfmt produced 507 changed lines, so the implementer
stopped at the published 500-line ceiling before acceptance checks, fresh
review, commit, or push.

The corrected implementation remains frozen, uncommitted, and unaccepted as
only `src/parser.rs` and `tools/check_all.ps1`, with exact two-path tree OID
`70d248f77d4b851520b3a5960060b4c2d085a85b` independently reproduced in
PowerShell and Git Bash. The proposed one-time exact-tree size-stop amendment
grants no acceptance or additional implementation authority. Increment
10B.1a.1.1 remains paused pending independent document acceptance, a
BDFL-authorized documentation commit, publication, terminal-green required
CI, durable status recording, and a separate explicit BDFL go signal for
checks and fresh review of those exact frozen bytes. Increment 10B.1a.1.2 and
every later item remain unauthorized.

The frozen-tree size-stop amendment received independent `ACCEPT` with no P0,
P1, or P2 findings and was BDFL-accepted, committed, and durably published as
`862f4a09f527b12e3ebf66059b7cef6be7c5d66c`. Workflow `29721006041`,
attempt 1, succeeded for that exact commit. Ubuntu job `88283875461` on
`ubuntu-latest` succeeded in 10m 14s, including 9m 51s in
`Run Hum preflight`; Windows job `88283875494` on `windows-latest` succeeded
in 16m 58s, including 16m 16s in `Run Hum preflight`. Both selected
`mode=full` with `reason=no_status_transition`; Cargo caching and Rust
toolchain preparation succeeded, and `Run status-only evidence` was skipped.

The corrected 507-line implementation remains frozen, uncommitted,
unaccepted, and byte-identical at two-path tree OID
`70d248f77d4b851520b3a5960060b4c2d085a85b`. No implementation acceptance
check has run against it. The only next eligible action is to reproduce that
exact OID, run the complete Increment 10B.1a.1.1 acceptance checks, and submit
those exact bytes for fresh independent implementation review. That action
remains unauthorized pending independent acceptance, commit, publication,
and terminal required CI for this status record plus a separate explicit BDFL
go signal. No implementation edit or further correction cycle is authorized.
Increment 10B.1a.1.2, 10B.1a.2, 10B.1b, 10B.2, 10C, Session AR, and every
later item remain unauthorized.

Session AR remains the next globally lettered session but is reserved for a
future fresh Hum IR/minimal compiler-ready standard-library Work Order. It has
not been authored, reviewed, issued, or authorized here. 10F completion must
stop and return to the BDFL for the broad independent foundation audit before
any AR planning.

No dirty-tree cleanup, 10B subincrement, 10C, GitHub Issue mutation,
`PRE-AQ-INTEGRITY`, dashboard, Session AR, Hum IR, standard-library, backend,
cumulative closure, foundation-audit, or later work is authorized.

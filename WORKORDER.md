# Hum Work Order 7: Predicate V2 And The Effect-Polymorphism Bake-Off

Date: 2026-07-11
Status: issued; independently reviewed and accepted by the BDFL; the complete
stack is commit `c10a210`, with Ubuntu and Windows CI passed in workflow
`29145110803`; Session AF is accepted and committed as `7991ef6`, with Ubuntu
and Windows CI passed in workflow `29160088909`; Session AG is next but remains
unauthorized pending a separate BDFL go signal; Sessions AH-AK and all later
work remain unauthorized; no implementation session, decision ruling, commit,
push, or scope expansion is implicitly authorized
Owner: BDFL (Ocean). Work-order author: architect-reviewer acting under the
bounded planning authorization. Independent pre-issuance reviewer: a fresh
architect-reviewer that did not author or edit this deliverable. Future
implementer: agent sessions only after the required gates.
Predecessor: Work Order 6, Sessions V-AE, closed and committed at `d601054`.
The closed Work Order 6 text remains in git history and is not reopened by this
order.

## Authority and issuance gate

This file is the issued Work Order 7 sequence, not authorization for an
implementation session or decision ruling. Independent pre-issuance review and
BDFL acceptance are complete in commit `c10a210`; Ubuntu and Windows CI passed
in workflow `29145110803`. Before Session AF implementation begins, the final
gate below must still occur:

1. the BDFL gives a separate go signal for Session AF.

Acceptance and issuance of this stack authorize only the sequence and gates
written here. Each session still requires its own independent verdict,
accepted commit and CI evidence recorded here, and a separate BDFL go signal for the next
session. No review, commit, push, CI success, decision proposal, or delegated
ruling silently authorizes the following session.

## Evidence authority and governance reconciliation

The accepted Session AE retrospective is the authorization source:

- `docs/CORE_LANGUAGE_SHAPE.md` records exactly three active Predicate v2
  vocabulary records and applies the three-strike rule;
- `docs/bakeoff/SCORECARD.md` requires Predicate v2 first, then the
  effect-polymorphism bake-off;
- `examples/probes/word_count.hum` currently checks only the weaker hard-coded
  equality `result == 2`, not its intended content-conditional relation;
- `examples/probes/list_builder.hum` keeps its list-content intent as unchecked
  prose; and
- `examples/probes/element_views.hum` keeps Text-literal equality as unchecked
  prose.

Governance names effect polymorphism and higher-order tasks as bake-off queue
item one, but also states that the newest retrospective ledger re-decides the
queue authorization and that Session AE's ledger currently holds that power.
This order does not amend governance or demote the effect bake-off. It pays the
triggered, bounded Predicate v2 debt first, then resumes governance queue item
one. The remaining governance bake-off queue is unchanged.

Research is input, not authority. The newest snapshots are skeptical evidence
links, not accepted recommendations:

- `docs/research/2026-07-10-overnight-research-triage.md`;
- `docs/research/deep/2026-07-10-effect-polymorphism-corpus.txt`; and
- `docs/research/deep/2026-07-10-flagship-wedge.txt`.

In particular, the research preference for Koka-style rows is not a Hum
decision. Koka-style rows, Flix-style Boolean formulas, and capture-oriented
checking remain genuine candidates until the repository corpus and prototypes
eliminate one transparently. Effekt-style second-class computation is not
silently rejected from a research summary: Session AG must record, against the
pinned model-neutral requirements, whether its inability to store callbacks or
return closures makes it ineligible without candidate-specific restructuring.

## Mandatory sequence

Work Order 7 takes one path:

```text
AF  exact-three Predicate v2 repair
AG  pinned model-neutral effect/higher-order corpus and experimental harness
AH  Koka-style row-polymorphism best-advocate prototype
AI  Flix-style Boolean-formula best-advocate prototype
AJ  capture-oriented checking best-advocate prototype
AK  cross-candidate scorecard and proposed effect-polymorphism decision
```

The order is mandatory. AF must be accepted before AG begins. The corpus must
be accepted and frozen before any candidate prototype begins. Every candidate
uses the same corpus and harness. Scoring and model selection occur only after
all three candidate sessions are independently accepted. Work Order 7 does not
implement the selected effect model in the production compiler or language.

## Global accepted-decision locks

### Decision 0015: contracts

Every recognized executable `needs:` and `ensures:` predicate continues to run
under `hum run`. Predicate v2 adds vocabulary only. This order may not add or
imply:

- a decision 0015 proof/trust contract classifier or output such as `proved`,
  `boundary`, `unproved`, or `external-trust`;
- debug/release contract profiles or build-mode policy;
- proof or evidence fields for contract discharge;
- contract elision or an optimization based on a written predicate;
- a conclusion that a defensive body guard is unreachable; or
- a global contract enable/disable toggle.

This ban does not prohibit AF's closed lexical/parser/type recognition status,
named `predicate_recognition_status`. That status says only whether contract
text is non-executable prose, a malformed candidate, semantically rejected, or
a recognized typed executable predicate. It carries no proof, trust-boundary,
build-mode, elision, optimization, enforcement-profile, or unreachable-code
meaning and is never a decision 0015 classification.

### Decision 0014: ownership

Session V earned only the exact local direct-field writable-alias slice already
recorded by decision 0014. This order does not add or claim internal references,
stored aliases, nested or element aliases, general disjoint-place inference,
broad flow-sensitive borrowing, general closure-capture soundness, full
ownership safety, borrow soundness, or memory-safety completeness.

Candidate prototypes must model captured owned and Transaction-shaped linear
resources honestly. Prototype evidence is not production ownership evidence
and cannot narrow a decision 0014 lock.

### Decision 0017: authority

App/task `uses:` declarations remain source budgets, never operator consent.
Exact operator grants, default deny, deny-wins, native Path identity, finite
source routes, and the source-policy/decision/exercise distinction remain
unchanged. Effect variables, formulas, or capture sets may describe latent or
captured requirements; they may not:

- create authority, widen an app maximum, or manufacture operator consent;
- erase exact capability identity or native Path identity;
- turn ambient process authority into an inferred effect;
- merge source policy, operator decision, and operation exercise facts;
- hide captured authority or resources inside an unannotated closure value; or
- call the Windows bootstrap adapter portable IO or a filesystem sandbox.

## Global scope and safety boundaries

Unless a session below names an exact exception, the following remain banned:

- production closures, lambdas, tasks-as-values, callbacks, higher-order
  standard-library APIs, effect variables, effect formulas, capture syntax,
  handlers, effect subtraction, or production effect inference;
- internal references or any other ownership repair;
- concurrency, threads, scheduling, parallel execution, cancellation, shared
  state, async runtime behavior, or fault-domain semantics;
- new IO operations, capabilities, grants, prompts, persistence, wildcards,
  host clocks, randomness, process execution, environment reads, network,
  unsafe, FFI, plugins, dynamic loading, or package behavior;
- Bytes, directories, enumeration, hashing, manifests, canonical JSON IO, file
  writing, evidence directories, signature verification, or sandbox claims;
- a new CLI subcommand, public report surface, runtime JSON surface, runtime
  profile, production schema identifier, backend, or pipeline stage; and
- public-alpha, complete effect-safety, deterministic-mode, full ownership,
  portable-IO, or flagship-tool claims.

Session AG is expressly allowed to add one isolated offline experimental Rust
crate under `experiments/effect-bakeoff/`, one model-neutral bake-off document,
one corpus data set, and a proportional existing-preflight invocation. Sessions
AH-AJ may add candidate modules and advocate documents only inside that
experimental boundary. Those artifacts are not production Hum syntax,
compiler stages, schemas, or runtime behavior.

All tests remain local and offline. They may use only checked-in fixtures or
isolated temporary data, and may not access the network, host clock, process
environment as semantic input, user files, or paths outside their test
directory.

## Standing evidence, diagnostic, and cross-stage gate

Every session must satisfy its local criteria plus these standing rules:

1. Positive evidence observes the claimed behavior. Declaration-only,
   unused-binding, weakened, TBD, or candidate-specific substitute fixtures do
   not count.
2. Every production source rejection introduced by AF has one stable H-code,
   one fundamental diagnostic, structured primary and related sites, and help
   naming the violated type/relationship and exact repair. Human and existing
   JSON surfaces agree. Runtime preflight uses the same shared fact and code.
3. Combined-cause fixtures pin precedence. A specific Predicate v2 type/shape
   failure cannot be masked by H0701 prose warning, resolver unknown-name,
   generic full-type blockers, or a runtime trap.
4. Resolver, type environment, type check, full type, effect, ownership,
   resource, Core preview/lower/verify, graph, and runtime agree on every AF
   form. A stage may expose an exact honest blocker, but may not call an
   accepted predicate prose, unchecked, unsupported, or more general than it
   is.
5. Candidate sessions add no Hum H-codes. Experimental diagnostics use stable
   corpus-case IDs, deterministic reason IDs, one primary explanation, and
   structured blame relationships, but are not production diagnostics or a
   public schema.
6. Candidate results separate `proven by this prototype` from `requires
   unimplemented machinery`. Hidden allocation, erased effects, ambient
   capabilities, unimplemented inference, future ownership repairs, and
   candidate-specific program restructuring receive no credit.
7. Host `cargo fmt --check`, `cargo test`, and warnings-denied Clippy must pass.
   The isolated experiment tests, targeted fixtures, `git diff --check`, and
   `.\tools\check_all.ps1` must pass. Platform/configuration gaps are enumerated
   honestly; no unproven cross-target command is mandated.
8. Leave each session uncommitted and stop for independent review. The reviewer
   issues exactly one verdict and may authorize only the scoped commit. The
   BDFL alone may authorize push and the next session.

## Session AF: exact-three Predicate v2 repair

Purpose: pay the active contract-vocabulary trigger as one coherent bounded
repair before higher-order work.

### Exact executable vocabulary

Predicate v2 retains Predicate v1's one-comparison-per-line structure,
arithmetic, `old(place)`, and `list_len(place)`. It adds only:

1. Text operands and Text literals for `==` and `!=`.
2. Exact ordered `List Text` content equality against a `List Text` literal for
   `==` and `!=`. Equality compares length and every UTF-8 Text value in order;
   it is not membership, subset, set, locale, normalization, or collation.
3. The contract-only operand form:

   ```hum
   list_count(list_text_operand, text_operand)
   ```

   It returns `UInt` equal to the number of elements exactly equal to the Text
   operand. The list operand may be an allowed `List Text` parameter/result
   place or a `List Text` literal. The match operand may be an allowed Text
   parameter/result place or Text literal. It is not a predicate task, closure,
   arbitrary helper call, general collection fold, or body/stdlib API.

The three source contracts become exactly:

```hum
result == list_count(["hum", "lang", "hum", "agent"], "hum")
result == ["parse", "check", "run"]
result == "parse"
```

The first replaces only `word_count`'s weaker hard-coded equality. The second
replaces only `builder_demo`'s unchecked list-content prose; its checked
`list_len(result) == 3` may remain as separate evidence. The third makes the
existing `element_views` Text equality executable. Existing result types and
program behavior do not change.

### Complete H0701/H0704 lexical ownership boundary

The shared analyzer first assigns recognition status to the complete trimmed
contract line. Blank lines, comments, and already-owned hollow-contract cases
retain their existing handling. Whitespace productions are exact:

```text
hws0 := *(SP / HTAB)
hws1 := 1*(SP / HTAB)
```

`hws0` is zero or more ASCII spaces or tabs where spacing is optional. `hws1`
is one or more ASCII spaces or tabs where a boundary rule requires separation.
Neither production consumes CR, LF, or any other Unicode whitespace, and a
contract line contains no executable newline.

An executable-predicate intent signal exists when, outside a well-formed Text
literal, any of these holds:

1. any punctuation from `=`, `<`, or `>` occurs;
2. `!` is followed by `hws0` and `=`, or is lexically between a complete
   operand-ending token and, after `hws0`, an accepted operand-starting token;
3. a token-boundary spelling of `old`, `list_len`, or `list_count` is followed
   immediately by `(` or by `hws1` and `(`.

For rule 2, operand-ending tokens are an identifier, boolean, integer, valid
Text literal, `)`, or `]`; operand-starting tokens are the exact starters listed
below. Thus `result ! 2`, `result!2`, and `(result) ! "x"` enter the malformed
candidate envelope, while terminal prose punctuation such as `must hold!` does
not.

Text-literal contents are opaque to this scan. Therefore prose such as
`completed list contains parse check run`, `result equals two`, and the whole
Text literal `"result == list_count(items, hum)"` has no intent signal and
remains H0701. `result = 2`, `result === 2`, `result ! = 2`, `result ! 2`,
`list_count (items, "hum") == result`, and `helper(items) == result` do have an
intent signal and enter the candidate envelope. A token such as `list_counted`
does not trigger rule 3, but an unquoted comparison operator later on the same
line still triggers rule 1. Only a Text literal accepted by the existing lexer
shields its contents. An unterminated or otherwise malformed quote does not
hide punctuation; both `result == "parse` and `"result == 2` enter the envelope
and receive H0704.

After an intent signal, the only accepted grammar is:

```text
predicate       := operand hws0 comparison hws0 operand
comparison      := "==" | "!=" | "<=" | ">=" | "<" | ">"
operand         := additive
additive        := multiplicative (hws0 ("+" | "-") hws0 multiplicative)*
multiplicative  := primary (hws0 ("*" | "/") hws0 primary)*
primary         := boolean | integer | place | text_literal
                 | list_text_literal | old_call | list_len_call
                 | list_count_call | "(" hws0 operand hws0 ")"
place           := identifier | identifier "." identifier
old_call        := "old(" hws0 place hws0 ")"
list_len_call   := "list_len(" hws0 place hws0 ")"
list_count_call := "list_count(" hws0 list_source hws0 "," hws0 text_source hws0 ")"
list_source     := place | list_text_literal
text_source     := place | text_literal
list_text_literal := "[]"
                   | "[" hws0 text_literal
                     (hws0 "," hws0 text_literal)* hws0 "]"
```

`boolean`, `integer`, `text_literal`, identifier, direct-field place, and
integer-literal spelling reuse the existing Hum token definitions. AF adds no
escape syntax or numeric syntax. `place` is purely syntactic and uses exactly
the already authorized predicate place surface: one identifier or one direct
field composed of two identifiers. It admits no element/index, nested field,
call, or permission-bearing form. Parsing never requires the place to resolve,
be a parameter/result, be section-eligible, or have a particular type.
Resolution, section eligibility, `old(...)` entry-readability, and operand type
are applied only after the complete grammar parses.

The accepted first non-whitespace operand tokens are `true`, `false`, an
existing decimal integer literal (including only the existing signed form), an
identifier/place, `"`, `[`, `(`, or the exact no-gap call tokens `old(`,
`list_len(`, and `list_count(`. No other starter is accepted. Once an intent
signal exists, empty input, a leading comparison/comma, `@`, `{`, `]`, or any
other starter is malformed H0704.

The complete trimmed line must match one `predicate`; chained comparisons and
trailing tokens are invalid. Multi-character comparison operators are atomic:
whitespace inside them is invalid. Optional spacing exists only at `hws0`
positions. No executable production uses `hws1`; the intent scan uses it to
distinguish a spaced known-call near miss from the accepted no-gap call token.
Accepted known-call spellings have no gap between the call name and `(`. Calls do not nest: `old` and `list_len` take one
syntactic place, and `list_count` takes only the two source forms above. Lists
are flat Text lists, have no trailing comma, and cannot contain expressions or
nested lists. Parentheses/brackets/call delimiters must match, and the shared
analyzer accepts at most 16 simultaneously open delimiter frames. Exceeding
that bound is H0704, not a recursion failure or trap.

Predicate recognition order is deterministic, and
`predicate_recognition_status` has exactly four closed values:

1. A meaningful line with no intent signal has
   `predicate_recognition_status = non_executable_prose_v0` and remains H0701.
   No later predicate parser or evaluator sees it.
2. A signaled line that cannot match the complete grammar has
   `predicate_recognition_status = malformed_executable_predicate_v2` and
   receives H0704. This includes an invalid/empty operand starter, punctuation
   near-miss operator, known-call spacing near miss, malformed place,
   missing/extra/mismatched delimiter, illegal nesting, invalid list separator,
   arbitrary call operand, or trailing token/prose.
3. A grammar-valid candidate proceeds through place resolution, section
   eligibility, and operand typing. Any failure has
   `predicate_recognition_status = rejected_executable_predicate_semantics_v2`.
   Existing H0630 retains precedence for a resolved, eligible, well-formed
   candidate that attempts to inspect opaque `Path`. Otherwise an unresolved
   place, resolved but predicate-ineligible place, invalid operator/type
   combination, cross-type equality, or invalid call argument type receives
   H0704 with the semantic reason.
4. Only a grammar-valid and type-valid fact has
   `predicate_recognition_status = recognized_typed_executable_predicate_v2`
   and reaches evaluation. A false `needs:` is H0702 and a false `ensures:` is
   H0703. A true predicate passes. No malformed or ill-typed candidate reaches
   the evaluator.

The one shared recognition fact contains `predicate_recognition_status`,
tokens, delimiter depth/result, intent-signal span, parsed operands/operator,
syntactic places, resolution/eligibility/type results when available, and every
source span. Static checking and runtime preflight consume that fact; neither
may rescan the line or independently decide H0701 versus H0704.

### Typed analysis and diagnostics

Move eligibility, operand parsing, type facts, and evaluation for executable
predicates into one shared Predicate v2 analysis consumed by full type,
downstream gates, graph facts where already represented, and runtime. Do not
maintain separate string-pattern interpretations in static and runtime paths.
The shared fact records section, comparison operator, operand kinds and types,
`list_count` argument types, syntactic places, resolution/eligibility results,
result type, and source spans. `predicate_recognition_status` is only a
lexical/parser/type-recognition fact and emits no decision 0015 proof/trust
classification.

Allocate H0704 to a signaled executable-predicate candidate whose complete
grammar, operator, arity, operand kind, or operand type is invalid. H0704 names the task
and section, expected and actual types/shapes, the offending operand/call site,
and a concrete valid replacement. It appears in full-type human and existing
JSON output and blocks runtime through the same fact. H0701 remains only the
warning for honest prose; it must not absorb malformed or ill-typed canonical
Predicate v2 forms. H0702 and H0703 remain caller-blamed `needs:` and
task-blamed `ensures:` runtime violations.

Required precedence probes include: one ill-typed canonical Predicate v2 line
that would otherwise fall through as prose (H0704, not H0701); Path use inside a
Predicate v2-looking comparison (the existing H0630 Path boundary owns it, not
H0704); and a false well-typed predicate (H0702/H0703, not H0704). Each offending
line has one owning diagnostic and no generic trap; independent errors on other
source lines are not suppressed.

### AF cross-stage surfaces

Resolver and type environment preserve referenced definition/type identity;
type/full type expose accepted or H0704-rejected operand facts; effect,
ownership, and resource consume the prior typed result without re-parsing the
contract; Core preview/lower/verify preserve the checked predicate or its exact
blocker; graph exposes the existing contract relationship honestly; and runtime
evaluates only the shared accepted fact after clean preflight. Existing schema
IDs and commands remain unchanged.

### Permanent evidence

Update the three hand-authored probes named above. Add non-degenerate fixtures
that prove:

- `word_count` returns the count relation, and a wrong implementation that
  still returns a plausible UInt fails H0703;
- `builder_demo` checks exact ordered content, and a same-length wrong-content
  implementation fails H0703;
- `element_views` checks Text equality, and a different Text result fails
  H0703;
- Text-vs-UInt comparison, mixed/non-Text list content, wrong `list_count`
  arity/type, list ordering comparison, arbitrary helper-call operands, and
  malformed canonical-looking forms fail with H0704 rather than H0701 or a
  trap; and
- ordinary non-predicate prose remains H0701 and existing Predicate v1
  `old(...)`/`list_len(...)` evidence remains green.

Place outcomes are pinned after a signaled line enters the envelope:

| Place outcome | Representative candidate | Recognition status and owner |
| --- | --- | --- |
| syntactically malformed | `item..done == true`, `.item == true`, `item. == true` | `malformed_executable_predicate_v2`; exactly H0704 |
| syntactically valid but unresolved | `missing_value == 2` | `rejected_executable_predicate_semantics_v2`; exactly H0704 with unresolved-place reason |
| resolved but predicate-ineligible | `helper_task == 2` where `helper_task` resolves to a task/non-contract value | `rejected_executable_predicate_semantics_v2`; exactly H0704 with ineligible-place reason |
| eligible place, wrong operand type | `text_value == 2` for a Text parameter/result | `rejected_executable_predicate_semantics_v2`; exactly H0704 with expected/actual types |
| eligible and correctly typed | `result == 2` for UInt result | `recognized_typed_executable_predicate_v2`; executable, then pass or H0703 |

Full-type human, JSON, and `hum run` preflight assert the same recognition
status, reason, syntactic place, resolution/eligibility/type facts, spans,
exact diagnostic count, and repair for every row. None becomes H0701, a generic
resolver/type diagnostic, or a runtime trap.

The permanent boundary matrix is exact:

| Boundary | Representative lines | Required owner |
| --- | --- | --- |
| punctuation operator near misses | `result = 2`, `result === 2`, `result ! = 2`, `result ! 2`, `result!2`, `result <> 2` | one H0704 each |
| prose operator word | `result equals two` | H0701 only |
| malformed known calls | `list_count (items, "hum") == result`, `list_count(items) == result`, `old (item) == result` | one H0704 each |
| missing/extra/mismatched delimiters | `result == ["parse"`, `result == ["parse")]`, `(result == "parse"`, `result == "parse`, `result == "parse""` | one H0704 each |
| invalid operand starters | `@result == 2`, `, result == 2`, `== result` | one H0704 each |
| canonical-looking trailing prose | `result == 2 matching words` | one H0704 |
| quoted predicate-like Text as prose | `"result == list_count(items, hum)"` | H0701 only; inner spelling is never scanned |
| token/call boundary near misses | `list_counted(items, "hum") == result`, `list_count (items, "hum") == result` | one H0704 each; first is arbitrary call, second is bad canonical spacing |
| delimiter-depth limit | a generated-in-test 16-frame valid operand and a 17-frame counterpart | accepted at 16; H0704 at 17 |
| lone predicate-shaped `!` | `result ! 2`, `result!2`, `(result) ! "x"` | one H0704 each in human/JSON/runtime preflight; no H0701 or trap |
| adjacent recognition triplet | three one-token-neighbor fixtures: prose `result equals two`, malformed `result = 2`, valid `result == 2` | respectively H0701, H0704, and executable fact/H0702-or-H0703 behavior |

Full-type human and existing JSON expose the same H0704 reason,
`predicate_recognition_status`, intent span, offending span, task/section site,
expected shape/type, and repair.
`hum run` preflight renders that same H0704 and exits 2 before any task body or
contract evaluation. H0701 remains a non-blocking runtime warning. Boundary
fixtures assert exact counts and absence of the competing H0701/H0704 code and
generic traps as applicable.

Suggested stable fixture families are
`fixtures/full_type_check/session_af_predicate_v2_*` and
`fixtures/run/session_af_predicate_v2_*`; names must describe the misuse, not
the desired implementation.

### AF bans

No boolean connectives, quantifiers, binders, lambdas, predicate task values,
general helper calls, general list equality, generic element equality, Text
ordering, normalization/collation, membership, `contains`, body-reachable
`list_count`, decision 0015 proof/trust classifier, profile/proof/elision work,
or new schema/command.

### AF acceptance criteria

- All three intended contracts execute and reject their wrong implementations;
  none remains H0701 prose and none is replaced by a hard-coded substitute.
- H0704 owns every pinned invalid canonical form with typed structured facts,
  stable blame, actionable repair, and static/runtime precedence.
- Every named compiler/report/Core/graph/runtime surface agrees, and no existing
  Predicate v1 or Path-boundary behavior regresses.
- The standing checks pass. Stop. Session AG remains unauthorized pending AF's
  independent verdict, accepted commit and CI record, and a separate BDFL go.

## Session AG: model-neutral corpus and experimental harness

Purpose: freeze the exam before any effect candidate can shape it.

### Authorized artifacts

This session may add exactly one maintained corpus document,
`docs/bakeoff/EFFECT_POLYMORPHISM_CORPUS.md`, and one isolated no-dependency
Rust crate under `experiments/effect-bakeoff/`. The crate is an offline test
harness only. It is not linked by the Hum binary, reachable from Hum source, or
exposed as a CLI/report/schema. Add one proportional invocation to the existing
preflight so corpus and experiment tests cannot silently rot.

The shared corpus representation describes behavior and relationships only:
values, callable inputs/results, call sites, stored or returned callables,
latent operations, captured values/authority/resources, ownership transfer,
registration lifetime, and expected blame sites. It contains no row variable,
Boolean formula, capture-set syntax, candidate name, handler mechanism, hidden
allocation, or candidate-specific rewrite.

### Frozen candidate-neutral executable result contract

Before any advocate begins, AG defines one closed Rust result type consumed by
the shared harness and AK scorecard. Every candidate emits exactly one result
per corpus case/variant with these required fields:

```text
candidate_id
case_id
variant_id
candidate_native_result
candidate_native_evidence
neutral_normalized_summary
status
required_source_annotations
inferred_requirement_facts
inferred_capture_facts
allocation_facts
resource_facts
added_machinery
primary_reason
primary_blame_site
related_blame_sites
repair_direction
implementation_cost
analysis_cost
diagnostic_cost
missing_evidence
```

`candidate_native_result` preserves the candidate's own normalized terminology.
`candidate_native_evidence` is a sorted key/value evidence bag for audit only;
it cannot create a score field. `neutral_normalized_summary` uses only corpus
terms: callable input/result, propagated/handled requirement, stored/returned
callable, capture/escape, exact authority requirement, ownership transfer, and
resource lifetime. `status` is exactly `accepted`, `rejected`, `unsupported`,
or `incomplete`; unsupported is not accepted, and incomplete is a hard evidence
failure.

Annotation facts record source site, purpose, and whether the annotation is
required or inferred. Requirement and capture facts record stable corpus
identity, origin/route, retained/propagated/handled disposition, and affected
callable. Allocation/resource facts record kind, trigger, lifetime, cleanup or
transfer owner, and whether the fact is explicit or merely prototype-assumed.
`added_machinery` records layer (`language`, `checker`, `runtime`, `ownership`,
or `tooling`), implementation state, and whether the scorecard may credit it;
anything not implemented and exercised is uncredited.

The diagnostic fields are mandatory for every rejected case. `primary_reason`
uses the frozen corpus reason ID, the primary and related sites use frozen
corpus site IDs, and `repair_direction` must preserve the model-neutral repair
goal. Candidate-native jargon may appear only in native evidence and rendered
explanation, not replace the neutral reason or sites.

AG freezes these execution and normalization rules:

- fixture inputs, positive/misuse variants, expected semantic observations,
  required reason IDs, and required blame-site sets are immutable corpus data;
- lists and maps sort by stable corpus ID; source routes retain semantic order;
  candidate variables alpha-normalize by first structural occurrence; path
  separators normalize only for identity, never display evidence;
- each candidate runs from a fresh harness state twice, and the complete neutral
  result must be byte-identical after canonical serialization;
- a missing field, empty required site set, unreported allocation/resource,
  omitted added machinery, failed normalization, or absent cost measurement
  appends `missing_evidence` and forces `status = incomplete`; missing evidence
  is never zero, pure, free, inferred, or favorable;
- extra candidate-specific top-level fields are rejected. Native terminology
  fits only in the two frozen native fields and has no independent scoring
  weight.

AG also freezes the cost method. `implementation_cost` is deterministic
nonblank/noncomment candidate-module lines plus dependency count.
`analysis_cost` is candidate-reported and harness-checked counts of visited
corpus nodes, generated facts/constraints, normalization steps, and maximum
live analysis items; process wall time may be recorded as non-scoring host
context only. `diagnostic_cost` records primary-diagnostic count, required-site
coverage, rendered UTF-8 byte count, candidate-native term count, and whether a
model-neutral repair is present. The hard gates are complete corpus coverage,
one primary diagnostic, full required-site coverage, no authority/ownership
laundering, and no missing evidence. AK compares the frozen raw measures and
may not invent a favorable default, hidden weight, or candidate-only metric.

The harness contains malformed-result tests for every required field and proves
that omission is incomplete. Any later change to this result contract,
normalization, corpus observations, or cost method is a common corpus correction
that stops the active advocate sequence, receives independent review, and
forces every completed candidate to rerun.

### Pinned distinguishing corpus

Pin positive and misuse variants, stable case IDs, and frequency/rationale for:

1. pure `map`;
2. effectful `map` with inferred callback effects;
3. `filter`/`retain`, including Program 3's two-list odd-filter positive,
   retain-style positive deletion, same-list mutation rejection, and stale
   retained-item-view rejection after deletion;
4. `fold` with an effectful step;
5. `retry` around a fallible callable without erasing its remaining effects;
6. `with_timeout`-shaped handled requirements as a type-system case only, with
   no scheduler or clock implementation credit;
7. `parallel_map` as a latent-effect composition case only, with no concurrency
   implementation credit;
8. a stored callback registry that captures caller state and has an explicit
   registration/lifetime misuse;
9. an event-handler factory returning a closure that captures authority;
10. a memoizing wrapper returning a callable that captures cache state and
    allocation/resource intent;
11. logging middleware that adds output while preserving the wrapped callable's
    effects; and
12. closure capture of an owned or Transaction-shaped linear resource,
    including move, escape, double-use, and outlives misuse.

Each misuse specifies one fundamental reason and structured callable,
definition, capture, resource, call, registration/return, and use sites as
applicable. The corpus separately measures effect inference, closure/resource
capture, authority preservation, blame, caller annotation burden, public
signature burden, and near-principal inference. `Near-principal` means the
prototype produces one stable most-general summary within its admitted model,
up to documented normalization/renaming, without requiring a caller annotation
for the ordinary positive cases; it is not a proof of global principal types.

### Corpus locks

- Candidate-specific restructuring receives no credit. An around-call helper
  cannot substitute for a returned wrapper; an owner table cannot substitute
  for captured caller state; copying cannot substitute for an owned view or
  resource; and a monomorphic callback cannot substitute for inferred latent
  effects.
- Existing Hum authority IDs remain exact. Corpus authority facts distinguish
  source requirement, operator consent, and operation exercise.
- Allocation and resource capture are explicit. A candidate cannot hide a
  closure environment, cache, box, reference count, or registry allocation.
- The harness rejects missing/duplicate case IDs, missing positive or misuse
  halves, missing blame relationships, candidate vocabulary in the neutral
  corpus, unpriced restructuring, incomplete candidate results, extra result
  fields, and candidate output that cannot normalize into the frozen contract.
- Session AG records the pure Effekt-style eligibility result transparently
  against stored-callback and returned-closure requirements. Research prose
  alone cannot eliminate it.

### AG bans

No production Hum module, syntax, compiler stage, runtime path, H-code, public
schema/report, candidate implementation, candidate score, model selection,
production higher-order fixture, or candidate-specific corpus vocabulary. The
session may validate the exam but may not answer it.

### AG cross-stage and diagnostic expectations

No production compiler stage changes. Existing Hum stages must remain exactly
as before and must not claim that corpus-only higher-order shapes are supported.
Experimental diagnostics use corpus IDs and deterministic reason/blame facts,
not H-codes. The new preflight invocation checks only the isolated crate and
corpus integrity; it is not a new Hum pipeline stage.

### AG acceptance criteria

- The corpus contains all twelve exact shapes and misuse relationships, with no
  model mechanism or candidate-specific substitute.
- The harness fails on a deliberately malformed in-test corpus and passes the
  checked-in corpus; no production Hum crate or surface depends on it.
- The frozen candidate-neutral result contract, normalization, expected
  observations, missing-evidence rule, and cost method reject malformed or
  incomplete mock-candidate output before AH begins.
- An independent reviewer verifies the corpus before any candidate sees an
  editable exam. After acceptance, corpus changes require a separately reviewed
  common correction and force every completed candidate to rerun.
- The standing checks pass. Stop. Session AH remains unauthorized pending the
  full gate and a separate BDFL go.

## Session AH: Koka-style row-polymorphism advocate

Purpose: give open effect rows their strongest honest prototype over the frozen
corpus.

### Scope

Add only a row-candidate module and best-advocate document inside the AG
experimental boundary. Implement open effect rows with stable labels and tail
variables, latent callable effects, application propagation, exact handled
label removal needed by the retry/timeout cases, alias-preserving rendering,
and stable normalization/renaming for near-principal comparisons. Duplicate
label behavior must be stated and tested rather than assumed.

The candidate must represent returned/stored callables and expose closure
environment allocation and captured owned/linear resources. If row effects
alone cannot enforce resource escape or exact authority retention, record the
additional capture/ownership side condition and price it as non-row machinery;
do not credit decision 0014 or 0017 for an unimplemented bridge.

### Evidence and diagnostics

Run every frozen positive/misuse case and emit exactly AG's frozen candidate-
neutral result contract. Preserve normalized latent rows only in the frozen
candidate-native evidence fields; comparison uses the neutral summary,
annotation, inference, allocation/resource, added-machinery, diagnostic, and
cost fields. Add order/renaming stability tests, row growth stress,
alias-preserving error snapshots, and a captured-linear-resource escape test.

### Bans

No production syntax/checker/runtime change, no corpus rewrite, no Boolean
formula or capture-set emulation hidden inside the row candidate, no hidden
effect labels or closure allocation, and no claim that the prototype proves
principal inference, ownership soundness, or authority safety.

### AH cross-stage surfaces

Only the isolated experiment crate and its preflight invocation may consume
row-candidate results. Production resolver, type, effect, ownership, resource,
Core, graph, and runtime output remain byte-for-byte unaffected by the
candidate mechanism except for unrelated deterministic build metadata.

### AH acceptance criteria

- The unchanged corpus produces a complete auditable row-candidate result; all
  failures and added side machinery are explicit and uncredited.
- Ordinary positive cases meet or clearly fail the near-principal/no-caller-
  annotation gate, and diagnostics are deterministic and single-cause.
- The standing checks pass. Stop. Session AI remains unauthorized pending the
  full gate and a separate BDFL go.

## Session AI: Flix-style Boolean-formula advocate

Purpose: give Boolean effect formulas their strongest honest prototype over the
same frozen corpus.

### Scope

Add only a formula-candidate module and best-advocate document inside the AG
experimental boundary. Implement latent callable effect formulas with variables
and the exact union, intersection, difference, complement, and equivalence/
normalization behavior needed to test the corpus. Inference results must be
stable modulo documented Boolean equivalence. Raw solver/formula output is not
an acceptable user diagnostic.

The candidate must represent stored/returned callables and price normalization,
unification, associated-effect pressure, compile-time cost, closure allocation,
and any separate ownership/capture rule. Authority labels remain exact and may
not be simplified into an ambient `IO` or erased top effect.

### Evidence and diagnostics

Run every unchanged positive/misuse case and emit exactly AG's frozen candidate-
neutral result contract. Preserve normalized Boolean formulas only in the
frozen candidate-native evidence fields; no formula-specific comparison field
or scoring rule may be added. Add formula-equivalence and order-stability tests,
an effect-exclusion case, a formula-growth/throughput measurement on a bounded
synthetic graph, a single-cause diagnostic snapshot that explains formulas in
domain language, and captured-linear-resource/authority escape tests.

### Bans

No production change, corpus rewrite, candidate-specific restructuring,
unpriced solver or associated-effect machinery, raw multi-error cascades,
hidden allocation, erased effects, or claim that principal inference or
performance is proved beyond the prototype measurements.

### AI cross-stage surfaces

Only the isolated experiment crate and its preflight invocation may consume
formula-candidate results. Production resolver, type, effect, ownership,
resource, Core, graph, and runtime gain no formula facts, syntax, or status.

### AI acceptance criteria

- The unchanged corpus produces a complete auditable formula-candidate result
  with equivalence normalization and measured bounded prototype cost.
- Every misuse has one comprehensible primary reason and the same relationship
  sites required by the corpus; formula machinery never substitutes noise for
  blame.
- The standing checks pass. Stop. Session AJ remains unauthorized pending the
  full gate and a separate BDFL go.

## Session AJ: capture-oriented checking advocate

Purpose: test whether capability/capture retention can provide Hum's effect
story without weakening decisions 0014 or 0017.

### Scope

Add only a capture-candidate module and best-advocate document inside the AG
experimental boundary. Implement explicit and inferred capture sets for
callable values, result capture for returned/stored closures, subcapture needed
by the corpus, and stable normalization/renaming. Distinguish:

- capturing an ordinary immutable value;
- moving or borrowing owned state;
- retaining a Transaction-shaped linear resource;
- retaining a source authority requirement; and
- actual operator consent and operation exercise, which are not captures and
  cannot be inferred into existence.

If failure, state, or handled requirements need effect facts in addition to
capture sets, expose and price the hybrid machinery. Do not call a capture-only
answer complete by silently dropping non-capability effects.

### Evidence and diagnostics

Run every unchanged positive/misuse case and emit exactly AG's frozen candidate-
neutral result contract. Preserve normalized capture sets only in the frozen
candidate-native evidence fields; no capture-specific comparison field or
scoring rule may be added. Add capture-substitution and order-stability tests,
strict-vs-lazy result-capture comparisons, captured native authority identity,
owned/linear move and escape misuses, and a test proving that a closure's
capture set neither grants operator consent nor widens its app authority
maximum.

### Bans

No production change, corpus rewrite, capability-as-consent, ambient capture
root, hidden box/allocation, decision 0014 narrowing, decision 0017 widening,
or claim that capture checking alone models effects the prototype represents
with additional machinery.

### AJ cross-stage surfaces

Only the isolated experiment crate and its preflight invocation may consume
capture-candidate results. Production resolver, type, effect, ownership,
resource, Core, graph, authority audit, and runtime gain no capture facts,
syntax, inference, or status.

### AJ acceptance criteria

- The unchanged corpus produces a complete auditable capture-candidate result;
  capability retention, resource ownership, and external consent remain
  distinct.
- Ordinary positive cases meet or clearly fail the near-principal/signature-
  burden gates, and every misuse produces one stable relationship diagnostic.
- The standing checks pass. Stop. Session AK remains unauthorized pending the
  full gate and a separate BDFL go.

## Session AK: cross-candidate scoring and proposed decision

Purpose: select a model only if repository evidence earns a selection.

### Scope

Freeze candidate code before scoring. Rerun all three prototypes against the
same corpus and compare:

1. exact positive and misuse coverage without restructuring;
2. near-principal inference and stability;
3. caller and public-signature annotation burden;
4. one-diagnostic blame quality and repair direction;
5. returned/stored closure expressiveness;
6. owned and linear resource capture safety;
7. decision 0017 authority integrity;
8. explicit allocation/resource cost;
9. checker complexity and bounded prototype throughput;
10. Core/graph/tooling fit and migration path; and
11. beginner and library-author pedagogy.

Separate proven prototype facts from proposed production repairs. Give no
credit for unimplemented machinery, hidden allocation, ambient capabilities,
erased effects, future ownership work, or candidate-specific restructuring.
Record eliminated candidates and salvageable ideas explicitly.

Update the existing bake-off scorecard only with verified candidate results.
Author `docs/decisions/0018-adopt-effect-polymorphism-model.md` with status
exactly `proposed` only if at least one candidate clears every non-negotiable
corpus, diagnostic, near-principal, ownership-capture, and authority gate. If no
candidate clears the gate, do not choose the least-bad score: leave 0018 absent
or proposed-blocked, identify the smallest shared evidence gap, and stop for a
new bounded BDFL ruling.

The proposed decision must state the chosen core rule, inference boundary,
callable/capture semantics, diagnostics, Core and graph implications,
allocation/resource visibility, rejected alternatives, salvaged mechanisms,
migration, performance evidence, pedagogy, and every honesty lock. It must not
implement production behavior.

### AK bans and cross-stage surfaces

No candidate code changes during scoring, corpus edits, production compiler or
runtime changes, Hum syntax, H-codes, schema/report additions, selected-model
hooks, effect implementation, internal references, higher-order stdlib, or
flagship-wedge surface. Production resolver, type, effect, ownership, resource,
Core, graph, and runtime remain unchanged; only the isolated experiment,
scorecard, and proposed decision may report the comparison.

### Decision gate

The implementer cannot accept decision 0018. Only an independent
architect-reviewer that verifies the frozen corpus, every candidate result, and
the scorecard may exercise delegated ruling authority by changing the status to
exactly `accepted under delegated authority (BDFL veto open)` and delivering the
required BDFL brief. The BDFL may veto or directly rule. A decision acceptance
does not authorize production effect-polymorphism implementation.

### AK acceptance criteria

- Every score traces to reproducible corpus output and prices all extra
  machinery symmetrically.
- The recommendation is decisive only if a candidate clears the gates; the ADR
  kills alternatives and records salvage without overstating prototype proof.
- Accepted decisions 0014, 0015, and 0017 remain unchanged and fully locked.
- The standing checks pass. Stop. Work Order 7 ends at the decision gate. No
  production effect implementation or later session is authorized.

## Deliberate deferrals and concrete cost

### Internal references

Internal references remain the next ownership repair whenever ownership work
resumes after this mandated sequence. They are not implemented or prototyped by
Work Order 7. Deferral keeps ownership corpus Program 5 blocked: a parser cannot
naturally own a buffer while storing a checked view into that buffer. Parser
state must continue using less-direct coordinates, re-presentation, copies, or
an explicitly different owner arrangement, none of which earns Program 5
credit.

### Production effect polymorphism

This order selects a model but does not ship it. Until a later accepted work
order implements the selected semantics, Program 3's retain positive, Program
4, closures, callbacks, tasks-as-values, higher-order stdlib, and higher-order
contract blame remain unsupported. Experimental acceptance is no production
language claim.

### Air-gapped update validator

The air-gapped update validator remains the next adoption destination, not a
Session AF-AK deliverable. Deferral costs Hum its flagship real-tool proof and
evidence dossier. Building it now would pre-decide or fake Bytes, directory
input, hashing, manifests, canonical JSON IO, file writing, evidence-directory
authority, signature/provenance policy, and sandbox claims. None may be
prebuilt, reserved through speculative hooks, or credited in this order.

### Other governance queue items

Structured concurrency/shared state, generics/coherence, allocation/container
identity, numeric modes, and Text/Bytes/Path remain in their existing governance
order. This proposal neither authorizes nor reorders them.

## Evidence links for independent pre-issuance review

The fresh reviewer must verify at least:

- predecessor closure and CI state from `git log`, this file's predecessor
  commit, and the accepted Session AE ledger;
- the exact three source programs and current H0701/runtime behavior;
- decisions 0014, 0015, and 0017 in full;
- `docs/bakeoff/CORPUS.md`, `docs/bakeoff/SCORECARD.md`, and the Session AE
  portion of `docs/CORE_LANGUAGE_SHAPE.md`;
- governance bake-off doctrine, delegated ruling, workflow continuity, and
  session-definition-of-done rules;
- all three named research snapshots with their provenance limitations; and
- whether AF remains one review-sized coherent predicate repair and AG-AK each
  remain independently review-sized without becoming documentation-only theater.

## Current authorization gate

Work Order 6 remains closed at `d601054`; Sessions V-AE are accepted and
committed. The independently reviewed Work Order 7 stack was accepted by the
BDFL and committed as `c10a210`; Ubuntu and Windows CI passed in workflow
`29145110803`. Work Order 7 is issued. The BDFL authorized Session AF; its
implementation was accepted and committed as `7991ef6`, and Ubuntu and Windows
CI passed in workflow `29160088909`. Session AG is next but remains unauthorized
pending a separate BDFL go signal. Sessions AH-AK and every later session are
unauthorized. No implementation session, decision ruling, commit, push, or
scope expansion is implicitly authorized.
Publishing remains a BDFL-reserved action under `docs/GOVERNANCE.md`.

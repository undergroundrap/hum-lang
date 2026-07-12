# 0018: Adopt Open Row-Polymorphic Effects With Explicit Capture Guards

Date: 2026-07-11
Status: accepted under delegated authority (BDFL veto open)

## Question

Which effect-polymorphism model should anchor Hum's first-class higher-order
callable design without weakening ownership or explicit authority?

## Context

Work Order 7 froze a model-neutral 12-case, 29-variant exam before any
candidate existed. Sessions AH-AJ then built best-advocate prototypes for open
effect rows, Boolean effect formulas, and capture-oriented checking. Session
AK froze all candidate code and reran each harness-owned factory twice through
the same closed result validator.

All three candidates accept the same 13 positives and reject the same 16
misuses with one diagnostic and complete relationship sites. The choice is
therefore about the core abstraction and the machinery it forces, not a
coverage winner.

## Proposed Decision

Adopt an open row-polymorphic latent-effect core for first-class callable
values, with explicit capture/ownership/authority companion checks.

The proposed core rule is:

1. A callable type carries an open multiset row of exact latent effect labels
   and, where polymorphic, one structural row tail.
2. Calling a callable propagates its latent row into the caller.
3. A handler removes exactly one matching label and preserves duplicate
   occurrences, unrelated labels, and the open tail.
4. Labels and aliases normalize deterministically; diagnostics preserve a
   written stable alias rather than expanding a raw solver representation.
5. Exact source-authority requirements may appear as latent requirements, but
   a row never creates authority, operator consent, or an exercised operation.
6. Stored and returned callables expose their environment allocation,
   retained values/authority/resources, cleanup owner, and lifetime through
   separate capture and ownership facts. A row is not an ownership proof.

This is a model selection only. No production syntax, compiler behavior,
runtime behavior, Core row, graph fact, handler, closure, or higher-order API
is authorized by this proposal.

## Typed Failure Compatibility

The proposed handler rule is type-level effect evidence only. For failure and
retry labels, removing one effect-row occurrence does not catch a runtime
failure, recover a value, erase a nominal error, or alter causal propagation.
Recovery and catch semantics remain separately undecided and require their own
pinned corpus and decision.

Decision 0016 remains fully binding. Typed failures retain exact nominal roots
and variants, explicit `try` propagation or structural wrapping, and every
root-origin, propagation, wrapping, and call site. This proposal does not
authorize implicit propagation, erased `any error`, exceptions, unwind, or an
ambient backtrace as the semantic carrier. Any future production effect-row
work must preserve H0901-H0907 ownership and precedence unless a separately
accepted decision explicitly changes them.

## Inference And Annotation Boundary

The bounded prototype earns near-principal inference only inside its admitted
row model. A future implementation should infer rows within task bodies and at
ordinary call sites while preserving alpha/order stability. Public
higher-order signatures, recursive inference boundaries, stored callable
types, and returned callable types must expose a stable row variable or named
alias when inference would otherwise become an invisible API contract.

This boundary is a production requirement, not proof supplied by the toy
front end. No global principal-inference, completeness, or arbitrary-program
scaling claim is accepted.

## Callable And Capture Semantics

Passed, stored, and returned callables remain their actual first-class shapes.
An around-call helper, monomorphic callback, copy, owner table, or program
restructuring is not equivalent evidence.

The effect row answers what invoking a callable may do. Separate retained-
environment facts answer what the callable keeps alive. Companion checks must
distinguish immutable captures, moved owned state, borrowed state, exact
authority identities, and owned or Transaction-shaped linear resources.
Operator consent and operation exercise never enter a capture set and cannot
be inferred from either a row or a capture.

Strict versus lazy result-retention analysis from the capture prototype is
salvaged for stored and returned callables. It remains separate from effect
inference and must be implemented and checked rather than credited to decision
0014.

## Diagnostics

A higher-order rejection has one fundamental model-neutral reason, one primary
site, every related relationship site, and an actionable repair. Effect-row
diagnostics preserve stable aliases and name the missing, propagated, or
handled domain operation. Raw rows, substitutions, capture sets, formulas, or
truth tables are audit evidence, not user blame.

Ownership, lifetime, allocation, and authority errors remain owned by their
specific companion checks. Effect mismatch must not mask a more fundamental
ownership or authority violation, and the companion error must not be recast
as a generic effect mismatch.

## Core And Semantic Graph Implications

A future Core design must represent callable input/result identity, an exact
latent open row, application propagation, exact one-occurrence handling, and
stable aliases. The semantic graph must preserve callable definition/use,
row-variable and label identity, handler relationships, source blame routes,
and separate allocation/resource/capture/authority facts.

Existing decision-0017 source policy, operator decision, and operation
exercise facts remain independent and joinable. Existing decision-0014 place,
transfer, lifetime, and linear-resource facts remain independent. No current
Core or graph schema changes in this decision session.

## Allocation And Resource Visibility

Closure environments, registry storage, cache storage, boxes, and any other
runtime representation are source/report-visible allocation facts with a
cleanup owner and lifetime. Captured resources retain exact ownership
transfer, use count, close/unregister boundary, and escape relationships.

Unimplemented callable environments, timeout execution, parallel execution,
and ownership bridges receive no credit. This proposal does not claim runtime
closure support, scheduler support, concurrency, ownership soundness, or
authority safety.

## Evidence And Bounded Cost

All three prototypes produce 13 accepted positives and 16 rejected misuses,
zero unsupported/incomplete results, 16 single-cause diagnostics, and complete
36/36 misuse-site coverage. All 13 positive annotations are inferred in each
bounded prototype.

The row prototype is the smallest direct effect model:

| Measurement | Rows | Formulas | Capture |
| --- | ---: | ---: | ---: |
| Implementation lines | 1,568 | 1,692 | 2,030 |
| Analysis visited nodes | 138 | 166 | 206 |
| Generated constraints | 109 | 137 | 177 |
| Normalization steps | 98 | 116 | 119 |
| Diagnostic UTF-8 bytes | 4,193 | 4,791 | 4,816 |

Rows also deterministically normalize a 256-label stress row. This is bounded
prototype evidence, not a wall-time, production-throughput, or whole-program
scaling claim.

## Pedagogy

The paved-road explanation is: a callable's row says what calling it may do;
its capture facts say what the callable keeps alive. Ordinary callers should
usually see inferred effects. Library authors see stable named rows at public
higher-order boundaries. Authority still requires the separate source maximum,
task closure, and exact operator grant from decision 0017.

No comparative user study exists. Beginner or library-author superiority is a
proposal grounded in the smaller direct model and diagnostics, not empirical
teachability proof.

## Alternatives Rejected

### Boolean Formulas As The Default Core

Rejected. The candidate is complete, but this corpus needs no complement,
intersection, general difference, or associated-effect abstraction that open
rows cannot express. Exact truth-function normalization already requires 4,096
assignments for 12 atoms and is exponential. The extra algebra, constraints,
diagnostic volume, and public-signature machinery have no demonstrated paved-
road payoff.

What dies: Boolean formulas, complement, intersection, general exclusion, and
associated effects as the default beginner/core model.

Salvage: exact equivalence as experimental verifier evidence, domain-language
diagnostics, and a future targeted exclusion feature only after a pinned real
API proves a need.

### Capture Checking As The Complete Effect Core

Rejected. Capture checking is strongest at retention and escape, but the
candidate needs a separate hybrid effect mechanism for failure, retry,
timeout, parallel composition, state, and wrapped requirements. It has the
largest measured implementation and analysis footprint. Calling the combined
system capture-only would hide its actual second effect channel.

What dies: capability/capture retention as Hum's complete effect semantics and
any rule that turns captured authority into operator consent or generic IO.

Salvage: retained-environment facts, subcapture/substitution discipline,
strict/lazy result retention, exact authority identity, and owned/linear
resource escape diagnostics as companion checks.

### Pure Second-Class Computations

Rejected as the core because stored callbacks and returned handler, memoizer,
logging-wrapper, and resource-bearing callable shapes are mandatory. Around-
call restructuring receives no credit.

Salvage: second-class blocks as a possible local implementation/library
pattern where no first-class escape is required.

## Migration

A future work order must first pin production syntax, Core/graph facts,
inference boundaries, diagnostics, closure allocation, and ownership/authority
precedence fixtures. Implementation should then proceed in review-sized
vertical slices: passed pure callable, propagated latent row, exact handling,
stored/returned callable environment, capture/ownership bridge, and only then
higher-order standard-library APIs. Existing first-order tasks retain their
current behavior and require no speculative rewrite.

No migration tool or compatibility promise is implemented here.

## Honesty Locks

Until separately implemented and corpus-verified, Hum must not claim:

- production effect polymorphism, closures, tasks-as-values, handlers, or a
  higher-order standard library;
- global principal inference, inference completeness, production throughput,
  or scalable row diagnostics;
- runtime callable environments, timeout, scheduling, parallel execution, or
  concurrency;
- ownership soundness, borrow soundness, general linear-resource checking, or
  memory-safety completeness;
- authority safety from an effect row or capture set;
- operator consent from source requirements or captured authority;
- recovery or catch semantics from type-level effect handling, or any erosion
  of decision 0016's nominal, explicit, causal typed-failure model;
- effect exclusion, Boolean formulas, associated effects, or capture-only
  effect semantics; or
- any narrowing of decisions 0014, 0015, 0016, or 0017.

## Consequences

Hum has one proposed effect core rather than three indefinite candidates.
Production work becomes evidence-gated around open rows plus explicit
capture/ownership/authority companions. Formula and capture research survive
where each proved strongest without making the paved road a union of all
candidates.

The cost is real: Hum must build a serious row inference/checking layer and a
separate sound capture/ownership bridge before higher-order claims are honest.
Decision acceptance would settle direction only and would not authorize that
implementation.

## BDFL Note

The hard call is whether Hum should buy extra algebra now, or split effect
semantics across capture and hybrid systems, when the frozen programs need
neither. This proposal chooses the smallest direct model that passes every
case, then explicitly salvages capture checking where rows are not the right
tool. The BDFL may veto or revise this proposal before implementation hardens
around it.

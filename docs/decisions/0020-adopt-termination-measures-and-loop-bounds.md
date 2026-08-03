# 0020: Adopt Termination Measures And Quantitative Loop Bounds As Separate Concepts

Date: 2026-08-03
Status: accepted as a design decision. This record authorizes no
implementation, syntax change, work order, or grammar amendment. It fixes the
semantic model and the stage boundary so that a later implementation cannot
adopt the wrong shape.

## Context

Hum makes correctness, resource, safety, and performance claims explicit and
evidence-bearing. Its Core design includes loops, explicit failure, effects,
and profiles, but defines no ranking measures and no quantitative loop bounds.
Three research passes examined whether to adopt such a feature, using the Milo
language as prior art.

Loops need two independent kinds of evidence, and they are not
interchangeable:

1. qualitative progress evidence that a loop or recursive descent cannot
   continue indefinitely;
2. quantitative evidence limiting how many times the body is entered.

An invariant is not progress evidence. This loop preserves `i <= items.len`
forever without terminating:

```text
while i < items.len {
  # i is never changed
}
```

A benchmark, a runtime observation, and a watchdog are likewise not
mathematical proof.

Milo supplied both a model and a caution. It implements `decreases` as a real
keyword producing static termination obligations; it has **no** `repeats`,
`iterations`, or `increases` keyword, and its quantitative counts come from a
separate WCET flow-fact extractor. That extractor infers a maximum from a
literal guard while assuming, without checking, a zero start and unit
increment: a loop starting at `-100` with guard `< 5` is labeled maximum 5 but
enters the body 105 times. Its safety-profile gate accepts any `while`
carrying at least one invariant, which proves no progress at all. Hum must not
reproduce either shortcut, and must not split one question across three
disagreeing analyses.

## Decision

Adopt two concepts for V0. Their meanings do not merge.

**Termination contract.** A well-founded progress measure, written at task
level for direct self-recursion or at loop level for back-edges:

```text
decreases:
  expression
```

This is a correctness and local-totality obligation. It is **not** part of
`cost:`.

**Quantitative loop bound.** A resource claim on body entries, written as
loop-local `cost:` metadata:

```text
cost:
  iterations: at most expression
```

`at most` bounds body entries per activation.

**`exactly` is compiler-derived only in V0.** It is not source syntax. Where
the compiler genuinely knows the count, it emits the fact itself with
`claim_origin: derived`. Exactness requires the absence of every early and
abnormal exit, so a source-written exact claim on an arbitrary `while` would
be an unsupported assertion. Initial derivation is limited to canonical
`for index A until B` loops with no early or abnormal exit. `for index`
otherwise derives a structural maximum. `for each` derives only `at most`
against an authenticated stable extent; exact `for each` waits until iterator
semantics are a stable Core contract.

### Canonical loop annotation order

When loop-level annotations exist, they appear as a prelude in this order,
before executable statements:

```text
while condition {
  keeps:
    invariant expression

  decreases:
    measure expression

  cost:
    iterations: at most bound expression

  # executable statements
}
```

The order reads as: what remains true, what progresses, what quantitative
limit follows, what executes. Pinning it now avoids a parser and formatter
migration when `keeps:` becomes executable.

### V0 scope

- one natural-number measure;
- ordinary local `while` and unconditional `loop` back-edges;
- direct self-recursion only;
- pure measure expressions over locals, parameters, integer constants, `+`,
  `-`, and approved stable-size operations such as `list_len`;
- `continue` is a back-edge; `break`, return, and typed failure are exits;
- call-totality and blocking assumptions stated explicitly;
- outcomes preserved honestly: `not_attempted`, `proved`, `refuted`,
  `conditional`, `unknown`, `unsupported`, `timeout`, `invalid`.

### Semantic constraints that must not be softened

- Natural-number semantics are **not** unchecked machine `UInt` arithmetic.
  `limit - i` is a valid natural only when `i <= limit`; measure evaluation
  must be proved safe against underflow.
- A ranking measure proves back-edge well-foundedness, **not** composed task
  completion. This loop has a valid decreasing measure and may never complete:

  ```text
  while remaining > 0 {
    decreases:
      remaining

    wait_forever()
    set remaining = remaining - 1
  }
  ```

  The measure decreases on every reached back-edge, but a back-edge may never
  be reached. Call-totality and blocking assumptions are therefore explicit.
- Completion vocabulary is layered and must stay distinct:
  `backedge_well_foundedness`, `finite_declared_outcome`, `finite_fail_stop`,
  `deadline_completion`. A typed failure is included in
  `finite_declared_outcome` because it is a declared language outcome, not
  because it is success. Avoid `normal_completion`, which implies successful
  return.
- **`ensures:` applies to successful return only.** Typed failure is governed
  by the declared failure type, `fails when:`, and causal propagation.
  Termination may classify success and typed failure alike as finite declared
  outcomes; that does not give them a shared postcondition.
- Iteration means one entry into the loop body per activation of that loop.
- Bound expressions refer to loop-entry snapshots. A collection whose extent
  can change during iteration yields `unknown`, never a reinterpreted moving
  expression.
- Watchdogs, cancellation, panic, traps, and abort do not satisfy any
  completion class above and are never proof of termination.
- Claim origin (`declared`, `derived`) and verification result remain separate
  dimensions. A compiler-derived candidate is not a verifier-proved fact; a
  measured benchmark never upgrades a bound to `proved`.
- One authenticated loop and termination analysis produces the facts consumed
  by graph output, termination checking, resource checking, profile checking,
  math-obligation export, and any future WCET adapter. Hum does not repeat
  Milo's split into three analyses that answer related questions differently.

### Stage ordering (normative)

Termination checking requires typed measure expressions, resolved places and
call targets, purity and effect facts, and stable-extent and alias facts.
Those exist only after full type, effect, and ownership checking.

The current dependency direction was verified in the source: `core_verify`
imports `ast`, `callable`, `core_contract`, `core_expr`, `core_lower`, and
`core_preview`, and does **not** import full-type, effect, or ownership;
`full_type_check` imports `core_verify`. Core verification therefore precedes
and is consumed by full type checking.

Placing termination checking inside `core_verify` would require it to depend
on later stages and would create a circular stage dependency, which active
work explicitly forbids. The required arrangement is:

```text
parse
-> resolve
-> declaration type authority
-> core preview
-> core lower
-> core verify          (structural transport only)
-> full type check
-> effect check
-> ownership check
-> termination check    (new stage)
-> resource check
-> profile check
-> IR readiness
```

Early Core stages transport `decreases:` and bound expressions honestly
without authenticating them. A new termination stage authenticates them after
the facts it needs exist. Resource and profile checking consume the
authenticated results.

### Stage interface (what termination checking publishes)

Downstream stages consume termination facts; they do not re-derive them. The
precise schema belongs to a later `hum.termination_check.v0` contract, but the
fact categories are fixed here so that resource and profile checking cannot
grow a competing control-flow analysis:

- `backedge_well_foundedness` -- proved, refuted, conditional, unknown, or
  unsupported;
- `finite_declared_outcome` -- proved, conditional, unknown, or unsupported;
- `early_exit_inventory` -- the `break`, return, typed-failure, and
  trap-possible exits present;
- `call_completion_dependencies` -- per call: proved, declared, assumed,
  unknown, may-diverge, or blocking.

Resource checking derives quantitative claims from those facts. A canonical
indexed cardinality of 10 with an empty early-exit inventory and a proved
finite declared outcome supports `exactly 10`; the same cardinality with a
`break` in the inventory supports only `at most 10`.

A structural maximum survives unknown or blocking call completion. If a body
call may block forever, the loop still cannot enter its body more than the
structural maximum, so `at most` remains valid. Completion and exactness are
strictly stronger claims and both fail in that case. Bounding entries is not
the same as proving the loop finishes.

### Implementation prerequisite

Loop-attached sections cannot be added with the existing section parser. Hum's
sections are item-level at fixed indentation, and the retained public body
representation (`ParsedBodyStatementKind`) is still coarse -- `Return`,
`Binding`, `Other` -- while richer loop facts live only in canonical parser
projections. Core body recognition is `partial_v0` and carries no ranking or
iteration semantics.

Loop-level annotations therefore require a **parser-owned block annotation
model** whose annotations are children of the exact loop node, carrying loop
identity, annotation identity, expression identity, source span, block
relationship, and stable downstream transport. Text scanning inside `does:` is
not acceptable. That model does not exist. Any implementation must build it
first and must not bypass canonical statement and expression identities.

## Consequences

- Termination becomes a contract-class fact with its own obligation kind,
  distinct from the resource-report inventory.
- Quantitative bounds extend resource reporting with an iteration relation
  that the current `classify_claim` does not recognize.
- Math-obligation export gains two kinds -- `termination` and
  `loop_iteration_bound` -- alongside the existing conservative
  allocation-freedom candidates.
- A new pipeline stage is added after ownership checking. No existing stage
  gains a backward dependency.
- Normal profile permits unknown termination. Strict profiles may require
  proved termination, a proved quantitative bound, or a separately classified
  watchdog policy, with the watchdog never presented as normal-termination
  proof.

## Alternatives rejected

- **`repeats:`** -- not a Milo construct, and less precise than `iterations`.
- **`decreases` inside `cost:`** -- conflates a correctness obligation with a
  resource estimate. Hum's formal core and performance-contract documents
  already separate these roles.
- **`increases:`** -- an increasing variant needs a known upper limit, and
  reaching a numeric bound can raise a runtime error rather than complete
  normally. `decreases: limit - i` expresses the same thing with one canonical
  proof direction.
- **`bounds:`, `loops:`, a dedicated `terminates:` section** -- vaguer than
  naming the metric and the relation.
- **`at least`** -- no established need in V0.
- **Source-written `iterations: exactly`** -- see above; derived only.
- **Runtime enforcement of `decreases:`** -- a debug observation of decreasing
  measures across one run is evidence, not a universal proof.
- **Deriving numeric bounds automatically from arbitrary ranking functions** --
  a measure may decrease by varying amounts, an exit may bypass the back-edge,
  and the initial value need not equal the trip count.

## Deferred

**Public `may diverge:` syntax.** Hum will eventually need to distinguish
"termination not proved" from "nontermination intentionally permitted." Prior
art exists: Why3's propagated `diverges`, Dafny's `decreases *`, F*'s
divergent `Dv` effect. But a declaration that propagates through calls is
effect-like, and Hum already has a callable effect-row model (decision 0018).
Adopting a public divergence section now risks building a second
call-propagation subsystem beside the effect checker. An internal
`may_diverge` completion fact may be reserved; the public surface waits for a
separate decision that settles its relationship to effect rows, transitive
propagation, conditional divergence, and effect polymorphism.

Also deferred: mutual and indirect recursion; recursion through task values,
callbacks, or dynamic dispatch; lexicographic tuples; structural descent over
recursive data; user-defined well-founded relations; general measure
inference; concurrency-sensitive termination; wall-clock deadline proof; exact
`for each`; external proof-receipt acceptance; WCET output format; strict
profile enforcement.

## Salvage

The research produced a regression matrix that should become permanent
fixtures before any public termination claim, including: proved countdown;
refuted growth (`n + 1`); refuted unchanged measure; a `continue` path that
skips the update; early `break` proving termination but rejecting an exact
count; mutable collection length in a measure; unknown callee totality;
possible measure underflow; mutual recursion reported unsupported; a `-100`
start with guard `< 5` never inferring maximum 5; a literal range containing a
`break` reported `at most` rather than `exactly`; nested loops bounded per
loop; and verifier timeout never reported as proved.

The research also proposed a phased implementation order that fits this
project's review discipline: compiler-derived bounds for existing `for index`
loops with no new syntax; then loop-local `decreases:` with the parser-owned
annotation model; then direct self-recursion. Receipts, WCET adapters, and
profile enforcement are later work, not first deliverables.

## Sources

Three research passes, 2026-08-03, verified against repository head
`6d859113`. Milo examined at `cf390123`. Research snapshots archived under
`docs/research/`. Comparative prior art: Dafny, F*, Why3, SPARK, Verus. Stage
dependency direction verified directly in `src/core_verify.rs` and
`src/full_type_check.rs`.

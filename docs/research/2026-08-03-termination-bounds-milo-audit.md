<!--
Research artifact imported on 2026-08-03.
Normalization: explicit UTF-8 decode, Deep Research UI citation markers stripped, typographic punctuation converted to ASCII, saved as UTF-8 without BOM.
Source names are preserved, but citation-only evidence cells may be blank; future runs should request direct source URLs in the Markdown body.
-->
# Hum x Milo Research: Termination Measures and Loop Bounds

## Research verdict

**Hum should adopt the concept, but not exactly as Grok framed it.**

The strongest design is:

1. **Add `decreases:` as a correctness contract**, separate from `cost:`.
2. **Represent concrete loop limits as loop-local resource claims**, preferably:

   ```hum
   cost:
     iterations: at most expression
   ```

   or:

   ```hum
   cost:
     iterations: exactly expression
   ```

3. **Do not add `repeats:`.** It is not current Milo syntax, and "repeats" is less precise than "iterations."
4. **Do not add `increases:` in the first version.** An increasing counter can normally be expressed as a decreasing remaining-distance measure.
5. **Keep annotations optional in normal Hum**, infer bounds for structurally finite loops, and require proof or watchdog evidence only in strict profiles.
6. **Introduce the semantic and evidence model before stabilizing the surface keyword.**

Hum is indeed early enough. Its current reference still labels the language `0.0.1 pre-alpha`, with Milestone 1 execution limited to a narrow interpreter subset. But Hum's own architecture says new syntax is not stable merely because it parses: it must lower into Core Hum and acquire graph, diagnostics, profile, evidence, verification, performance, and teaching semantics.

Sources: Hum README (`README.md`), Hum Language Reference (`docs/LANGUAGE_REFERENCE.md`), Hum Architecture (`docs/ARCHITECTURE.md`).

## Audit of Grok's claims

| Grok claim | Assessment |
|---|---|
| It is not too late for Hum | **Correct.** Hum is still pre-alpha and its loop/core semantics are not frozen. |
| Milo has `decreases` | **Correct.** It is implemented for functions and loops and produces static termination obligations. |
| Milo has or suggests `repeats` | **Not supported by current Milo.** The authoritative grammar has `invariant` and `decreases` as loop contracts, but no `repeats` construct. |
| Put `decreases` inside `cost:` | **I disagree.** Termination is a correctness/totality property, not merely a resource estimate. |
| Put it near Hum's existing `keeps:` | **Conceptually adjacent, but factually premature.** `keeps:` is still a broader reference direction rather than a current stable executable section. |
| Exact bounds are useful but should not be mandatory | **Correct.** This matches both Hum's progressive-disclosure philosophy and the realities of data-dependent loops. |
| Hum should preserve `unknown` honestly | **Strongly correct.** This already matches Hum's external-verifier doctrine. |

Milo's grammar explicitly defines `decreases expr` as an integer measure and permits it on functions and loops; there is no `repeats` production. Milo's verification documentation says the measure must be nonnegative and strictly decrease at recursive calls or loop back-edges, and proofs depending on recursive induction remain conditional when termination has not been discharged.

Sources: [Milo grammar](https://github.com/milo-language/milo/blob/main/docs/grammar.ebnf), [Milo verification roadmap](https://github.com/milo-language/milo/blob/main/docs/verification-roadmap.md), [Milo contracts and safety guide](https://github.com/milo-language/milo/blob/main/docs/site/language/safety.md).

Hum's current task-section table marks `cost:` as current and `proves:` as reference, while `keeps:` is listed among broader constructs that still need checker and graph work.

Source: Hum Language Reference (`docs/LANGUAGE_REFERENCE.md`).

# The essential semantic separation

These concepts are related, but they are not interchangeable:

| Construct | What it establishes | What it does **not** establish |
|---|---|---|
| `keeps:` | A state invariant remains true | That the loop ever finishes |
| `decreases:` | A well-founded ranking measure progresses toward termination | An exact runtime or WCET |
| `iterations: at most n` | A quantitative upper bound on body entries | The loop's postcondition |
| `iterations: exactly n` | A precise trip count under all relevant paths | Wall-clock duration |
| `time: O(n)` | Asymptotic growth | A concrete limit for a particular invocation |
| watchdog | A bounded fail-safe response | Normal termination or successful completion |

An invariant such as:

```hum
keeps:
  i <= items.len
```

does not prove termination. This loop preserves that invariant forever:

```hum
while i < items.len {
  # i is never changed
}
```

A ranking measure adds the missing progress condition:

```hum
decreases:
  items.len - i
```

Conversely, a ranking measure does not automatically give an **exact** count. Early `break`, typed failure, return, or other exits can terminate the loop before the measure is exhausted.

This distinction matters especially for Hum because its Formal Core already anticipates three different mechanisms: loop variants, watchdogs, and measured bounds. Its safety-critical profile already says unbounded loops need variants or watchdogs, while realtime policy calls for WCET or measured-bound evidence.

Sources: Hum Formal Core (`docs/FORMAL_CORE.md`), Hum Runtime Profiles (`docs/RUNTIME_PROFILES.md`).

# What Milo actually implements

## `decreases` is real and useful

A Milo recursive function can declare:

```milo
fn countdown(n: i64): i64
requires n >= 0
decreases n
{
    if n == 0 {
        return 0
    }
    return countdown(n - 1)
}
```

Milo's regression fixtures include both this valid countdown and a `runaway` function that recurses on `n + 1`; the latter's termination obligation is refuted. Milo also rejects a boolean expression such as `decreases n > 0`, because the clause must provide an integer measure rather than assert a proposition.

Sources: [Milo decreases termination fixture](https://github.com/milo-language/milo/blob/main/tests/prove/decreasesTermination.milo), [Milo non-integer decreases error fixture](https://github.com/milo-language/milo/blob/main/tests/errors/decreasesNotInteger.milo).

The important part is not the keyword. The compiler generates obligations equivalent to:

- the measure is in a well-founded domain;
- it is nonnegative when another recursive step or loop iteration is possible;
- the new measure is strictly less than the old measure at every relevant recursive call or back-edge.

This is directly relevant to Hum's eventual `ensures:` proof semantics. A recursive implementation cannot soundly use its own postcondition as an induction hypothesis unless the induction is well-founded.

## Milo does not currently have `repeats`

The closest Milo feature is a **separate WCET flow-fact extractor**. It classifies:

- literal `for i in A..B` ranges as exact counts;
- simple `while i < literal` conditions as maximum counts;
- other loops as unresolved.

That subsystem emits source-level flow facts for WCET tools; it is not the same thing as the `decreases` verifier.

Source: [Milo WCET flow-fact extractor](https://github.com/milo-language/milo/blob/main/src/wcet.ts).

My best inference is that Grok merged two Milo ideas:

1. the language-level `decreases` contract; and
2. Milo's WCET flow-fact output.

That is understandable, but copying them as one Hum construct would erase an important boundary between proof and resource evidence.

## A caution Hum should learn from Milo

Milo's current WCET extractor contains assumptions that are not checked by the extractor itself. For `while i < N`, it looks at the guard and literal bound while assuming a zero start and unit increment. It does not establish those facts from initialization and all body updates.

By inspection, both of these would be classified as having `MAX 5` by that extractor:

```milo
var j: i32 = -100
while j < 5 {
    j = j + 1
}
```

This actually enters the body 105 times.

```milo
var j: i32 = 0
while j < 5 {
    j = j
}
```

This never progresses.

Similarly, a literal range is labeled "exact" before the extractor analyzes whether the body contains an early `break`, return, or failure. The tests cover canonical incrementing examples, but not those adversarial cases.

Sources: [Milo WCET implementation](https://github.com/milo-language/milo/blob/main/src/wcet.ts), [Milo WCET tests](https://github.com/milo-language/milo/blob/main/tests/wcet.test.ts).

Milo's safety-profile gate has a related weakness: the current implementation checks that a `while` has a nonempty loop-contract list, rather than requiring and discharging an actual termination measure. A state invariant can therefore satisfy the syntactic bounded-loop gate even though it proves no progress.

Source: [Milo safety-profile implementation](https://github.com/milo-language/milo/blob/main/src/safety.ts).

That does not make Milo's `decreases` verifier unsound. It means the separate WCET and profile shortcuts should not be treated as proof evidence. Hum should require one of these:

- derive the bound from initializer, guard, updates, aliases, and exits;
- validate a declared bound through a proof obligation;
- or report it as declared/unknown rather than proven.

# Recommended Hum surface

## 1. Task-level `decreases:`

For direct recursion:

```hum
task countdown(n: UInt) -> UInt {
  decreases:
    n

  ensures:
    result == 0

  does:
    if n == 0 {
      return 0
    }

    return countdown(n - 1)
}
```

This should be classified as a **contract**, not a cost claim.

A first version should allow exactly one meaningful measure line. Multiple measures and lexicographic order can come later.

## 2. Loop-local `decreases:`

The exact loop-annotation grammar is not pinned yet, so this is an illustrative future surface rather than currently valid Hum:

```hum
does:
  change i: UInt = 0

  while i < list_len(items) {
    keeps:
      i <= list_len(items)

    decreases:
      list_len(items) - i

    set i = i + 1
  }
```

Hum's Formal Core already says critical loops may carry `keeps:`, `changes:`, `watch for:`, and `cost:` metadata, so adding loop-local `decreases:` extends an existing design direction rather than inventing a foreign annotation model.

Source: Hum Formal Core (`docs/FORMAL_CORE.md`).

## 3. Concrete bounds under loop-local `cost:`

```hum
while i < list_len(items) {
  decreases:
    list_len(items) - i

  cost:
    iterations: at most list_len(items)
    check: prove

  set i = i + 1
}
```

For a structurally counted loop:

```hum
for index i from 0 until item_count {
  cost:
    iterations: exactly item_count
}
```

The compiler should normally derive the latter rather than require the user to repeat it.

I recommend these canonical spellings:

```hum
iterations: at most expression
iterations: exactly expression
```

Avoid:

```hum
repeats: <= n
loops: at most n
bound: n
```

`iterations` names the metric. `at most` and `exactly` name the relation. The result is easier for people, agents, diagnostics, and schemas to interpret without guessing.

# Precise V0 semantics

## Measure domain

Start with **`UInt` measures only**.

That avoids immediately needing signed nonnegativity proofs and arbitrary well-founded orders. Hum's checked arithmetic must still prove that subtraction or other measure computation does not underflow.

Later versions can add:

- signed integers with a proven lower bound;
- lexicographic tuples;
- structural size measures;
- mutually recursive call-graph components;
- user-defined well-founded relations.

Dafny, F*, and Why3 all support richer or lexicographic variants, while Why3 also treats structured `for` loops as intrinsically terminating. That supports a small natural-number V0 plus automatic inference for Hum's structured loops.

References: [Dafny reference](https://dafny.org/dafny/DafnyRef/DafnyRef), [F* termination tutorial](https://fstar-lang.org/tutorial/book/part1/part1_termination.html), [WhyML reference](https://why3.org/doc/whyml.html).

## Loop obligation

For a loop measure `M`, Hum should establish:

1. `M` is side-effect-free and well typed.
2. Whenever the loop condition permits another iteration, `M` is in the natural-number domain.
3. On every normal back-edge, including every `continue` path:

   ```text
   M_after < M_before
   ```

4. Every operation needed to reach that back-edge terminates under recorded assumptions.
5. Every dependency used by `M` has stable resolver/place identity.

A `break`, return, or typed `fail` ends that path, so it does not need to decrease the measure.

A panic, arithmetic trap, or watchdog termination must not be silently reported as normal total correctness. Those are separate safety or fail-stop outcomes.

## Recursive-task obligation

For direct self-recursion:

```hum
decreases:
  n
```

each recursive call must prove:

```text
callee_measure < caller_measure
```

For example:

```hum
return countdown(n - 1)
```

is accepted only on a path that also proves `n > 0`, including freedom from unsigned underflow.

Initially, Hum should report mutual or indirect recursion as `unsupported`, not guess. Supporting it correctly requires reasoning over the strongly connected component of the call graph, usually with a shared or lexicographic ranking function.

## Calls and blocking

A local ranking function proves only that the loop or recursion cannot take infinitely many internal progress steps. It does **not** prove wall-clock completion if the body can:

- call an unknown or divergent task;
- block on IO;
- wait for another thread;
- depend on scheduler fairness;
- await an external device indefinitely.

A termination receipt therefore needs explicit assumptions such as:

```text
callee_totality
no_unbounded_blocking
scheduler_fairness
stable_collection_extent
```

Hum's math-engine boundary already requires assumptions, effects, program shape, limits, and verifier results to remain explicit rather than allowing a solver to become hidden compiler authority.

Source: Hum Math Engine Boundary (`docs/MATH_ENGINE_BOUNDARY.md`).

## Intentional nontermination

Normal Hum should continue to allow servers, event loops, schedulers, and device polling loops. Therefore:

- normal profile: missing termination evidence is `unknown`, not an error;
- safety/realtime profiles: require proof, a derived structural bound, or an explicitly classified watchdog/fail-safe policy;
- future Hum may need an explicit `may diverge` or long-running-task classification.

F*'s explicit distinction between total and divergent computations is useful prior art here: divergence should be represented deliberately rather than inferred from the absence of a measure.

Reference: [F* divergence tutorial](https://fstar-lang.org/tutorial/book/part4/part4_div.html).

# Exact and maximum bounds need strict definitions

Hum should define an iteration as:

> **One entry into the loop body, per activation of that loop.**

That avoids disagreements over whether the initial condition check or final failed condition check counts.

## `at most`

```hum
iterations: at most n
```

means that every activation satisfying the task and loop preconditions enters the body no more than `n` times.

Early `break`, return, or typed failure is compatible with an upper bound.

## `exactly`

```hum
iterations: exactly n
```

should require substantially more evidence:

- the loop is reached;
- the body cannot exit early;
- the loop cannot fail or trap before completing;
- range endpoints or collection extent are snapshotted or proven stable;
- each iteration makes exactly the expected progression;
- all called operations complete;
- arithmetic remains defined.

For example, this is not exact-10 body execution:

```hum
for index i from 0 until 10 {
  if found {
    break
  }
}
```

It remains **at most 10**.

## Snapshot semantics

A bound such as:

```hum
iterations: at most list_len(items)
```

must refer to a well-defined state. Hum should normalize it to the collection length at loop entry when the collection cannot structurally change during iteration.

If the loop can append or remove elements, the compiler should require a different bound or return `unknown`; it must not reinterpret a moving expression as an entry-state constant.

## Relationship to `decreases`

A finite, proven upper bound normally implies termination, assuming each iteration itself completes.

A `decreases` proof may sometimes yield an upper bound, but not automatically:

- the measure may decrease by varying amounts;
- an exit may bypass the back-edge;
- its initial value may not equal the trip count;
- a structural measure may not map directly to an integer cost.

Therefore the compiler may derive one fact from the other only when it can emit the derivation and its assumptions. The graph should preserve both the original claim and the derived claim.

# Why not `increases:` yet?

SPARK supports both increasing and decreasing loop variants, but an increasing variant needs a known upper limit. Moreover, reaching the numeric type's bound may produce a runtime error rather than normal loop completion, so absence-of-runtime-error proof is also part of the total-correctness story.

Reference: [SPARK assertion pragmas](https://docs.adacore.com/spark2014-docs/html/ug/en/source/assertion_pragmas.html).

Hum can avoid those subtleties in V0:

```hum
# Instead of:
increases:
  i

# Write:
decreases:
  limit - i
```

This gives one canonical proof direction and fits Hum's preference for one precise spelling per concept.

# Evidence and schema design

Hum already has the correct architecture for this. `hum math-obligations` currently exports only conservative allocation-freedom candidates, while `hum resource-report` inventories declared time, space, allocation, and optimization claims without upgrading them to proof.

Sources: Hum Math Obligations Schema (`docs/MATH_OBLIGATIONS_SCHEMA.md`), Hum Resource Report Schema (`docs/RESOURCE_REPORT_SCHEMA.md`).

## Add two distinct obligation kinds

### Termination

```json
{
  "obligation_kind": "termination",
  "scope": "loop",
  "normalized_formal_claim": {
    "representation": "hum_ranking_function_v0",
    "measure": "list_len(items) - i",
    "domain": "nat",
    "relation": "strictly_decreases"
  },
  "assumptions": [
    "items extent is stable during the loop",
    "called tasks terminate"
  ]
}
```

### Iteration bound

```json
{
  "obligation_kind": "loop_iteration_bound",
  "normalized_formal_claim": {
    "metric": "body_entries_per_activation",
    "relation": "<=",
    "bound": "list_len(items)"
  }
}
```

## Keep provenance and verdict separate

A useful representation would have two dimensions:

```text
claim_origin:
  declared
  derived

verification_result:
  not_attempted
  proved
  refuted
  unknown
  unsupported
  timeout
```

This prevents a compiler-derived candidate from being confused with a verifier-proved fact.

A measured benchmark must not upgrade a mathematical iteration maximum to `proved`. Likewise, a debug runtime check that observed decreasing measures during one run is valuable evidence, but it is not a universal termination proof.

## Report placement

- `decreases:` -> contract/termination graph fact and `termination` math obligation.
- `iterations:` -> resource report plus `loop_iteration_bound` math obligation.
- watchdog -> profile and operational-safety fact.
- benchmark result -> performance evidence.
- verifier receipt -> proof evidence.

This separation follows Hum's existing doctrine that external engines may prove, refute, or return unknown, while Hum retains ownership of semantics, assumptions, source spans, graph identity, and final policy.

Source: Hum Math Engine Boundary (`docs/MATH_ENGINE_BOUNDARY.md`).

# Concrete integration map for `hum-lang`

The present implementation already has most of the required stage boundaries: syntax catalog, canonical AST events, loop statement kinds, Core preview/lowering/verification, full type and effect checks, resource reports, math obligations, profile checks, and graph output.

Relevant sources:

- Hum CLI/module spine (`src/main.rs`)
- Hum syntax catalog (`src/syntax.rs`)
- Hum AST (`src/ast.rs`)
- Hum Core body classifier (`src/core_body.rs`)
- Hum Resource Report implementation (`src/resource_report.rs`)
- Hum Math Obligations implementation (`src/math_obligations.rs`)

A disciplined implementation sequence would be:

### Phase 1 - decision and fixtures

Write the semantic decision before changing accepted syntax.

Pin:

- definition of termination;
- treatment of return, typed failure, panic, and watchdog;
- body-entry iteration metric;
- loop-entry snapshot semantics;
- normal versus strict-profile behavior;
- V0 exclusions such as mutual recursion and lexicographic measures.

Add fixtures for accepted, refuted, unknown, and unsupported cases.

### Phase 2 - syntax and graph facts only

Touch:

- `src/syntax.rs`
- parser and `src/ast.rs`
- `src/core_body.rs`
- graph/schema projections

At this stage:

```text
claim_origin: declared
verification_result: not_attempted
```

No proof claim.

The current syntax catalog centralizes task section order, hover text, and semantic tokens, while the current AST already owns canonical `While`, `ForEach`, indexed-loop, and unconditional-loop identities.

### Phase 3 - structured-loop derivation

Infer bounds for the simplest honest cases:

- `for index` over snapshotted checked ranges;
- `for each` over a finite collection whose extent is stable;
- downgrade `exactly` to `at most` when early exits exist;
- report unknown when calls, mutation, aliases, or unsupported control flow prevent proof.

This should be structural compiler reasoning, not an SMT dependency.

### Phase 4 - local ranking-function checker

Support:

- one `UInt` measure;
- simple arithmetic;
- direct locals and parameters;
- pure length operations;
- all `continue` and normal back-edge paths;
- direct self-recursion.

Refute obvious growth or unchanged measures.

### Phase 5 - obligation export

Refactor `src/math_obligations.rs`, whose current payload is shaped around allocation freedom, into kind-specific normalized payloads. Add `termination` and `loop_iteration_bound`. Extend `src/resource_report.rs` only with quantitative iteration claims, not ranking functions.

### Phase 6 - external receipts

Allow compatible local verifiers to return:

```text
proved
refuted
unknown
unsupported
timeout
```

Record:

- solver and version;
- assumptions;
- timeout and memory budget;
- certificate or trace;
- source and graph identities;
- trust classification.

### Phase 7 - profile enforcement

Only after the previous stages are stable:

- normal profile allows unknown;
- hard-realtime profile requires a concrete maximum plus WCET evidence;
- safety-critical profile requires termination evidence or a separately classified watchdog policy;
- a watchdog never masquerades as a proof of normal termination.

# Required regression cases

These should be permanent before any public termination claim:

```text
1. countdown(n - 1)                       -> proved
2. runaway(n + 1)                         -> refuted
3. recursive call with unchanged n        -> refuted
4. while with limit - i and i += 1        -> proved
5. continue path that skips i += 1        -> refuted
6. early break                            -> termination proved, exact count rejected
7. mutable collection length in measure   -> unknown or refuted
8. call with unknown totality             -> conditional/unknown
9. measure subtraction may underflow      -> blocked
10. mutual recursion                      -> unsupported in V0
11. start at -100 with guard < 5           -> never infer max 5
12. unchanged counter with guard < 5       -> never infer max 5
13. literal range containing break         -> at most, not exactly
14. nested loops                           -> separate per-loop bounds
15. verifier timeout                       -> timeout, never proved
```

# Proposed decision text

> **Adopt explicit ranking functions and quantitative loop bounds as separate Hum concepts.**
>
> `decreases:` is a correctness contract used to establish control-flow termination under explicit call and effect assumptions. It does not belong inside `cost:`.
>
> Concrete trip limits are resource claims written as `iterations: at most E` or `iterations: exactly E` under loop-local `cost:` metadata. Exactness has stronger control-flow requirements than an upper bound.
>
> Structured finite loops should receive inferred termination and bounds when the compiler can preserve the derivation. Normal Hum permits unknown termination. Strict profiles may require proved termination, a proved quantitative bound, or a separately classified watchdog policy.
>
> Hum V0 supports one natural-number measure, direct self-recursion, and simple local loops. Mutual recursion, lexicographic measures, structural relations, and explicit divergence effects remain future work.
>
> Source declarations, compiler derivations, external-verifier receipts, runtime monitoring, benchmark evidence, and watchdog evidence remain distinct graph facts and must never be silently upgraded into one another.

# Bottom line

Grok identified the right design territory, but the best Hum design is not "add Milo's `decreases` and `repeats` to `cost:`." It is **a termination-contract layer plus a separate quantitative loop-bound layer**, connected through Core Hum and evidence schemas but kept semantically distinct.

# Primary source index

## Hum

- Repository README (`README.md`)
- Architecture (`docs/ARCHITECTURE.md`)
- Language Reference (`docs/LANGUAGE_REFERENCE.md`)
- Formal Core (`docs/FORMAL_CORE.md`)
- Milestone 0 Grammar (`docs/MILESTONE_0_GRAMMAR.md`)
- Performance Contracts (`docs/PERFORMANCE_CONTRACTS.md`)
- Runtime Profiles (`docs/RUNTIME_PROFILES.md`)
- Math Engine Boundary (`docs/MATH_ENGINE_BOUNDARY.md`)
- Math Obligations Schema (`docs/MATH_OBLIGATIONS_SCHEMA.md`)
- Resource Report Schema (`docs/RESOURCE_REPORT_SCHEMA.md`)
- Resource Check Schema (`docs/HUM_RESOURCE_CHECK_SCHEMA.md`)
- Core Contract Schema (`docs/HUM_CORE_CONTRACT_SCHEMA.md`)

## Milo

- [Repository README](https://github.com/milo-language/milo/blob/main/README.md)
- [Language Guide](https://github.com/milo-language/milo/blob/main/docs/language-reference.md)
- [Contracts and Safety Guide](https://github.com/milo-language/milo/blob/main/docs/site/language/safety.md)
- [Grammar](https://github.com/milo-language/milo/blob/main/docs/grammar.ebnf)
- [Verification Roadmap](https://github.com/milo-language/milo/blob/main/docs/verification-roadmap.md)
- [Roadmap](https://github.com/milo-language/milo/blob/main/docs/roadmap.md)
- [Termination regression fixture](https://github.com/milo-language/milo/blob/main/tests/prove/decreasesTermination.milo)
- [Invalid decreases fixture](https://github.com/milo-language/milo/blob/main/tests/errors/decreasesNotInteger.milo)
- [WCET flow facts](https://github.com/milo-language/milo/blob/main/src/wcet.ts)
- [WCET tests](https://github.com/milo-language/milo/blob/main/tests/wcet.test.ts)
- [Safety profiles](https://github.com/milo-language/milo/blob/main/src/safety.ts)

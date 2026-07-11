# Hum Effect-Polymorphism Bake-Off Corpus

Date: 2026-07-11
Status: Session AG model-neutral corpus, pending independent review
Machine data: `../../experiments/effect-bakeoff/corpus.txt`

## Purpose

This document freezes the exam used by Sessions AH-AK. It specifies behavior,
relationships, observations, fundamental reasons, and blame sites. It does not
specify a candidate mechanism, score a candidate, or claim production Hum
support for higher-order programs.

The machine data is authoritative for stable identities and the harness checks
that every identity below remains documented. The harness independently pins
the exact ordered 29-variant identity set and a full-relationship fingerprint;
additions, removals, renames, invalid site kinds, or relationship changes fail.
A common corpus correction stops
the advocate sequence, requires independent review, and forces every completed
candidate to rerun.

## Neutral Representation

Every variant records: `case_id`, `variant_id`, polarity, frequency and
rationale; values; callable inputs/result and call sites; whether a callable is
passed, stored, or returned; latent operations; captured values, authorities,
and resources; ownership transfer; registration lifetime; expected semantic
observation; one reason ID; all relationship sites and the required site set;
and separate source requirement, operator consent, and operation-exercise
facts.

The representation contains no candidate mechanism. `none_explicit` is a
reported observation, never an omitted or favorable default. Type-only timeout
and parallel cases receive no scheduler, clock, or concurrency implementation
credit.

Site kinds are `definition`, `callable`, `call`, `capture`, `resource`,
`registration`, `return`, and `use`. A misuse has one fundamental reason and a
nonempty required site set. Routes preserve semantic order.

## Frozen Case Matrix

| Case | Frequency | Required shape and positive observations |
| --- | --- | --- |
| `effect.pure_map` | pervasive | Transform each input in order through a passed callable and preserve its input/result relationship. |
| `effect.effectful_map` | pervasive | Infer and propagate the passed callable's latent operation requirements. |
| `effect.filter_retain` | pervasive | Preserve Program 3's distinct-output odd filter and in-place retain deletion, ordinary same-list mutation rejection, and stale deleted-item-view rejection. |
| `effect.fold` | common | Preserve an effectful step's requirements through ordered accumulator application. |
| `effect.retry` | common | Handle retry failure without erasing any remaining action requirement. |
| `effect.with_timeout` | occasional | Model a handled timeout requirement only; receive no scheduler or clock implementation credit. |
| `effect.parallel_map` | occasional | Compose parallel and callback requirements only; receive no concurrency implementation credit. |
| `effect.callback_registry` | common | Store a callback capturing caller state under an explicit registration lifetime and unregister boundary. |
| `effect.event_handler_factory` | common | Return a callable retaining an exact authority requirement. |
| `effect.memoizing_wrapper` | common | Return a callable retaining wrapped requirements plus explicit cache allocation, ownership, lifetime, and cleanup. |
| `effect.logging_middleware` | common | Return a wrapper adding output while preserving every wrapped requirement. |
| `effect.linear_capture` | common | Move owned or Transaction-shaped linear state into a callable with explicit transfer and reject escape, double use, and outliving closure paths. |

Near-principal inference means one stable most-general summary within the
candidate's admitted model, up to documented normalization and renaming,
without caller annotations for ordinary positives. It is not a proof of global
principal types.

## Frozen Variants, Reasons, And Required Sites

| Case | Variant | Observation or fundamental reason | Required sites |
| --- | --- | --- | --- |
| `effect.pure_map` | `positive` | ordered mapped values; `none_expected` | `map.call`, `callback.call` |
| `effect.pure_map` | `misuse_callback_shape` | `callable_shape_mismatch` | `callback.definition`, `callback.call` |
| `effect.effectful_map` | `positive` | inferred requirement propagates; `none_expected` | `callback.call`, `map.call` |
| `effect.effectful_map` | `misuse_erased_requirement` | `latent_requirement_erased` | `callback.call`, `map.call` |
| `effect.filter_retain` | `positive_two_list_odd_filter` | input unchanged, output exactly `1, 3`; `none_expected` | `filter.call`, `predicate.call` |
| `effect.filter_retain` | `positive_retain_delete` | final list exactly `1, 3`, no deleted view escapes; `none_expected` | `retain.call`, `retain.delete` |
| `effect.filter_retain` | `misuse_same_list_mutation` | `active_iteration_mutation` | `source.mutation`, `filter.call` |
| `effect.filter_retain` | `misuse_stale_retained_view` | `stale_retained_item_view` | `retain.delete`, `stale.use` |
| `effect.fold` | `positive` | ordered result and propagated step requirement; `none_expected` | `fold.call`, `step.call` |
| `effect.fold` | `misuse_erased_step` | `latent_requirement_erased` | `step.call`, `fold.call` |
| `effect.retry` | `positive` | retry failure handled, other requirements retained; `none_expected` | `retry.call`, `action.call` |
| `effect.retry` | `misuse_erased_remaining` | `unhandled_requirement_erased` | `action.call`, `retry.call` |
| `effect.with_timeout` | `positive_type_only` | handled requirement with no runtime credit; `none_expected` | `timeout.call`, `action.call` |
| `effect.with_timeout` | `misuse_claims_runtime` | `unimplemented_machinery_credited` | `timeout.call`, `timeout.handle` |
| `effect.parallel_map` | `positive_type_only` | composed requirements with no runtime credit; `none_expected` | `parallel.call`, `callback.call` |
| `effect.parallel_map` | `misuse_erased_callback` | `latent_requirement_erased` | `callback.call`, `parallel.call` |
| `effect.callback_registry` | `positive` | count reaches two and unregister ends firing; `none_expected` | `register.call`, `unregister.call` |
| `effect.callback_registry` | `misuse_outlives_state` | `registration_outlives_capture` | `register.call`, `caller.state`, `return.site` |
| `effect.event_handler_factory` | `positive` | returned handler retains exact authority; `none_expected` | `factory.call`, `handler.call` |
| `effect.event_handler_factory` | `misuse_laundered_authority` | `captured_authority_erased` | `authority.capture`, `factory.call`, `handler.call` |
| `effect.memoizing_wrapper` | `positive` | explicit cache allocation/resource and retained wrapped requirement; `none_expected` | `memoize.call`, `wrapper.call`, `cache.allocation` |
| `effect.memoizing_wrapper` | `misuse_hidden_cache` | `hidden_allocation_or_resource` | `cache.allocation`, `memoize.call` |
| `effect.logging_middleware` | `positive` | output added and wrapped requirement preserved; `none_expected` | `wrapper.call`, `wrapped.call` |
| `effect.logging_middleware` | `misuse_erased_wrapped` | `latent_requirement_erased` | `wrapped.call`, `wrapper.call` |
| `effect.linear_capture` | `positive_move` | exactly one owner and one close/transfer; `none_expected` | `factory.call`, `closure.call` |
| `effect.linear_capture` | `misuse_move_without_transfer` | `ownership_transfer_missing` | `resource.capture`, `factory.call` |
| `effect.linear_capture` | `misuse_escape` | `captured_resource_escapes` | `resource.capture`, `return.site`, `factory.call` |
| `effect.linear_capture` | `misuse_double_use` | `linear_resource_double_use` | `first.use`, `second.use` |
| `effect.linear_capture` | `misuse_outlives` | `captured_resource_outlives` | `close.site`, `late.use`, `register.call` |

The complete site inventory remains in the machine data. Candidate diagnostics
must use these reason and site IDs rather than replacing them with native
jargon. Candidate-specific terminology is permitted only in the two native
result fields and rendered explanation.

## Frozen Candidate Result Contract

Each candidate emits one closed Rust result per case/variant with exactly:

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

Status is exactly `accepted`, `rejected`, `unsupported`, or `incomplete`.
Unsupported is not accepted. Incomplete is a hard evidence failure.

Annotations record site, purpose, and required/inferred mode. Requirement and
capture facts record stable identity, origin, semantic route, disposition, and
affected callable. Allocation/resource facts record kind, trigger, lifetime,
cleanup or transfer owner, and explicit versus prototype-assumed evidence.
Added machinery records its stable identity, layer (`language`, `checker`,
`runtime`, `ownership`, or `tooling`), implementation state, and whether it may
receive credit. Unimplemented or unexercised machinery receives none.

Rejected results require exactly one primary diagnostic, the frozen reason,
full required-site coverage, and a model-neutral repair. Missing fields, empty
required facts/sites, unreported allocation/resource/machinery, absent costs,
failed normalization, misuse acceptance, authority/ownership laundering, and
unknown/duplicate case results force `incomplete`. Unknown or duplicate
top-level fields are rejected.

Presence-shaped placeholders do not satisfy the contract. Each result must
match its selected variant's callable disposition, input/result shapes,
expected observation, capture domains, ownership transfer, registration
lifetime, source requirement, operator consent, operation exercise, and
allocation/resource domains. Structured facts must also match the frozen
stable identity, origin, disposition, affected callable, capture kind, and
semantic route; contradictory extras and copied generic facts reject.
The corpus owns explicit domain-to-origin joins independently for source
requirements, operator consent, exercised operations, and captures; no origin
is inferred from list position. Source annotations name an existing corpus
site, use a closed neutral purpose and `required`/`inferred` mode, and retain
the frozen call route. Every structured fact category has an exact attribute
key set, so nested candidate vocabulary or scoring fields reject.
Allocation and resource kind and evidence must match their frozen domain
meanings rather than candidate-supplied labels. Captures are typed
separately as ordinary values, exact authority, or actual resources, and only
the last category creates resource facts. `explicit_none` machinery is never
creditable; machinery identities and state/Boolean values have closed forms,
and other machinery is creditable only when implemented, exercised, and not
candidate-specific restructuring. Explicit-none machinery has one fixed
harness route; other machinery retains the selected variant's complete call
route. Allocation and resource identities are unique, and duplicate evidence
rejects even when its domains otherwise match.

## Normalization And Execution

- Lists/maps sort by stable corpus identity; source routes retain semantic
  order.
- Candidate variables normalize by first structural occurrence.
- Path separators normalize only for identity; original display spelling is
  evidence and is not rewritten.
- Candidate-native evidence is a sorted key/value bag with no scoring weight.
- Neutral summaries use only callable input/result, propagated/handled
  requirement, stored/returned callable, capture/escape, exact authority
  requirement, ownership transfer, and resource lifetime terms.
- A harness-owned factory constructs two new candidate run states. The harness
  executes both and complete canonical bytes must match exactly.

## Frozen Cost Method

`implementation_cost` is recomputed from a central harness-owned inventory of
the actual checked-in candidate modules and manifests; candidate execution
cannot supply or omit those strings. Rust attributes count as code while line
and block comments do not; comment-shaped contents of strings, raw strings,
byte strings, and character literals remain code. The measurement is
nonblank/noncomment candidate-module lines plus every ordinary, development,
build, and target-specific dependency entry. `analysis_cost` is recomputed from
a harness-owned trace of visited corpus nodes,
generated facts, generated constraints, normalization steps, and maximum live
analysis items. Wall time may be non-scoring host context only.
`diagnostic_cost` is recomputed from harness-owned rendered output and records primary count, required/covered site counts, rendered
UTF-8 bytes, native-term count, and presence of a neutral repair.

Hard gates are complete corpus coverage, one primary diagnostic for each
rejection, full required-site coverage, no authority/ownership laundering, and
no missing evidence. Session AK compares raw measurements and may add no hidden
weight, favorable default, or candidate-only metric.

## Corpus Locks

An around-call helper is not a returned wrapper. An owner table is not captured
caller state. Copying is not an owned view/resource. A monomorphic callback is
not inferred latent behavior. Closure environments, caches, boxes, reference
counts, and registry allocations are explicit. Exact authority facts keep
source requirement, operator consent, and exercised operation separate.

## Pure Second-Class-Computation Eligibility Observation

This section is an eligibility record, not neutral corpus data, advocacy, or a
candidate score. The research calls the family "pure Effekt-style". Against the
unchanged exam it is `ineligible_without_restructuring` for:

- `effect.callback_registry`: `stored_callable` is required;
- `effect.event_handler_factory`: `returned_callable` is required;
- `effect.memoizing_wrapper`: a returned callable retaining cache state is
  required; and
- `effect.logging_middleware`: a returned wrapper is required, so an immediate
  around-call substitute does not count; and
- `effect.linear_capture`: the resource-bearing callable must be returned or
  stored beyond its defining call.

This observation eliminates nothing by prose. The harness records the exact
relationship failure; candidate selection remains Session AK-only.

## Scope

The corpus and harness are offline experiments. They add no Hum syntax,
production higher-order value, compiler stage, H-code, command, schema, runtime
path, candidate score, or decision. Sessions AH-AK and all production work
remain separately gated.

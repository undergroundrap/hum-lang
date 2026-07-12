# Capture-Oriented Checking Best Advocate

Date: 2026-07-11
Status: bounded Session AJ prototype; not a candidate selection or production decision

## Strongest Honest Case

Capture-oriented checking fits the part of Hum's effect story that is already
value-shaped. A callable retains a typed, exact environment rather than an
ambient capability root. The prototype distinguishes immutable values, moved
owned state, borrowed owned state, Transaction-shaped linear resources, and
exact source-authority requirements. It never treats operator consent or an
operation exercise as something a closure can capture or infer.

That is a particularly direct fit for returned handlers, callback registries,
memoizers, logging wrappers, and linear-resource closures. The checker can say
what the callable keeps alive, which owner moved, and which exact authority
requirement survives the boundary. It also makes closure-environment
allocation and cleanup ownership difficult to hide.

The prototype does not pretend that retention is the whole effect system.
Failure, retry handling, timeout, parallel composition, stateful operations,
and wrapped callable requirements are represented by separate hybrid effect
facts. That machinery is implemented and measured inside this experiment; it
is not smuggled into the capture relation or credited to decisions 0014/0017.

## Executable Model

The native model has:

- closed explicit sets and inferred sets with one structural tail variable;
- deterministic atom order and alpha-renamed `$cN` variables;
- one-use substitutions whose rejected duplicate bindings leave the original
  mapping unchanged;
- one shared 64-atom validation applied to canonicalization, closed/open
  substitution, and both sides of every structural subcapture check;
- strict result capture, which retains the complete lexical environment;
- lazy result capture, which retains only identities joined to frozen
  domain-to-origin use facts;
- separate hybrid requirement sets with explicit handled-requirement removal;
- side checks for callable shape, active-place conflicts, stale views,
  registration lifetime, exact authority retention, allocation visibility,
  ownership transfer, linear use count, resource escape, and runtime-credit
  honesty.

Capture atoms are pairs of exact identity and class. A logging grant stays
`logging.grant_exact`; it never widens to `IO`. Source requirement, operator
consent, and exercised operation remain three frozen policy facts outside the
capture set. Removing consent from an otherwise unchanged handler leaves the
retained requirement visible but creates no permission.

## Frozen Corpus Result

The unchanged corpus contains 12 cases and 29 variants. This prototype emits
13 accepted positive results and 16 single-cause misuse rejections through the
unchanged candidate-neutral contract.

| Case | Positive evidence | Misuse evidence |
| --- | --- | --- |
| pure map | callable shape and empty environment | callback shape mismatch |
| effectful map | hybrid callback requirement propagates | latent requirement erasure |
| filter/retain | two-list and retain behavior retain view facts | active mutation and stale view |
| fold | step requirement preserved | erased step requirement |
| retry | retry failure handled; remaining requirement preserved | unhandled requirement erasure |
| timeout | type-only handling with no runtime credit | false runtime credit |
| parallel map | callback and parallel requirements coexist | erased callback requirement |
| callback registry | stored environment and registration lifetime | registration outlives borrowed state |
| event handler factory | immutable prefix plus exact authority retention | authority erasure/widening |
| memoizing wrapper | moved cache state, box, cache allocation, cleanup owner | hidden allocation/resource |
| logging middleware | returned callable preserves wrapped and output requirements | wrapped requirement erasure |
| linear capture | explicit move and one use | missing transfer, escape, double use, outlives |

Every result is produced twice from fresh candidate state and canonical bytes
must match. Poisoning expected polarity, reason, observation, or blame sites
does not change native analysis. Independent semantic mutations cover the
capture classes, authority identity, hybrid requirements, allocation,
registration, ownership transfer, resource lifetime, use count, and runtime
credit.

## Strict Versus Lazy Result Capture

The frozen programs name only semantically retained facts, so strict and lazy
result capture agree for their stored and returned callables. The executable
result path derives used identities from the candidate program's frozen
domain-to-origin capture relationships rather than declaring every environment
atom used. A permanent adversarial program adds an unused borrowed environment
atom without a use relationship: the same result path keeps it in strict mode,
removes it in lazy mode, and continues to retain the used immutable value and
exact authority identity.

This difference is real prototype behavior, not free production machinery.
The static result-retention analysis is priced as implemented checker work.
Actual callable environments remain explicitly unimplemented runtime
machinery, uncreditable in the neutral result contract.

## Inference And Signature Burden

All 13 frozen positives use inferred source annotations in the bounded
prototype. That is near-principal evidence only for this finite constraint
language; it is not a proof of principal inference. Library signatures for
stored and returned callables must expose retained environment identities and
hybrid requirements. Ordinary callers add no annotation in the frozen cases,
but library authors pay the public-signature burden when an exact authority,
borrowed owner, or linear resource escapes through a callable value.

The 64-atom bound, closed substitution table, deterministic `BTreeSet` order,
and bounded synthetic growth test make analysis cost reproducible. They do not
establish production scaling or performance.

## Allocation, Resource, And Added Machinery Honesty

Returned and stored callables report `callable_environment`; registries report
`registry_storage`; memoizers report `cache_storage`. Resource facts remain
separate from ordinary captured values and authority. Registration lifetime,
resource lifetime, allocation lifetime, cleanup owner, and ownership transfer
remain distinct neutral facts.

Creditable experimental checker machinery is limited to retained-environment
inference/subcapture, result-retention analysis, and hybrid requirement facts.
Ownership/place/lifetime guards are explicit but uncreditable as candidate
effect machinery. Callable-environment runtime support, timeout execution, and
parallel execution remain unimplemented and uncreditable.

Implementation cost is recomputed from the harness-owned checked-in module and
manifest. Candidate output cannot choose its inventory, source set, manifest,
line count, or dependency count.

## Diagnostics

Misuses produce one model-neutral reason with the frozen primary and related
sites and the repair direction `preserve the model-neutral relationship`.
Rendered messages speak about callable requirements, retained authority,
ownership transfer, allocation, and lifetime. They do not expose raw solver
or capture-set dumps as user blame.

## Limits And No Claims

This is an offline experimental Rust module with no dependencies. It changes
no Hum syntax, compiler stage, runtime, schema, diagnostic, command, Core fact,
graph fact, or authority audit surface.

It does not prove principal inference, ownership soundness, authority safety,
memory safety, production performance, runtime closure representation, timeout
execution, parallel execution, or that capture checking alone is a complete
effect system. It does not narrow decision 0014, widen decision 0017, select a
candidate, or propose decision 0018.

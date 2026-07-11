# Open Effect Rows: Best Advocate Record

Date: 2026-07-11
Status: Session AH experimental advocate; not a Hum language decision

## The strongest case

Open effect rows give higher-order APIs a compact answer to the question
“what else may this callable do?” A callable carries a latent row of stable
effect labels and an open tail. Application propagates that latent row to the
caller. A handler removes exactly one occurrence of the label it handles while
leaving the open tail and all unrelated labels intact. In the frozen corpus,
this directly represents pure and effectful map, filter/retain, fold, retry,
timeout-shaped handling, parallel composition, returned wrappers, and stored
callbacks without asking ordinary callers to repeat effect annotations.

The prototype normalizes labels lexically and tail variables by first
structural occurrence. Distinct open tails remain distinct variables until the
application solver records a structural substitution that unifies them; no
tail names are concatenated into a synthetic variable. Written aliases survive
rendering. Rows are multisets:
duplicate labels are meaningful, deterministic entries, and exact handling
removes one occurrence rather than silently collapsing every equal label.
Those rules make order changes and alpha-renaming byte-stable while retaining
the precision needed for nested handling.

## Callable and handler model

The candidate represents a callable as an input shape, result shape, latent
open row, capture relationships, and a disposition: passed, stored, or
returned. Applying the callable propagates its row. Retry removes one
`retry.failure`; timeout-shaped checking removes one `timeout.requirement`.
Logging middleware adds `output.requirement` and preserves the wrapped tail.
The candidate does not claim a scheduler, clock, parallel runtime, or handler
runtime implementation from these type relationships.

The ordinary positive cases need no caller-written effect annotation in this
prototype. Their normalized summaries are stable within the admitted row
model, modulo the documented label ordering and tail renaming. The prototype
does not implement a general Hum type-inference front end, so this is bounded
near-principal corpus evidence rather than a proof of global principal types,
inference completeness, or arbitrary-program behavior.

## Allocation, authority, and ownership honesty

First-class returned and stored callables need closure environments. The
candidate reports those allocations explicitly as prototype-assumed and
records the missing runtime environment implementation without credit. Cache
storage, registry storage, and captured resources remain separate facts rather
than being hidden inside an effect label.

Rows alone do not prove any of the following:

- an exact authority value was retained rather than widened or laundered;
- operator consent exists because a source requirement appears in a row;
- an owned or linear resource was transferred exactly once;
- a captured view or registration remains within its lifetime; or
- a closure environment was allocated, cleaned up, or made runtime-safe.

The prototype therefore uses candidate-owned callable-shape, capture,
authority-retention, allocation-visibility, place/view,
registration-lifetime, machinery-credit, and linear-resource facts. The side
checks compare actual shapes, sets, lifetimes, transfer state, and use counts;
they are not selected from the corpus verdict or reason. They are surfaced as
separate machinery, included in measured implementation and analysis cost, and
given no row-model credit. Decision 0014 ownership and decision 0017 authority
are not treated as a free bridge.

## Diagnostic case

Rows provide a useful local explanation when an effect is erased: name the
preserved alias, show the normalized latent row, identify the missing label,
and blame the frozen callable and call sites. The prototype emits exactly one
primary diagnostic for each misuse and retains the model-neutral reason,
required sites, and repair. Candidate terminology stays in the native result
and rendered explanation; the shared comparison remains corpus-neutral.

The known ergonomic risk is row growth. The stress test exercises 256 labels
with deterministic ordering, but that is only bounded prototype evidence. It
does not establish production throughput or prove that large public rows will
be teachable. Alias preservation reduces expansion noise; it does not make
effect vocabulary discipline optional.

## Frozen-exam result

The unchanged exam contains 12 cases and 29 variants: 13 positives and 16
misuses. Each variant is translated into candidate-owned callable, row,
handler, capture, lifetime, authority, allocation, and ownership constraints.
Analysis does not read corpus polarity, expected reason, required blame sites,
or expected observation. Poisoning those answer fields leaves native analysis
byte-identical, while mutating a semantic row claim or lifetime fact changes
the derived outcome. The candidate emits one complete frozen result for every
variant and fresh-state execution is byte-identical twice. Type-only timeout
and parallel cases receive no runtime credit. Returned/stored callable
allocation, resource capture, exact authority separation, and all added side
machinery are present in the same result contract used by every later
candidate.

Implementation cost is recomputed from the checked-in candidate module and
manifest owned by the harness. Analysis and diagnostic costs are reconstructed
from harness-owned traces and rendered diagnostics. No candidate-supplied
score, favorable default, dependency, production Hum hook, corpus rewrite, or
candidate-only comparison field is introduced.

## Honest conclusion

Open rows are a strong direct effect-polymorphism baseline for Hum because
they express the entire higher-order shape space while keeping ordinary caller
burden low and handling subtraction precise. They are not, by themselves, an
ownership system, capture checker, authority proof, runtime handler system, or
allocation implementation. Session AH establishes an auditable advocate result
and its costs. Candidate selection remains Session AK work after the two other
advocates run against the unchanged exam.

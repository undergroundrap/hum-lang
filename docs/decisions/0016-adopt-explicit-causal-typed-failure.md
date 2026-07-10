# 0016: Adopt Explicit Causal Typed Failure

Date: 2026-07-10
Status: accepted under delegated authority (BDFL veto open)

## Context

The first IO-capability slice will need errors that cross task boundaries
without becoming invisible control flow or losing the source site that created
the failure. Before Session W, the interpreter could carry a typed-looking
variant, but ordinary task calls propagated it implicitly and retained no
origin or call-site chain.

Session W deliberately tests only direct named calls with ordinary value
arguments. It does not settle first-class `Result` values, variant membership,
recovery, effect polymorphism, or general call typing.

## Decision

The proposed initial surface contains exactly:

```hum
let value = try fallible_call()
let value = try fallible_call() or fail OuterError.context
```

Unwrapped `try` is valid only when caller and callee declare the same nominal
error root. The wrapping form must use the caller's declared root and preserves
the inner cause. A direct `fail Root.case` must also use the caller's root.

Known fallible calls in every currently executable expression position never
propagate implicitly, including calls nested in operators or arguments and
calls used as loop collections. Typed failure is an ordinary exit-1 outcome,
distinct from a runtime trap. The runtime carrier retains the root identity and
origin and every recognized propagation or wrapping site, then renders them
outer-to-root. The existing `fails when:` section must state a meaningful
condition for every task that can fail through this slice.
Nonempty placeholder text is not meaningful: the same hollow-contract rule
used by source checking rejects `todo`, `tbd`, tautologies, and generic claims
for direct failure, propagation, and wrapping.

## Consequences

Call sites disclose failure movement, nominal incompatibility is rejected
before execution, and context addition cannot erase a root cause. Full-type
and effect rows expose the same roots and source sites used by runtime
diagnostics. This is sufficient groundwork for the first IO error path without
granting IO authority or adding an exception mechanism.

Unsupported `try` shapes remain explicit H0906 blockers through Core preview,
lower, and verify rather than being represented as executable operations.
`try` recognition is keyword-bounded; identifiers such as `trying` and
`try_value` are ordinary names.

Permission-bearing call arguments, first-class `Result`, checked variant
membership, exhaustiveness, recovery or catch, narrowing, exceptions, unwind,
ambient backtraces, erased any-error propagation, and general call typing
remain outside the decision.

## Alternatives Rejected

### Implicit Fallible Calls

Rejected because innocent-looking calls would hide an effect and its blame
site.

### Incompatible Implicit Propagation

Rejected because it erases the caller's nominal failure contract.

### String-Only Context

Rejected because prose detached from a typed outer failure cannot be checked
against the caller root.

### Exceptions Or Unwind

Rejected for this slice because non-local hidden control flow conflicts with
Hum's explicit-effect model.

### Erased Any-Error Propagation

Rejected because it discards nominal compatibility and weakens repair quality.

### Ambient Backtraces

Rejected as the semantic carrier. A host backtrace is neither stable source
blame nor a substitute for explicit causal sites.

## BDFL Note

This proposal chooses a small, visible bridge from typed task failure to later
IO errors. It intentionally makes context structural and causal while leaving
recovery and richer `Result` design for evidence after the IO slice.

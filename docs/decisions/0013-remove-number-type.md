# 0013: Remove the Number Type

Date: 2026-07-08
Status: accepted

## Context

Early examples (README, SPEC, app sketches) use a `Number` type alongside
`Int`, `UInt`, and `Float`. `Number` was never defined: not its width, its
signedness, its overflow behavior, or its relationship to the other numeric
types. Vague numerics are a classic early language trap; every month `Number`
survives, more examples and habits accrete around an unspecified type.

FORMAL_CORE already keeps floating point outside the trusted core until safety
and replay profiles have a strict float policy.

## Decision

`Number` is removed from Hum.

Accepted rules:

1. Core numeric types are `Int` and `UInt`, with no implicit narrowing, no
   implicit signedness conversion, and defined overflow diagnostics before the
   core is called stable.
2. `Float` exists in surface Hum design but stays outside the trusted core
   until a float policy (determinism, NaN handling, replay profiles) is
   accepted.
3. All examples, docs, and fixtures that use `Number` migrate to `Int` or
   `UInt`, whichever the example actually means.
4. A future high-level numeric alias may be proposed only through a new
   decision record that pins exact semantics.

## Consequences

README and SPEC examples change. The type-check and type-env surfaces have one
less unspecified name to carry. Beginners learn two honest types instead of
one comfortable lie.

## Alternatives Rejected

- Define `Number` as an alias for `Int`: an alias with a friendlier name
  becomes the spelling everyone uses, violating one-spelling-per-concept.
- Define `Number` as an arbitrary-precision integer: wrong default cost model
  for a systems language; belongs in the stdlib as an explicit type if ever.
- Keep it undefined a while longer: that is how languages get `long long`.

## BDFL Note

If Hum ever needs a beginner-facing numeric story, it should be a teaching
story about `Int` and `UInt`, not a type that hides the machine.

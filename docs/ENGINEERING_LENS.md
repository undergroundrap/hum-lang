# Hum Engineering Lens

Date: 2026-07-06

## Thesis

Hum should write down the mental translation a senior engineer performs while
reading code.

When a strong systems programmer reads a loop, they do not only see syntax. They
hear:

- this scans the whole collection
- this is inside another loop
- this allocates in the hot path
- this mutates shared state
- this can turn into O(n^2)
- this assumes the input is small
- this is fine only because the data is bounded

Hum should let that inner narration become checked source.

## Recommended Blocks

### `cost:`

Declares expected complexity and resource behavior.

```text
cost:
  time: O(tasks)
  space: O(1)
  allocates: nothing
  io: none
```

This gives humans and agents a fast review surface. It also gives the compiler a
claim to compare against static analysis, instrumentation, and benchmarks.

### `avoids:`

Declares implementation shapes the author does not want.

```text
avoids:
  nested scan over users and sessions
  allocation inside loop
  network call inside loop
  unbounded recursion
```

This is the compiler-grade version of `dislikes:`. A beginner tool can ask "what
should this code avoid?" or even "what do you dislike here?" but the source
should use `avoids:` because it is actionable.

### `tradeoffs:`

Declares the compromise a senior engineer would normally explain in review.

```text
tradeoffs:
  linear scan is acceptable because task lists are expected to stay small
  dense storage is preferred over stable addresses
```

A tradeoff is not an excuse. It is a named decision that future tools can
re-check when the program changes.

## Enforcement Model

`cost:` should support increasing strictness:

```text
check: graph    record the claim in JSON
check: warn     warn on likely contradictions
check: compile  fail when the compiler can disprove the claim
check: prove    fail unless the claim can be proven
check: bench    fail benchmark or release profiles when measurements miss
```

The default should be gentle enough for beginners. Critical code can demand hard
enforcement.

See [PERFORMANCE_CONTRACTS.md](PERFORMANCE_CONTRACTS.md).

## Complexity Notation

Hum should support a small complexity vocabulary first:

```text
O(1)
O(log n)
O(n)
O(n log n)
O(n^2)
O(input)
O(items)
O(users * sessions)
```

Complexity symbols are source-level claims. In v0, Hum may only parse and emit
them into the semantic graph. Later, Hum can compare them against loop structure,
collection operations, profiling, and benchmark data.

## Example

```text
task find active session(user: UserId, sessions: Sessions) -> Result Session, SessionError {
  why:
    find the first non-expired session for a user

  uses:
    sessions
    clock.now

  cost:
    time: O(sessions)
    space: O(1)
    allocates: nothing

  avoids:
    nested scan over users and sessions
    allocation inside loop

  tradeoffs:
    linear scan is acceptable for small local session lists

  does:
    for each session in sessions {
      if session.user == user and session.expires_at > clock.now {
        return session
      }
    }

    fail SessionError.not_found
}
```

## Senior Engineer Diagnostics

If a function declares:

```text
cost:
  time: O(users)
```

but the compiler sees a loop over users inside a loop over sessions, it should
say:

```text
error: declared cost says O(users), but loop structure looks like O(users * sessions)

why:
  this task loops over `sessions` inside a loop over `users`

fix:
  update `cost:` if the nested scan is intentional, or index sessions by user
```

This is where Hum starts sounding like a senior reviewer.

## Relationship To Existing Blocks

- `optimizes:` says what we want better.
- `cost:` says what we believe the current shape costs.
- `allocates:` says what memory behavior is allowed.
- `avoids:` says what implementation shapes are forbidden or suspicious.
- `tradeoffs:` says why the accepted shape is reasonable.
- `watch for:` says what future failures or edge cases deserve attention.

The time-space simulation research in
[research/2026-07-07-time-space-simulation.md](research/2026-07-07-time-space-simulation.md)
adds one more senior-engineer question: should this code store intermediate
state, recompute it, or expose a bounded cache/checkpoint policy?

These blocks should work together, not replace each other.

## Brutal Warning

Do not let `cost:` become decorative.

A false complexity claim is worse than no claim because it trains reviewers to
trust lies. Milestone 0 should emit `cost:` into JSON. Later milestones must
check it against loops, calls, allocation, and benchmarks.
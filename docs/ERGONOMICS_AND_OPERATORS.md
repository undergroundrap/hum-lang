# Hum Ergonomics And Operators

Date: 2026-07-06

## Purpose

Hum needs the comforts programmers love without becoming a language of clever
shortcuts.

The goal is not to have the most operators. The goal is to have the right few
operators, the right English-shaped constructs, and the right tool feedback so
code remains fast, safe, and obvious under pressure.

Quality of life is not sugar if it removes whole classes of mistakes.

## Ergonomics Rule

A convenience feature is Hum-shaped only if it satisfies all five:

1. It makes common code clearer at the use site.
2. It removes repetition without hiding effects, allocation, mutation, failure, or cost.
3. It has one canonical spelling in formatted code.
4. It appears clearly in the semantic graph.
5. It can be taught to a beginner and defended to a systems programmer.

If it only saves characters, it is not enough.

## Operator Philosophy

Operators are for concepts programmers already read as operators:

- arithmetic
- comparison
- indexing
- field access
- assignment inside explicit mutation
- low-level bit work in explicit bit contexts

Everything else should start as a named construct.

Hum should prefer this shape:

```text
if user is verified and device is allowed {
  try create session for user
}
```

before this shape:

```text
if (u.ok && d.ok) create(u)?;
```

The second form is compact. The first form carries intent.

## Core Operator Set

### Calls, Grouping, And Access

Keep:

- `task name(a, b)`: call syntax
- `(expr)`: grouping
- `value.field`: field access
- `items[index]`: checked indexing
- `items[range]`: checked slicing
- `module.name`: module/package access unless a stronger module syntax wins

Rules:

- indexing is bounds-checked in safe Hum
- unchecked indexing requires an unsafe or proven-bound context
- field access must not trigger hidden IO or allocation
- computed properties with nontrivial cost must expose `cost:` metadata

### Arithmetic

Keep:

```text
+  -  *  /  %
```

Rules:

- integer overflow behavior is explicit by mode or type family
- checked arithmetic is the beginner-safe default
- wrapping, saturating, and unchecked arithmetic are named or typed
- exponentiation should be named (`pow`) until a clear operator story exists

### Comparison

Keep:

```text
==  !=  <  <=  >  >=
```

Rules:

- equality should be explicit about value identity versus object identity
- floating-point comparison diagnostics should warn when exact equality is suspicious
- ordering must be total only when the type promises total ordering

### Boolean Logic

Prefer words:

```text
and  or  not
```

Reason:

- beginners read them immediately
- experts do not lose power
- `&&`, `||`, and `!` create visual noise near borrow, bit, and unsafe syntax

`chirp` should reject mixed symbolic and word boolean logic in one expression.

### Mutation And Compound Assignment

Keep mutation explicit:

```text
let total = 0
change count = 0
set total += price
set count += 1
```

Rules:

- `++` and `--` do not enter Hum
- compound assignment requires `set`
- assignment is not an expression
- mutation must match `changes:` or a local `change` binding

This preserves the convenience of `+=` without importing C-style ambiguity.

### Bit Operators

Reserve symbolic bit operators for numeric and bitset contexts:

```text
&  |  ^  <<  >>
```

Rules:

- bitwise operators never mean boolean logic
- shifts must define behavior for oversized or negative shift counts
- security-sensitive bit work should carry `protects:` and tests

### Ranges

Hum needs ranges, but range endpoints are a classic bug source.

Preferred readable forms:

```text
for index i from 0 until count
for index i from 1 through last
items[from 0 until count]
```

Possible compact forms can be considered later:

```text
0..<count
1..=last
```

Rules:

- beginner docs use `until` and `through`
- formatter keeps range spacing canonical
- diagnostics always say whether the end is included or excluded

### Error Propagation

Rust's `?` is excellent for experts but invisible to beginners.

Hum should start with:

```text
try load profile(id)
```

not:

```text
load_profile(id)?
```

Rules:

- `try` propagates typed failure
- the propagated failure must be listed under `fails when:` or inferred into it
- a postfix shorthand can be reconsidered only after diagnostics and pedagogy are excellent

### Option And Presence

No ordinary null.

Avoid importing optional chaining as-is:

```text
user?.address?.city
```

because it can hide where absence happened.

Prefer explicit presence handling:

```text
when user.address is present as address {
  return address.city
}
```

and defaulting:

```text
let city = user.address.city or "unknown"
```

Rules:

- `or` defaulting must apply only to `Option`, not falsy values
- no truthy/falsy coercion
- `chirp` should flag long chains of presence/default operations

### Pipelines

Pipelines are useful, but they can hide cost.

Candidate syntax:

```text
users
  |> keep active
  |> sort by last login descending
  |> take first 100
```

Rules before stabilization:

- each stage appears in the semantic graph
- allocation and laziness are visible
- `cost:` can account for the whole pipeline
- formatter breaks long pipelines one stage per line

### Pattern Matching And Destructuring

Keep:

- exhaustive `match`
- destructuring in `let`
- named variants
- no fallthrough

Example:

```text
match result {
  ok user:
    return user
  error NotFound:
    fail user missing
}
```

Rules:

- non-exhaustive matches are errors unless a deliberate fallback is named
- public APIs should prefer named fields over positional mystery

### Record Update And Builders

Programmers love update syntax because it removes boilerplate.

Candidate syntax:

```text
let updated = user with {
  name: new name
  last seen: clock.now
}
```

Rules:

- `with` creates a new value unless used inside explicit `set`
- hidden mutation is not allowed
- missing required fields are compile errors

### Collection Literals

Hum needs literals for common data:

```text
[1, 2, 3]
map {
  "host": "localhost"
  "port": "8080"
}
set { "read", "write" }
```

Rules:

- list, map, and set literals are visually distinct
- large literals should report allocation/cost where relevant
- duplicate map keys are diagnostics when statically visible

### String Interpolation

String interpolation is quality of life, but it is also a security boundary.

Candidate syntax:

```text
text "hello {user.name}"
```

Rules:

- format strings are parsed by the compiler
- SQL, shell, HTML, and URL contexts require typed escaping APIs
- secrets must not be formatted without an explicit reveal boundary

## Operators To Reject Early

Reject from the core language:

- `++` and `--`
- ternary `?:`
- arbitrary custom operators
- assignment as an expression
- implicit numeric widening or narrowing
- truthy/falsy coercion
- hidden optional chaining over ordinary null
- overloaded control-flow operators
- user-defined precedence
- macros that create unparseable mini-languages

These are not forbidden forever because of taste. They are rejected until someone
can prove they improve Hum without recreating old regrets.

## Bevy Lessons For Hum

Bevy ECS is a useful design teacher even outside games.

The important ideas:

- app data is split into plain components
- systems declare what data they read and write
- the scheduler can run non-conflicting systems in parallel
- resources are unique typed data outside ordinary entities
- storage layout can differ by access pattern
- change detection is first-class
- messages and observers separate scheduled flow from reactive flow
- bundles make common groups of data easy to spawn together

Hum already has the beginning of this with `uses:` and `changes:`.

The deeper lesson is that declared access is not only documentation. It is an
optimization and scheduling primitive.

## Hum Data-Oriented Response

Hum should eventually make this kind of code possible:

```text
task move players(world: GameWorld) {
  uses:
    world.velocity

  changes:
    world.position

  cost:
    time: O(players)
    space: O(1)

  does:
    for each entity in world where has position and velocity {
      set entity.position += entity.velocity
    }
}
```

The compiler should understand:

- this task reads velocity
- this task changes position
- it can run beside tasks that do not touch position
- storage layout affects iteration speed
- change tracking can wake dependent tasks
- the query has a measurable cost

That is Bevy's best lesson translated into Hum.

## Data-Oriented Features To Design

Before Hum claims to be great for data-heavy systems, design these deliberately:

1. `store` layout declarations: dense, sparse, table, arena, stable, cache.
2. query syntax for selecting records/entities/components.
3. change detection: `when changed`, `added`, `removed`, `touched`.
4. deterministic schedules for tasks with declared access.
5. parallel scheduling from `uses:` and `changes:`.
6. event/message channels with explicit ordering and replay behavior.
7. bundle/group syntax for common data sets.
8. relationship syntax for parent/child, ownership, graph, and index links.
9. profiler output that maps hot loops back to source promises.
10. storage diagnostics when declared access conflicts with chosen layout.

This belongs in the stdlib/data model, not in Milestone 0 syntax.

## Quality-Of-Life Checklist

Hum still needs design answers for:

- named and default arguments
- optional arguments without overload sprawl
- pattern matching syntax
- destructuring syntax
- collection literals
- string interpolation
- range and slice syntax
- resource cleanup and `defer`-like behavior
- scoped capabilities
- package aliases and imports
- visibility modifiers
- test data builders
- generated mocks/fakes for tests
- benchmark fixtures
- REPL or scratch runner
- compiler quick fixes
- semantic search over `why:`, `uses:`, `changes:`, and `cost:`

These are not side quests. They decide whether Hum feels alive.

## Admission Gate

No operator or QoL feature should stabilize unless it answers:

1. What repetition does this remove?
2. What bug does this prevent?
3. What meaning does this hide?
4. Can `humfmt` make it canonical?
5. Can `chirp` catch its most common misuse?
6. Does it preserve `uses:`, `changes:`, `fails when:`, `cost:`, and semantic graph clarity?
7. Is it readable to a beginner after one example?
8. Is it still useful to an expert after 100,000 lines?
9. Does it compose with ownership, effects, tests, and Nectar?
10. Which old language regret could it recreate?

## Brutal Rule

Hum should not be afraid of convenience.

Hum should be afraid of convenience that makes code lie.

## Sources

- Bevy ECS crate documentation: https://docs.rs/bevy_ecs/latest/bevy_ecs/
- Rust Reference, operator expressions: https://doc.rust-lang.org/reference/expressions/operator-expr.html
- Rust `std::ops`: https://doc.rust-lang.org/std/ops/index.html
- Swift API Design Guidelines: https://www.swift.org/documentation/api-design-guidelines/
- TypeScript 3.7 optional chaining and nullish coalescing notes: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3-7.html
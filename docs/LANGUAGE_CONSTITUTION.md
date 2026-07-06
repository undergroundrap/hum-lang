# Hum Language Constitution

Date: 2026-07-06

## Purpose

Hum exists to make systems programming clearer, safer, faster, and more
auditable without hiding the machine.

Hum is not trying to make programming look like casual English. It is trying to
make program intent explicit enough that humans, compilers, and agents can share
the same facts.

## Prime Directive

```text
Humans write intent.
The compiler enforces promises.
Agents propose, repair, and explain.
Proofs, tests, and benchmarks decide.
```

## Safety Of Maker And User

Hum must protect both sides of the work:

- the maker building the language, toolchain, packages, and release artifacts
- the user running Hum programs, depending on Hum packages, or operating Hum
  systems

The toolchain should never quietly expand authority on the developer machine.
Hum programs should never hide authority from the people who run or audit them.

See [SAFETY_OF_MAKER_AND_USER.md](SAFETY_OF_MAKER_AND_USER.md).

## Non-Negotiable Rules

1. Important claims belong in syntax, not comments.
2. The executable core must be precise.
3. Effects are part of the interface.
4. Mutation is a declared capability.
5. Allocation is visible.
6. Unsafe code is small, named, justified, and checked.
7. Security boundaries are explicit.
8. Performance claims require benchmarks.
9. Agents are never part of the trusted base.
10. The language and toolchain must protect both the maker and the user.
11. A feature is not accepted until it can be taught.

## Comment Philosophy

Hum should eliminate most comments by giving programmers better places to put
meaning.

Instead of:

```text
// Make sure the token does not leak the user's password.
// Also beware hash flooding.
```

Hum should prefer:

```text
protects:
  token does not reveal user secrets

watch for:
  attacker may create many sessions quickly
  hash collisions must not make lookup slow
```

Comments are still allowed, but they are second-class. If a sentence affects
correctness, safety, security, performance, ownership, allocation, or failure
behavior, it should live in a checked block.

## The Three Audiences

Hum code must serve three audiences at once:

- humans reading source
- compilers enforcing facts
- agents navigating, repairing, and explaining code

If a design helps only one audience while making the others blind, it is suspect.

## The Mission-Critical Standard

Hum should be designed as if it will be used in software where failure matters:

- kernels
- databases
- browsers
- compilers
- embedded devices
- financial infrastructure
- security tools
- medical and industrial systems
- long-lived infrastructure services

That does not mean every Hum program must be heavy. It means the language's
defaults should not train programmers into habits that fail under pressure.

## Readability Standard

Readable does not mean vague.

Readable Hum should make these facts obvious:

- why the task exists
- what it reads
- what it mutates
- what must be true before it runs
- what must be true after it succeeds
- how it can fail
- what it protects
- what edge cases matter
- how memory behaves
- what effects escape the task

If a beginner can read a task and understand its promise, and a senior engineer
can inspect the same task for safety, the syntax is doing its job.

## Precision Standard

The `does:` block cannot remain fuzzy natural language forever.

Hum may start with controlled readable phrases, but every executable phrase must
lower to a precise internal operation. When a phrase is ambiguous, the compiler
must reject it or ask for a more precise form.

No magic English execution.

## Optimization Standard

Hum should optimize from declared intent, not from folklore.

Good:

```text
optimizes:
  lookup speed
  memory density
  security before speed
```

Better:

```text
expects:
  up to 20 million active sessions

optimizes:
  p99 lookup under 200 ns on target server
  memory overhead below 2 bytes per entry
  resist hash flooding from hostile keys
```

The compiler should turn optimization claims into benchmark obligations whenever
possible.

## Security Standard

Security properties must have names and blame.

`protects:` declares what must not be broken.
`trusts:` declares what the task relies on outside itself.
`unsafe` declares where the checker needs extra proof.

If a security-sensitive task cannot explain what it protects, it is not ready.

See [SECURITY_MODEL.md](SECURITY_MODEL.md) and [UNSAFE_POLICY.md](UNSAFE_POLICY.md).

## Agent Standard

Hum should be unusually good for agents, but never dependent on agents for
correctness.

Agents should receive:

- stable syntax
- canonical formatting
- JSON semantic graphs
- source spans
- effect graphs
- ownership graphs
- blame diagnostics
- generated test obligations
- repair hints

Agents may suggest code. The compiler, tests, proofs, fuzzers, and benchmarks
accept or reject it.

## The Senior Engineer Lens

Hum should write down the review thoughts experienced engineers usually keep in
their heads:

- expected time complexity
- expected space complexity
- hidden allocation risk
- nested-loop risk
- unacceptable implementation shapes
- accepted tradeoffs

These belong in checked blocks such as `cost:`, `avoids:`, and `tradeoffs:`.

## Governance Standard

Hum uses a BDFL model with evidence-first review. The BDFL preserves taste and
final direction, while contributors provide examples, diagnostics, semantic graph
impact, tests, benchmarks, migration plans, and rejected alternatives.

See [GOVERNANCE.md](GOVERNANCE.md).

## Feature Admission Test

Before adding a feature, answer:

1. What confusion does it remove?
2. What bugs does it prevent?
3. What performance does it make possible?
4. What does it cost the compiler?
5. What does it cost the programmer?
6. Can it be represented in the semantic graph?
7. Can an agent understand it from compiler docs?
8. Can it be tested, fuzzed, proven, or benchmarked?
9. Is there one obvious way to use it?
10. What happens if we do not add it?

If these answers are weak, wait.

## Design Debt We Refuse

Hum must not accumulate:

- hidden effects
- invisible allocation
- undocumented unsafe escape hatches
- macro systems that defeat tooling
- multiple equivalent spellings for core ideas
- comments that carry critical contracts
- ambiguous natural-language execution
- performance claims without measurement
- features added because another language has them

## Ecosystem Standard

A language is the whole system around the syntax: formatter, diagnostics,
tests, package manager, docs, debugger, profiler, build system, standard
library, and compatibility story.

Hum must design those pieces as part of the language, not as afterthoughts.

See [SYSTEMS_LANGUAGE_AUDIT.md](SYSTEMS_LANGUAGE_AUDIT.md).

## Definition Of Progress

Hum progresses when it makes a promise mechanically stronger.

Examples:

- A comment becomes a checked block.
- A checked block becomes a diagnostic.
- A diagnostic becomes a generated test.
- A generated test becomes a proof obligation.
- A proof obligation becomes an optimization permission.
- An optimization permission becomes measured performance.

This is how Hum earns trust.
# Language Pain Sweep 2026

Date: 2026-07-06

## Purpose

This is a research sweep of what developers love and hate about major languages,
with a bias toward systems programming, tooling, package ecosystems, and agent-era
software engineering.

The point is not to dunk on good languages. The best languages are loved because
they removed a major tax. They are hated where they introduced a new one.

Hum should study both.

## Core Finding

Developers rarely hate a language because of one syntax choice.

They hate recurring taxes:

- slow feedback loops
- invisible allocation, mutation, IO, failure, or control flow
- package and environment chaos
- weak or confusing diagnostics
- unsafe escape hatches that look too casual
- cleverness that only the author can read
- ecosystem churn without a migration path
- performance cliffs hidden behind pleasant syntax
- build systems that become their own language
- concurrency models that are easy to start and hard to reason about

Hum's job is not to be every language at once.

Hum's job is to keep the good parts and make the taxes visible, checkable, and
teachable.

## Survey Signal

Stack Overflow's 2025 survey is a useful reality check: Rust is still the most
admired programming language, and Zig is also highly admired, but their usage is
far lower than Python, JavaScript, TypeScript, C++, and Go.

Interpretation:

- Rust and Zig prove systems programmers want better foundations.
- Python proves readability and approachability win enormous mindshare.
- TypeScript proves tooling can pull a chaotic ecosystem toward structure.
- C++ proves power and installed base can survive decades of pain.
- Go proves boring tooling and fast builds are product features.

Hum should not chase popularity first. It should chase the reason people stay.

## Rust

### What People Love

- memory safety without garbage collection
- ownership and borrowing as a serious alternative to C++
- Cargo as a first-party build/package experience
- strong enum, pattern matching, and `Result` culture
- performance close enough to C++ for many systems workloads
- a community that cares about correctness

### What People Hate

- compile times
- ownership learning cliff
- trait and lifetime errors that can feel like compiler negotiations
- macro expansion hiding generated code and compile-time cost
- async/executor/cancellation complexity
- unsafe obligations still living partly in prose
- FFI boundaries where Rust's guarantees become harder to audit

### Hum Response

- keep Rust's safety soul: ownership, borrowing, moves, explicit mutation
- make ownership words beginner-readable: `owned`, `borrow`, `change`, `shared`
- make `hum check` fast and separate from optimized builds
- make macro-like features wait until the semantic graph can expose them
- make unsafe review packets syntax, not comments
- make async wait for effect, cancellation, allocation, and scheduler clarity
- make FFI a `foreign` trust boundary with layout and ownership contracts

### Brutal Rule

Do not advertise "better Rust" until Hum has a better explanation for every
place Rust currently has a hard but correct rule.

## Zig

### What People Love

- no hidden allocation
- simple language core compared with C++
- explicit allocators
- excellent cross-compilation instincts
- C interop and `zig cc`
- `comptime` as a powerful replacement for macros/templates/preprocessors
- build modes that make safety/performance tradeoffs explicit

### What People Hate

- manual memory burden for everyday code
- young ecosystem and pre-1.0 churn
- build/package story still maturing
- `comptime` can become a second language if overused
- low-level pointer/cast details are visible often
- C translation and ABI work still have edge cases
- safety depends heavily on build mode and programmer discipline

### Hum Response

- keep "no hidden allocation" as `allocates:` and `cost:` promises
- use Rust-like safety as the default instead of manual memory by default
- expose allocator/arena choice at important boundaries without making every API noisy
- make Nectar declarative first; build scripting comes later and under budgets
- allow compile-time execution only with cost, graph, and termination discipline
- make C interop explicit, audited, and tested

### Brutal Rule

Zig is a warning that transparency alone is not enough. Humans also need the
compiler to carry routine safety work.

## C++

### What People Love

- raw power and control
- enormous ecosystem and industry adoption
- zero-overhead abstraction when used well
- deep access to hardware and ABIs
- long-lived code and toolchain availability

### What People Hate

- undefined behavior as a normal risk surface
- headers, templates, and slow rebuilds
- too many ways to express the same thing
- fragmented build/package ecosystem
- complex generics and diagnostics
- unsafe defaults and pointer/lifetime traps
- style fragmentation across teams and eras
- backwards compatibility freezing old mistakes

### Hum Response

- no headers
- no casual undefined behavior
- one obvious spelling for common operations
- default private modules and explicit visibility
- `humfmt`, `chirp`, Nectar, LSP, debugger, profiler as first-party pillars
- unsafe exists only in small audited scopes
- generic power must be explainable in the semantic graph

### Brutal Rule

C++ proves that power without a small safe subset becomes culture-dependent.
Hum must make the safe subset the language, not a guideline PDF.

## Python

### What People Love

- readable syntax
- fast first program
- enormous libraries
- interactive workflow
- approachable mental model
- excellent fit for data, scripting, automation, and AI

### What People Hate

- packaging and environment fragmentation
- runtime type errors in large systems
- performance cliffs
- version fragmentation
- native extension and binary dependency pain
- supply-chain risk through huge dependency graphs
- concurrency limits and deployment surprises

### Hum Response

- keep Python's readability ambition, but not dynamic uncertainty
- static types by default
- Nectar owns packages, lockfiles, tools, and build profiles
- dependency provenance and trust are visible metadata
- performance contracts are source-level promises
- no hidden dynamic fallback in systems code
- interactive/prototype modes can exist, but production Hum is checked Hum

### Brutal Rule

Python friendliness is real. Hum should learn from it. But systems code cannot
pay with runtime surprises.

## Go

### What People Love

- fast builds
- simple syntax
- strong standard tooling
- formatting culture
- easy deployment
- concurrency that is easy to start using
- boring production ergonomics

### What People Hate

- repetitive error handling
- `nil` mistakes
- GC/runtime assumptions limiting low-level use
- omitted abstraction tools causing workaround patterns
- `unsafe` and `cgo` becoming sharp edges
- map concurrency behavior that requires discipline
- generics arriving late and still feeling constrained in places

### Hum Response

- treat fast builds as language design
- keep formatting and package tooling first-party
- typed `Result` plus `fails when:` instead of exception or `nil` conventions
- no ordinary null
- include sum types/variants early
- no mandatory GC for systems code
- make C interop explicit and contract-heavy
- concurrency waits for effect and memory-model clarity

### Brutal Rule

Go proves that boring can win. Hum should be exciting in capability, not in daily
friction.

## JavaScript And TypeScript

### What People Love

- ubiquity
- instant feedback
- web platform reach
- flexible object and JSON-shaped programming
- TypeScript editor tooling
- ability to gradually add types to existing code

### What People Hate

- package ecosystem chaos
- dependency and build-tool churn
- semantic gap between TypeScript source and JavaScript runtime
- erased/unsound types
- config sprawl
- many authoritative tools disagreeing about the project
- runtime surprises from historical JavaScript behavior

### Hum Response

- compiler is the source of truth
- semantic graph comes from the compiler, not tool inference
- Nectar owns package/build metadata
- type/effect/safety claims are not erased decoration
- escape hatches are explicit unsafe or foreign boundaries
- configuration starts small and typed

### Brutal Rule

Do not let Hum's toolchain become a stack of translators that each understand a
different language.

## Java And C#

### What People Love

- mature tooling
- excellent IDE support
- stable ecosystems
- managed memory for application code
- strong enterprise libraries
- runtime introspection and deployment conventions

### What People Hate

- null as ambient debt
- framework and annotation sprawl
- GC and runtime assumptions where low-level control matters
- type erasure and boxed identity/layout costs
- checked exceptions as paperwork
- verbose patterns around effects, dependency injection, and serialization

### Hum Response

- no ordinary null
- effects are language-level promises, not annotation culture
- value records and explicit identity from the start
- generics preserve enough layout/type information for optimization
- typed `Result` and `fails when:` without exception signature sprawl
- stdlib stays intentional instead of becoming a framework maze

### Brutal Rule

Enterprise structure is useful. Enterprise ceremony is not.

## Swift, Kotlin, And Modern App Languages

### What People Love

- expressive syntax
- strong IDE support
- approachable type systems
- optionals/null-safety direction
- good standard libraries for app work
- polished high-level APIs

### What People Hate

- source churn and migration cost
- platform coupling
- compile-time surprises from inference/generics
- opaque framework behavior
- concurrency model transitions that take years to feel obvious

### Hum Response

- use editions sparingly
- keep platform portability early
- bound inference
- explain concurrency and effects before stabilizing syntax
- make generated docs/contracts show what framework-like APIs actually do

### Brutal Rule

Nice surface syntax does not excuse invisible machinery.

## Verse

Verse is worth studying because it is not just another C-like systems language.
It explores functional logic programming ideas for a large creative platform.

### Ideas Worth Stealing Carefully

- failure as a first-class control concept
- expressions that can succeed or fail, not only return booleans
- logic variables and equality constraints as a way to describe desired results
- `one` / `all` style result selection
- a formal core calculus with rewrite semantics
- deterministic handling of choice through spatial choice
- language design aimed at creators, not only professional engineers

### What Hum Should Not Copy Blindly

- hidden search or backtracking in systems hot paths
- logic-programming semantics that make cost hard to see
- platform-specific language assumptions
- young-language churn before toolchain maturity
- magical control flow that agents cannot explain from the semantic graph

### Hum Response

- study Verse-style failure contexts for tests, queries, pattern matching, and `proves:`
- keep failure, branching, and search cost visible in `cost:` and semantic graph output
- consider `one` / `all`-like concepts only for bounded query/test/proof contexts
- keep the executable core deterministic unless nondeterminism is explicit
- build a formal Hum IR before native backend work

### Brutal Rule

Verse hints at a future where code states what must be true and the language
helps find it. Hum should borrow that ambition, but systems code still needs
visible cost, memory, effects, and trust.

## Cross-Language Pain Patterns Hum Must Fix

### 1. Feedback Latency

Rust and C++ show that powerful abstractions can make feedback slow.

Hum rule:

```text
fast check first, optimized build second
```

### 2. Package And Build Friction

Python, C++, JavaScript, and C all show that language plus third-party build
chaos becomes ecosystem debt.

Hum rule:

```text
Nectar is part of the language experience
```

### 3. Unsafe Escape Hatches

C++, Rust, Go, Zig, and FFI-heavy Python all show that unsafe boundaries become
where guarantees go to be renegotiated.

Hum rule:

```text
unsafe and foreign require contracts, tests, trust notes, and small scopes
```

### 4. Hidden Runtime Cost

Python, Java, C#, JavaScript, and high-level frameworks show the pain of hidden
allocation, reflection, dispatch, dependency injection, and runtime behavior.

Hum rule:

```text
allocation, IO, mutation, failure, and scheduling are visible promises
```

### 5. Clever Abstraction

C++ templates, Rust trait gymnastics, Haskell/Scala encodings, Zig `comptime`,
and TypeScript type-level programming all show the same risk: library authors can
write a private language inside the language.

Hum rule:

```text
if the semantic graph cannot explain it, the feature is not stable
```

### 6. Concurrency Ambiguity

Go makes concurrency easy to start. Rust makes some concurrency safe but async
can be hard to reason about. C++ exposes too many low-level details. Python hides
limits until scale.

Hum rule:

```text
concurrency syntax waits for effects, cancellation, scheduling, and memory model
```

### 7. Migration Pain

Swift, Python, JavaScript, C++, and TypeScript show that evolution without a clear
migration story becomes a permanent tax.

Hum rule:

```text
editions are rare, migrations are tool-assisted, and old decisions have explicit exits
```

## New Hum Design Gates From This Sweep

Every major feature must answer:

1. Which existing language pain does it remove?
2. Which existing language pain could it recreate?
3. Is the cost visible in source?
4. Is the behavior visible in the semantic graph?
5. Can `humfmt` keep it readable?
6. Can `chirp` catch misuse?
7. Can `hum lsp` explain it interactively?
8. Can Nectar build, cache, and reproduce it?
9. Can a beginner read the happy path?
10. Can a systems engineer see allocation, failure, effects, and trust?
11. Can an agent repair it without guessing?

If not, the feature stays experimental.

## What This Changes For Hum

The next priorities become even clearer:

1. Finish Milestone 0 test skeletons and richer semantic graph lines.
2. Write `docs/PACKAGE_AND_BUILD.md` for Nectar.
3. Write `docs/UNSAFE_POLICY.md` and `docs/FFI_AND_ABI.md` before unsafe syntax.
4. Keep `hum check` fast and boring.
5. Delay macros, async, and compile-time execution until the graph/tooling story is strong.
6. Study Verse in a bounded way for tests, proofs, queries, and failure contexts.

## Bottom Line

Hum should feel like Python to read, Rust to trust, Zig to inspect, Go to build,
C++ to reach hardware, TypeScript to tool, and Verse to express intent.

But it must not inherit their taxes by accident.

The only way to do that is to make every powerful feature pass through checked
intent, stable diagnostics, semantic graph output, and boring first-party tools.

## Sources

- Stack Overflow Developer Survey 2025, Technology: https://survey.stackoverflow.co/2025/technology
- Rust Performance Book, Compile Times: https://nnethercote.github.io/perf-book/compile-times.html
- The Rust Programming Language, Ownership: https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html
- Rust Reference, Behavior considered undefined: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
- Zig Language Reference: https://ziglang.org/documentation/master/
- Go FAQ: https://go.dev/doc/faq
- Go blog, Errors are values: https://go.dev/blog/errors-are-values
- C++ Core Guidelines: https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines
- TypeScript Design Goals: https://github.com/microsoft/TypeScript/wiki/TypeScript-Design-Goals
- TypeScript Type Compatibility: https://www.typescriptlang.org/docs/handbook/type-compatibility.html
- PyPitfall, dependency chaos in Python: https://arxiv.org/abs/2507.18075
- PyTy, repairing Python type errors: https://arxiv.org/abs/2401.06619
- Verse Calculus paper: https://simon.peytonjones.org/assets/pdfs/verse-conf.pdf
- Epic documentation, Programming with Verse: https://dev.epicgames.com/documentation/en-us/uefn/programming-with-verse-in-unreal-editor-for-fortnite
# External Advice Review

Date: 2026-07-06

## Purpose

This document turns outside advice about competing with C++ and Rust into Hum
engineering commitments.

The advice is useful because it names the classic systems-language cliffs:
memory management, LLVM/backend work, self-hosting timing, C interop, compiler
front-end design, and compile-time pressure.

It is incomplete because it treats language design mostly as syntax plus LLVM.
Hum's real bet is larger: checked intent, explicit effects, cost promises,
security contracts, semantic tooling, and agent-readable compiler facts.

## Verdict

The advice helps, but it should not steer Hum into building a generic Rust/Zig
clone.

It confirms these decisions:

- keep the bootstrap compiler in Rust for a long time
- do not self-host early
- do not build machine-code backends before the front end is trustworthy
- treat C FFI as a survival requirement
- treat memory safety as the core design decision
- treat compile time as a first-class product feature

It does not answer Hum's hard questions:

- how source code carries intent without becoming prose
- how the compiler checks promises instead of merely parsing comments
- how agents consume structured meaning instead of guessing from text
- how cost, allocation, effects, and security claims become enforceable
- how beginners read systems code without hiding systems truth

## What The Advice Gets Right

### Memory Management Is Existential

Hum cannot compete with Rust by being casual about memory.

Decision:

- Hum defaults to Rust-style ownership, borrowing, moves, and exclusive mutation.
- Hum adds clearer source-level intent with `uses:`, `changes:`, `needs:`,
  `ensures:`, `protects:`, and `cost:`.
- Hum should support explicit regions, arenas, and allocator parameters where
  they make systems code clearer.
- Reference counting may exist as a library type, but it is not the default
  memory model.
- Garbage collection is not the systems baseline.

Hard rule:

Hum does not get to say "safer than Rust" until the ownership/effect checker can
reject real memory, aliasing, lifetime, and data-race bugs.

### LLVM Is Useful, But Not The Language

LLVM is a strong backend path, especially for optimized native code, object
files, debug information, and many target architectures.

Decision:

- Hum should eventually lower to LLVM for optimized AOT builds.
- Hum may use Cranelift or an interpreter earlier for fast feedback and a simpler
  executable core.
- Hum IR must come before LLVM IR so intent, effects, contracts, source spans,
  and semantic graph identity are not thrown away too early.

Hard rule:

Do not start by writing LLVM codegen. Start by proving the front end can parse,
validate, explain, and serialize meaning.

### Self-Hosting Is A Trap If Done For Ego

A self-hosted compiler is a milestone, not proof of a good language.

Decision:

- Rust remains the stage0 compiler implementation language.
- Hum starts self-hosting only with leaf tools and non-critical compiler pieces.
- The parser, checker, borrow/effect checker, optimizer, and backend stay in Rust
  until Hum versions are simpler, tested, fuzzed, and differentially checked.

Hard rule:

Self-hosting cannot slow down diagnostics, safety work, or beginner tooling.

### C Interop Is A Survival Requirement

Systems programmers live near C ABIs, OS APIs, drivers, runtimes, databases,
graphics libraries, and embedded SDKs.

Decision:

- Hum needs explicit `foreign` blocks.
- Foreign calls must declare ABI, layout, ownership transfer, failure behavior,
  trust boundary, and safety obligations.
- The first stable foreign target should be C. C++ interop is a separate layer
  because C++ ABI, templates, exceptions, and ownership conventions are much
  messier.

Hard rule:

No foreign function is "just a call". It is a trust boundary.

### Parsing Strategy Should Serve Diagnostics

Parser generators can help, but Hum's first parser should optimize for clear
errors, spans, recovery, and semantic graph construction.

Decision:

- The Rust bootstrap can keep a hand-written parser while syntax is fluid.
- Tree-sitter should exist for editors and incremental syntax trees.
- The compiler parser remains authoritative for semantics.

Hard rule:

If a syntax feature makes error recovery, highlighting, LSP, or agent repair bad,
it does not stabilize.

## What The Advice Misses

### Diagnostics Are Product Surface

A language for beginners and systems engineers lives or dies on errors.

Hum diagnostics need stable codes, blame blocks, suggested fixes, JSON output,
LSP mapping, related contracts, and examples.

### Package And Build Design Is Language Design

A Rust competitor without a Cargo-class experience is not a Rust competitor.

Nectar must be boring, reproducible, fast, and security-aware from the start.

### Tooling Must Be Designed Before Syntax Freezes

Syntax highlighting, semantic tokens, formatting, LSP, debugger source maps,
profiler links, and agent repair all need stable node identities.

Hum should reject clever syntax that tools cannot explain.

### The Standard Library Is A Research Program

Hum's standard library should not be a pile of wrappers.

It needs benchmarked containers, allocation visibility, adversarial tests,
property tests, fuzzing, cache-aware layouts, and clear cost contracts.

### Security Needs A Threat Model

Memory safety is not all of security.

Hum also needs secret types, capability boundaries, constant-time claims, package
trust, unsafe review rules, FFI auditability, sanitizer hooks, and supply-chain
policy.

### Adoption Requires Migration Stories

Hum cannot win by being theoretically better.

It needs:

- C interop
- Rust interop plan
- great docs
- small examples
- editor support
- build system stability
- real benchmark honesty
- a few killer libraries or domains

## Answers To The Advice's Questions

### Which Memory Strategy?

Hybrid, but with a clear default:

```text
owned values by default
borrow for shared read access
change borrow for exclusive mutation
regions and arenas when declared
reference counting as an explicit type
unsafe only inside reviewed, contract-heavy boundaries
```

Hum should feel less mysterious than Rust, but it should not throw away Rust's
core safety lesson.

### What Fatal Flaw Does Hum Fix?

Hum's target flaw is not one syntax wart.

The target flaw is that most languages hide senior-engineer reasoning in
comments, naming conventions, review culture, tests, benchmarks, and tribal
knowledge.

Hum moves that reasoning into checked source:

```text
why:
uses:
changes:
needs:
ensures:
protects:
watch for:
cost:
tests:
does:
```

That is the wedge. If this does not work, Hum is just another syntax experiment.

### Compile Speed Or Runtime Optimization?

Both, but in different modes.

Default developer loop:

- parse fast
- type/effect check fast
- emit semantic graph fast
- run targeted tests fast
- keep editor feedback under strict budgets

Release loop:

- optimize deeply
- use profile-guided data where available
- prove or benchmark declared cost claims
- emit full debug/profiling/source-map metadata

Hard rule:

Hum should never make every edit pay for every possible optimization.

## Risk Register

### Risk: We Invent Too Much Syntax Before Semantics

Mitigation:

Milestone 0 stays focused on parsing, diagnostics, semantic graph, and intent
checks.

### Risk: We Promise More Than The Compiler Can Enforce

Mitigation:

Every promise has a level:

```text
noted
warned
checked
proved
benchmarked
```

Docs must not blur those levels.

### Risk: Hum Becomes Rust With Sentences

Mitigation:

Every feature must improve at least one of:

- human readability
- machine-checkable intent
- agent repair accuracy
- safety clarity
- performance visibility
- tooling quality

### Risk: FFI And Unsafe Become Escape Hatches

Mitigation:

Foreign and unsafe blocks require contracts, proofs or tests, trust notes, and
small scopes.

### Risk: Compile Times Become Rust's Biggest Complaint Again

Mitigation:

Semantic graph caching, incremental checks, feature budgets, no hidden macro
explosions, and separate fast-check versus optimized-build modes.

### Risk: Hum Optimizes For The Founder Instead Of Users

Mitigation:

Beginner examples, systems examples, Rust/C++ migration examples, editor tests,
error-message tests, and usability reviews must be release gates.

## Design Commitments Added From This Review

1. `docs/DIAGNOSTICS.md` must be written before more syntax expansion.
2. `docs/FFI_AND_ABI.md` must define C interop before native backend work.
3. `docs/UNSAFE_POLICY.md` must exist before any unsafe syntax stabilizes.
4. `docs/PACKAGE_AND_BUILD.md` must make Nectar a first-class language pillar.
5. `docs/LSP_AND_DEBUG_PROTOCOLS.md` must define stable node identities before
   syntax freezes.
6. The Rust bootstrap remains the trusted compiler until Hum earns self-hosting.
7. LLVM is an optimized backend target, not the starting point.

## Bottom Line

The advice is a good warning label.

Hum's response is not to get smaller in ambition. It is to get stricter about
proof.

A serious Hum claim must always answer:

```text
What does the human read?
What does the compiler check?
What does the toolchain expose?
What can the agent repair?
What can the machine prove or measure?
```
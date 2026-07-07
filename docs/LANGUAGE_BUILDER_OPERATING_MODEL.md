# Hum Language Builder Operating Model

Date: 2026-07-07
Source status: distilled from a user-provided Chris Lattner interview transcript; raw transcript is not committed.

## Purpose

This document captures the operating model Hum should learn from Chris
Lattner's work on LLVM, Clang, Swift, MLIR, and Mojo.

The companion research note captures language and compiler lessons. This file
captures the builder method: how to select problems, prototype, persuade,
migrate users, validate hard claims, and keep a language coherent while it grows.

This is not hero worship and not a plan to copy Swift, Mojo, Apple, or Modular.
It is a set of working habits Hum can use to become more serious.

## Core Pattern

The repeated pattern is:

```text
fall in love with a real problem
-> build privately or narrowly enough to understand it
-> prove the hard hypothesis with a concrete artifact
-> explain the value in the old world's terms
-> make adoption incremental
-> add tooling, docs, debugger/editor/API polish
-> scale only after the core can carry more people
```

Hum should treat that as a build discipline.

## Mental Model

The transcript suggests a recurring decision frame:

1. What platform shift makes the old answer insufficient?
2. What old ecosystem or expert community must be honored rather than erased?
3. What hard technical hypothesis must be proved before the claim is real?
4. What can be deliberately left out until the core gets mileage?
5. What lower-level mechanism can remove many surface special cases?
6. What artifact can skeptics inspect instead of trusting taste?
7. What migration path lets users adopt one boundary at a time?
8. What layer makes deep machinery usable by normal programmers?

Hum should use these questions when it feels tempted to add syntax, make a
performance claim, copy another language, or widen the roadmap.

## Third-Pass Tacit Wisdom

These lessons are easy to miss because they are not feature advice. They are
judgment advice.

### Success Is A Stress Test

Fast adoption can make a language worse if the core is not ready. Product
pressure, framework deadlines, and demo needs can push local syntax and special
cases into permanent language shape.

Hum rule: when a feature attracts attention, spend more energy simplifying the
core, compiler, docs, diagnostics, and migration path before adding sugar around
that attention.

### Credibility Is Earned Before Persuasion

Bold technical proposals land better after the builder has shipped useful work.
Credibility is not a substitute for evidence, but it buys enough attention for
evidence to be heard.

Hum rule: earn the right to make ambitious claims through working commands,
fixtures, reports, CI, examples, and honest release notes.

### Adoption Has A Status Model

Experts may resist because a new system threatens their accumulated mastery. A
technical explanation that ignores status, identity, and transfer of expertise
will miss the real objection.

Hum rule: migration docs should show existing experts how their knowledge becomes
more powerful in Hum. Do not present them as obsolete.

### Familiarity Sometimes Beats Taste

A migration-focused language or bridge sometimes has to preserve syntax,
terminology, or behavior that the designer would not choose from scratch. This is
not weakness if it lowers adoption risk at the boundary.

Hum rule: inside native Hum, choose the clean paved road. At interop and
migration boundaries, preserve enough old-world shape that users can move safely.

### Do Not Own What A Stable Boundary Can Use

Mojo's Python story highlights a discipline: use stable public boundaries where
they exist instead of trying to own or replace the whole upstream ecosystem at
once.

Hum rule: prefer stable C ABI, process, Wasm, interpreter, file-format, and
protocol boundaries before promising deep native integration.

### The Base Operation Must Carry The Future

Mojo's emphasis on function calls is a substrate lesson. If the fundamental
operation is powerful and regular enough, many advanced features compose through
it instead of becoming special cases.

Hum rule: `task` invocation, typed failure, effects, ownership, source spans,
contracts, and graph facts are Hum's substrate. Do not add features that bypass
that substrate.

### Wedge Humility Protects Ambition

Go's strongest use became cloud/platform work even if early positioning sounded
more systems-general. A language can keep large ambition while being honest about
where it first wins.

Hum rule: Hum can aim at Rust/C++ scale eventually while first proving itself in
evidence-native offline tools, security utilities, and reviewable automation.

### Hype Must Be Separated From Infrastructure

AI, accelerator, and breakthrough-theory conversations include noise, but the
underlying infrastructure problems can still be real. The skill is extracting the
durable systems problem without swallowing the hype.

Hum rule: every exciting research or market signal becomes a hypothesis, not a
slogan, until Hum can turn it into a proof, benchmark, diagnostic, or profile.

### If Only Experts Understand It, Product Work Remains

A powerful compiler or verifier that only a few people can use is not yet a
language product. The next bottleneck may be syntax, explanation, diagnostics,
or tooling rather than deeper machinery.

Hum rule: after hard internal proof, build the layer that lets ordinary users and
agents benefit from the machinery.

### Platform Shifts Are The Right Time To Be Brave

LLVM, Swift, MLIR, and Mojo each connect to a platform shift: compiler
infrastructure, mobile apps, heterogeneous hardware, and AI/accelerator stacks.
Big language moves need a reason the old equilibrium is no longer enough.

Hum rule: tie major bets to real shifts: supply-chain evidence, agent-readable
code, safety regulation, heterogeneous hardware, local-first security, and
software that must explain its own behavior.

### The Right Answer May Improve Competitors

LLVM made other languages faster. A deep infrastructure contribution can help an
ecosystem even when those languages compete for users.

Hum rule: some Hum tools should be useful even before people write all-Hum code:
reports, wrappers, validators, migration aids, and evidence bundles can improve
mixed systems.

### Do Not Celebrate Running Code Too Early

A compiler that emits code proves only that code can run. It does not prove the
language is understandable, safe, portable, optimizable, debuggable, or worth
adopting.

Hum rule: every execution milestone must preserve the semantic graph, diagnostic
quality, evidence story, and user-facing explanation.

## Builder Principles

### 1. Enthusiasm Is Infrastructure

Long projects need more than rational justification. They need a problem that is
important enough to matter and interesting enough to keep the builder engaged
through years of uncertainty.

Hum rule: do not spend months on a feature only because it seems strategically
useful. It must also teach the project something and make the next proof easier.

### 2. Build To Understand

The transcript repeatedly returns to the idea that some ideas become clear only
by building them. Early prototypes are not public promises. They are instruments
for understanding.

Hum rule: unclear core ideas should become small runnable or report-emitting
prototypes before they become broad governance debates. For Milestone 0 this
means source fixtures, semantic graph facts, diagnostics, reports, and local
checks before ambitious syntax.

### 3. Build At The Leverage Layer

LLVM's long tail shows that one compiler-infrastructure improvement can benefit
many languages and ecosystems for decades. The leverage layer is often below the
surface syntax: IR, diagnostics, optimizer facts, runtime interfaces, debug info,
package metadata, or editor protocols.

Hum rule: prefer improvements that compound through the semantic graph, Hum IR,
reports, schemas, and tooling instead of one-off surface wins.

### 4. The Hard Part Is Usually Not The Syntax

A language launch requires much more than a parser and pretty source:

docs, examples, editor support, debugger path, API shape, migration tooling,
compatibility story, package/build story, diagnostics, and evidence.

Hum rule: public alpha is a product gate, not a parser milestone.

### 5. Adoption Preserves Existing Expertise

Resistance to a new language is often about identity, sunk knowledge, and fear
that existing expertise becomes worthless. The technical answer is not enough.
The adoption answer must show how old expertise transfers.

Hum rule: explain Hum to Rust, C, Python, ops, security, and enterprise users in
terms of what they already know. Make them more capable, not obsolete.

### 6. Pull The Old World Forward

Swift did not only replace Objective-C; it improved Objective-C enough to make
the bridge work. Mojo does not start by replacing all Python packages; it uses
stable interpreter boundaries and lets code move gradually.

Hum rule: interop is not a compromise afterthought. Safe wrappers, process
boundaries, C ABI paths, Wasm, Python/Rust bridges, and migrators are adoption
architecture.

### 7. Honest Instability Is A Release Valve

Early Swift could break source because the breakage was explicit. Users could
choose the new path without believing they had received a false stability
promise.

Hum rule: pre-alpha and alpha docs must say what can break. Breaking changes
need versions, release notes, and migration paths before Hum asks for trust.

### 8. Smaller Coherent Groups Can Discover Faster

Large consensus too early can turn every unknown into a meeting. A small group
can build enough shape that the larger group has something concrete to evaluate.

Hum rule: use BDFL direction, focused slices, and written design records. Do not
copy secrecy as policy, but do protect early discovery from premature process.

### 9. Write To Scale The Conversation

One-on-one persuasion does not scale. Concerns become useful when they are
written down and answered in a form other people can inspect.

Hum rule: major concerns become docs, decision records, schemas, fixtures, or
checks. Chat memory is not ground truth.

### 10. Progressive Disclosure Is Also For Experts

Simple surface area is not only beginner kindness. It reduces the mental load of
ordinary expert work.

Hum rule: advanced power must be reachable, but normal tasks should not require
users to understand ownership, FFI, profiles, comptime, backend details, or proof
tools before they can read the program.

### 11. Feature Absence Can Be A Strength

Go delayed generics until the design had enough mileage. Mojo delayed classes
while focusing on the lower-level model. A missing feature can protect the core.

Hum rule: the default answer to major features is wait until the core can explain
it through semantics, diagnostics, graph facts, tools, profile policy, tests,
benchmarks, and docs.

### 12. General Mechanisms Beat Local Sugar

Swift's special-case accretion is the warning. Local syntax can make one demo
look good while making the compiler, tooling, and language harder forever.

Hum rule: reject feature-specific sugar until a general mechanism exists and can
preserve formatting, diagnostics, semantic graph facts, source spans, timings,
and profile restrictions.

### 13. Modern Hardware Invalidates Old Folklore

Old algorithm lessons can mislead when memory layout, cache locality,
indirection, allocation, branch behavior, SIMD, NUMA, and accelerators dominate.

Hum rule: stdlib and optimization choices need hardware-aware evidence, not only
Big-O arguments.

### 14. Value Semantics Are The Systems-Friendly Middle

Strict pure-functional copying often fights the machine. Raw mutation fights
reasoning. Value semantics plus ownership can keep logical composition while
allowing efficient in-place updates when uniqueness is known.

Hum rule: favor value-oriented defaults, explicit ownership, visible mutation,
and profile-aware strictness. Persistent or boxed structures should be explicit
side roads.

### 15. Dynamic-Feeling Code And Strict Code Can Coexist

A strong language can let ordinary code stay light while hot paths, embedded
code, and safety-critical profiles opt into stricter control.

Hum rule: progressive strictness should be profile-backed and visible. Do not
make every user pay the full systems tax, and do not hide the tax from systems
users who need control.

### 16. Survey Before Inventing

The method is not to reimplement the last familiar thing. It is to study old and
new systems, borrow sharp ideas, and understand the tradeoffs before hardening a
feature.

Hum rule: significant language, stdlib, backend, and tooling features need a
research sweep or design note before they stabilize.

### 17. Validate The Hard Hypothesis First

The Modular/Mojo story is not syntax-first. The hard compiler hypothesis was
validated against serious vendor baselines before the language surface became
the main product story.

Hum rule: before any big performance or backend claim, build the smallest kernel
that tests the actual impossible thing, compare against a serious baseline, and
record the result with target details.

### 18. If Only A Few People Understand It, Build A Layer

Powerful compiler internals are not enough if almost nobody can use them. Syntax,
APIs, docs, diagnostics, and tools turn deep machinery into a usable platform.

Hum rule: any advanced engine needs a Hum-facing explanation layer and graph
facts before it becomes product direction.

### 19. Use Good Parts, Not Whole Systems

LLVM, MLIR, Python, Swift, Zig, Rust, Go, and C++ all contain useful lessons.
None should become Hum's identity.

Hum rule: borrow mechanisms by reason, not by brand. Hum IR and the semantic
graph remain the center.

### 20. Blank-Page Work Needs Scaffolding

Excellent engineers can be strong at mature-codebase hill climbing and still be
paralyzed by greenfield invention.

Hum rule: contributors need small vertical slices, fixtures, schemas, commands,
roadmap gates, and examples. Do not ask people to invent from fog.

### 21. Handoff Is Part Of Invention

A founder can catalyze an idea, but mature subsystems need maintainers, tests,
and written ownership so the founder can move to the next unknown.

Hum rule: every maturing subsystem should grow a maintainer path, evidence gate,
and regression suite. The project should not depend on one mind remembering why
everything exists.

## Operating Loops

### Prototype Loop

Use for unclear ideas:

1. State the smallest question.
2. Build the smallest local artifact that can answer it.
3. Add a fixture or report so the answer is inspectable.
4. Write what changed in the design.
5. Decide: continue, narrow, defer, or reject.

### Adoption Loop

Use before public-facing features:

1. Name the old ecosystem or workflow being improved.
2. Preserve the user's existing expertise where possible.
3. Provide a one-boundary adoption path.
4. Provide diagnostics and docs in the user's vocabulary.
5. Provide a migration story for intentional breaks.

### Feature Gate Loop

Use before accepting major syntax or stdlib surface:

1. Can ordinary users ignore this until they need it?
2. Is there one general mechanism instead of local sugar?
3. Can the semantic graph represent it?
4. Can `humfmt`, `chirp`, LSP, debugger, profiler, and agents explain it?
5. Does a profile need to forbid or require it?
6. What prior systems were studied?
7. What migration path exists if the design changes?

### Performance Claim Loop

Use before speed claims:

1. What exact workload is the claim about?
2. What is the serious baseline?
3. What hardware, OS, compiler, profile, and inputs were used?
4. What resource report and benchmark evidence exists?
5. What workloads lose?
6. What fallback path exists?
7. What claim is still forbidden?

### Handoff Loop

Use when a subsystem matures:

1. Write the architecture note.
2. Add command or schema surfaces.
3. Add fixtures and regression checks.
4. Document ownership and open risks.
5. Let someone else maintain it without needing the original chat context.

## Continue, Narrow, Defer, Or Reject

Use this when deciding whether to keep pushing on an ambitious direction:

- Continue when the hard hypothesis is validated and the next bottleneck is
  usability, syntax, docs, tooling, or adoption.
- Narrow when one slice is real but the broad story is still too vague.
- Defer when the feature is implementable but cannot yet be taught, migrated,
  represented in graph facts, or governed by profiles.
- Reject when the idea creates a rewrite cliff, weakens the core, depends on
  backend magic, or cannot name the evidence that would make it true.

A beautiful idea that cannot pass through this gate belongs in research, not in
stable Hum.

## Questions Before Hum Copies Any Idea

Before copying a feature from any language or compiler project, answer:

1. What user pain does this solve in Hum's mission?
2. What charter or tradeoff does this serve?
3. Which existing expertise does it preserve?
4. Does it make the beginner path smaller or larger?
5. Does it create a rewrite cliff?
6. Can it be adopted one module, boundary, tool, or hot path at a time?
7. What leverage layer should carry it: syntax, stdlib, IR, runtime, package metadata, or tool protocol?
8. What graph facts does it require?
9. What diagnostics does it enable?
10. What docs teach it without glamour?
11. What profile gates does it need?
12. What benchmarks or proofs would make the claim honest?
13. What simpler mechanism could subsume it later?
14. What would make us reject or defer it?

## Anti-Patterns

Hum should avoid:

- syntax added because one example looks better
- public alpha branding without executable artifacts and tooling
- broad consensus before a prototype exists
- private taste without written reasoning
- compatibility promises before mileage
- all-or-nothing rewrites
- performance claims without baselines
- backend worship
- copying a language's surface while missing its adoption machinery
- making old experts feel discarded
- asking contributors to start from a blank page

## Practical Consequence

Hum should feel ambitious, but the work should stay humble:

```text
small proof -> written lesson -> graph/report/check -> migration path -> public claim
```

The next hard proof should be executable truth without losing Hum's unique
surface: one tiny program lowered through a formal core, run locally, and still
connected to diagnostics, effects, resource claims, test obligations, and graph
facts. That proof matters more than adding another attractive language feature.

That is the part of the Lattner method Hum most needs to carry forward.

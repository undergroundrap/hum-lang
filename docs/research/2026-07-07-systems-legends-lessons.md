# Systems Legends Lessons For Hum

Date: 2026-07-07

Status: distilled research note

Source status: user-provided candidate list, checked against primary or
near-primary sources where practical.

## Executive Thesis

Hum should not imitate any one legendary engineer's taste wholesale. The useful
lesson is the overlap among their best work: small cores, stable interfaces,
portable artifacts, source availability, clear writing, measured performance,
and an unwillingness to hide hard tradeoffs behind fashion.

The systems legends disagree in style. Unix minimalism is not TypeScript
pragmatism. Wirth's austerity is not Carmack's hardware-chasing intensity.
Stallman's freedom argument is not Torvalds's maintainer discipline. But their
combined signal is strong: durable systems come from a precise core surrounded by
boring tools that people can inspect, port, compose, and teach.

## Primary Signals

Dennis Ritchie's history of C describes C as a system implementation language
created for early Unix, born on small machines and shaped by implementation
constraints, simple compilers, and portability pressure.

Ken Thompson's "Reflections on Trusting Trust" shows that compiler provenance is
part of security. Source-level checking alone cannot prove trust when the lower
toolchain can learn hidden behavior.

The Linux kernel coding style is blunt but valuable: readable maintainable code
uses simple expressions, short functions, controlled nesting, meaningful names,
and conventions that make bad structure visible.

Git's origin story shows a tool built because existing source-control systems did
not satisfy kernel-scale distributed development needs. It optimized for speed,
integrity, and decentralized work because those were product requirements, not
nice-to-have traits.

TypeScript's design goals show mature language pragmatism: model existing
ecosystems, avoid surprising users, make tooling extensible, and balance
correctness with productivity instead of pretending a single axis wins.

Wirth and Gutknecht's Project Oberon shows the opposite pressure from sprawl:
design a whole system so it can be described and understood, omit nonessential
features, and make compiler, OS, and even hardware regular enough to study.

GNU's manifesto shows infrastructure as a social contract: source availability,
user freedom, and rebuildable tools determine who can repair, teach, and control
the system.

id Software's source releases show the compounding effect of preserving and
publishing real engine code. Carmack and Abrash also show that optimization needs
hardware facts, algorithmic structure, and user-visible mod/tool boundaries.

CP/M and Digital Research show the value of a hardware abstraction boundary.
Kildall's BIOS split let one microcomputer OS reach many machines without
rewriting the whole system for each vendor.

Brian Kernighan's influence is mostly the discipline of explanation: small
programming models, examples, manuals, and books can turn deep systems ideas into
shared culture.

## Hum Lessons By Legend

### Thompson And Ritchie: Mechanism Before Myth

Unix and C were not successful because they were large or perfect. They were
small enough to rebuild, close enough to hardware to be useful, and abstract
enough to move.

Hum consequence:

- Keep Core Hum small and executable before widening the language.
- Make bootstrapping and compiler provenance explicit design topics.
- Treat portability as a language feature, not a backend afterthought.

### Kernighan: Documentation Is Part Of The Language

A language wins when its mental model can be taught. K&R, AWK, and Unix writing
made terse systems ideas understandable without making them vague.

Hum consequence:

- Examples must be canonical, small, and runnable as soon as execution exists.
- The language reference should be a first-class artifact, not a marketing doc.
- Error text, manuals, and schemas should teach the same model.

### Torvalds: Maintainer Reality Is A Design Constraint

Linux and Git are reminders that successful infrastructure must survive patches,
ports, scale, and long-term review. Tooling must make the right workflow faster
than the wrong one.

Hum consequence:

- Hum should make bad patches hard to hide: stable diagnostics, evidence diffs,
  source-derived ids, and reproducible checks.
- Performance claims require reproducible baselines.
- Contributor workflow is part of language design.

### Carmack And Abrash: Measure The Machine

Game engines made performance visual, concrete, and impossible to fake. The best
optimization work combines algorithm choices, data layout, hardware awareness,
and ruthless profiling.

Hum consequence:

- `cost:` should eventually connect source intent to measured facts.
- Optimization must start with representation and algorithm choice before exotic
  backend tricks.
- Runtime and stdlib work need adversarial benchmarks, not just happy paths.

### Bill Joy: Distribution And Networking Matter

BSD, vi, and Sun's early workstation story show that infrastructure spreads when
it meets people where the network, editor, and machine already are.

Hum consequence:

- Editor integration, portable installs, and network/deployment boundaries are
  core adoption work.
- Hum should support serious terminal and editor workflows before fancy surfaces.
- Network-facing APIs need conservative defaults and inspectable interfaces.

### Hejlsberg: Pragmatism Beats Purity Theater

Turbo Pascal, Delphi, C#, and TypeScript show a recurring pattern: strong tools,
fast feedback, understandable type systems, and adoption through compatibility
with the world developers already inhabit.

Hum consequence:

- Evidence cannot make ordinary coding exhausting.
- The type/evidence system must have escape hatches, staged strictness, and clear
  defaults.
- Tooling latency and editor experience are language features.

### Wirth: Leave Things Out On Purpose

Pascal, Modula, Oberon, and Project Oberon are an argument for disciplined
subtraction. A small complete system can teach more than a huge incomplete one.

Hum consequence:

- Every new feature must justify its permanent cognitive cost.
- Profiles may deliberately remove features when safety, footprint, or audit
  pressure demands it.
- A small self-hosting or self-describing core is more valuable than broad syntax
  without execution.

### Stallman: Freedom And Repairability Are Architecture

GNU's value was not only code volume. It was the claim that users need source,
tools, rights, and documentation to repair their own systems.

Hum consequence:

- Public Hum artifacts should be rebuildable, inspectable, and license-clear.
- Avoid hidden cloud, telemetry, or proprietary dependencies in the core path.
- Package evidence should help users know what authority code needs.

### Kildall: Portability Needs A Hardware Boundary

CP/M's BIOS split is a simple but deep adoption lesson: isolate the
machine-specific surface so the rest of the system can stay stable.

Hum consequence:

- Backend, runtime, OS, filesystem, clock, randomness, network, and device access
  need explicit boundary contracts.
- Embedded and enterprise profiles should see the same language core through
  different authority layers.
- Porting Hum should be a checklist, not folklore.

## Combined Legends Test For Hum

Before we add a major language/compiler/runtime feature, ask:

1. Would Thompson and Ritchie recognize the underlying mechanism?
2. Could Kernighan explain it with one clean example?
3. Would Torvalds accept the maintenance and patch-review burden?
4. Would Carmack or Abrash know how to measure it on real hardware?
5. Would Bill Joy see how it travels through editors, networks, and machines?
6. Would Hejlsberg see a practical adoption path for ordinary developers?
7. Would Wirth say the complexity has earned its place?
8. Would Stallman be able to rebuild, inspect, and modify the core path?
9. Would Kildall see the hardware/platform boundary clearly?
10. Would Bellard see a small enough artifact to audit and port?

If the answer is mostly no, the feature belongs in research, not the paved road.

## Near-Term Hum Consequences

- Start a traditional language reference early, with syntax, semantics, profiles,
  diagnostics, and examples evolving together.
- Keep Core Hum small until parsing, checking, graph export, editor recovery,
  math obligations, resource reports, and debug facts agree on the same model.
- Add a future portability-boundary document that names OS, filesystem, clock,
  randomness, network, device, backend, and runtime authorities.
- Make reproducible performance and footprint reporting part of the compiler
  contract before claiming best-in-class stdlib or backend wins.
- Treat license, provenance, rebuildability, and supply-chain trust as core
  product facts.

## What Hum Should Not Copy Blindly

- C's unchecked memory and undefined-behavior culture.
- Unix's tendency to leave global security policy outside the language.
- Linux's abrasive communication style.
- Game-engine crunch and burnout as a technical strategy.
- TypeScript's deliberate unsoundness as a default for safety-critical profiles.
- Wirth-style minimalism when it blocks real-world integration.
- GNU-style ideology when it prevents pragmatic adoption boundaries.
- CP/M's business fragility and hardware-era assumptions.

## Sources

- Dennis Ritchie, "The Development of the C Language": https://www.nokia.com/bell-labs/about/dennis-m-ritchie/chist.html
- Ken Thompson, "Reflections on Trusting Trust": https://www.cs.cmu.edu/~rdriley/487/papers/Thompson_1984_ReflectionsonTrustingTrust.pdf
- Linux kernel coding style: https://www.kernel.org/doc/html/latest/process/coding-style.html
- Git history, Pro Git: https://git-scm.com/book/en/v2/Getting-Started-A-Short-History-of-Git
- TypeScript design goals: https://github.com/microsoft/TypeScript/wiki/TypeScript-Design-Goals
- Project Oberon, revised edition: https://people.inf.ethz.ch/wirth/ProjectOberon/PO.System.pdf
- GNU Manifesto: https://www.gnu.org/gnu/manifesto.en.html
- Quake GPL source release: https://github.com/id-Software/Quake
- Quake III Arena GPL source release: https://github.com/id-Software/Quake-III-Arena
- Digital Research CP/M page: https://digitalresearch.biz/CPM.HTM
- AWK and Brian Kernighan background: https://awk.dev/

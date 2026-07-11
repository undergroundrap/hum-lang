# Hum Governance And Evolution

Date: 2026-07-06

## Purpose

Hum needs a way to accept ideas without becoming a pile of good intentions.

If hundreds of people eventually argue about the language, the process must keep
the language small, teachable, safe, fast, and agent-readable.

Hum should learn from C++ without becoming C++:

- welcome deep expertise
- require written rationale
- preserve compatibility deliberately
- reject feature sprawl
- avoid committee compromise syntax
- make evidence stronger than preference

## Governance Model

Hum uses a BDFL model with evidence-first review.

BDFL authority is constrained by the BDFR safety directive:

- protect the developer machine
- protect users and downstream systems that Hum programs touch
- keep Hum tooling offline-first until trust gates exist
- prioritize parser, diagnostics, semantic graph, and local proof before risky features
- adopt existing ecosystems through safe interop before trying to rewrite them
- preserve portability as a product feature

See [BDFR_SCOPE_AND_SAFETY_DIRECTIVE.md](BDFR_SCOPE_AND_SAFETY_DIRECTIVE.md).
See [SAFETY_OF_MAKER_AND_USER.md](SAFETY_OF_MAKER_AND_USER.md) for the
two-sided safety philosophy.

The project founder is the Benevolent Dictator For Life for the language vision
and final direction. That role exists to preserve taste, coherence, and courage.

But final decisions should still leave a public reasoning trail:

- what problem was solved
- what alternatives were rejected
- what evidence was considered
- what risks remain
- what migration path exists

The BDFL can say no because a feature feels wrong for Hum. The decision record
should still explain the taste, because future contributors need to learn the
language's judgment.

## Agent Roles And Mandates

Hum is built by one BDFL working with AI agents. There is one BDFL. Agent
titles do not dilute that, and no agent mandate creates a committee.

Standing roles, accepted 2026-07-08:

- BDFL (Ocean): taste, goals, scope, final authority. Accepts or rejects
  decision records directly and retains the final veto over delegated rulings
  under the amendment below. The review gates in active work orders (corpus
  reads, candidate reads, ADR acceptance) exist to force BDFL engagement at
  the moments that matter and may not be waived by agents.
- Architect-reviewer agent: holds the direction between sessions, reviews
  every session deliverable against the active work order, and must
  challenge decisively — including challenging the BDFL. May recommend
  with force. May hold a deliverable for BDFL input. May accept a decision
  record only through the delegated-ruling process below; may not accept one
  outside that process or change governance.
- Implementer agent: makes strong local implementation calls within
  accepted decisions and the active work order, and must push back before
  building anything it believes is wrong. May not change accepted
  decisions or governance.

Shared mandate for both agents:

- Optimize for the language the BDFL would still be proud of in 20 years.
- Treat accepted decision records and the architecture docs as ground
  truth.
- Keep every claim honest and compiler-checkable.
- Prefer simple, teachable, powerful foundations over clever surface
  sugar.
- Push back when the BDFL is about to trade away safety, clarity, or
  long-term coherence — and also when excessive purity would make Hum
  miserable to use.
- Make hard tradeoffs explicit, especially where ergonomics,
  implementation maturity, and safety collide.
- Never rubber-stamp. Never dominate. Think with the BDFL.

Mission and scope statements live in the accepted docs
([ARCHITECTURE.md](ARCHITECTURE.md),
[ADOPTION_STRATEGY_2026.md](ADOPTION_STRATEGY_2026.md),
[LANGUAGE_CONSTITUTION.md](LANGUAGE_CONSTITUTION.md)); this mandate does
not restate them, and agents must not widen scope by paraphrase.

### Delegated Ruling (BDFL-directed amendment, 2026-07-08)

The BDFL has delegated default ruling authority on decision records to the
architect-reviewer agent, under these terms:

- Every delegated ruling is recorded transparently as
  `accepted under delegated authority (BDFL veto open)` — never presented
  as a direct BDFL ruling.
- Each ruling is delivered to the BDFL as a one-page decision brief:
  the question, the recommendation, the reasoning, and what accepting it
  forecloses. The conversation record of these briefs is part of the
  project's decision history.
- The BDFL holds a standing veto: any delegated ruling can be reversed
  with one recorded sentence at any time before implementation hardens
  around it. Silence is consent.
- Delegation is borrowed, not transferred: the BDFL may reclaim any
  decision, or the whole delegation, by saying so.

Reserved matters that remain BDFL-only and cannot be delegated:

- licensing and any legal act of the copyright holder
- publishing to the outside world: pushes to remotes, tags, releases,
  announcements
- spending money or acting in the BDFL's name or identity
- changes to this delegation or to agent roles

The implementer agent's pushback mandate applies with extra force to
delegated rulings: it is the second reviewer when the first reviewer is
also the one ruling.

### Workflow Continuity (BDFL-directed amendment, 2026-07-11)

Hum's operating cycle is implementer -> independent architect-reviewer ->
implementer commit -> BDFL-authorized push and CI -> Work Order status update
-> separate next-session go signal. Each prompt names the active role and exact
scope. Each stopped gate names the next actor and provides a paste-ready prompt
when the next action is known; if authority is missing, that prompt requests
it. Routing work creates no standing manager or coordinator role and carries
no authority of its own. No stage silently authorizes the next one.

Roles are repository runbooks, not model or chat identities. Full context is a
cold start from repository ground truth; remembered conversation is only a
cache. Independence attaches to a deliverable: an agent that authored, edited,
generated, or directly directed any part of it cannot issue its independent
architect-reviewer verdict. Direct direction means producing or controlling
implementation work, including through another agent; findings, acceptance
criteria, and bounded corrective requirements remain review. If a reviewer
accidentally implements, the work is preserved rather than reverted to restore
the appearance of independence, its report is reclassified as implementation
evidence, and a fresh cold-start reviewer verifies the unchanged deliverable.
The accidental author may provide facts but may not issue or advocate the
verdict. It may work on a later unrelated deliverable, but never becomes
retrospectively independent for work it authored.

Git history, the active Work Order, accepted decisions, fixtures, diagnostics,
and check evidence are the durable handoff. Chats are replaceable; essential
state that exists only in conversation must be moved into an existing
authoritative repository artifact, not a separate handoff file or required
transcript. The per-lane one-pen and independent-reviewer rules in "Scaling
Notes: From One Crew To Many" preserve this boundary as contributors multiply;
cross-lane and cross-model-family review remains preferred for keystone work,
while ruling and publishing authority do not multiply.

The operational mutation gates and platform-configuration review discipline
are specified in `AGENTS.md`. They do not alter the architect-reviewer's duty
to challenge, the implementer's pushback mandate, delegated ruling with the
BDFL veto open, the red-main CI exception, or any BDFL-only reserved matter.
Agents may record an exact, explicit BDFL governance ruling as a constrained
writing task; they may not originate, broaden, accept, or silently reinterpret
one. Such a patch remains uncommitted until an independent review checks
authority validity, internal consistency, non-duplication, and unintended
power expansion.

## Bake-Off Doctrine

Decision 0014 (the ownership model) was decided by a method, not a mood,
and the method is reusable. Every remaining hard design axis with
competing plausible models gets the same instrument:

1. Pin a model-neutral program corpus first: behavior specifications and
   required rejections, never mechanisms. The corpus is the exam; write
   it before any candidate can study for it.
2. Each candidate is written by its best advocate, engaging its known
   hard cases honestly. Cross-candidate criticism is deferred to the
   scoring session. Strawmen make the bake-off theater.
3. Scoring separates proven-today from with-planned-repairs counts,
   weights by real-world pattern frequency, credits no unimplemented
   machinery (elision, hidden costs, future checkers) to any candidate,
   and prices every escape hatch.
4. The losers are salvaged: their surviving ideas become escape hatches,
   library patterns, or profile features, named in the decision record.
5. The decision lands as an ADR under the delegated-ruling process, veto
   open, with honesty locks that bind all claims until each planned
   repair ships and passes the corpus as probe programs.

A bake-off is authorized only when all three hold: genuinely competing
models are plausible, the choice has long-term lock-in, and a pinned
corpus can actually distinguish the candidates. Decisions that fail the
test are ordinary ADRs; do not ceremonialize the obvious.

Current recommended queue, in order — re-decided by each work-order
retrospective ledger, which grants the next authorization (currently
Session AE's ledger holds that power):

1. Effect polymorphism and higher-order tasks.
2. Structured concurrency and shared state.
3. Generic abstraction and coherence.
4. Allocation, containers, stable identity, arenas, and handles.
5. Numeric modes, floating-point policy, and replay semantics.
6. Text/Bytes/Path and Unicode boundaries.

None are authorized until a retrospective ledger says so. Do not
prebuild candidates, add speculative hooks, or widen a current session's
surface toward an unauthorized bake-off. The strategic thesis this
doctrine encodes: the ownership bake-off produced the project's best
decision; running the same instrument deliberately down the remaining
axes, in evidence-gated order, is the path to a language whose every
load-bearing choice was made in the open against a corpus that could
have falsified it.

## Scaling Notes: From One Crew To Many

The operating model above assumes one crew: one BDFL, one architect-
reviewer, one implementer, serial sessions, one pen. These notes
pre-decide what changes when contributors multiply — written while the
answers are cheap, not during the crisis. A "contributor" here is a
crewed lane: a human operating their own implementer and reviewer
agents. The unit of contribution is the lane, not the individual agent.

Break points and their pre-chosen answers:

1. The one-pen rule becomes per-lane. Serial dirty-tree review on `main`
   is a solo-scale optimization. With parallel lanes, sessions move to
   branches, review happens on the branch diff, and `main` holds only
   reviewer-accepted work. The one-pen rule survives inside each lane.
2. Session letters become per-work-order lanes. The global odometer
   assumes one writer. With parallel work orders, each order owns its
   session sequence ("Work Order 9 Session C"), and the odometer that
   matters is the work-order number.
3. Review authority stays independent, not centralized. A lane's
   reviewer never reviews its own lane's implementation. Keystone
   sessions (new binding forms, new diagnostic families, ADR-adjacent
   work) escalate to the architect office for second review. Cross-lane
   and cross-model-family review is preferred where available, because
   same-family reviewers share blind spots.
4. Ruling authority does not multiply. However many lanes exist, there
   is one BDFL and one architect office holding delegated ruling. Lanes
   propose through decision records and the RFC template; the review
   bodies sketched below advise. Eight rulers is a committee, and the
   regret ledger documents what committees did.
5. Contended ledger files become append-only. The friction ledger and
   scorecard are merge-conflict magnets under parallelism. Lanes append
   session-scoped records; only retrospectives consolidate, and only the
   architect office edits consolidated tables.
6. Culture becomes CI. At one crew, the probe sets and honesty locks
   transmit by practice. At many, anything not mechanically gated will
   be skipped by someone sincere and busy. Every standing discipline in
   AGENTS.md that can become a preflight check must become one before a
   second lane opens.
7. The work order remains the interface. Lanes coordinate through
   issued work orders, accepted decision records, and `main` — never
   through shared session state, chat context, or agent memory. If two
   lanes need to talk, the conversation produces a work-order amendment
   or an ADR, or it did not happen.

None of this activates while the project is one crew. It exists so that
opening the second lane is a mechanical step, not a redesign.

## Decision Principle

```text
Taste chooses the direction.
Evidence earns stability.
Compatibility earns trust.
Pedagogy earns adoption.
```

Hum's working method is captured in [LANGUAGE_BUILDER_OPERATING_MODEL.md](LANGUAGE_BUILDER_OPERATING_MODEL.md): small proofs, written lessons, graph/report/check surfaces, migration paths, and then public claims.

## Review Bodies

As Hum grows, create small domain groups. They advise; they do not own the soul
of the language.

Suggested groups:

- language design
- compiler and IR
- safety and security
- performance and benchmarks
- standard library
- tooling and diagnostics
- pedagogy and beginner experience
- agent semantics and semantic graph

Each group should be responsible for evidence, not politics.

## Change Types

### Design Note

Explores an idea. No commitment.

### Experiment

Implemented behind an experimental flag or in examples. No stability promise.

### Candidate

Feature is coherent enough for serious use, but may still change.

### Stable

Feature has compatibility guarantees.

### Deprecated

Feature remains available but has a planned replacement.

### Removed

Only for experimental features or edition boundaries with a migration path.

## Feature Admission Gates

A feature cannot become stable unless it has:

1. Problem statement.
2. Beginner explanation.
3. Senior-engineer explanation.
4. Syntax examples.
5. Semantic graph representation.
6. Diagnostics design.
7. Formatter behavior.
8. Ergonomics and operator impact.
9. Tests and generated-test story.
10. Runtime performance and allocation impact.
11. Compile-time impact.
12. Optimization and DSA evidence when relevant.
13. Toolchain impact: syntax highlighting, LSP, debugger, profiler, and source maps.
14. Runtime profile impact: normal, realtime, engine, safety-critical, or certified-toolchain behavior.
15. Safety and security analysis.
16. Maker/user safety impact.
17. Agent documentation impact.
18. Cross-language regret ledger check.
19. Language project risk register check.
20. Migration and compatibility story.
21. Rejected alternatives.
22. BDFL decision note.

If a feature cannot pass these gates, it can remain experimental.

## Stability Levels

```text
sketch        docs only, may disappear
experimental  available behind a flag, no compatibility promise
candidate     expected shape, still allowed to change
stable        compatibility promise
legacy        supported only for migration
```

Hum should make stability visible in docs and semantic graph output.

## Compatibility And Editions

Hum should not break stable code casually.

Use editions for rare language-level shifts:

```text
edition 2026
edition 2027
```

Edition changes must include:

- formatter migration
- semantic graph migration
- diagnostic migration
- package metadata migration
- compatibility report
- old/new examples

Editions are a release valve, not a broom.

## Experimental Features

Experimental features must be visibly marked:

```text
use experimental feature checked cost prove
```

or by package/build config.

Experimental features may be removed. Stable features should not.

## BDFL Vetoes

The BDFL should veto features that:

- add multiple ways to express the same core idea
- make source harder for humans to scan
- make semantic graphs weaker
- make agents guess instead of consume facts
- hide effects, allocation, mutation, or failure
- weaken safety defaults
- make the compiler much harder without major user benefit
- only exist because another language has them

A veto should be short, direct, and recorded.

## Standard Library Governance

The standard library should be harder to grow than a package ecosystem.

No API enters `std` unless it has:

- contract blocks
- examples
- tests
- fuzzing when input-facing
- benchmarks when performance-facing
- optimization and DSA evidence when structure-facing
- misuse guidance
- semantic graph docs
- stability promise

The standard library should prefer small, sharp, composable APIs over a huge
surface area.

See [STDLIB_CONSTITUTION.md](STDLIB_CONSTITUTION.md) for the full admission packet and stability rules.

## Decision Records

Every accepted or rejected major decision should get a short decision record.
The active index is [decisions/README.md](decisions/README.md).

Example paths:

```text
docs/decisions/0001-adopt-evidence-native-architecture.md
docs/decisions/0002-use-rust-bootstrap-until-self-hosting.md
```

A decision record should include:

- status
- context
- decision
- consequences
- alternatives rejected
- BDFL note

## Versioning And Tags

Hum follows the local release policy in [RELEASE_AND_VERSIONING.md](RELEASE_AND_VERSIONING.md).

Rules:

- keep `VERSION` and `Cargo.toml` in sync
- use SemVer-shaped versions from the beginning
- treat `0.0.x` as pre-alpha snapshots, not compatibility promises
- use annotated `vX.Y.Z` Git tags only after release-readiness checks pass
- do not push tags or remotes without explicit human approval

## Release Rhythm

Early Hum should move in milestones, not calendar promises.

Suggested rhythm:

- Milestone 0: semantic graph and diagnostics
- Milestone 1: executable core and tests
- Milestone 2: ownership and effects
- Milestone 3: stores and stdlib lab
- Milestone 4: unsafe and FFI
- Milestone 5: compile-time discipline, Nectar, and editor spine
- Milestone 6: native backend
- Milestone 7: self-hosting preparation
- Milestone 8: agent-native tooling and 2050 developer experience

After real users exist, add regular preview releases and rare stable releases.

## Community Culture

Hum should welcome strong opinions but require strong artifacts.

Good contribution:

- shows the problem
- gives before/after Hum examples
- explains safety/performance impact
- adds diagnostics
- updates semantic graph schema
- adds tests or benchmarks

Weak contribution:

- asks for a feature because another language has it
- argues from taste without examples
- adds syntax without diagnostics
- adds magic agents must infer
- grows the language without reducing confusion

## Brutal Warning

A language can die from bad governance before it dies from bad code.

If Hum becomes a place where every smart person adds their favorite abstraction,
it will fail.

Hum should be opinionated enough to say no, transparent enough to earn trust,
and empirical enough to improve when evidence beats taste.

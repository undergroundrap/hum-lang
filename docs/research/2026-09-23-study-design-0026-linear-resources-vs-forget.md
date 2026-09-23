# Study Design: Exactly-Once Linear Resources vs Rust's Safe `mem::forget`

Date: 2026-09-23
Status: draft. Pre-issuance independent review requested before any PR.
Recommendation only; the BDFL rules.

## Why this study exists

Decision 0026 (accepted 2026-09-23, revision 3) keeps closed-world
accountability as a scoped design objective and commissions a
comparative-evidence plan. The language argument that survived review is
not exclusive ownership of a ledger: it is that Hum can make a valuable
discipline **coherent, default, teachable, and economical**. The first
comparative program in that plan pits Hum's exactly-once linear resources
against the strongest Rust encodings of the same discipline. This document
designs that study — the protocol, the two implementations to be compared,
the strongest honest Rust opponent, and exactly what gets measured.

## The protocol: a transaction that must be settled exactly once

**Chosen protocol: a database-style transaction that must be committed or
rolled back exactly once.** The settlement must be an explicit decision by
program code — commit or abort — not a silent default.

Rationale, against the alternatives:

- **File handle (close exactly once).** Weak discriminator. Closing is
  drop-natural and usually idempotent; `mem::forget` on a handle leaks a
  descriptor, which is waste rather than a broken protocol. The OS reclaims
  it. The difference between the two sides is small and mostly about leaks.
- **Lock (acquire/release pairing).** Weak discriminator. Releasing is
  drop-natural; the interesting bug (double release) is already
  unrepresentable in both languages via move semantics. Forgetting a guard
  deadlocks, which is a liveness failure, not a settlement failure.
- **Transaction (commit-or-rollback exactly once).** Sharp discriminator.
  The whole point of the protocol is the *decision*: a transaction that is
  silently dropped never commits and never explicitly aborts. Real
  consequences are observable — server-side state left open until timeout,
  escrow never settled, a journal never flushed. Rust's own language team
  names exactly this case: "transactions that must be either committed or
  rolled back, but can't be dropped" is their stated want for linear
  types, and it is still a want, not a feature **[External]**
  (rust-lang team meetup notes, 2024). Decision 0014 lists commit/rollback
  first among its exactly-once protocol examples **[Requirement]**
  (0014-adopt-ownership-model.md).

The study fixes one concrete shape: open a transaction, perform a bounded
sequence of writes, then settle it. Every path — success, expected failure
(0016's typed failure), and early return — must end in exactly one explicit
settlement. The adversarial cases are a forgotten guard, a
`ManuallyDrop`-wrapped guard, and a settlement decision that arrives via a
destructor default instead of explicit code.

## The Hum version (design sketch under 0014)

Decision 0014 adopts linear resources as the first-class model for
exactly-once protocols and gives the vocabulary: values are owned by
default, `consume` transfers or closes linear authority **[Requirement]**
(0014 §Decision items 1–2, 5). The sketch:

```hum
task transfer_funds(db: change Database, from: Text, to: Text, amount: Int)
    fails when: InsufficientFunds | ConnectionLost
{
    let tx = begin(db)              # tx: linear Transaction
    try debit(tx, from, amount)
    try credit(tx, to, amount)
    consume tx.commit()             # commit consumes linear authority
    # any path that reaches the end of this task without consuming tx
    # is a checker error, not a silent drop
}
```

And the failure path makes the decision explicit rather than defaulted:

```hum
    let tx = begin(db)
    let outcome = try debit(tx, from, amount) or fail InsufficientFunds.context
    consume tx.rollback()           # abort is also an explicit settlement
```

What the sketch claims, labeled per 0026's evidence model:

- **[Requirement]** A linear `Transaction` must be consumed exactly once on
  every path; falling off the end of a task with a live linear value is a
  checker error. There is no `mem::forget` equivalent in the language:
  dropping is not a settlement, and the checker does not accept it as one.
- **[Requirement]** Settlement is one of two consuming operations,
  `commit` or `rollback`, each transferring linear authority. Transition
  *order* (no commit after rollback, no use after settle) follows from
  the same linearity.
- **[Inference]** The study's hypothesis: this is coherent (one concept —
  consumption — covers ordering and completion), default (no annotations
  beyond declaring the type linear), teachable (the diagnostic names the
  unconsumed value and the path that abandoned it), and economical (the
  protocol author writes the two consuming operations; the client writes
  no protocol machinery).

### What Hum has not implemented (honesty section)

The comparison cannot be run as an implemented-vs-implemented experiment
today. These are the dependencies:

- **Linear resource path checking is unimplemented** (0014 consequence
  roadmap, item 2). It is the exact feature under study. Until it exists,
  the Hum side is a design sketch, and the study's first deliverable is
  the sketch plus the checker's acceptance criteria, not a measured
  implementation.
- **The 0015 classifier is unimplemented** ("current Hum assigns none").
  Under 0015's vocabulary the settlement obligation is today `unproved` —
  checked at runtime at best. The study must say this plainly: Hum's
  current evidence for the transaction protocol is weaker than the Rust
  typestate encoding's compile-time ordering guarantee.
- **Returned/stored views are unimplemented** (0014 §3, roadmap items
  3–5). If the transaction carries views into its connection's buffers,
  that half of a realistic implementation is also future work. The study
  keeps the transaction opaque to avoid depending on it.
- **Tasks-as-values and effect polymorphism are not settled** (0014). If
  settlement spans task boundaries (e.g. a transaction handed to a
  worker), the study depends on decisions that do not exist yet. The
  study's first phase keeps settlement inside one task.
- **Borrow-soundness completeness** (disjoint-field projection, internal
  references) is claim-locked until implemented. The study makes no
  memory-safety claim beyond what 0014's honesty locks permit.

## The strongest Rust encodings (the honest opponent)

Ranked by strength, with each encoding's exact weak point. The study
implements and measures against the strongest *implementable today*
encoding, and separately costs the restricted-profile encoding.

**Encoding A — typestate + `#[must_use]` + `clippy::mem_forget = "deny"` +
`#[forbid(unsafe_code)]`, workspace lint config enforced in CI.**
State as a type parameter, transitions consume `self` by value, per-state
`impl` blocks so wrong-order calls are compile errors. This is the
strongest stack stable Rust offers today **[External]** (typestate pattern
docs; clippy `mem_forget` lint in the restriction group, allow-by-default;
`#[must_use]`/`unused_must_use` semantics).

Its exact weak points, each measured by the study's adversarial matrix:

1. `mem::forget` is safe code; the lint matches only direct
   `mem::forget` calls on `Drop` types. `ManuallyDrop::new(tx)` achieves
   the same settlement-skip with **no lint firing** — `ManuallyDrop<T>`
   has no `Drop` impl **[External]** (clippy lint source; std docs).
2. Typestate enforces transition *order*, never *completion*. Nothing
   forces the final `commit(self)` call; the "must consume" obligation is
   not expressible in the type system.
3. `#[must_use]` is silenced by `let _ = value;` — binding and dropping
   at end of scope counts as "use".
4. Lint configuration does not reach third-party dependencies
   (`--cap-lints` lowers even forbidden lints) **[External]**
   (rustc lint-level docs; 0026 revision 3). A dependency can forget the
   guard invisibly to the profile.

**Encoding B — restricted compilation profile with a trusted checker.**
A driver that forbids `mem::forget`, `ManuallyDrop::new`,
`Box::leak`, and cycle-forming `Rc`/`Arc` use on admitted paths, binds
evidence to the actual build inputs and artifact, and rejects stale or
missing evidence — the enforced-profile steelman from 0026 **[Inference]**.
This is the encoding that could, in principle, close Encoding A's holes.

Its cost is the study's central measurement: it is a one-to-two-year
project for a constrained profile (0026), and the study must price what it
takes to enforce what Encoding A only suggests — the checker itself, the
dependency-audit burden (every newly admitted crate is an admission
decision), and the interop damage from rejecting `forget`/`ManuallyDrop`/
`Box::leak` in admitted code.

**Encoding C — whole-program verifier campaign** (Kani harnesses over the
closed artifact, `forget` denied by the profile, dependencies audited).
**[External]** No current tool verifies exactly-once consumption on all
paths including `forget` as a property a type carries: Kani is bounded
per-harness and cannot quantify over all uses of a type across crates;
Creusot verifies attached functional specs, not global consumption.
Encoding C is a per-artifact verification campaign, bounded and expensive —
measured as the upper bound on what verification buys, not as a practical
encoding.

**Not considered:** GhostCell (proves aliasing discipline, not
consumption) **[External]**; session-type crates (same completion gap as
typestate); proposed linear-types extensions (`?Leak` bound — unshipped,
and the internals discussion itself flags linearity as "easy to defeat"
at generic boundaries) **[External]**.

The study's Rust baseline is Encoding A. Encoding B is costed as a design
exercise (what must be built and audited), Encoding C as the verification
upper bound.

## What gets measured

For Encoding A and the Hum sketch (and, as projections, Encodings B and C):

1. **Lines.** Protocol implementation plus one representative client
   (the transfer above), counted identically on both sides. Includes
   everything the protocol author must write: state types, impl blocks,
   lint configuration, wrapper types.
2. **Annotation burden.** Count of protocol-machinery annotations per
   client call site: `PhantomData` markers, state type parameters,
   `#[must_use]` placements, lint-config lines, and any
   `ManuallyDrop`-shaped defensive code. The hypothesis under test is
   that Encoding A concentrates this burden on every client while the
   Hum sketch concentrates it once in the linear type declaration.
3. **Interop restrictions.** What the admitted code universe must give up:
   for Encoding A — nothing enforced (advisory only); for Encoding B —
   `mem::forget`, `ManuallyDrop::new`, `Box::leak`, and unaudited
   dependency updates become admission decisions, each priced.
   Measured as: the list of rejected constructs, and what breaks in a
   realistic dependency closure when they are rejected.
4. **Proved vs trusted** (0015's vocabulary). For each encoding, every
   protocol property is labeled:
   - *Encoding A:* transition order — `proved` (compile error otherwise);
     final settlement happens — **trusted** (Drop + convention + lints);
     no forget in dependencies — **trusted** (lint capping);
     lint config applied — **trusted** (CI convention).
   - *Hum sketch:* transition order and final settlement — `proved` by
     the linear checker **[Requirement, unimplemented]**; today —
     `unproved` (0015: classifier assigns nothing).
   The deliverable is the labeled table, not a winner's banner.
5. **Adversarial matrix.** Each encoding faces: `mem::forget` on the
   guard; `ManuallyDrop::new` on the guard; `let _ =` settlement-skip;
   a dependency that forgets; a stale or missing lint configuration;
   settlement via destructor default instead of explicit code (the
   sqlx abort-on-drop shape — a fail-closed default, not an explicit
   decision **[External]**). Each case records: caught by whom, at what
   stage, with what diagnostic — or silently accepted.

## Falsification criteria

The study is designed to be losable, per 0026's symmetric falsifier:

- If Encoding B can be specified and costed at a price comparable to
  implementing Hum's linear checker, and its admitted-code restrictions
  do not damage ordinary library interop, then exactly-once settlement
  does not discriminate between the language and the profile — the
  investment argument for Hum's linear resources weakens to convenience.
- If the Hum sketch's annotation or diagnostic burden turns out
  comparable to Encoding A's typestate machinery once real programs are
  written (the friction ledger is the instrument), the "economical and
  teachable" claim fails on its own terms.
- If the adversarial matrix shows Encoding A catching every realistic
  misuse in practice (forget-lint deny + code review + CI), the study
  records that the residual risk is theoretical for the studied
  population — and says so.

What the study does **not** conclude: failing to establish exclusivity
does not establish that Hum's integrated design has no value (0026). The
study measures the cost of enforcement, not the metaphysics of bolt-on.

## Sequence

1. **Study 1 (this document):** exactly-once linear resources vs
   `mem::forget`, transaction protocol. Deliverable: the sketch, the
   Rust baseline implementation, the measurement table, the adversarial
   matrix.
2. **Study 2:** stored and returned views (0014) vs the cost of safe-Rust
   encodings — only after a concrete Hum example establishes what Hum
   accepts naturally (0026's comparative plan). Gated on 0014's
   returned-view implementation, not started here.

## Review requested

Pre-issuance independent review of this study design before any PR, per
the commissioning instruction. Reviewer checks: the protocol choice is
the sharpest available discriminator; the Rust opponent is the strongest
honest one (no strawman); every Hum claim is labeled with its
implementation status; the falsification criteria can actually fire; the
measurements are countable, not vibes.

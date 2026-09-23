# Study Design: Exactly-Once Linear Resources vs Rust's Safe `mem::forget`

Date: 2026-09-23
Status: draft (revision 2). Pre-issuance review: Claude, 2026-09-23 —
verdict accept with required fixes; all four applied (see Revision
history). Re-check requested before any PR. Recommendation only; the
BDFL rules.

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

## The Hum version (real surface, per the reference and the probe)

The sketch below uses only surface that exists in
`docs/LANGUAGE_REFERENCE.md` and `examples/probes/transaction_once.hum`.
`consume` is a parameter permission (`consume txn: Transaction`) and a
call-site argument marker (`rollback(consume txn)`), not an expression
keyword; tasks carry sections (`why:`, `fails when:`, `does:`); prose lives
in sections. Decision 0014 adopts linear resources as the first-class model
for exactly-once protocols and gives the vocabulary **[Requirement]**
(0014 §Decision items 1–2, 5):

```hum
module study.transaction_settle

type Transaction {
  id: UInt
}

type TransferError {
  code: Text
}

task begin_transaction() -> Transaction {
  why:
    open a transaction resource; the Transaction shape is recognized
    as the first narrow linear-resource class

  does:
    return Transaction.open
}

task commit(consume txn: Transaction) -> Unit {
  why:
    settle by committing; consumes linear authority

  does:
    return
}

task rollback(consume txn: Transaction) -> Unit {
  why:
    settle by aborting; consumes linear authority

  does:
    return
}

task transfer_funds(amount: UInt) -> Result Text, TransferError {
  why:
    commit or roll back exactly once on every path

  fails when:
    debit fails
    credit fails

  does:
    let txn: Transaction = begin_transaction()
    if debit(change txn, amount) == false {
      let settled: Unit = rollback(consume txn)
      fail TransferError.debit_failed
    }
    if credit(change txn, amount) == false {
      let settled: Unit = rollback(consume txn)
      fail TransferError.credit_failed
    }
    let settled: Unit = commit(consume txn)
    return "ok"
}
```

What this claims, labeled per 0026's evidence model:

- **[Implemented, scoped]** For the recognized Transaction-shaped class,
  H0803 (linear resource not consumed) and H0804 (consumed twice) fire as
  checker diagnostics — compile-time rejection, with the interpreter
  trapping on the same violations as a runtime backstop. A linear value
  that reaches a return, failure, or fallthrough path without exactly one
  visible consume action is an error — there is no `mem::forget`
  equivalent in the language, and dropping is not accepted as settlement.
  For the recognized shape this is stronger than a drop-bomb and
  comparable to the closure API's guarantee; for general linear types
  there is no check today at all, weaker than a drop-bomb.
- **[Requirement]** Settlement is one of two consuming operations,
  `commit` or `rollback`, each taking `consume txn: Transaction`.
  Transition *order* (no commit after rollback, no use after settle)
  follows from the same linearity; use-after-consume traps with H0801.
- **[Proposed surface]** How a type is *declared* linear. Today the
  checker recognizes the Transaction shape (friction ledger: "recognize
  Transaction-shaped type annotations as the first narrow
  linear-resource class"); the source-visible linear resource marker
  that generalizes exactly-once checking beyond transaction probes is
  still a proposal ("design a source-visible linear resource marker
  before generalizing"), not surface.
- **[Inference]** The study's hypothesis: this is coherent (one concept —
  consumption — covers ordering and completion), default (no annotations
  beyond the type and `consume` at the boundary), teachable (the
  diagnostic names the unconsumed value and the path that abandoned
  it), and economical (the protocol author writes the two consuming
  operations; the client writes settlement calls, not protocol
  machinery).

### The central open question: linearity × 0016 failure propagation

The probe above handles failure by branching on a `Bool` and settling
explicitly before `fail`. With 0016's typed failure, the natural sketch is:

```hum
  does:
    let txn: Transaction = begin_transaction()
    let receipt: Receipt = try charge(change txn, amount)
    let settled: Unit = commit(consume txn)
    return "ok"
```

If `charge` fails, `try` propagates the failure out of the task while
`txn` is live — violating the sketch's own exactly-once rule on that
path. H0803 names failure paths explicitly, so today this is a checker
error (and a runtime trap); the question is what the *designed* answer
is. Rust's scoped-closure API handles this for free: the closure's `Err`
return triggers the library's rollback with no per-path code. Hum's
options, listed without deciding:

1. **Reject `try` while a linear value is live.** The programmer keeps
   the probe's shape: branch on results, settle explicitly, then `fail`.
   Cost is explicit rollback code per failure path — see the measured
   dimension below.
2. **A settle-on-failure construct.** A scoped form that guarantees
   settlement on the failure path. Shape undecided.
3. **The failure carrier owns settlement.** The typed failure value takes
   the linear resource with it as it propagates — settlement travels
   with the error.

**Measured dimension:** lines of explicit settlement code per failure
path under each option, counted on the transfer client with N fallible
calls. This is exactly where "economical" is won or lost: if option 1
costs one rollback call per `try` site and option 2 costs one construct
per task, the study reports both numbers. The Rust scoped-closure
baseline for this dimension is zero per-path lines — the library pays
once.

### What Hum has not implemented (honesty section)

The comparison cannot be run as an implemented-vs-implemented experiment
for the *general* linear checker. The narrow case, however, is further
along than a pure sketch:

- **Narrow linear-resource checking exists; general checking does not.**
  H0803 (linear resource not consumed) and H0804 (consumed twice) exist
  as checker diagnostics and interpreter traps for the recognized
  Transaction-shaped class **[Implemented, scoped]** (DIAGNOSTICS.md;
  examples/probes/transaction_once.hum). What is unimplemented: the
  static linear checker beyond the recognized shape, and the
  source-visible linear resource marker (friction ledger: "design a
  source-visible linear resource marker before generalizing
  exactly-once checking beyond transaction probes") **[Proposed
  surface]**.
- **The 0015 classifier is unimplemented** ("current Hum assigns none"),
  so the study does not call the H0803 enforcement `proved` — it is
  **checker-enforced**, and the vocabularies stay separate. Split by
  scope: for the recognized Transaction shape, settlement is
  checker-enforced at compile time — stronger than a drop-bomb and
  comparable to the closure API's guarantee. For general linear types
  there is no check today at all — weaker than a drop-bomb, which would
  at least panic. The drop-bomb idiom (below) is the honest
  current-state Rust baseline for the *general* case, not the
  recognized shape; the study measures the delta the general checker
  would buy.
- **The linearity × 0016 question is open** (see above). Until it is
  decided, the study cannot claim Hum handles the failure path more
  economically than Rust's scoped-closure API — that is a measured
  dimension, not an assumed win.
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

**Encoding A1 — scoped closure API** (`db.transaction(|tx| { ... })`).
The library owns the guard; the closure receives `&mut Transaction`; the
library commits on `Ok` and rolls back on `Err`. Exactly-once by
construction in safe Rust: the client never owns the guard, so
`mem::forget` is unavailable to the client; the transaction cannot escape
the closure (lifetime-bound) or be handed to another task; a panic inside
the closure unwinds through the guard's `Drop`, which rolls back
(fail-closed default). This is the strongest *practical* encoding — the
one real Rust code uses today (Diesel, sqlx connection pools) — and
omitting it would make the comparison a strawman.

Its exact costs, each a measured dimension:

1. **No handoff, no storage.** A transaction that must span tasks, be
   stored, or be settled conditionally on values computed after the
   closure returns is unrepresentable. The pattern must be
   re-implemented per resource type — there is no language-level
   linearity.
2. **Settlement policy is the library's, tied to `Result`.** `Ok` maps
   to commit, `Err` to rollback. Custom per-outcome settlement logic
   (commit on one error kind, roll back on another) is unrepresentable
   without a second API.
3. **The library is the trusted settler.** Exactly-once holds because
   one audited place owns the guard — the same shape as Hum's
   conditional guarantee, with the trust concentrated in the library
   rather than the checker.

**Encoding A2 — typestate + `#[must_use]` + `clippy::mem_forget = "deny"` +
`#[forbid(unsafe_code)]`, workspace lint config enforced in CI.**
State as a type parameter, transitions consume `self` by value, per-state
`impl` blocks so wrong-order calls are compile errors. This is the
strongest *type-level* stack stable Rust offers today **[External]**
(typestate pattern docs; clippy `mem_forget` lint in the restriction
group, allow-by-default; `#[must_use]`/`unused_must_use` semantics).

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

**Encoding A3 — drop-bomb idiom.** A guard whose `Drop` panics if the
resource was never explicitly settled:

```rust
impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.settled { panic!("transaction dropped without settlement") }
    }
}
```

This is a runtime check — fail-stop, not a compile error — and it is the
honest current-state baseline for the *general* linear case: for the
recognized Transaction shape Hum already rejects at compile time via
H0803/H0804, past drop-bomb parity; for general linear types Hum has no
check today, so the drop-bomb is ahead of Hum there. The study measures
the delta the general linear checker would buy. Weak points: defeated by
`mem::forget` (no `Drop` runs); the diagnostic is a panic message, not
a source-site blame; panics are the wrong tool where aborting the
process is unacceptable.

**Encoding B — restricted compilation profile with a trusted checker.**
A driver that forbids `mem::forget`, `ManuallyDrop::new`,
`Box::leak`, and cycle-forming `Rc`/`Arc` use on admitted paths, binds
evidence to the actual build inputs and artifact, and rejects stale or
missing evidence — the enforced-profile steelman from 0026 **[Inference]**.
This is the encoding that could, in principle, close Encoding A2's holes.

Its cost is the study's central measurement: it is a one-to-two-year
project for a constrained profile **[Inference]** (0026's estimate), and
the study must price what it takes to enforce what Encoding A2 only
suggests — the checker itself, the dependency-audit burden (every newly
admitted crate is an admission decision), and the interop damage from
rejecting `forget`/`ManuallyDrop`/`Box::leak` in admitted code.

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

The study's Rust baseline is all three A encodings, implemented and
measured. Encoding B is costed as a design exercise (what must be built
and audited), Encoding C as the verification upper bound.

## What gets measured

For the three A encodings and the Hum sketch (and, as projections,
Encodings B and C):

1. **Lines.** Protocol implementation plus one representative client
   (the transfer above), counted identically on both sides. Includes
   everything the protocol author must write: state types, impl blocks,
   lint configuration, wrapper types, the closure-API library function.
2. **Annotation burden.** Count of protocol-machinery annotations per
   client call site: `PhantomData` markers, state type parameters,
   `#[must_use]` placements, lint-config lines, and any
   `ManuallyDrop`-shaped defensive code. The hypothesis under test is
   that Encoding A2 concentrates this burden on every client while the
   Hum sketch concentrates it once in the linear type declaration —
   while Encoding A1 concentrates it once in the library function.
3. **Settlement code per failure path** (the linearity × 0016
   dimension). Lines of explicit settlement/rollback code the client
   writes per fallible call: zero for Encoding A1 (the library pays
   once); N rollback branches for the Hum probe's option-1 shape; one
   construct per task for option 2; to be counted once the 0016 question
   is decided. This is where "economical" is won or lost.
4. **Interop restrictions.** What the admitted code universe must give up:
   for Encoding A2 — nothing enforced (advisory only); for Encoding A1 —
   no handoff, no storage, no custom settlement policy (expressive
   restriction, not a lint); for Encoding B — `mem::forget`,
   `ManuallyDrop::new`, `Box::leak`, and unaudited dependency updates
   become admission decisions, each priced. Measured as: the list of
   rejected constructs, and what breaks in a realistic dependency
   closure when they are rejected.
5. **Proved vs trusted** (0015's vocabulary). For each encoding, every
   protocol property is labeled:
   - *Encoding A1:* exactly-once settlement — `proved` by construction
     (the library owns the guard; the client cannot name the owned
     value); settlement *policy* — trusted to the library's
     Ok→commit/Err→rollback mapping; no escape — `proved` (lifetime).
   - *Encoding A2:* transition order — `proved` (compile error
     otherwise); final settlement happens — **trusted** (Drop +
     convention + lints); no forget in dependencies — **trusted**
     (lint capping); lint config applied — **trusted** (CI convention).
   - *Encoding A3:* unsettled drop detected — `proved` at runtime
     (panic); settlement happens — **trusted** (no `forget`); the
     process survives — **trusted** (a panic aborts the task at best).
   - *Hum sketch:* transition order and final settlement for the
     Transaction-shaped class — checker-enforced by H0803/H0804
     **[Implemented, scoped]** (not called `proved`: the 0015
     classifier still assigns nothing); the general linear checker —
     **[Requirement, unimplemented]**; today the general obligation is
     `unproved` (0015: classifier assigns nothing), weaker than a
     drop-bomb.
   The deliverable is the labeled table, not a winner's banner.
6. **Adversarial matrix.** Each encoding faces: `mem::forget` on the
   guard; `ManuallyDrop::new` on the guard; `let _ =` settlement-skip;
   a dependency that forgets; a stale or missing lint configuration;
   settlement via destructor default instead of explicit code (the
   sqlx abort-on-drop shape — a fail-closed default, not an explicit
   decision **[External]**); a panic on the settlement path; an attempt
   to hand the transaction to another task. Each case records: caught
   by whom, at what stage, with what diagnostic — or silently
   accepted. Expected honest outcomes: A1 is immune to client-side
   forget (no owned value) but cannot represent handoff; A3 is
   defeated by `mem::forget`; Hum's H0803 trap fires on the
   `try`-propagation path today.

## Falsification criteria

The study is designed to be losable, per 0026's symmetric falsifier:

- If Encoding B can be specified and costed at a price comparable to
  implementing Hum's linear checker, and its admitted-code restrictions
  do not damage ordinary library interop, then exactly-once settlement
  does not discriminate between the language and the profile — the
  investment argument for Hum's linear resources weakens to convenience.
- If Encoding A1 covers the realistic program population at negligible
  cost — in-task settlement with Ok/Err policy — and the study's
  programs never need handoff or custom settlement policy, then the
  remaining discriminator is cross-task settlement and the teachability
  of one uniform concept versus per-library patterns. The study must
  say which population it measured.
- If the Hum sketch's annotation or diagnostic burden turns out
  comparable to Encoding A2's typestate machinery once real programs are
  written (the friction ledger is the instrument), the "economical and
  teachable" claim fails on its own terms.
- If the adversarial matrix shows Encoding A2 catching every realistic
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
honest one (no strawman — the scoped closure API and drop-bomb are in);
every Hum claim is labeled with its implementation status and uses real
surface; the linearity × 0016 question is named with undecided options
and a measured dimension; the falsification criteria can actually fire;
the measurements are countable, not vibes.

**Revision history.** Draft at f92c84b. Pre-issuance review: Claude,
2026-09-23 — verdict accept with required fixes: (1) Hum sketch rewritten
in real Hum surface (`consume` as parameter permission and call-site
marker, sections, `fail`, no invented syntax), with the linear-type
declaration marked [Proposed surface]; (2) the linearity × 0016 failure
propagation question named explicitly with three undecided options and a
per-failure-path settlement-code measured dimension; (3) scoped closure
API and drop-bomb idiom added as Rust encodings; (4) Encoding B's
one-to-two-year cost labeled [Inference]. Revision 2 (7d29f8a) applied
all four. Re-check: Claude, 2026-09-23 — passes, with one consistency
fix applied here: the recognized-shape vs general-linear split (H0803 is
compile-time checker rejection for the recognized shape, stronger than
a drop-bomb and comparable to the closure API; no check at all for
general linear types, weaker than a drop-bomb) and "checker-enforced"
instead of `proved` for H0803, keeping the vocabularies separate from
0015. No further review round needed.

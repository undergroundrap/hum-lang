# 0024: State the Performance North Star and Its Implications

Date: 2026-09-22
Status: accepted 2026-09-22. BDFL ruling (relayed 2026-09-22): accepted with
the verification-speed addition below (Claude's review addressed).

## Context

Hum's north star has never been written down in the repo: Hum should be the
best language in the world for humans and agents to read, write and verify,
with performance in the class of C++ and Rust. A north star that lives only
in chat cannot constrain decisions. This record states it and, crucially,
states what it implies for tradeoffs being made right now -- including
wordfreq's text_split, whose owned-copy return was chosen to dodge the
ownership debt (decision 0021).

"Best ... to read, write and verify" is the primary clause; "performance in
the class of C++ and Rust" is the constraint that keeps the primary clause
honest. A language that is a pleasure to verify but cannot ship inside
performance-critical systems has not met its goal.

## Options considered

**A. State the north star with implications** (recommended): the goal plus
what it commits us to, what it does not, which current choices are debt
rather than settled, the going-forward rule, and the backend gates. Costs one
short record; constrains every later tradeoff explicitly.

**B. State the north star without implications**: cheaper, but a slogan with
no teeth. Every hard tradeoff (owned copies vs views, runtime vs
compile-time checks) would re-litigate what the goal means.

**C. Defer until a backend exists**: avoids premature commitment, but the
tradeoffs are being made *now* -- text_split already chose copies over
views. Deferring the statement does not defer the decisions; it only defers
naming them, which is how debt gets quietly accepted as final.

## Decision

The north star is adopted as stated above, with the following implications.

### What "C++/Rust class" commits us to

- **Predictable performance.** No hidden costs. Allocation is visible in the
  language (`allocates:`), and cost claims are honest (decision 0021's
  "no zero-cost claim" is the model).
- **Compile-time checking preferred.** Rust's answer to the safe-and-fast
  tension is that most checks are compile-time. Any check Hum defers to
  runtime is a performance decision and must be named as one; decision
  0015's proved/boundary/unproved/external-trust classifier is the
  mechanism for naming it.
- **Deterministic execution.** Performance in this class is meaningless
  without specified, deterministic semantics.
- **Fast verification feedback.** Check and CI time is a performance budget
  too -- and for agents it is the tighter one. An agent's loop speed is
  bounded by how fast Hum can tell it that it is wrong. A language that is
  fast at runtime but slow to check is not the best language for agents.
  Honest note: today's ~40-minute CI wall-clock is process cost (the
  fixture re-execution model the validation-cost discipline already
  names), not compiler speed. This goal constrains the compiler's check
  path and the harness design going forward; it does not retroactively
  bill the prototype.

### What it does not commit us to

- **No benchmark claims today.** The tree-walking interpreter is a
  prototype, not a plan. Every claim about Hum's speed today is a claim
  about a tree-walker, and must be labeled as such.
- **No native backend now.** The gates below decide when a backend is
  attempted; this record does not move them.

### Performance debt, not settled choices

The following existing choices are recorded as performance debt in the
perf-debt ledger (`docs/research/2026-09-22-performance-debt-ledger.md`),
not quietly accepted as final:

- **text_split returns owned copies** (decision 0021, "Ownership and cost"):
  views would need element-alias and stored-view ownership relationships
  that decision 0014 locks. Revisit when views are expressible.
- **List growth API**: unsettled (decision 0014, "Not Settled").
  Allocation/growth semantics unnamed.
- **Contract checks deferred to runtime** (decision 0015): each deferral is
  a performance decision; it must be named per check, not made by accident.
- **Termination-measure checks** (decision 0020): adopted; enforcement cost
  in the prototype unnamed.

### The rule going forward

Any decision that trades performance for checker simplicity says so
explicitly in its decision record and gets a perf-debt ledger entry.
Entries resurface when a real backend exists: backend design reviews every
open entry and dispositions it (paid, kept deliberately, or moved).

### Before a native backend is attempted

These gates are tied to the improvement backlog and are **not loosened** by
this record:

- Deterministic semantics specified (backlog item 7).
- Verification architecturally separable from code generation (backlog
  item 7).
- The safety-profile architecture defines what the backend must preserve
  (backlog items 7 and 10 trigger).
- The perf-debt ledger is an input to backend design: every open entry is
  dispositioned before the backend is considered complete.

## Consequences

- Future decision records that trade performance for simplicity must name
  the trade and file a ledger entry; reviewers check this.
- No speed claims about Hum-the-language until a backend past the
  tree-walker exists; prototype speed claims are labeled as prototype
  claims.
- The "not bolt-on-able" record (backlog item 6) is unaffected; this
  strengthens it -- the performance clause is part of what a bolt-on
  cannot supply.
- The ledger entries stand as facts about past decisions.

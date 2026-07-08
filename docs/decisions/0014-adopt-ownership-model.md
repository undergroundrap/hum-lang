# 0014: Adopt Ownership And Borrowing As The Core Ownership Model

Date: 2026-07-08
Status: accepted under delegated authority (BDFL veto open)

## Context

Work Order 2 ran a three-candidate ownership bake-off against the pinned
twelve-program corpus in [../bakeoff/CORPUS.md](../bakeoff/CORPUS.md):

- Candidate A: ownership and borrowing with planned repairs, linear resources,
  and explicit arenas.
- Candidate B: mutable value semantics with second-class references.
- Candidate C: region-first ownership with dynamic regions, unique pointers,
  and explicit RC as priced escape hatches.

The scorecard is [../bakeoff/SCORECARD.md](../bakeoff/SCORECARD.md). It applies
the reviewer notes from Sessions F-H: repair maturity is separated from
with-repairs counts, programs 3 and 4 are marked as effect-polymorphism
interactions, pattern frequency is weighted, hidden copy-on-write and hidden
reference counts are rejected, and signature plumbing is counted for all
candidates.

The core philosophical axis is now explicit:

- Candidate A grows language/checker complexity so natural programs check.
- Candidate B restructures programs so the language stays simpler.
- Candidate C keeps natural programs but makes region annotations pervasive.

## Decision

Adopt Candidate A as Hum's proposed core ownership model:

1. Values are owned by default. Moving a value transfers cleanup authority.
2. `borrow` grants bounded read authority; `change` grants bounded exclusive
   mutation authority; `consume` transfers or closes linear authority.
3. Returned views and stored views must state their source with checked
   relationships such as `Slice Text from text` or `Slice Text from
   parser.buffer`.
4. The checker is place-based: records, fields, list elements, indexes,
   parameters, and returned views are checked as source-visible places.
5. Linear resources are the first-class model for exactly-once protocols such
   as commit/rollback, unregister, send, close, and finish.
6. Arenas and regions survive as explicit opt-in mechanisms for cyclic graphs,
   stable node identity, pool allocation, and profile-specific storage.
7. Internal references, disjoint-field projection, and flow-sensitive borrowing
   are required repairs before the model can be called mature.

This is a proposed decision only. Under the delegated-ruling amendment, this
session does not flip the status to accepted.

## Why This Wins

Candidate A is the best long-term fit because it keeps the high-frequency
programs natural:

- returning a view derived from a parameter stays a returned view, not a range
  token or forced copy;
- parsers can own a buffer and hold a token view into it, once internal
  references are implemented;
- field updates and record updates stay direct, once disjoint-field projection
  is implemented;
- worker handoff and builder finish use ordinary ownership transfer;
- transactions and subscriptions use linear resources, which matches the
  exactly-once nature of the work.

The quantified gate is not free. Candidate A clears eight of twelve programs
without declared escape hatches only when its planned repairs and the
effect-polymorphism gate are counted. Under the stricter proven-today maturity
count, it clears four. The gate should move for this decision because the ADR is
choosing the language's long-term ownership model, not certifying the current
checker. What that concedes: Hum takes on real implementation risk, and Hum must
not claim full ownership safety, internal-reference support, disjoint-field
precision, or memory-safety completeness until those repairs are built and
tested against the corpus.

That debt is preferable to making every common view-returning API pay
coordinate bookkeeping forever, or making every ordinary signature carry region
parameters as the default mental model.

## Consequences

Hum's ownership vocabulary centers on `borrow`, `change`, `consume`, and
checked source relationships. The language should optimize pedagogy,
diagnostics, semantic graph facts, and tooling around those concepts.

The ownership checker roadmap must prioritize:

1. ordinary owned values, moves, `borrow`, `change`, and `consume`;
2. linear resource path checking;
3. returned-view dependencies from parameters;
4. disjoint-field projection;
5. internal references for owned values that store views into their own storage;
6. flow-sensitive borrowing for conditional returns and lending iterators.

Until these exist, reports must stay honest: no full memory-safety, borrow
soundness, or safety-critical readiness claim may be emitted from partial
coverage.

## Alternatives Rejected

Candidate B is rejected as the core model. What dies with it:

- second-class references as Hum's central answer;
- returned ranges or owned copies as the default replacement for returned
  views;
- self-referential parser state expressed primarily as coordinates;
- owner-table restructuring as the ordinary answer to direct relationships.

Candidate C is rejected as the core model. What dies with it:

- named regions as the everyday ownership fact users must write;
- pervasive `in r` parameters in ordinary type and task signatures;
- dynamic regions, unique pointers, or RC as the default escape from lexical
  lifetime limits;
- region-first pedagogy as the main story for beginners.

## Salvage

From Candidate B, keep:

- second-class temporary access as a checker implementation technique;
- range descriptors and owned-copy APIs for serialization, FFI, and no-borrow
  boundaries;
- owner tables and generation handles as explicit library patterns;
- the discipline that a simple value API is often better than an exposed borrow.

From Candidate C, keep:

- explicit arenas and regions for cyclic graphs, parsers, pools, and
  profile-controlled allocation;
- dynamic regions only as a visible, priced escape hatch;
- unique pointers as a visible linear/heap mechanism where arenas are wrong;
- explicit RC only as a source-visible, effect-visible, profile-gated type.

No salvage item may hide allocation, reference counts, mutation, unsafe
responsibility, or profile restrictions.

## Not Settled

Concurrency sharing rules are not settled. Route to a future concurrency/shared
state decision after effect polymorphism and ownership basics land.

Record-update syntax sugar is not settled. Route to a future syntax and stdlib
ergonomics decision after disjoint-field projection has a checker proof.

The list growth API is not settled. Route to stdlib design probes for append,
retain, builder finish, element-view invalidation, and profile-specific
allocation behavior.

Tasks-as-values and effect polymorphism are not settled. Programs 3 and 4
remain gated on the effect-polymorphism decision; monomorphic callback effects
are not an acceptable long-term answer for `retain`, callbacks, `map`, `fold`,
or retry-style APIs.

Contract-check-mode policy is not settled. The Session D observation that
`divide`'s body guard is unreachable when `needs:` checks are on remains in the
friction ledger for the future contract-check-mode decision.

## BDFL Note

The hard call is not whether Candidate A is easier to implement today. It is
not. The hard call is whether Hum should make common systems programs natural
and pay for a serious checker, or keep the checker simpler by asking users to
write less direct programs. This proposal chooses the serious checker, with the
claims locked down until the evidence exists.

## Ruling

Accepted 2026-07-08 by the architect-reviewer under the delegated-ruling
process in [../GOVERNANCE.md](../GOVERNANCE.md).

Basis: the scorecard was verified against all six standing reviewer notes
and applies them symmetrically to every candidate. The gate-move is
accepted because this decision selects the long-term model, not the
current checker, and the concession is bound by explicit honesty locks:
no full ownership-safety, internal-reference, disjoint-field, or
memory-safety-completeness claim until each repair is implemented and
passes the corpus as probe programs. Candidate B's superior proven-today
count (5 versus 4) was weighed and overridden by pattern frequency: B's
tax is permanent and lands on pervasive program shapes; A's debt is
temporary and claim-locked.

The BDFL veto is open. This ruling reverses with one recorded sentence at
any time before implementation hardens around it.
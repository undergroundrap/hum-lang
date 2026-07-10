# 0015: Adopt Evidence-Classified Runtime Contract Checking

Date: 2026-07-09
Status: accepted under delegated authority (BDFL veto open)

## Context

The friction ledger records a real policy conflict. `examples/core/divide.hum`
declares `needs: b != 0` and also keeps a typed body guard for division by zero.
With current `hum run` behavior, the checked entry contract makes that guard
unreachable on the ordinary path. Deleting the guard would be premature,
however, because Hum does not yet classify trust boundaries, prove contracts,
or select an enforced runtime contract profile.

Session U reapplied the three-strike rule after Predicate v1 shipped. Contracts
remain triggered, and the contract-check-mode question is the mandated policy
decision. The distilled contract research rejects both a universal
"always-on" answer and a debug-only answer. Mature systems distinguish proved
internal obligations from boundary and unproved obligations.

## Decision

This decision defines the classifications that a future checker must assign;
current Hum assigns none. Every contract eligible for mode-dependent treatment
must eventually be classified as one of:

- `proved`: mechanically established for the relevant build inputs;
- `boundary`: enforced where authority or trust crosses a program boundary;
- `unproved`: checked but not mechanically established;
- `external-trust`: depends on data, code, or authority outside Hum's proof.

Classification is conservative and ordered. An obligation at a declared trust
or authority crossing is `boundary`; otherwise one whose transitive proof
inputs depend outside Hum is `external-trust`; otherwise a mechanically
established obligation is `proved`; every remaining or unknown obligation is
`unproved`. The first matching class wins, so `proved` can never override a
boundary or external dependency.

The runtime policy is:

1. Current `hum run` continues checking every executable predicate contract.
   This decision does not implement a classifier, profile selector, or elision.
2. Any future debug or test execution mode must check every executable
   contract.
3. A future release mode may elide only a mechanically proved contract whose
   transitive proof inputs remain inside the proved Hum unit and cross no
   declared trust or authority boundary.
4. Boundary, unproved, and external-trust contracts remain checked. Unknown
   classification fails closed as `unproved`.
5. A `needs:` contract is not a substitute for parsing or validating external
   input. External data enters as untrusted until a checked boundary validates
   it.
6. No body guard is called unreachable, removed, or elided until the contract
   classification and enforcement evidence make that claim true.
7. Any elision must emit machine-readable classification and proof evidence
   tied to the source, compiler, profile, target, and relevant proof inputs.

## Consequences

Hum keeps current behavior simple while reserving a policy that can eventually
avoid redundant internal checks without weakening trust boundaries. IO makes
the distinction concrete: filesystem results, runner grants, host adapters,
and other external authority cannot acquire `proved` status merely because a
caller wrote `needs:`.

The future classifier, profile syntax, evidence fields, and optimization are
implementation work. Until they ship, output must say that all recognized
predicate contracts run and that no contract has been mechanically classified
or elided.

Higher-order blame is not settled. Tasks-as-values and effect polymorphism must
land before Hum claims correct higher-order contract blame.

## Alternatives Rejected

### Strip Contracts In Release Builds

Rejected. It removes checks precisely where external and unproved obligations
still matter.

### Debug-Only Contracts

Rejected. Debug coverage is not authority or proof, and boundary validation
cannot depend on build mode.

### Check Every Contract Forever

Rejected as a permanent policy. A mechanically proved internal obligation may
be safely elided when equivalent evidence is emitted.

### User-Selected Global On/Off Switch

Rejected. A global knob erases the distinction between proved internals and
untrusted boundaries and cannot support honest evidence.

### Delete Defensive Guards When `needs:` Exists

Rejected. Presence of contract text does not establish classification,
enforcement, or trust.

## BDFL Note

This decision chooses the evidence boundary, not a performance mode. The safe
default remains checking. Optimization is earned only by proof, and external
trust never becomes internal proof by assertion.

## Ruling

Accepted 2026-07-09 by the architect-reviewer under the delegated-ruling
process in [../GOVERNANCE.md](../GOVERNANCE.md).

Basis: the corrected Session U ledger leaves contracts triggered after
Predicate v1, current runtime behavior was independently verified, and the
distilled research supports classification rather than a global mode. The
ruling changes no current runtime behavior and makes no classifier, profile,
proof, or elision claim.

The BDFL veto is open. This ruling reverses with one recorded sentence at any
time before implementation hardens around it.

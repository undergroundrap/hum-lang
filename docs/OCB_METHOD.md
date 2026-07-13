# The OCB Method: Orthogonal, Causal, Bound

Date: 2026-07-12
Status: synthesis of the method as actually practiced in this repository.
Author: Ocean Bennett (OCB). This document is descriptive, not marketing.

## What this is

OCB is the engineering method this project actually ran to build Hum with
semi-autonomous AI agents. It is synthesized from the repository's real
practice -- the governance rules, agent runbooks, decision records, work
orders, and friction ledger cited throughout -- not from aspiration. Every
claim below points to where it is enforced in the repo.

The name is a functional backronym -- Orthogonal, Causal, Bound -- that
happens to carry the author's initials. The three words are picked for what
they mean, not whose they are.

## Honest scope

Two of the three pillars are domain principles for semantic-pipeline
systems (compilers, analyzers, data-lineage engines). One is a general
engineering method that transfers to any software project. This document
keeps that line explicit rather than pretending three compiler principles
are universal laws.

- General method (transfers anywhere): **Bound**, plus the process
  disciplines in Part 2.
- Domain principles (transfer to systems with identity and data flow):
  **Orthogonal**, **Causal**.

## What this is NOT

The method does not produce mathematical proofs, "flawless" software, or a
"self-correcting factory." Hum's contracts are runtime-checked, not proven.
The method has been validated on one project, by one person plus agents,
and it is slow: review cycles cost real hours. Its power is not perfection.
Its power is that it catches more, earlier, and refuses to trust its own
agents. A dishonest description of an honesty method is self-refuting, so
this document holds itself to the same standard it teaches.

---

## Part 1: The Three Pillars

### Bound (general method): zero-trust promotion through adversarial review

Principle: implementation and review are strictly separate, competing
roles, and nothing merges on an agent's word.

As practiced:

- Implementer and reviewer are different agents, preferably different model
  families, so blind spots are not shared (`AGENTS.md`, Role Runbooks).
- The reviewer never trusts the implementer's report. It re-runs the
  fixtures, the misuse cases, the tests, and the full preflight itself.
  "Tests pass" from the implementer is a claim to verify, never evidence.
  This is the single most load-bearing rule in the method.
- The reviewer runs a fixed probe set on every change: a name-identity
  attack set, a fail-closed check, a positive-evidence rule, precedence
  probes, masking analysis, a docs-claims sweep, and P0/P1/P2 severity
  tagging (`AGENTS.md`, review probe sets, distilled 2026-07-09).
- Verdicts are accept / accept-with-required-fix / reject with reasons.
  One session, one verdict, then a hard stop.
- Even the reviewer is reviewed: work orders and decision records get an
  independent pre-issuance pass; delegated rulings carry a standing BDFL
  veto (`GOVERNANCE.md`, Delegated Ruling; Pre-issuance review).
- The acceptance gate is a multi-platform CI loop proving functional
  behavior and structural discipline together, not just a green local run.

This is the pillar that transfers to any project: separate the hands from
the eyes, make the eyes independent, and never let code merge because the
author said it works.

### Orthogonal (domain principle): identity is intrinsic, not derived from presentation

Principle: a fact's identity is decoupled from how it is displayed.

As practiced in Hum:

- A diagnostic, cause, or semantic fact carries an opaque, owner-assigned
  identity from the moment it is born at the producer (parser, resolver,
  analyzer). It is not reconstructed from filenames, string names, line
  numbers, or source spans (Work Order 9, Sessions AN-AP; the diagnostic
  registry as single source of truth).
- Rendering consumes identity; it never mints it.

General form (transfers to data systems): identity should be intrinsic and
provenance-bearing, never inferred from a presentation layer that can drift
or collide. It does not apply to software without a meaningful identity/
representation split.

### Causal (domain principle): provenance is transported, not reconstructed

Principle: downstream stages consume upstream facts; they never guess them.

As practiced in Hum:

- Typed failure carries its causal origin, propagation sites, and wrapping
  sites explicitly; nothing is heuristically reconstructed (decision 0016,
  causal typed failure).
- A downstream check that could re-derive an upstream fact from its own
  projection must instead compare against the authoritative upstream set,
  or fail closed. Ambiguity is an error, not a guess (Work Order 9 review
  gaps).
- Fail-closed is the default: missing, duplicate, or ambiguous associations
  crash at compile time (exit status 2) rather than proceeding on a guess.

General form (transfers to pipelines): carry provenance through every
stage; never let a later stage reconstruct what an earlier stage owned.

---

## Part 2: The Process Disciplines (all general)

These are the transferable practices that surround the Bound pillar.

### Synthesis-first decision gating

Research happens at decision points, never ambiently. Before a major design
choice, synthesize historical failures and state-of-the-art literature into
dated, confidence-labeled snapshots (`docs/research/`), then decide. Research
is input, never authority: repository evidence and accepted decisions
override research prose whenever they disagree. When implementation hits a
real architectural ambiguity, the pipeline pauses for a scoped research pass
rather than a hotfix. (Seen across the ownership, effect, stdlib, and
capability research triages.)

### The bake-off for lock-in decisions

For the handful of choices with long-term lock-in and genuinely competing
options, run a bake-off: pin a model-neutral corpus first, have each
candidate written by its best advocate, score against the frozen corpus
crediting no unimplemented machinery, salvage the losers' surviving ideas,
and land the result as a decision record with a veto open. (`GOVERNANCE.md`,
Bake-Off Doctrine; decisions 0014 ownership, 0018 effects.)

### Work orders with bounded sessions

Work proceeds in lettered sessions inside issued work orders. Each session
has one deliverable, hard acceptance criteria, and a full stop for review.
Sessions do not batch, and the next does not start without an explicit go
signal. (`AGENTS.md`, Session rhythm.)

### Honesty locks

No output -- code, docs, or reports -- may claim more than the checker
proves. Locks narrow only by shipping the feature that retires them, never
by rewording. Every checker emits an explicit non-claims list. (Enforced
across every work order.)

### Docs as CI invariant

Documentation is validated in CI, not trusted to stay current. Fixture-
extracted examples must match the code; the diagnostic catalog must match
the registry; a docs-claims sweep flags prose that overclaims the shipped
rule. Drift fails the build. (`tools/check_all.ps1`; the docs-claims sweep
probe.)

### Decision records that kill alternatives

Every major decision is recorded with context, the decision, consequences,
alternatives rejected (what dies), and a salvage note. Nineteen such records
exist. Rulings are transparent about their authority (BDFL-direct vs
delegated-with-veto). (`docs/decisions/`.)

### The friction ledger and the three-strike rule

Design pain is recorded in a structured ledger (wanted / forced / severity /
indicts / proposal). Three unresolved records indicting one area
mechanically trigger a decision record or work-order item. Pain converts to
policy by rule, not by mood. (`docs/CORE_LANGUAGE_SHAPE.md`.)

### Portable roles, repo-first state

Roles are defined by the repo, not by which model plays them, so any capable
agent can assume either role cold. There is no handoff document to maintain:
state lives in git history, the active work order, and the decisions index.
Anything that lives only in someone's head is a defect to write down.
(`AGENTS.md`, Role Runbooks and Handoff.)

---

## Honest costs and failure modes

The method is not free, and this section is part of the method.

- It is slow. Adversarial review and per-session stops cost hours per
  session. Feature sessions pass cleanly; consolidation sessions grind.
- Over-tight envelopes backfire. When a work order partitions a cross-
  cutting change into an exact per-file list, it generates governance-
  boundary rejections (file-not-on-list) instead of correctness rejections.
  Observed live in Work Order 9 Session AP: the implementation was sound but
  touched one file the map omitted, and review stopped at the envelope. The
  lesson: cross-cutting semantic work needs envelopes scoped by intent
  ("the producer files needed to establish X"), not by a pre-guessed exact
  list.
- It is validated at n=1. One project, one BDFL, agent implementers. It has
  not been shown to scale to human teams or to beat conventional methods in
  a controlled comparison. The scaling notes in `GOVERNANCE.md` pre-decide
  the many-crew case but have not been exercised.
- Its credibility is downstream of the project's success. The method's
  public value arrives when the project it built visibly works, not before.

## Provenance

Synthesized 2026-07-12 from `GOVERNANCE.md`, `AGENTS.md`,
`docs/decisions/` (0001-0019), the Work Order history in git, the friction
ledger in `docs/CORE_LANGUAGE_SHAPE.md`, and the research triage snapshots
in `docs/research/`. Where this document and those sources disagree, the
sources win and this document is the bug.

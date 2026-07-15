# Why Hum: The Honest Competitive Case

Date: 2026-07-15
Status: positioning synthesis. Consolidates the per-language lessons in
`CROSS_LANGUAGE_REGRET_LEDGER.md`, the accepted decisions in
`docs/decisions/`, and the go-to-market posture in
`ADOPTION_STRATEGY_2026.md` into one place. Scope is the incumbents a
systems audience actually measures Hum against -- Rust, Zig, Go, C++.
Comparisons to niche or adjacent-market languages live in their own
design-input docs, not here.

## Reading rule: what the maturity tags mean

Every claim below is tagged, so this document can never quietly overclaim:

- **Designed** -- decided in an accepted decision record, not yet built.
- **Checked** -- enforced today by a real checker at the interpreter/analyzer
  level (a named `src/*.rs`), on real fixtures.
- **Shipped** -- works end-to-end in compiled, distributable output.

As of this date, Hum has **Designed** and **Checked** claims. It has almost
no **Shipped** claims, because the executable backend is not built yet. A
reader should weight the tags accordingly. The point of this document is to
be believed, and it can only be believed if it is honest about that line.

## Posture (inherited from the adoption strategy)

Hum does not claim to have replaced Rust, C++, or Go. The stated posture is:
compete later as a full systems language; win first as an evidence-native
offline tool. This document describes the *designed* competitive case, not a
shipped verdict. It exists to make the case legible and testable, not to
declare victory.

---

## The one-sentence thesis

Keep Rust's safety soul; file down the ergonomic cliffs; and add the two
things no mainstream systems language has first-class -- **checked intent**
(contracts with blame) and **checked authority** (capability-based effects).

Everything below is that sentence, expanded and sourced.

---

## Axis 1 -- Memory safety without the learning cliff

- **The incumbent pain:** Rust's most-cited regret in our ledger is the
  "ownership learning cliff." Zig's critics say it hands memory safety back
  to programmer discipline entirely.
- **Hum's answer (Checked):** an ownership model (decision 0014,
  `src/ownership_check.rs`) with borrow/change/consume, moves, field and
  element views, and linear resources -- enforced today. Its design rule:
  *"ownership diagnostics explain responsibility and permission,"* rather
  than making every programmer reconstruct the borrow checker's model.
- **Honest standing:** on *guarantees*, this is parity with Rust, not a win
  -- and it is proven at the checker level, not in compiled output. The
  intended win is on the *diagnostics*, and that win is unproven until real
  users hit real ownership errors and report whether the messages actually
  teach. This is a claim to earn, not to bank.

## Axis 2 -- Checked intent (contracts with blame)

- **The incumbent gap:** Rust has `debug_assert!` and prose invariants. No
  mainstream systems language makes preconditions, postconditions, and
  pre-state (`old()`) first-class with caller/task blame.
- **Hum's answer (Checked):** executable contracts (decision 0015,
  `src/core_contract.rs`) -- `needs:` / `ensures:` / `old()` checked at
  runtime with caller-vs-task attribution.
- **Honest standing:** genuinely a new axis versus the incumbents, and it
  runs today. Caveat from our own honesty locks: these are *runtime-checked*,
  not statically proven. Do not describe them as proofs.

## Axis 3 -- Checked authority (capability-based effects)

- **The incumbent gap:** in Rust, any code that can call a function can do
  its IO. There is no deny-by-default authority in the language.
- **Hum's answer (Checked):** structural app-authority boundary (decision
  0017, `src/capabilities.rs`, `src/capability_root.rs`) plus effect
  checking (`src/effect_check.rs`, row-polymorphic effects in decision
  0018). Authority is deny-by-default and granted explicitly.
- **Honest standing:** a category the incumbents do not have at the language
  level. Strongest differentiator on paper. Still Checked, not Shipped.

## Axis 4 -- Errors that carry their cause

- **The incumbent pain:** our ledger flags "unsafe requirements living in
  prose" and error handling that is either boilerplate (Go) or hand-plumbed.
- **Hum's answer (Checked):** nominal causal typed failure (decision 0016,
  `src/typed_failure.rs`) -- failures carry origin, propagation, and
  wrapping sites explicitly, with `fails when:` instead of exceptions or
  unchecked error values. Fail-closed is the default: ambiguity is a compile
  error, not a guess.
- **Honest standing:** better than Go's boilerplate and Rust's prose
  invariants on the provenance axis; Checked today.

## Axis 5 -- Power without a second language

- **The incumbent pain:** Rust macros and trait solving, Zig's `comptime`
  and `anytype` sprawl -- compile-time machinery that becomes an unbounded
  second language.
- **Hum's answer (Designed):** compile-time execution is delayed and
  budgeted; macros must be visible in the semantic graph; generics use
  explicit bounds rather than duck typing.
- **Honest standing:** mostly **Designed**, not yet Checked. This is a
  promise about restraint, and restraint is only proven by what ships.

---

## Where Hum is behind -- and it is not close

This section is the reason the document is credible. It is not an appendix;
it is the current reality. Sourced from `ADOPTION_STRATEGY_2026.md` "Top
Blockers" and the research triage in `docs/research/`.

1. **No executable backend yet.** Every axis above is proven at the
   interpreter/checker level. Rust, Zig, and Go emit optimized native
   binaries today. Until Hum's backend emits binaries, the honest answer to
   "can it build the thing I ship?" is *not yet*. This single gap outweighs
   every advantage above.
2. **No ecosystem.** Rust has ~150k crates, a mature LSP, debuggers, and a
   decade of production hardening. Hum has a stdlib *strategy* and a
   pre-self-host checklist (`docs/research/2026-07-12-minimum-compiler-
   ready-stdlib-triage.md`), not a stdlib.
3. **No async/concurrency model.** Deliberately deferred until cancellation,
   allocation, effects, and executor boundaries are clear. The right call --
   but it means a large class of Rust's real workloads has no Hum story yet,
   on purpose.
4. **No FFI/interop story shipped.** Credible C ABI / interop is a named
   top blocker, not a solved problem.
5. **No reproducible-build / provenance / SBOM tooling shipped.** These are
   central to the offline-tool wedge and are still ahead of us.
6. **Tooling gaps.** Formatter, LSP, docs, test runner, benchmark runner,
   stable machine-readable diagnostics -- named blockers, partially designed.

The pattern, stated in the adoption strategy and repeated here without
softening: **Hum's risk is product completeness, not syntax novelty.** The
design case is strong and largely decided. The shipping case has barely
begun.

---

## How to use this document

- **Do** use the Designed/Checked axes to explain what Hum *is for* and why
  its shape is different from Rust and Zig.
- **Do** lead with Axes 2 and 3 (contracts, capabilities) -- they are the
  things the incumbents structurally lack, not just do differently.
- **Do not** make "Rust replacement" claims. The posture is compete-later.
- **Do not** upgrade a Checked claim to a Shipped one in any external
  material. When the backend ships, update the tags here first, then speak.

## Provenance

Synthesized 2026-07-15 from `CROSS_LANGUAGE_REGRET_LEDGER.md` (Rust, Zig, Go
lessons), `docs/decisions/` 0014-0018, `ADOPTION_STRATEGY_2026.md`
(Competition Stance, Top Blockers), and the stdlib research triage in
`docs/research/`. Where this document and those sources
disagree, the sources win and this document is the bug. Maturity tags must
be re-audited whenever the backend or stdlib status changes.

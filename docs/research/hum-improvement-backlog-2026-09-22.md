# Hum Improvement Backlog — Prioritized, Research-Only

Date: 2026-09-22. Baseline: pinned commit `ca3f7cf5a5fe004e855de3df7b9bbd1d827bf9a4`.
Primary source: `workspace/user/files/hum-research-2026-09-20.zip` (round-one package).
Later `~/workspace/hum-research` files are **supplemental later research**, not attached-package content.
Research and recommendations only. Ocean decides what becomes work.

## Verdict first

**No. Ocean did not design a bad language.**

The highest-leverage gaps below are *completion and evidence* gaps, not proof the core
ownership or intent choices were wrong. Decision 0014's own ruling states the choice
plainly: Candidate A was chosen because it keeps common systems patterns natural and
pays for a more serious checker — the difficult-but-serious model, not the wrong model.
The 2026-09-20 critique landed as a verdict on *completion* (how much of the model is
proven today), not on the model choice. Keep those separate.

## Score-ambiguity note

The archive does **not** currently substantiate a literal "5/10" design score.
What it substantiates, all from the pinned tree:

- **8/12** programs cleared with planned repairs + the effect-polymorphism gate counted
  (the decision gate — choosing the long-term model, not certifying the checker).
- **4/12** proven-today for Candidate A under the stricter maturity count (decision 0014).
- **5/12** proven-today for Candidate B, weighed and deliberately overridden by pattern
  frequency (the ruling).

The remembered "5/10" is most plausibly a blend of these figures. The complete
2026-09-20 08:36:40 / 08:39:10 UTC messages could not be recovered from the archive;
the dated summary of that exchange is the authority worked from here.

## How to read provenance

- **[Pinned fact]** — read directly from the pinned tree or the attached ZIP.
- **[External]** — from web research (landscape notes, supplemental).
- **[Inference]** — my synthesis from the sources; argue with it.
- **[Unexecuted]** — proposed in research notes; nothing built or tested.

---

## 1. wordfreq: the first real program — NOW, no new language surface
**[Pinned fact + Inference]**

**Weakness it answers:** Every utility claim is currently asserted, not demonstrated.
The agent-first/human-review thesis is coherent but unevidenced [Pinned fact, REPORT.md].
A real program finds probe-grade rough edges faster than probes do [Inference,
three_program_sequence.md].

**Smallest credible change:** None to the language. Write `examples/tools/wordfreq.hum`
composing only implemented probes (capability-root file reads, bounded stdout, fallible
app entry, recognized `list_count`/`list_len`/`old(...)` predicates), and record every
friction point classified as program / tooling / documentation / unresolved-language-question.
The decision-ready appendix already exists in the research notes [Unexecuted].

**Evidence bar:** `hum check` clean with honesty locks intact; positive + boundary + misuse
cases through `hum run` (misuse fails closed with typed errors); `hum evidence` shows every
`needs:`/`ensures:` obligation linked; an explicit statement of what the evidence does
**not** prove; a friction ledger; a record that no language extension was required.

**Leverage:** Maximum information per effort. It adjudicates the adoption thesis (can a human
review the intent sections instead of the body?), the Zero counterargument (does intent need
to live in the artifact at all? [External]), and produces the corpus programs items 3–5
consume.

**Domain scrutiny:** For medical/aerospace/OS/engine claims, wordfreq proves nothing about
certification — say so in the completion record. Its value is as the first *real* program
under capability boundaries, the shape every later safety argument needs.

**Blast radius:** New `examples/tools/` directory (taxonomy decision); no checker/parser
changes; possible interpreter fixes in `src/run.rs` as separate reviewed commits;
`docs/` work-order record only; CI fixture family: new `.hum` example corpus, no schema
changes.

**Timing:** Now. It is the prerequisite for nearly everything below.

---

## 2. Replace the self-selected 12-program corpus — NOW, alongside
**[Pinned fact + Inference]**

**Weakness:** The ownership bake-off ran against a 12-program corpus selected by Ocean;
frequency weighting reduced but did not remove selection bias [Pinned fact, from the
2026-09-20 assessment]. A corpus you chose can only confirm what you already suspected
[Inference].

**Smallest credible change:** No language change. Commission external programs: real-world
parser/iterator/builder/worker-handoff shapes from outside Ocean's circle, plus adversarial
cases aimed at the four debt items (item 4). Record provenance per program.

**Evidence bar:** Each new program has a provenance note; re-run the Candidate-A scorecard
against it; publish where the score moves and where it doesn't.

**Leverage:** Every checker claim below (items 4–5) is only as credible as the corpus it
passes against. This is the cheapest credibility upgrade available.

**Blast radius:** Bake-off corpus files only; scorecard docs; no compiler, diagnostic, or
schema changes.

**Timing:** Now, in parallel with wordfreq. Research/evidence work, not syntax.

---

## 3. Human-review evidence — with wordfreq, cheap precursor first
**[Pinned fact + Unexecuted]**

**Weakness:** "Beginner story" ratings were asserted; no human study exists; wordfreq with
a real first user was named as the first genuine data point [Pinned fact, from the
2026-09-20 assessment].

**Smallest credible change:** None to the language. Run the corpus-study precursor first
(zero participants): take agent-written Hum tasks, strip intent sections, have independent
raters judge whether each section states anything the body doesn't already make obvious.
If a large share is body-redundant, the ceremony problem is real before any lab study runs
[Unexecuted proposal, usability_evaluation_sketch.md].

**Evidence bar:** Pre-registered predictions (the sketch names three, including the
contract-erosion cell as the most important); reported confidence intervals, not just
p-values; the ceremony detector (reported section use vs. sections that predicted
detection).

**Leverage:** This is the experiment the whole "Rust for agents" thesis stands on. It also
feeds item 9 (erosion) — the weakened-`ensures:` cell is the direct measurement of
contract erosion by agents.

**Domain scrutiny:** Medical/aerospace reviewers will ask "who verified the verifier's
input?" — a human-review study is the closest thing to an answer. Do not claim benefit
before it runs.

**Blast radius:** No compiler changes; new study materials and rater protocols as docs;
possibly new example programs.

**Timing:** Precursor with wordfreq; full study after the precursor says the surface is
worth testing.

---

## 4. Burn down the ownership debt — after wordfreq, claim-bounded
**[Pinned fact]**

**Weakness:** Decision 0014 accepted Candidate A with four named repairs outstanding;
Session V narrowed only the exact local direct-field slice. The remaining lock is broad:
no general aliases, stored aliases, internal references, nested-place projection, element
aliases, general flow-sensitive borrowing, complete borrow soundness, full ownership
safety, or memory-safety completeness [Pinned fact, decision 0014 + ROADMAP_LEDGER.md].

**Smallest credible change:** Architecture/checker completion, in the ledger's order, each
as a claim-bounded evidence campaign: (1) internal references — the parser-holds-view-
into-own-buffer pattern; (2) general disjoint-field projection — beyond the Session-V
slice; (3) flow-sensitive borrowing — control-flow-sensitive cases with acceptance *and*
rejection evidence; (4) general linear resources — a source-visible general mechanism, not
special-case-shaped results. No ownership-model replacement is on the table; the debt is
completion, not redesign [Inference from the ruling].

**Evidence bar (per stage):** Direct acceptance evidence for the intended pattern; relevant
invalid cases remain rejected (negative evidence preserved); claims name the exact accepted
shape; a checker improvement does not become a language guarantee until stable in the
declared evidence set (the ledger's honesty locks).

**Leverage:** Unblocks the programs the corpus says are high-frequency (parsers, iterators,
builders). Each stage that lands raises the proven-today count honestly.

**Domain scrutiny:** Memory-safety completeness is the load-bearing claim for OS/engine work
and a prerequisite input for any safety argument. Until a stage is proven, the
corresponding claim stays locked — the decision already requires this.

**Blast radius:** Checker internals; ownership-trap diagnostics (H0802/H0808/H0809 family
and neighbors); ownership fixtures (positive + misuse); graph facts for place
relationships; DIAGNOSTICS.md wording (docs-claims sweep per the repo's own review
discipline). CI: ownership fixture selectors and the canonical-seal pair matrix if
parser/checker paths move.

**Timing:** After wordfreq. Utility evidence first, so checker work chases real friction,
not imagined shapes.

---

## 5. Measure checked-vs-unchecked contract coverage; implement the 0015 classifier narrowly — before the offline alpha
**[Pinned fact + Unexecuted]**

**Weakness:** `why`/`protects`/`trusts`/`avoids`/`tradeoffs`/`optimizes` remain
declared-only; predicate classification exists but its value depends on realistic
contracts not mostly falling into unchecked prose; decision 0015's
proved/boundary/unproved/external-trust classifier is documented but unimplemented
[Pinned fact, REPORT.md]. Seventeen sections may encourage checklist ceremony —
measurement, not assumption [Pinned fact].

**Smallest credible change:** No new sections, no deletions — the visible intent structure
is part of the product. First, measure: over wordfreq + the corpus, what fraction of
contract lines are recognized/checked vs. `UNCHECKED_PROSE_CONTRACT`. Then implement the
0015 classifier only where the measurement shows a blocker, growing the predicate ladder
one rung at a time (e.g., an ordering predicate only if a real program needs it — the
`ordered_insert` question, kept deferred until then).

**Evidence bar:** A coverage report with per-section checked/declared ratios on real
programs; each new classifier rung ships with positive + misuse fixtures and a decision
record for "why this predicate but not its generalization" (the `is_sorted`-vs-`forall`
problem the notes already name).

**Leverage:** The honesty-lock machinery made quantitative. Feeds the offline alpha:
reviewers need to know which promises the compiler actually enforces.

**Domain scrutiny:** For medical/aerospace, the checked/declared ratio *is* the
qualification-relevance story. A contract suite that is 80% prose is documentation, not
evidence — the measurement forces that conversation early.

**Blast radius:** `src/predicate.rs` recognizer; `UNCHECKED_PROSE_CONTRACT` diagnostics;
`hum evidence` obligation statuses; evidence JSON schemas if new statuses are added;
LANGUAGE_REFERENCE.md predicate docs.

**Timing:** Before the offline-tool alpha. The alpha's review story depends on it.

---

## 6. Write the "not bolt-on-able" decision record — before broad feature growth
**[External + Inference]**

**Weakness:** The landscape's sharpest counterargument: everything except the intent ladder
could be bolted onto rustc tomorrow as JSON output + a linter. If intent sections turn out
to be expressible as Rust attributes/macros + a linter, Hum's language bet weakens to
"a really good toolchain" [External, supplemental zero-lang.md]. Currently unanswered
[Inference].

**Smallest credible change:** A decision record, not code: state precisely which Hum
property requires the semantic core and cannot be bolted on. Candidates from the notes:
checked source relationships for returned/stored views (decision 0014 §3), the
checked-vs-declared labeling as a language-level machine-readable construct, the
predicate ladder's evaluation semantics.

**Evidence bar:** The record names the property, the bolt-on attempt that fails, and what
dies if the claim is wrong. Ocean approves, revises, or rejects — it is his call whether
Hum is a language project or a toolchain project.

**Leverage:** Determines the shape of every later investment. A toolchain project should
be stealing AI-Co's diagnostic contract and PyBun's SARIF/CycloneDX emission, not growing
a checker.

**Blast radius:** Docs only (`docs/decisions/`). No compiler, fixture, or schema changes.

**Timing:** Before broad feature growth. It is the fork in the road.

---

## 7. Safety-profile architecture — before safety claims scale
**[Supplemental pinned-derivable fact + Unexecuted]**

**Weakness:** Hum is "certification-shaped, not certification-sufficient": no baselined
requirements layer, no requirements-based vs. structural coverage distinction or MC/DC
story, no independent evidence/tool qualification, no deterministic specified execution,
no defined safety subset [supplemental later research, REPORT.md round 2 / ROADMAP.md].
Safety profiles should emit evidence rather than act as labels [Unexecuted].

**Smallest credible change:** Architecture decisions first, per the ledger: (a) profiles
emit qualification-relevant evidence, not labels; (b) deterministic semantics settled
before broad feature growth; (c) verification stays architecturally separable from code
generation. Then the smallest profile that can emit meaningful evidence for the selected
tool path.

**Evidence bar:** An owner-approved architectural statement; a profile that produces
evidence a third party can check (not a label); determinism specified before features
multiply.

**Leverage:** Without this, every safety-domain conversation is premature. With it,
medical/aerospace engagement has a defined on-ramp.

**Domain scrutiny:** This *is* the domain-scrutiny item. For medical devices:
requirements traceability with stable IDs and change control. For aerospace: the MC/DC
and structural-coverage story. For OS/engines: determinism and the safety subset boundary.
None of these exist yet — the backlog must not imply otherwise.

**Blast radius:** New profile machinery in the compiler; evidence schemas (new emitted
artifact kinds); docs (safety doctrine); CI: new profile-gated fixture selectors. Do not
let it leak into the general language surface — profiles are opt-in.

**Timing:** Before scaling any safety claim. It is a decision phase, not a program.

---

## 8. Transitive, machine-checked allocation claims — before engine/OS claims
**[Supplemental + Unexecuted]**

**Weakness:** `allocates:` is honesty-checked for contradictions but not semantically
verified; engine/OS claims need a checked, call-graph-transitive allocation effect, which
does not exist [supplemental later research, notes/engines-os.md; pinned fact that
`allocates:` is contradiction-checked only, three_program_sequence.md].

**Smallest credible change:** An allocation effect that is call-graph-transitive and
machine-checked — the proposed direction from the engines/OS note [Unexecuted]. Start with
the wordfreq-shaped claim ("one line buffer at a time") as the proving ground: either the
compiler verifies it or the honesty lock stays visible.

**Evidence bar:** A program whose `allocates:` claim the compiler checks transitively
across calls; misuse (claim violated by a callee) produces a diagnostic; the wordfreq
appendix's open question — "may `allocates:` prose ship unverified in a human-facing
example?" — is answered by owner decision.

**Leverage:** The engine/OS credibility gate. Game engines and operating systems live or
die on allocation discipline; a declared-only `allocates:` is a comment.

**Blast radius:** `src/resource_check.rs`; new diagnostics; resource-report evidence format
(`hum.resource_report.v0`); examples carrying `allocates:` lines; docs.

**Timing:** Before any engine/OS claim. After wordfreq (which surfaces the real allocation
shapes).

---

## 9. Contract-regression and erosion gates — before trusting agent-authored contracts at scale
**[External + Unexecuted]**

**Weakness:** The highest-risk erosion vector for agent-authored code is an agent weakening
a contract to make the verifier pass; the literature doesn't name "contract erosion" but
attests its components (assertion disabling, C++26's `ignore` semantics, test-weakening as
an anti-pattern) [External, supplemental language_lessons.md]. No mechanism exists yet
[Unexecuted, supplemental dbc-agent-era.md].

**Smallest credible change:** Process + tooling, not language: refinement-diff gates
(contract edits are higher-privilege than code edits and require evidence/justification),
laundering detectors, privileged contract review. The BASE methodology's ratchet is the
industry precedent: a `needs:` that becomes a checked contract must never silently
regress to prose [External, supplemental agent-first-landscape.md].

**Evidence bar:** A contract edit without justification fails the gate; a weakened
`ensures:` that hides a real bug is caught (this is the usability study's most important
cell — item 3 measures it); the ladder state is versioned per task.

**Leverage:** The offline-tool alpha puts agents in the loop writing contracts. Without
erosion gates, the alpha's evidence rots silently.

**Blast radius:** CI gates (new required checks); evidence formats (ladder-state
versioning); work-order/review process docs; no language-surface change.

**Timing:** Before the offline alpha trusts agent-authored contracts at scale. The study
in item 3 provides the measurement.

---

## 10. Defer with triggers — evidence-format polish, agent interfaces, geography, bootstrap, backend ladder
**[Mostly Unexecuted]**

Real items, each with a trigger — invest when the trigger fires, not before:

- **Normative diagnostic/evidence contract** (stable machine-readable codes à la AI-Co;
  SARIF/CycloneDX emission à la PyBun) [External, Unexecuted]. Trigger: a tool or CI
  consumer needs to parse Hum output. Until then, the existing schema-based diagnostics
  suffice.
- **Version-matched agent skills served by the compiler** (`hum skills get <topic>`,
  Zero's lesson) [External, Unexecuted]. Trigger: doc-drift causes a real agent failure.
- **Structural agent interface** (`hum graph --json`, impact/slice) [External,
  Unexecuted]. Trigger: the Ralph-loop repair bottleneck is measured, not assumed.
- **`protects:` mapped onto WASI capabilities** [External, Unexecuted]. Trigger: a WASM
  compile target exists.
- **Contractual program geography** [supplemental, Unexecuted]. Trigger: corpus +
  navigation evidence; begin as convention/warning, never harden prematurely.
- **Bootstrap/self-hosting** [Pinned fact + Inference]. Trigger: utility and evidence
  exist first — self-hosting is a leverage milestone, not a substitute.
- **Beyond the tree-walker** (general native backend) [Pinned fact]. Trigger: the
  safety-profile architecture (item 7) defines what the backend must preserve;
  verification stays separable from codegen.
- **Effect polymorphism, concurrency sharing rules, record-update sugar, list-growth
  API, contract-check-mode policy** [Pinned fact, decision 0014 "Not Settled"]. Trigger:
  a real program is blocked, per the work-order filter.

---

## What this backlog does not claim

- No syntax is invented here; no passing evidence is asserted; no estimates or guaranteed
  benefits are given.
- Every "smallest credible change" above is a direction for a future work order, not a
  work order. Ocean decides what becomes work.
- Provenance is labeled per item. Where the attached ZIP and later research disagree in
  emphasis, the ZIP is the primary source and later research is marked supplemental.
- Deleting sections or docs is not treated as simplification anywhere above — the intent
  structure is part of the product; the lever is measurement (item 5), not removal.

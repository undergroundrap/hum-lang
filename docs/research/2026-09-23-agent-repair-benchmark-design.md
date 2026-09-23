# Agent Repair Benchmark Design

Date: 2026-09-23
Status: proposed 2026-09-23. BDFL rules and merges.

## Context

0027 lists the agent repair benchmark as "to be built": agents fixing broken
programs, scored by success rate and attempts. It is one of four measures of
the north star's "best for humans and agents" claim (0024). This design
specifies the benchmark before any harness is built.

The question it answers: do Hum's diagnostics make agents better at repairing
broken programs than Rust's diagnostics do, on analogous bugs? The benchmark
is designed to be losable.

## The corpus: deliberately broken Hum programs

### Source programs

Broken items are generated from working corpus programs: wordfreq today; the
config parser and state machine as they finish (0027's loop). Every broken
item derives from a program that was green before breaking — the agent
repairs toward a known-good state, not an open-ended rewrite.

### Error classes

| Class | Hum codes | Rust analogue | Generation |
|---|---|---|---|
| use after move | H0801 | E0382 borrow of moved value | delete or duplicate a `consume` |
| borrowed parameter written | H0802 | E0596 cannot borrow as mutable | mutate through a `borrow` parameter |
| linear resource not consumed | H0803 | none direct; typestate / drop-bomb per the 0026 study | drop the commit/rollback call |
| linear resource consumed twice | H0804 | double-close / use-after-close shapes | duplicate the consume call |
| iteration mutation conflict | H0806 | iterator invalidation (E0499 family) | `list_append` inside `for each` |
| stale view / alias overlap | H0807 / H0808 | borrow-checker overlap errors | write through an alias; grow a list under a view |
| fallible call requires try | H0901 | unhandled `Result` (`unused_must_use`) | delete `try` |
| failure propagation mismatch | H0902 / H0903 / H0905 | `?` with incompatible error types (E0277) | swap the error roots |
| missing failure declaration | H0907 | none (Hum-specific) | hollow `fails when:` |
| undeclared mutation | H0201 / H0202 / H0603 | none (Hum-specific) | delete a `changes:` entry |
| invalid text_split call | H0636 | none (Hum-specific) | swap or empty the separator argument |
| missing intent section | H0105 | none (Hum-specific) | delete `why:` / `does:` |
| contract violation | H0702 / H0703 | debug_assert / contract crates (approximate) | break the `ensures:` body |
| missing source authority | H0621 / H0631 | none (Hum-specific) | call a stdout / file op without authority |

Classes with no Rust analogue are measured Hum-only and reported separately;
they never enter the head-to-head score.

### Generation method

1. **Mutation operators** [To be built]: one operator per class above, applied
   to a green program. Each operator targets exactly one diagnostic; the
   expected code is recorded with the item.
2. **Hand-written adversarial items** for shapes mutation cannot reach
   (cross-task linear settlement, authority-boundary crossings).
3. **Deduplication**: items producing the same diagnostic on the same
   construct family collapse to one.

### Size

5 items per class, 14 classes, 70 items to start **[Inference: starting size;
tune after the first run]**. Fewer per class leaves per-class rates too noisy
to act on; more delays the first measurement.

### Each item ships with

- the green original and the broken variant, with the mutation recorded;
- the expected diagnostic code;
- a hidden semantic test: the fixed program must behave correctly, not just
  pass the checker. This is what catches wrong-fix-that-compiles.

## An attempt

### Setup given to the agent

The broken program in a fresh isolated directory (see Leakage control), with
this instruction: "Repair the program so `hum check` is clean and the
program behaves as documented." The agent may read any file in the directory
and run `hum check` (`--format json` or text), `hum explain <code>`, and
`hum run`.

### Budget [Inference: starting parameters]

10 `hum check` / `hum run` invocations or 15 minutes, whichever comes first.
The attempt ends when the agent declares done, breaks the budget, or makes no
new tool call for 3 minutes.

### Scoring

| Outcome | Definition |
|---|---|
| fixed | `hum check` clean AND hidden semantic tests pass |
| attempts | check/run invocations used to reach fixed |
| wrong-fix-that-compiles | `hum check` clean but semantic tests fail — counted separately, ranked worse than unfixed |
| gave-up | budget exhausted or the agent declares it cannot fix |

Primary metrics: fix rate per class, median attempts-to-fixed, and the
wrong-fix rate. Wrong-fix-that-compiles is the dangerous outcome: a
diagnostic that guides the agent to a plausible-but-wrong repair is worse
than one that leaves the agent stuck.

### Diagnostics ablation

Two conditions per item: full JSON diagnostics (code + span + related_spans
+ help) versus code-and-span-only (help text stripped). If the help prose
does not move the fix rate, the blame-style-help investment is unjustified.

## Statistics

- **Runs per item**: 5 attempts per item per condition (350 attempts per arm
  per condition at 70 items). This smooths agent nondeterminism; a single
  run per item measures luck.
- **Recorded per run**: model name and version, harness version,
  temperature / top-p and other sampling settings, the compact Hum guide's
  content hash, the `hum` binary version, and the date. A run without this
  record is not reproducible and not comparable across releases.
- **Confidence intervals, never bare rates**: report Wilson 95% intervals
  for every rate. At 5 items × 5 runs per class (25 attempts per class per
  condition), per-class arm-vs-arm differences below roughly 25–30pp are
  noise — per-class rates are directional, not decisive. Decisive
  comparisons pool across classes: at n=70 an arm's rate carries roughly
  ±12pp, and arm-vs-arm differences below about 15pp are noise. If
  per-class decisiveness is ever needed, raise items per class; the cost
  is linear.
- **Pre-registered loss thresholds with a margin**: a loss criterion fires
  only if the entire 95% interval clears the threshold (for ceilings) or
  the intervals do not overlap (for comparisons). Noise alone cannot fire
  a loss.

## Leakage control

Every item is a mutation of a green corpus program, so each attempt runs in
a fresh isolated directory containing exactly the broken program file(s) and
the compact Hum guide — no green originals, no corpus checkout, no git
history (no `.git`), no network access to fetch the original. The harness
lists the directory before the attempt and fails the run if anything
unexpected is present. Otherwise a "fix" can be a copy, which measures
retrieval, not diagnostic quality.

## How Hum's diagnostics are fed to the agent

The feed is the existing machine surface, per the Agent Contract
(docs/DIAGNOSTICS.md): agents fix by code, not message substring.

- `hum check --format json`: code, title, severity, message, span,
  related_spans, help.
- `hum explain <code> --format json`: the stable code documentation, offline.
- Terminal prose is forbidden input in the primary condition (the Contract
  lists scraping prose when JSON is available as bad behavior).

Precondition: every code in the corpus must have JSON coverage. The
Contract's Test Requirements demand JSON coverage before a code is used by
agents; any code lacking it is blocked from the corpus until covered
**[verify per code; do not assume]**.

## The Rust arm

For each class with a Rust analogue, an equivalent broken Rust program with
the same bug shape (use-after-move, mutable borrow through a shared
reference, unhandled `Result`, incompatible `?` conversions, iterator
invalidation). The agent gets `cargo check --message-format=json` diagnostics
and may run `cargo check` / `cargo test` under the same budget.

Fairness rules:

- **Familiarity confound (named)**: agents train on vast Rust and
  approximately no Hum, so the Rust arm measures familiarity plus
  diagnostics, not diagnostics alone. The confound is reduced, not removed,
  by the mitigation below — it must ride along as a caveat on every
  cross-language number.
- **Compact Hum guide in context**: the Hum arm runs with a purpose-built
  guide in the isolated directory — the syntax the items use, the
  diagnostic codes in play, and `hum check` / `hum explain` usage, excerpted
  from LANGUAGE_REFERENCE.md. The agent is tested on repairing with the
  diagnostics, not on discovering the language from scratch. The guide's
  content hash is recorded per run (see Statistics).
- Same attempt budget, same scoring, same hidden-test oracle discipline
  (`cargo test` semantic tests, not just a clean compile).
- The Rust programs are written by someone fluent in Rust and reviewed for
  idiomaticity — a straw-man Rust arm invalidates the comparison.
- rustc's diagnostics are mature; the comparison measures diagnostic quality
  for agents, not language superiority.
- Hum-only classes are reported alongside, never averaged in.

## What counts as Hum losing

Primary results are within-Hum: per-class fix rates, the help-text ablation,
and trends across releases (the benchmark doubles as a regression suite for
Hum's own diagnostics). The cross-language comparison is secondary and
always reported with the familiarity caveat.

Primary (within-Hum) loss criteria:

- The help-text ablation shows no effect, with intervals: the
  blame-style-help investment (a core 0014-era claim) is unjustified.
- Hum's wrong-fix-that-compiles rate exceeds its pre-registered ceiling,
  with the entire 95% interval above the ceiling: Hum's diagnostics
  mislead more than the design tolerates.
- Regression across releases: fix rate drops or wrong-fix rate rises
  versus the previous benchmark run, with non-overlapping intervals.

Secondary (cross-language) signal:

- Rust's fix rate exceeds Hum's by more than the pre-registered margin
  (15pp), with non-overlapping 95% intervals. This fires an investigation,
  not a verdict: the familiarity confound may explain it. It becomes a
  loss only if the investigation rules the confound out.

Other findings that are reported as findings, not failures:

- Agents systematically need the structural interface (`hum graph --json`)
  to make progress: the benchmark then fires backlog item 10's deferred
  trigger.

## Tooling dependencies [To be built]

Everything in this section is unexecuted; the design must not assume it:

- **The harness**: drives agents, enforces budgets, enforces attempt-directory
  isolation (lists the directory pre-attempt, fails the run on unexpected
  files), scores outcomes, runs hidden tests. Does not exist.
- **Compact Hum guide**: a purpose-built LANGUAGE_REFERENCE excerpt for the
  agent context (syntax used by the items, codes in play, `hum check` /
  `hum explain` usage). A docs artifact, not tooling; its content hash is
  recorded per run. Does not exist.
- **Mutation operators**: generate the broken corpus from green programs.
  Do not exist; hand-write the first 70.
- **JSON coverage audit**: verify every corpus code has JSON coverage per
  the Contract's Test Requirements. Unknown until audited.
- **Hidden semantic tests**: per-item oracles. Misuse fixtures exist for
  some codes; a per-item oracle harness does not.
- **Rust-arm corpus**: equivalent broken programs plus semantic oracles.
  Does not exist.
- **Agent pool**: at least two different agent systems (different models or
  harnesses) to avoid single-model bias. Not arranged.
- **Structural interface** (`hum graph --json`, backlog item 10):
  unexecuted. The benchmark measures whether the Ralph-loop repair
  bottleneck is real; if agents stall on navigation, that is the trigger
  measurement item 10 waits for.

## What this does not measure

- Human repair skill: that is the human review study (backlog item 3).
- Greenfield writing: this benchmark is repair-only. Writing agents are a
  separate design.
- Whether the diagnostics are liked: only whether they produce correct
  fixes cheaply.

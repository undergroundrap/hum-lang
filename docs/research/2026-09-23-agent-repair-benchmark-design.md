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

The broken program in its repo directory, with this instruction: "Repair the
program so `hum check` is clean and the program behaves as documented." The
agent may read any file in the directory and run `hum check` (`--format json`
or text), `hum explain <code>`, and `hum run`.

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

- Same attempt budget, same scoring, same hidden-test oracle discipline
  (`cargo test` semantic tests, not just a clean compile).
- The Rust programs are written by someone fluent in Rust and reviewed for
  idiomaticity — a straw-man Rust arm invalidates the comparison.
- rustc's diagnostics are mature; the comparison measures diagnostic quality
  for agents, not language superiority. Hum wins only on a higher fix rate,
  fewer attempts, or a lower wrong-fix rate on the analogous classes.
- Hum-only classes are reported alongside, never averaged in.

## What counts as Hum losing

- Agents repair the Rust analogues at an equal or higher fix rate with equal
  or fewer attempts: the "best for agents" claim fails for the measured
  population.
- Hum's wrong-fix-that-compiles rate exceeds Rust's: Hum's diagnostics
  mislead more than rustc's — a loss even at equal fix rates.
- The help-text ablation shows no effect: the blame-style-help investment
  (a core 0014-era claim) is unjustified.
- Agents systematically need the structural interface (`hum graph --json`)
  to make progress: the benchmark then fires backlog item 10's deferred
  trigger — a finding, not a failure, but reported as one.

## Tooling dependencies [To be built]

Everything in this section is unexecuted; the design must not assume it:

- **The harness**: drives agents, enforces budgets, scores outcomes, runs
  hidden tests. Does not exist.
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

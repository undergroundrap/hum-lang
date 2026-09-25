# Ledger #18 decision brief: `hum check` versus predicate analysis

Date: 2026-09-25. Lane: research. Status: brief only — no surface invented,
no decision taken. The question is BDFL-reserved.

## 1. The question

Ledger #18 (wordfreq friction ledger): `hum check` never runs predicate
analysis, so contract errors — the H0704 family ("invalid executable
predicate") and the H0638-in-contract case from WO28 #4 — are invisible to
the agent loop until `hum full-type-check` or `hum run`'s preflight. The
0027 question: is running predicate analysis in `check` the GENERAL fix, or
is the cheap-check / full-pipeline boundary the right shape and the docs
the right fix?

## 2. Demonstrated friction (measured 2026-09-25, this VM, debug build)

A task with a malformed `ensures:` predicate (`result ~= "hello"`):

- `hum check`: `checked 1 file(s): 0 error(s), 0 warning(s)`, exit 0.
- `hum full-type-check`: exit 1, `error[H0704]` —
  `malformed_executable_predicate_v2`, with a repair hint
  (`Use one complete Predicate v2 comparison ...`).

An agent writing contracts in the `hum check --format json` loop sees green
until it runs the much slower, capability-demanding full pipeline.

## 3. What `hum check` runs today

`src/main.rs`: parse → resolve → `path_boundary` → `callable` diagnostics
→ `capability_root` analysis → per-stage type checks. Each stage is gated on
"no errors so far" (`main.rs:451`). Predicate analysis is not among them.

Predicate analysis itself (`predicate::analyze_program(&program)`) is
self-contained given `&Program`: it builds its own field-type map and
lexical context (`predicate.rs:284`), and is cached per process via a
thread-local keyed on the program (`predicate.rs:559`), so one invocation
computes it once. It surfaces today in two places:

- `hum full-type-check`, via `build_report_with` (`full_type_check.rs:521`),
  alongside five other added passes (type-check summary, callable analysis,
  typed-failure analysis, field types, core verification).
- `hum run`'s preflight: when full-type-check has *only* predicate errors,
  run prints just the predicate diagnostics with exit 2 (`main.rs:1256`).

## 4. Cost (measured 2026-09-25)

| Input | `hum check` | `hum full-type-check` | Delta |
|---|---|---|---:|
| `examples/tools/wordfreq.hum` (190 lines) | ~1.35 s | ~2.28 s | ~0.9 s |
| `examples/` (33 files) | ~40 s | ~50 s | ~10 s |

(`hum type-check` on wordfreq: ~1.4 s — check's extra stages over
type-check are negligible; the delta sits in full-type-check's bundle.)

The delta covers all six added passes, not predicate analysis alone; the
CLI exposes no per-stage timings (same instrumentation gap recorded in the
2026-09-25 preflight time profile), so predicate analysis's individual
share is unmeasured but bounded above by the delta — a fraction of ~0.9 s
on a 190-line program.

`docs/COMPILE_TIME_STRATEGY.md` frames the trade: "`hum check`: fastest
correctness loop" is loop #1, and non-negotiable #4 requires expensive
features to justify their compile-time cost. A sub-second fraction on a
real program is the cost to justify.

## 5. Precedent: ledger #15 → H0639

The directly analogous disagreement — `hum check` accepted contract-only
`list_count` in task bodies (0 errors) while `hum run` trapped — was
resolved by WO28 #15 surfacing the error *in check* (H0639), not by
documenting the boundary. Verified 2026-09-25: H0639 fires in `hum check`
today. The enforcement-status page records this as the project's answer to
that shape of gap.

## 6. Options

- **A. Run predicate analysis in `hum check`,** gated on no earlier errors
  (the gating pattern check already uses at `main.rs:451`, and the
  precedence design run's preflight already implements). Cost: an
  unmeasured fraction of the ~0.9 s wordfreq delta. Benefit: the agent
  loop sees H0704/H0638-contract errors immediately; consistent with the
  H0639 precedent and with "fastest correctness loop" — a loop that misses
  correctness errors is not a correctness loop.
- **B. Keep the boundary; document check's scope.** State explicitly which
  stages `check` covers and that contract predicates require
  `hum full-type-check`. Cost ~0. Risk: the loop keeps lying by omission,
  and it contradicts the H0639 precedent set weeks earlier for the same
  shape of gap.
- **C. New opt-in surface** (flag, or JSON-only predicate diagnostics).
  Default-to-no applies (decision 0027); not recommended without BDFL
  appetite for new surface.
- **D. Status quo via `run` preflight.** Already exists (exit 2), but `run`
  demands capabilities and an entry point — it is not the iteration loop.

## 7. Recommendation

Option A, on three grounds: (1) the H0639 precedent — the project already
decided this exact shape of check/pipeline disagreement belongs in check;
(2) the strategy doc's own framing — predicate errors are correctness
errors agents introduce while writing contracts, which is what the
fastest *correctness* loop is for; (3) measured cost is a fraction of ~0.9 s
on a real 190-line program, with per-process caching already in place.

## 8. What this brief does not decide

Implementation design (Builder lane when commissioned): exactly where the
gating sits, diagnostic rendering in both formats, and whether the
predicate-error precedence needs the run-style only-predicate-errors
narrowing. It also does not move any other full-type-check stage, and it
moves nothing out of CI — this only adds surfacing.

## 9. Sources

- `docs/research/wordfreq-friction-ledger.md` #18; `src/main.rs:451`,
  `:854`, `:1256`; `src/predicate.rs:284`, `:559`;
  `src/full_type_check.rs:521`; `docs/COMPILE_TIME_STRATEGY.md`;
  enforcement-status page (ledger #15 → WO28 #15, H0639).

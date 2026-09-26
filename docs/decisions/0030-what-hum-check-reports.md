# Decision 0030 — What `hum check` reports

Status: **accepted 2026-09-26 (BDFL ruling: Option B, with D3 and the cost gate).**

> **Standing of this record.** This decision is *accepted*. It records the
> BDFL's ruling (§10) and the recommendation it adopted. This record
> generalizes ledger #18 (wordfreq friction ledger): `hum check` never
> runs predicate analysis, so contract errors are invisible to the agent
> loop until `hum full-type-check` or `hum run`'s preflight. The question
> is broader than predicates: should `hum check` report everything the
> static checker knows? The ruling answers yes, gated on a clean cost
> measurement.

## 1. The finding

Ledger #18 (demonstrated 2026-09-25): a task with a malformed `ensures:`
predicate (`result ~= "hello"`) gets `checked 1 file(s): 0 error(s), 0
warning(s)`, exit 0, from `hum check`, while `hum full-type-check` reports
`error[H0704]` (`malformed_executable_predicate_v2`) with a repair hint.
An agent iterating in the `hum check --format json` loop sees green until
it runs the slower, capability-demanding full pipeline.

Why now: WO30 (active) puts the new call-shape errors H0640 (arity),
H0641 (argument type), H0642 (statically-known negative in UInt position)
in `full-type-check` — "the stage that owns expression checking" — and its
acceptance criterion 2 pins the boundary in fixtures: the new diagnostics
surface in `hum full-type-check` but not in `hum check`'s pipeline; `hum
check` "stays silent on call shapes by pipeline design"
(`workorders/active/WORKORDER_30.md:252-275`). So the generalized question
is live today, not hypothetical: after WO30 lands, `hum check` will stay
silent on H0640–H0642, H0636 (invalid `text_split` call), H0638 (invalid
text escape), H0704/H0701 (predicate family), and H0901–H0906 (typed
failures) — while `hum run`'s preflight refuses all of them with exit 1
before execution.

## 2. The current boundary, pinned

The diagnostic catalog (`src/diagnostic_catalog.rs`) attributes each of
the 94 codes to an owning analysis (the 7th tuple field — a provenance
label, not a CLI surface map):

| Owning analysis | Codes | Families |
|---|---|---|
| parser | 10 | H0001–H0010 (top-level shape, identifiers) |
| check (check.rs per-file) | 28 | H0101–H0110, H0201–H0202, H0301–H0305, H0401–H0402, H0501–H0502, H0637, H0639, H1201–H1205 |
| resolve | 4 | H0601–H0604 |
| type_check | 2 | H0605, H0606 |
| callable | 2 | H1401, H1402 (plus UNRESOLVED_NAME mapping, `src/callable.rs:1196,4492`) |
| capability_root | 13 | H0617–H0628, H0631 |
| path_boundary | 2 | H0629, H0630 |
| app_entry | 8 | H0610–H0616, H0634 |
| full_type_check | 8 | H0636, H0638, H0901–H0906 |
| predicate | 2 | H0701, H0704 |
| run (execution) | 2 | H0702, H0703 (needs/ensures violations — require execution) |
| ownership_check | 9 | H0801–H0809 |
| effect_check / file_read / native_admission | 4 | remaining |

(`src/diagnostic_catalog.rs:4749` pins `all().len() == 94`.)

The CLI surface per command, from `src/main.rs`:

- **All commands** run `load_program` (`src/main.rs:3424`): parse
  (`:3440`), then `check::check_parse_output` per file (`:3444`) — this is
  where the 28 "check"-attributed codes fire, including H0637
  (`src/check.rs:163`) and H0639 (`src/check.rs:266`) — then app-entry
  per-file diagnostics gated on no errors so far (`:3457`–`:3494`).
- **Shared non-run orchestration** (`src/main.rs:431`–`:477`): each stage
  is gated on "no errors so far": `path_boundary` (`:447`), callable
  diagnostics **only for the `check` command** (`:452`–`:458`), then
  `capability_root` (`:465`). The occurrence invariant is validated at
  `:470`.
- **`hum check`** (`:479`) renders exactly the above and nothing else.
- **`hum resolve`** (`:741`) adds the resolve report (H0601–H0604).
  (Name-resolution failures also surface in `hum check` through callable's
  UNRESOLVED_NAME mapping, `src/callable.rs:4492`.)
- **`hum type-check`** (`:817`) adds the type_check stage (H0605, H0606).
- **`hum full-type-check`** (`:855`) runs `build_report_with`
  (`src/full_type_check.rs:508`–`:528`), which computes six passes:
  type-check summary, callable analysis, typed-failure analysis, field
  types, predicate analysis (`:522`), and core verification. Per-item
  collection is gated on `blocked` (any earlier error), but all six
  analyses run.
- **`hum run`** preflight (`execute_run_command`, `src/main.rs:1125`):
  early return on errors (`:1212`–`:1225`), then resolve (`:1228`),
  type-check (`:1237`), full-type-check (`:1256`) in order. When
  full-type-check has *only* predicate errors, run prints just the
  predicate diagnostics with exit 2 (`:1257`–`:1275`) — the precedence
  design already exists here.

So today the agent's fast loop (`hum check --format json`, schema
`hum.check.v0`, `src/diagnostics.rs:5`) reports: parse + check.rs +
app-entry + path-boundary + callable + capability families. Everything the
checker knows beyond that — type_check, full_type_check, predicate —
requires another command. H0702/H0703 are execution-requiring and out of
scope for any static command.

## 3. Options

No option may introduce ambient flags or env vars (per the commission).

- **A. Predicate analysis in `hum check` only.** Slot
  `predicate::analyze_program` into the shared orchestration after the
  existing stages, gated on no errors so far (the pattern at
  `src/main.rs:431`–`:477`, and the precedence design run's preflight
  already implements). This is the brief's recommendation
  (`docs/research/2026-09-25-ledger-18-check-vs-predicate-analysis-brief.md`
  §6–§7). Fixes ledger #18 exactly; leaves H0640–H0642 (WO30), H0636,
  H0638, H0901–H0906 silent in `hum check`.
- **B. `hum check` runs the full static pipeline.** `hum check` becomes:
  today's stages, then type-check, then full-type-check (which includes
  predicate analysis), with stage precedence kept — later stages blocked
  by earlier errors, exactly as run's preflight does today
  (`src/main.rs:1228`–`:1290`). The boundary becomes principled: `hum
  check` reports everything statically knowable; `hum run` additionally
  enforces what requires execution (H0702/H0703). `hum type-check` and
  `hum full-type-check` remain as stage-scoped views (CI routing per
  decision 0025, debugging).
- **C. Keep the boundary; document it.** State explicitly which stages
  `hum check` covers and that contract predicates, call shapes, and typed
  failures require `hum full-type-check` (or `hum run`). Zero
  implementation cost. The brief's Option B.
- **D. Other shapes found.**
  - **D1. New explicit command** (e.g. `hum verify`) running the full
    static pipeline; `hum check` untouched. Zero cost to `check`, but new
    surface — and agents must learn a new command to get what B gives in
    the command they already use.
  - **D2. Visible scope.** `hum check`'s summary names the stages it ran
    (human line and/or docs), so the "0 error(s)" claim is explicitly
    scoped. Near-zero cost; honest, but keeps the omission.
  - **D3. Additive JSON field.** `hum.check.v0` gains a machine-readable
    scope field (e.g. the stage list); backward compatible for tolerant
    readers. Composable with any option.

## 4. Per-option analysis

### Agent-loop impact

- **A:** the `hum check --format json` loop catches H0704/H0701 (contract
  errors agents introduce while writing contracts). It still misses
  WO30's H0640–H0642, H0636, H0638, H0901–H0906.
- **B:** the loop catches everything statically knowable. An agent that
  gets green from `hum check` can trust that `hum run`'s preflight will
  not refuse on static grounds — the only remaining preflight refusals
  are execution-requiring (H0702/H0703).
- **C:** unchanged; agents must call `hum full-type-check` for the full
  picture. The loop keeps being green-by-omission on whole families.
- **D1:** agents must learn the new command; the default loop is
  unchanged. **D2/D3:** no new errors surface; agents can at least see
  the scope.

### Human impact

- **A/B:** richer diagnostics in the editor loop — `hum check --format
  json` (`hum.check.v0`) is the specified LSP/editor diagnostics source
  (`docs/LSP_CAPABILITIES_SCHEMA.md:69`,
  `docs/LSP_CAPABILITY_MATRIX.md:37`,
  `docs/EDITOR_AND_INTEGRATION_STRATEGY.md:25`). Contract and call-shape
  errors surface while typing, not at run time.
- **C:** docs state the scope; humans keep two commands in their head.
- **D2:** the summary line makes the scope visible to humans.

### JSON schema / version impact

- `hum check --format json` emits schema `hum.check.v0` with fields
  `schema`, `summary{files,errors,warnings}`, `diagnostics[]`
  (`src/diagnostics.rs:42`–`:73`). Under **A** and **B** the shape is
  unchanged — only the *code universe* in `diagnostics[]` grows
  (A: +H0701/H0704; B: +H0605/H0606, +H0636/H0638, +H0640–H0642 once WO30
  lands, +H0701/H0704, +H0901–H0906). No version bump is required by the
  shape, but the semantic contract changes: agents with code allowlists
  must handle the new codes, and `summary` counts shift. Under **B**,
  `hum check` does *not* adopt full-type-check's richer report schema
  (`callable_json_report` with `predicate_facts` etc.); it keeps
  `hum.check.v0` with a longer diagnostics list — the exact rendering is
  builder-lane design (see §7).
- **D3** is the only option that changes the schema, additively
  (backward compatible for tolerant readers; repo tests pin exact JSON
  and would be updated).

### Precedence and duplicate-diagnostic risk

Exactly one diagnostic per issue across stages — the invariant the
existing machinery enforces:

- **A:** predicate analysis runs nowhere else in the `hum check`
  process, so there is no double-report. Facts produce diagnostics only
  for `MalformedExecutable | RejectedSemantics` statuses
  (`src/predicate.rs:461`–`:475`), all mapped to H0704. Gating on no
  earlier errors mirrors the existing pattern.
- **B:** mirror run's preflight ordering (resolve → type-check →
  full-type-check, each gated). Within one process the thread-local
  caches mean no recomputation: callable (`src/callable.rs:258`–`:259`),
  typed-failure (`src/typed_failure.rs:140`–`:141`), predicate
  (`src/predicate.rs:558`–`:568`). The anti-duplication machinery
  already exists: `callable::diagnostics` filters facts against prior
  diagnostics (`prior_owns`, `src/callable.rs:306`–`:315`), and the
  occurrence invariant is validated (`src/main.rs:470`). Risk to name
  in the implementation: `build_report_with` computes all six analyses
  unconditionally (`src/full_type_check.rs:508`–`:528`) — the `hum
  check` arm must gate the *call* on no-earlier-errors (as the shared
  prefix does) rather than relying on the internal `blocked` flag, or it
  pays the full computation for nothing.
- **C/D:** no change.

### 0024 verification-speed cost

Decision 0024: "Check and CI time is a performance budget too — and for
agents it is the tighter one. An agent's loop speed is bounded by how
fast Hum can tell it that it is wrong"
(`docs/decisions/0024-state-performance-north-star.md:53`–`:58`).
`docs/COMPILE_TIME_STRATEGY.md` names `hum check` the "fastest
correctness loop" (loop #1) and requires expensive features to justify
their compile-time cost (non-negotiable #4).

Cost evidence is governed by §6 below. In brief: **A** costs an
unmeasured fraction of the ~0.9 s upper-bound delta on a 190-line
program; **B** costs the full delta (~0.9 s on 190 lines, ~10 s on 33
files — upper bounds, shared-VM debug builds). **C/D2** cost ~0.
**D1** costs 0 to `check` but moves the cost to wherever agents invoke
the new command.

The editor-loop coupling cuts both ways and must be weighed openly:
`hum.check.v0` is the LSP diagnostics contract, so **B** makes the
editor loop pay full-type-check cost on every run. Mitigations: release
builds are far faster than the measured debug builds, editors debounce,
and the real fix is incremental compilation plus semantic-graph caching
(strategy non-negotiables #2/#3 — not yet built). This is the strongest
argument against **B**, and it is why §6 requires a clean number before
implementation.

## 5. Decision 0027's general-vs-specific test

Decision 0027: "An addition must be the general fix, not a one-program
special case"
(`docs/decisions/0027-adopt-program-driven-language-development.md:29`–`:31`).
Applied to tooling surface: **A** fixes one family's silence; the
identical shape of gap remains for every other full-type-check family —
WO30's H0640–H0642 silence is already pinned by design and would need its
own future decision. **B** answers the generalized question once, for all
current and future static families: `hum check` reports everything the
static checker knows. Under 0027's lens, **B** is the general fix and
**A** is the specific one. The H0639 precedent (WO28 #15: the same shape
of check/pipeline disagreement was fixed by surfacing the error *in
check*, verified 2026-09-25; Session AG pins it at
`tools/check_all.ps1:5145`–`:5149`) points the same way — the project has
already decided this shape of gap belongs in `check`.

## 6. Cost evidence rules

No wall-clock timing was run on the shared VM for this record. The only
figures used are the brief's, and only as **labelled upper bounds**
(shared-VM, possibly contended, debug builds):

| Input | `hum check` | `hum full-type-check` | Delta (all six added passes) |
|---|---|---|---:|
| `examples/tools/wordfreq.hum` (190 lines) | ~1.35 s | ~2.28 s | ~0.9 s |
| `examples/` (33 files) | ~40 s | ~50 s | ~10 s |

(`hum type-check` on wordfreq: ~1.4 s — check's extra stages over
type-check are negligible; the CLI exposes no per-stage timings, the
same instrumentation gap the brief records. `--timings` reports
per-file read/parse/check only, `src/main.rs:3630`–`:3641`.)

Structural facts (no timing needed):

- Analyses are cached per process via thread-locals keyed on the
  program: callable (`src/callable.rs:258`), typed-failure
  (`src/typed_failure.rs:140`), predicate (`src/predicate.rs:558`).
  Within one `hum check` process, **B** recomputes nothing already
  computed — notably, callable analysis is already paid by `hum check`
  today (`callable::diagnostics`, `src/main.rs:452`–`:458`).
- Predicate analysis is self-contained given `&Program`: it builds its
  own field-type map and lexical context (`src/predicate.rs:284`–`:291`).
- Existing redundancy, for the record: field types are collected twice
  today (predicate `:285` and full-type-check `:520`).
- If a clean number is needed before implementation: measure on CI
  (release build, uncontended) or add per-section instrumentation to
  `--timings`. Do not re-time on the shared VM.

## 7. Consequences if B is accepted

- WO30 acceptance criterion 2 is amended: the fixtures asserting `hum
  check`'s silence on call shapes flip to assert rejection with
  H0640/H0641/H0642. (A decision outranks a Work Order; the Work Order
  text is updated, not silently contradicted.)
- `hum type-check` and `hum full-type-check` remain as stage-scoped
  views (0025 CI routing, debugging, the enforcement-status page). No
  command is removed.
- Session AG's `hum check examples` assertion
  (`tools/check_all.ps1:1865`) is re-verified under the wider surface —
  examples must be clean under all static stages.
- Builder-lane design questions left open (not decided here): the exact
  gating site in the `check` arm, rendering of the added stage
  diagnostics in human and JSON formats (keeping `hum.check.v0`'s
  shape), and whether the only-predicate-errors narrowing
  (`src/main.rs:1257`–`:1275`) has any `check`-side analogue.
- D3 (additive `stages` field) is recommended as a compatible adjunct:
  it makes the scope machine-readable for agents under any option.
- H0702/H0703 stay `run`-only: they require execution, so they are
  outside every static option by construction.

## 8. Recommendation

**Recommend Option B**, with D3 as an adjunct and a cost gate:

`hum check` should report everything the static checker knows — today's
stages, then type-check, then full-type-check (predicate analysis
included), with stage precedence kept exactly as run's preflight does
today. The boundary becomes principled instead of accidental: static
(`hum check`) versus execution-requiring (`hum run`).

Grounds: (1) 0027's general-vs-specific test — B answers the generalized
question once; A leaves WO30's call-shape silence in place by design.
(2) The H0639 precedent — the project already decided this exact shape
of check/pipeline disagreement belongs in `check`. (3) 0024 — the
agent's loop is bounded by how fast Hum tells it it is wrong; a
correctness loop that stays silent on correctness errors is not a
correctness loop. (4) The 0026 accountability angle —
`docs/decisions/0026-closed-world-accountability-as-a-scoped-design-objective.md:94`–`:100`:
completeness is relative to an explicit specification, and unknown
obligations must remain visible. Under B, `hum check`'s "0 error(s)" is
complete relative to the static specification, with the
execution-requiring remainder explicitly out of scope — instead of
today's implicit, accidental scope.

**Cost gate:** no implementation until a clean number exists (CI
release-build timing or per-section `--timings` instrumentation). The
brief's figures are upper bounds only. The number must cover the
editor-loop coupling (`hum.check.v0` is the LSP contract): if clean
measurement shows the full static pipeline costs ≥2× `hum check` on
representative programs in release builds, fall back to **A** now and
revisit **B** when incremental compilation lands.

### What would change my mind

1. A clean (uncontended, release-build) measurement showing the full
   static pipeline at ≥2× `hum check` on representative programs —
   then A now, B after incremental.
2. Evidence that the editor loop cannot tolerate the delta even
   debounced (the LSP contract rides on the same command) — same
   fallback.
3. A BDFL ruling that `hum check`'s identity is the fast surface gate
   and the command ladder is load-bearing as-is — then C plus D2/D3
   (documented, visible, machine-readable scope).

## 9. Sources
- `docs/research/2026-09-25-ledger-18-check-vs-predicate-analysis-brief.md`
  (Option A recommendation; cost figures reused here as upper bounds)
- `docs/research/wordfreq-friction-ledger.md` #18
- `workorders/active/WORKORDER_30.md:252`–`:275` (criterion 2, the
  pinned #18 boundary)
- `src/main.rs:431`–`:477` (shared orchestration + gating), `:479`
  (`check` arm), `:741` (resolve), `:817` (type-check), `:855`
  (full-type-check), `:1125`–`:1290` (run preflight), `:1257`–`:1275`
  (only-predicate-errors narrowing), `:3424`–`:3502` (load_program),
  `:3630`–`:3641` (`--timings`)
- `src/full_type_check.rs:508`–`:528` (six passes), `:522` (predicates)
- `src/predicate.rs:284`–`:291` (self-contained build), `:461`–`:475`
  (diagnostic emission), `:558`–`:568` (per-process cache)
- `src/callable.rs:258`–`:259` (cache), `:306`–`:315` (prior_owns),
  `:349` (stage_blockers), `:452`–`:458` via main.rs (check-only)
- `src/check.rs:12` (check_file), `:163` (H0637), `:266` (H0639)
- `src/diagnostics.rs:5` (`hum.check.v0`), `:42`–`:73` (check_json)
- `src/diagnostic_catalog.rs:4749` (94 codes); 7th tuple field = owning
  analysis
- `docs/decisions/0024-state-performance-north-star.md:53`–`:58` (0024
  verification-speed clause)
- `docs/decisions/0026-closed-world-accountability-as-a-scoped-design-objective.md:94`–`:100`
  (unknown obligations remain visible)
- `docs/decisions/0027-adopt-program-driven-language-development.md:29`–`:31`
  (general-vs-specific test)
- `docs/COMPILE_TIME_STRATEGY.md` (loop #1; non-negotiable #4)
- `docs/LSP_CAPABILITIES_SCHEMA.md:69`,
  `docs/LSP_CAPABILITY_MATRIX.md:37`,
  `docs/EDITOR_AND_INTEGRATION_STRATEGY.md:25` (LSP rides on
  `hum.check.v0`)
- `tools/check_all.ps1:1865` (`hum check examples`, Session AG),
  `:5145`–`:5149` (H0639-in-check pin)

## 10. Ruling (BDFL, 2026-09-26)

Ocean accepts **Option B**, with **D3** and the cost gate.

- **B is accepted.** `hum check` will report everything the static
  checker knows: today's stages, then type-check, then full-type-check
  (predicate analysis included), with stage precedence kept exactly as
  `hum run`'s preflight does today. The boundary becomes principled:
  static (`hum check`) versus execution-requiring (`hum run`).
- **D3 as adjunct.** `hum.check.v0` gains an additive, machine-readable
  scope field (the stage list), backward compatible for tolerant
  readers.
- **Cost gate.** Implementation does not start until a clean measurement
  exists: uncontended release build, `hum check` vs `hum full-type-check`
  on `examples/tools/wordfreq.hum` and on `examples/`. If the full
  pipeline costs ≥2× the current `hum check`, fall back to Option A now
  and revisit B after incremental checking lands.
- **WO30 sequencing.** WO30's criterion-2 fixtures (asserting `hum check`
  stays silent on call shapes) flip in the B implementation PR, not in
  this record and not now. Until that PR lands, the pinned boundary
  stands.

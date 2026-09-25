# Preflight time profile — pinned cost structure (timings unmeasured)

Date: 2026-09-25 (rewritten; supersedes the withdrawn 2026-09-25 measurement
note). Lane: research.
Assignment (Claude 2026-09-25): profile the local preflight
(`tools/check_all.ps1 -EvidenceTier Fast`) and propose a faster pre-push
tier, with CI as documentation, all evidence kept, and a pre-push loop
measured in minutes.

**Provenance — the 36m47s measurement is withdrawn.** The 2026-09-25 local
Fast run (commit `11c5de2`, published as `1bdfd3a`, 36m47s) was
contaminated: concurrent Cargo work on the shared Linux VM caused CPU/disk
contention, and prior runs had already warmed the target directory — so the
timing was neither a valid cold measurement nor a controlled warm/prebuilt
measurement. **That measurement and every wall-clock figure derived from it
are withdrawn.** BDFL disposition 2026-09-25: do not time anything on the
shared VM; mark all wall-clock figures in this note **unmeasured**; clean
numbers come later from per-section timing instrumentation (option A, Builder
work) run on uncontended CI runners.

**Standing change.** The pre-push process has already changed
(`docs/TESTING_STRATEGY.md`, "Pre-push gates", BDFL ruling 2026-09-25,
landed as #40): the local Fast preflight is dropped; pre-push is now
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D
warnings`, `cargo test --bin hum`, `tools/test_ci_policy.ps1`, plus
hand-running the exact commands behind any added/changed Session AB
assertions; CI Full is the gate and the PR goes up as a draft. This note is
the evidence behind that change: what the Fast tier *is*, what its pinned
cost structure looks like, where each piece's evidence now lives, and what
further speed-ups remain on the table. No new decision is drafted — the
process decision is already made.

Sections are labelled by standing: **(pinned)** = verifiable in the repo
now; **(unmeasured)** = wall-clock not yet measured on clean runners;
**(inference)** = this note's argument. Nothing below presents an estimate
as a measurement.

## 1. What the Fast tier is (pinned)

- `EvidenceTier` defaults to `'Fast'` (`tools/check_all.ps1:2–3`). It is
  the tier a bare local `./tools/check_all.ps1` runs, and the tier the
  builder used as the pre-push gate until 2026-09-25.
- Fast runs nearly the whole suite in one process: Fast evidence-capture
  tests → policy/selector self-tests and source assertions → `format`,
  `check`, `tests`, `clippy`, `build` → compiler-front checks →
  WO25 Unit A/B/C focused evidence + Unit C mutation evidence → selector
  ledger bookkeeping → compiler-corpus checks → `hygiene` (policy,
  bootstrap, discovery, boundary classifier, text hygiene, public
  readiness, release readiness). It differs from CI Full mainly by the
  evidence tier: no `HUM_EVIDENCE_RECEIPT` production, no isolated
  `hum-dev evidence exhaustive` route. (The 14,226-pair canonical-seal
  matrix is a plain `#[test]` in `parser.rs` and runs inside `cargo test`
  in *both* tiers — the tiers differ in receipt/isolation machinery, not
  in that matrix.)
- CI never runs Fast. CI runs fixed profiles (`Language`/`Runtime`/
  `Compiler`, via `Invoke-HumFixedProfile`) or the exhaustive Full route
  (`hum-dev evidence exhaustive`). Fast exists only for local runs, so its
  timing evidence can only come from a local run — not from CI logs.

## 2. Timing instrumentation: what exists and what does not (pinned)

- `check_all.ps1` emits per-section timings **only for fixed profiles**:
  `profile_group=$Group;passed=$Passed;elapsed_ms=...`
  (`Invoke-HumFixedProfile`, `:1895–1930`). CI's Language runs therefore
  carry per-group timings in their logs.
- The Fast path and the Full path emit **no per-section timings at all**.
  `Invoke-Native`, `Invoke-RepoScript`, `Read-NativeChannelsWithExit`, and
  `Invoke-HumCoreCheck` print only `==> Label` markers (`:97`, `:148`,
  `:606`, `:1824`). Per-function wall-clock inside Fast/Full is
  not recorded by the tooling — that is the central finding of this note,
  and adding it is Builder work (see §5, option A).
- CI job-log download returns HTTP 401 for this lane's credentials
  (verified 2026-09-25 against the jobs API, which does work), so the
  `profile_group` lines from CI's Language runs are not retrievable here.
  Reconstructing them from log timestamps is Claude's side.

## 3. Pinned cost structure (pinned, countable — no wall-clock)

Every count below was verified 2026-09-25 against `tools/check_all.ps1` at
main `18a31cd`. Re-pin with the cited greps; the counts are the facts, the
minutes are not.

1. **14 `RUSTFLAGS --cfg` recompiles.** Fourteen distinct
   `hum_compile_fail_*` cfgs
   (`grep -o "hum_compile_fail_[a-z_0-9]*" tools/check_all.ps1 | sort -u`;
   e.g. `hum_compile_fail_backend_adapter_raw_inputs`,
   `hum_compile_fail_canonical_minimal_add_backend_facts_escape`,
   `hum_compile_fail_verified_backend_input_authority`, …
   `hum_compile_fail_verified_integer_sign_backend_input_substitution`).
   Each forbidden-construction proof sets one `RUSTFLAGS --cfg ...` and
   runs `cargo check --bin hum` (e.g. `:2402`); a changed `RUSTFLAGS`
   changes the fingerprint, so each proof is a full recompile of the
   binary. Fourteen proofs ⇒ fourteen full recompiles, by construction.
   Wall-clock per recompile: unmeasured.
2. **Mutation rows: 15 + 8, each recompile-driven.** WO22 Unit B:
   15 production-mutation rows `B01`–`B15`
   (`sed -n '780,900p' tools/check_all.ps1 | grep -c "Label = 'B"` ⇒ 15);
   each row rewrites source, runs the exact test (recompile), restores,
   and re-verifies — about two recompiles per row, structurally. WO25:
   8 initialized-mutation rows `I01`–`I08` (the receipt function pins
   exactly eight records: "full mutation ledger must contain exactly
   eight records", `:33–34`); each row runs an honest-before exact test,
   mutates, expects the failure, restores, and byte-compares
   (`Invoke-Wo25UnitAProductionMutationEvidence`, `:1249`;
   `Invoke-Wo25UnitBProductionMutationEvidence`, `:1480`). Recompile
   count per row: unmeasured.
3. **75 exact-selector call sites, two cargo spawns each.** 72
   `Invoke-ExactRustTest` call sites and 3
   `Invoke-HumContainedExactRustTest` call sites
   (`grep -c`). Every call is **two** cargo invocations
   (`cargo test <sel> -- --exact --list`, then `cargo test <sel> --
   --exact`; `test_exact_rust_selector.ps1:103–140`,
   `run_fast_evidence.ps1:1331–1348`) — about 150 cargo process spawns,
   each paying workspace resolution and target fingerprinting. The
   `hum-dev` selectors are the *only* execution of those tests in the
   Fast path (bare `cargo test` never touches `hum-dev`) — not repeats,
   but each still costs two spawns. The root-package selectors
   (WO22/23/24 rows, the diagnostic registry) genuinely re-execute tests
   the `tests` group's full root `cargo test` already ran.
4. **Four root-package compile passes.** `cargo check --all-targets`,
   `cargo test`, `cargo clippy --all-targets`, `cargo build` each do a
   full fingerprint + compile pass of the root package (bare cargo
   commands at the workspace root select the root package only —
   verified: `cargo test --no-run` builds `unittests src/main.rs` and
   nothing else). Cargo caches aggressively, but check/clippy use
   different compiler drivers, so each is real compile work — four builds
   of the binary per Fast run, structurally. Wall-clock: unmeasured.
5. **Member crates run once, explicitly.** `windows-drive-locality` and
   `hum-dev` are workspace members (`Cargo.toml:8`) but bare cargo
   commands select the root package only, so the explicit `cargo test -p
   windows-drive-locality` and `cargo clippy -p windows-drive-locality`
   (`:6597`, `:6600`) are those crates' *only* runs — not
   duplicates. (An earlier draft of the withdrawn note claimed otherwise;
   the `--no-run` check corrected it.)
6. **Effect bake-off builds in isolation.** `--target-dir
   target/effect-bakeoff` (`:6534`) keeps it out of the shared target
   dir — a full separate build by design, not a repeat, but it cannot
   reuse the main cache.
7. **The boundary classifier suite always runs on local Fast.** 153
   cases, classifier invoked twice per case
   (`test_workorder_status_boundary.ps1`, 1933 lines). On CI fixed
   profiles it is skipped when `boundary_required=false`; locally
   `HUM_CI_BOUNDARY_REQUIRED` is unset, so fail-safe runs it every time
   — even for docs-only changes whose consumers CI would never run it
   for.
8. **The full capture-test script runs at the top of every Fast
   invocation** (`test_fast_evidence_capture.ps1`, 2362 lines); fixed
   profiles run only its `-ProfileSmokeOnly` smoke
   (`Invoke-HumCaptureSmoke`, `:1879–1894`).

The single biggest cost cluster is therefore **recompilation, by
count**: 4 core cargo passes + 14 cfg-gated `cargo check --bin hum`
proofs + ~23 mutation rows at ~1–2 recompiles each. The per-invocation
corpus `hum` calls are sub-second each; the corpus cost is volume
(`Invoke-HumCompilerCorpusChecks`, `:3623`, and the compiler-front section), matching the
2026-07-30 profile's "fixtures × stages" finding. How many minutes each
cluster costs: unmeasured — that is what option A instruments.

## 4. CI job-level contrast (pinned runs, unmeasured wall-clock)

| Run | Profile | Ubuntu preflight | Windows preflight |
|---|---|---|---|
| 36042043106 (PR #37, WO28 #4) | Full | unmeasured | unmeasured |
| 35969187628 (PR #36, docs) | Language | unmeasured | unmeasured |

The runs are pinned (IDs, profiles); their job-level wall-clock is
retrievable from the CI logs via the Jobs API
(`/repos/{owner}/{repo}/actions/runs/{id}/jobs`, `started_at` →
`completed_at` per job, including checkout/toolchain setup). The earlier
note's minute figures for these runs were session notes, not clean
evidence, and are withdrawn with the rest — re-pull them from CI when
needed. The Language docs run carries the boundary-skip dividend (skip
line on both OSes, §5 of the enforcement page); per-section timings for
the fixed profiles live in the `profile_group` log lines (§2). What CI
cannot supply is per-section timing for Fast/Full — that needs option A.

## 5. Where each Fast section's evidence lives without a local Fast (inference)

The adopted pre-push tier keeps CI Full as the gate, so nothing below is
dropped — it moves. Mapping, Fast section → where its evidence runs now:

- `format` → pre-push `cargo fmt --all -- --check` (same command, local)
  and CI Full.
- `check`, `clippy` → pre-push `cargo clippy --workspace --all-targets`
  (covers check's ground; clippy implies a compile) and CI Full.
- `tests` → pre-push `cargo test --bin hum` (the language binary's suite)
  for the fast signal; the full workspace suite runs in CI Full.
- Policy/selector self-tests, source assertions, WO25 A/B/C evidence,
  compiler-front/corpus, hygiene group → CI Full (and the fixed profiles
  where routed). These are the CI-tooling's own tests; they change only
  when `tools/` or the workflows change, which is exactly when CI runs
  them.
- Boundary classifier suite → CI fixed profiles (skipped when
  `boundary_required=false`) and Full; locally it ran unconditionally,
  which was pure cost for non-consumer changes.

The deliberate trade: feedback latency for the deep sections moves from
"before push" to "CI Full", absorbed by the draft-PR workflow (push as
draft, mark ready when green). The pre-push loop itself is the five cheap
gates above; their wall-clock on a clean runner is unmeasured (the
withdrawn note's 121 s figure came from the contaminated VM session).

## 6. Ranked options (inference)

Ranked by expected time saved against evidence kept. **Option 0 is
adopted** (Claude, 2026-09-25); the rest are future work, ordered by
expected value. All savings below are structural (counts), not minutes —
the minutes arrive with option A.

- **0. Adopted: drop local Fast; CI Full is the gate.** Saves the whole
  local run (wall-clock unmeasured — the withdrawn 36m47s is not
  evidence). Pre-push is the five cheap gates (§5). Evidence kept:
  everything above runs in CI Full or a routed fixed profile. Cost:
  deep-section feedback waits for CI; the draft-PR workflow is the
  mitigation. No decision record needed from this lane — the ruling is
  recorded in `docs/TESTING_STRATEGY.md`.
- **A. Per-section timing lines in `check_all` (Builder work).** Wrap the
  Fast/Full section calls the way `Invoke-HumFixedProfile` already does
  (`:1916–1930`) so the next profiling question is answerable from logs.
  Saves nothing by itself; it is what makes §3 countable in minutes
  instead of recompiles. Per the BDFL disposition, the clean numbers come
  from this instrumentation run on uncontended CI runners — not from the
  shared VM. Cheap, high value for the next round.
- **B. Collapse exact-selector double invocations.** Each selector test
  currently spawns cargo twice (list, then run). The list step exists to
  prove the selector resolves to exactly one test; a single
  `cargo test <sel> -- --exact` whose output is asserted to show exactly
  one run proves the same thing. ~75 spawns saved per Fast/Full run.
- **C. Retired — was "deduplicate locality test/clippy"; the
  `--no-run` check (§3.5) proved the explicit `-p` invocations are the
  crates' only runs, not duplicates.** Kept here so nobody re-proposes
  it. The real variant: `cargo test --workspace` + `cargo clippy
  --workspace --all-targets` once, instead of root-only commands plus
  explicit `-p` invocations — one resolution, one fingerprint pass, same
  evidence. (Verify the `-p` invocations carry no extra flags first.)
- **D. Route the local boundary suite like CI does.** Skip
  `test_workorder_status_boundary.ps1` locally when no consumer of a
  changed path changed (git diff against main, same consumer list the
  plan step uses). With the local Fast dropped this is moot for
  pre-push, but it applies to any future local full run.
- **E. Share or warm the target dir across check/clippy/test/build.**
  Toolchain-level (sccache or a warm shared target dir); biggest
  potential saving against the four compile passes, but it touches the
  build environment rather than the evidence, so it is Builder + infra
  work with its own correctness review.
- **F. Batch independent compile-fail cfgs.** The 14
  forbidden-construction proofs each set one `RUSTFLAGS --cfg` and
  recompile (§3.1). Where the cfgs are independent, combining them into
  fewer `cargo check` invocations would cut recompiles — but each proof
  asserts on its own error output, so attribution must be preserved (one
  combined failing compile must still prove *which* construction was
  rejected). Saving unmeasured; Builder design work, evidence kept only
  if the per-proof assertions survive.

No option above silently drops evidence: in every case the section still
runs in CI Full (or the routed fixed profile), which is now the
documented gate.

## 7. Open questions for the Builder lane

- Answered during drafting: the explicit `cargo test -p
  windows-drive-locality` / `cargo clippy -p windows-drive-locality` are
  *not* duplicates — bare cargo commands select the root package only
  (§3.5). The remaining question is whether `--workspace` variants would
  preserve the exact evidence.
- Open: §3 holds the pinned count structure (14 cfg recompiles, 23
  mutation rows, 75 double-spawn selector sites, 4 compile passes). The
  minute-level profile is option A on uncontended CI runners.

---
*Research lane, docs-only. No code, tooling, or workflow changes made or
proposed as edits — options are recommendations for Builder-lane decision
records.*

# 0025: Close the CI Routing Gaps for Docs, Governance, and Tooling Changes

Date: 2026-09-22
Status: proposed 2026-09-22. BDFL rules and merges.

## Context

Tiered validation profiles landed (PR #4): pull requests classify to a
language, runtime, compiler, or full profile instead of every change paying
for everything. The routing policy itself -- what each change must prove --
is the BDFL's call alone, and the standing directive is that no step-3 cuts
are implemented until throwaway PRs per route are measured and the cuts
re-ranked against those numbers.

PR #11 (docs: accept 0024; seed perf-debt ledger) is the first real routing
measurement. It did not select the cheap Language profile. This record
states why, measures what that costs, and proposes the narrow fixes. A
correction folded in during drafting: the ownership registry covers
essentially nothing outside src/, fixtures/, and examples/ -- tools/ has
zero registered paths, so every CI-tooling PR, which is most of what today
consisted of, also pays Full.

## Finding 1: additions and deletions always pay Full

`Get-HumCiChangeProfile` (tools/check_ci_policy.ps1) returns `full` for any
change whose status is not an ordinary same-mode modification. The code
comment states the rationale: "Additions/deletions (including both rename
sides) require renewed ownership."

Measured: PR #11 changed exactly two files, both new
(docs/decisions/0024-state-performance-north-star.md,
docs/research/2026-09-22-performance-debt-ledger.md). The plan step selected
`full`; validation run 35787826906 finished success with a wall-clock of
39m33s (2026-09-22T21:38:15Z to 22:17:48Z). The green health anchor
(35781027455) was never consulted -- the health check only gates non-full
inner profiles.

## Finding 2: unregistered paths always pay Full

`Get-HumCiProfile` ranks any path not in the ownership registry as 3
(`full`). The registry is 309 literal paths: 307 at rank compiler
(fixtures/ 217, src/ 60, examples/ 30) and 2 at rank language (README.md,
docs/LANGUAGE_REFERENCE.md).

Unregistered: docs/decisions/, docs/research/, workorders/, tools/,
.github/, and nearly all of docs/. Two consequences:

- Even a one-word edit to a decision record costs a full ~40-minute run on
  both platforms. The documentation and governance surface the project runs
  on is, to the router, indistinguishable from a compiler change.
- tools/ has no registered paths at all. Every CI-tooling change -- the
  validation scripts, the readiness checks, the policy script itself, most
  of today's PRs -- pays Full. The project's own automation is priced like
  a compiler rewrite.

## Options considered

A. Status quo. Docs, governance, and tooling changes keep paying Full.
   Simplest, no policy risk; cost is measured and recurring (see below).

B. The fixes proposed here: (a) additions/deletions under an already-owned
   prefix classify by path; (b) prefix-level ownership for documentation,
   governance, and tooling paths, with explicit exceptions for paths that
   can affect compiled behaviour or CI integrity.

C. Register the missing paths literally at cheap ranks, without prefix
   matching. Rejected as incomplete: every new file would need
   registration, so the additions problem (Finding 1) survives and the
   registry rots.

## Decision

Adopt option B, in two parts:

(a) Additions and deletions under an already-owned prefix classify by the
    path, not by the change's novelty. Ownership attaches to a path's blast
    radius: a new fixture under fixtures/ is no riskier than editing one,
    and a new source file under src/ is still a compiler-rank change.
    Renames classify both sides and take the maximum rank. Mode and type
    changes -- symlink, gitlink, and executable-bit flips -- continue to
    force Full, as the current classifier already requires (only ordinary
    same-mode 100644 modifications escape Full); this record does not
    loosen that rule.

(b) The registry gains prefix-level ownership for documentation,
    governance, and tooling paths -- docs/decisions/, docs/research/,
    workorders/, tools/, and their peers -- so they classify by path
    instead of defaulting to Full. One general rule governs the ranks: a
    path's rank must include the checks that consume it. The consumers are
    concrete: the status-boundary classifier and the hum-dev status
    tooling read workorders/ (the active marker drives Work Order
    discovery); the public and release readiness checks scan docs/; every
    gate consumes tools/. Verification against the current workflow shows
    those consuming checks run in the Full preflight
    (tools/check_all.ps1) and the fast status-only path -- not in the
    normal language/runtime/compiler profiles. A cheap rank is therefore
    only safe if the implementation wires the path's consumers into that
    rank's profile, or keeps the path at a rank whose profile already
    executes them. Any prefix not in the ownership table still classifies
    Full: prefix ownership narrows the default; it does not make the
    unregistered cheap. Two kinds of exception keep a code-level profile:
    - Paths that can affect compiled behaviour. docs/DIAGNOSTICS.md is
      compiled into the binary via `include_str!`
      (src/diagnostic_catalog.rs:4270, 5233); editing it changes the
      binary, so it keeps a compiler-rank profile.
    - Paths that define routing or gate integrity. The policy script
      itself (tools/check_ci_policy.ps1) is the highest-sensitivity
      tooling path: its changes must run the classification's own real
      gates, not a re-implementation.

## Measured cost of the status quo

- PR #11: 39m33s, success, full profile, for two new docs files.
- Bootstrap Full reference: ~40m08s wall (run 35702815736); process cost,
  not compiler speed.
- At current velocity, every decision record, ledger entry, work-order
  update, and CI-tooling iteration costs ~40 CI-minutes on both platforms.
  The paper trail that keeps the project honest, and the automation that
  keeps it green, are both priced like compiler rewrites.

## What this record does not decide

- The step-3 cuts. Per-route measurements for language, runtime, and
  compiler are still owed before any cut is implemented.
- The exact prefix list and ranks, and whether a cheap rank's profile gains
  the path's consuming checks or the path keeps a higher rank. That mapping
  is the BDFL's call; this record proposes the shape and the rule, not the
  table. In particular, the right rank for tools/ paths -- which must prove
  CI integrity rather than compiled behaviour -- needs the prefix table,
  not a guess here.
- Whether buying the fix is worth its price: the policy edit is itself a
  tools/ change, and tools/ is unregistered, so implementing this record
  costs one more full-price run (~40 minutes) to earn cheaper docs,
  governance, and tooling runs thereafter.

## Consequences

- If accepted, the implementation is a tools/check_ci_policy.ps1 change
  (prefix-aware ownership, path-classified additions/deletions) validated
  by a Full run under the current policy.
- The ownership registry becomes prefix-based where prefixes are stable
  (docs, governance, tooling); literal file pins remain for code paths.
- Decision records, the perf-debt ledger, and CI tooling become cheap to
  evolve, which is the point: the project's memory and its automation
  should not cost a compiler build to update.
- Like 0023, if this record is accepted, the written routing ruling in
  AGENTS.md changes in the same PR as the policy code, so the written rule
  and the code stay in step.

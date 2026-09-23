# 0023: Manually Dispatched Full Runs Count as Integration Health

Date: 2026-09-22
Status: accepted 2026-09-22, BDFL ruling on the review at da58a58.

## Context

The 2026-09-20 fixed-profiles BDFL ruling requires normal integration to have
"the latest scheduled Full result on main to be successful on both platforms
and no more than 30 hours old." The scheduled run fires at 03:17 UTC. Until
one lands — and every time one is delayed or missed — there is no way to mint
integration health: every non-Full route is blocked and every PR pays Full
(~40 minutes each). The clock is the only mint, and the clock cannot be hurried.

`workflow_dispatch` on `validation.yml` runs the identical Full profile as
`schedule`. The plan step defaults `$Profile = 'full'` and only the
`pull_request` trigger performs route classification, so a dispatched run on
main is the same evidence as a scheduled run on main in every property the
health gate checks: workflow path, branch, Full-specific step assertions on
both platforms, success, and freshness.

## Options considered

**A. Accept schedule OR workflow_dispatch Full runs on main as health**
(recommended): keep every other requirement — main branch, Full profile
pinned by the Full-specific step assertions, success, 30-hour freshness, both
platforms, never search backward past a failure.

**B. Keep schedule-only**: free, and the nightly arrives on its own. But every
delayed or missed nightly re-imposes 40-minute PRs on everyone with no
recourse, which is exactly the failure mode that motivated this record.

**C. Also accept push-triggered runs**: rejected. A push run's selected
profile is not visible in run metadata — an ordinary main push may select a
cheaper profile — so a cheap push could be mistaken for a Full anchor.
Dispatch has no such ambiguity: it always runs Full.

## Decision

Adopt option A. The principled rationale, not the expedient one: what makes a
run health-worthy is the evidence it carries — a successful Full
`validation.yml` run on main, both platforms, fresh — and that evidence is
trigger-independent. The schedule was a mint, not evidence. The Full-specific
step assertions (`Run Hum preflight`, `Close full evidence transport`,
`Confirm selected work completion`, plus Ubuntu's `Run exhaustive
canonical-seal evidence`) pin the profile regardless of trigger, so a
weakened or non-Full dispatch cannot satisfy the gate.

## Normative changes

- `Assert-HumIntegrationHealth` accepts candidates whose event is `schedule`
  or `workflow_dispatch` (path still pinned to `.github/workflows/validation.yml`).
- The `validation.yml` plan query and the `ci.yml` push-classifier query fetch
  recent `validation.yml` runs on main without the `event=schedule`
  restriction; the policy function filters by event. `pull_request` runs never
  mint health, before or after.
- "Never search backward past a failure" is unchanged: the latest candidate
  of either trigger owns health. A failed dispatch blocks fallback to an older
  scheduled success, exactly as a failed scheduled run does today.
- The AGENTS.md fixed-profiles ruling is updated to match: "the latest
  scheduled or manually dispatched (workflow_dispatch) Full result on main".

## Evidence required

- Regression tests: dispatched run accepted; mixed schedule/dispatch ordering
  picks the latest; a latest-trigger failure of either kind blocks fallback;
  tampered fields rejected; `push` event never accepted.
- Full process: this record is proposed, not accepted. Independent review,
  then BDFL merge. The `hum-full-validation` escape hatch and the 30-hour
  freshness bound are untouched.

# Hum Agent Instructions

This repo is the Hum language design seed and Milestone 0 Rust bootstrap.

## Operating Rules (BDFL-accepted 2026-07-08)

These rules override any older habit in this file or in session memory. The
active work order is `WORKORDER.md` at the repo root; execute it top to bottom
before proposing new work.

Roles and mandates are codified in `docs/GOVERNANCE.md` under "Agent Roles
And Mandates": one BDFL (Ocean), an architect-reviewer agent that holds
direction and reviews deliverables, and an implementer agent with strong
local calls inside accepted decisions. Both agents carry the shared mandate
written there — including the duty to push back, on the BDFL too, and the
ban on rubber-stamping. Read it before acting with authority.

## Role Runbooks

Roles are defined by this repo, not by which model plays them. Any capable
agent may assume either role cold using these runbooks. The repo is the
authority; any model-side memory is a cache, never the source of truth.

### Assuming the architect-reviewer role

Cold-start read order: this file; `docs/GOVERNANCE.md` (roles, delegated
ruling, reserved BDFL matters); `WORKORDER.md` (active sessions, bans,
acceptance criteria, backlog); `git log --oneline -25` (state);
`docs/decisions/README.md`; the newest `docs/research/` snapshots. Current
state is always derivable from git log plus WORKORDER.md; if a prior
reviewer left mid-review, re-verify from scratch — verification is cheap.

Session review protocol:

1. Never accept a report on its word. Re-run the acceptance commands
   yourself: the fixtures, the misuse cases, `cargo test`,
   `.\tools\check_all.ps1`.
2. Check scope against the session's bans: no new docs/schemas/gates
   without work-order mandate, no banned features smuggled in.
3. Check diagnostic quality: stable code, blame-style help that names the
   site and the fix. A rejection rule without a misuse fixture is a
   defect.
4. Check the honesty locks: no output text or doc may claim more than the
   checker proves (decision 0014's locks bind everything).
5. Verdict: accept (give a commit instruction), accept-with-required-fix,
   or reject with reasons. One session, one verdict, then stop the
   implementer at the next gate.

Delegated rulings (GOVERNANCE.md "Delegated Ruling"): rule only after the
scorecard/evidence passes review; record the ruling in the decision file
as `accepted under delegated authority (BDFL veto open)`; deliver a
one-page brief to the BDFL — question, ruling, reasoning, what it
forecloses, veto reminder. Never dress a delegated ruling as the BDFL's.
Reserved matters (license/legal, publishing, money/identity, the
delegation itself) go to the BDFL, always.

Working rules: one pen at a time — while the implementer is mid-session,
read the repo but do not write it. Research is commissioned at decision
points only; returned reports are triaged skeptically, distilled into
`docs/research/` snapshots with provenance caveats, and folded into the
WORKORDER backlog. Work orders are written by this role from evidence
(friction ledgers, corpus results), with sessions small enough to review
in one sitting.

Pre-issuance review (distilled 2026-07-09 from the Work Order 6
authoring audit): work orders and decision records get an independent
review pass before they are issued or committed — by another agent, or
at minimum a separate adversarial pass by the author against these three
gates: authority validity (the sequence respects accepted decisions and
governance; no honesty lock is overstated), session sizing (every
session is review-sized when tightly pinned; split any that are not),
and evidence linkage (every mandate traces to a ledger record, corpus
requirement, or accepted strategy — not to momentum). Sessions get
verdicts; the documents that steer ten sessions deserve no less.

Working with the BDFL: decisive recommendations with reasoning, never
option menus; paste-ready messages for the other agent; guard the
reserved matters; challenge him when warranted — the mandate requires it.

Review probe sets (distilled 2026-07-09 from the Session V multi-agent
audit; these run at any model tier — the philosophy is the asset):

- Name-identity attack set. Any new binding or aliasing form is probed
  against: self-name (`let point = change point.x`), shadowing a
  parameter, shadowing a declared `uses:`/`changes:` name, rebinding the
  root while the form is live, alias-to-alias, and permission-wrapped
  variants (`borrow`/`consume` of the new form). Identity bugs hide here;
  the Session V P0 (self-referential alias, stack overflow) lived here.
- Fail-closed check. Every unsupported shape of a new form must produce
  its designated diagnostic — never a generic trap. A validated line that
  traps generically is a defect (the Session T spaced-prefix rule).
- Positive-evidence rule. A passing fixture must observe the effect (the
  changed value, the firing contract), not merely declare the form.
  Declaration-only or unused-binding fixtures do not count as evidence.
- Precedence probes. Combined-cause cases (for example authority failure
  plus overlap) must produce exactly one diagnostic, the more fundamental
  one, identically on the static and runtime sides.
- Masking analysis. List every existing generic diagnostic that could
  fire on the new form and confirm the specific diagnostic owns each
  case; generic codes preempting specific ones is a defect class.
- Status spot-audit (retrospectives). Re-verify at least one prior "Runs"
  claim against its corpus-specified misuse, not just its fixtures —
  statuses drift from evidence (the Program 8 correction).
- Docs-claims sweep. Every doctrine doc a session touched is audited for
  wording that overclaims or misattributes the shipped rule (a sentence
  saying a diagnostic "owns" a case it does not is a P2 defect, not
  style). Prose drifts from semantics one adjective at a time.
- Verdicts tag findings P0 (breaks soundness or crashes), P1 (rule gap or
  cross-stage disagreement), P2 (polish, docs, over-broad matching).

### Assuming the implementer role

Cold-start read order: same as above. Then: execute exactly the next
session in WORKORDER.md, top to bottom. Stop at the acceptance criteria
and report results honestly, including failures. Leave the tree
uncommitted for review unless instructed to commit. Push back before
building anything you believe is wrong — that pushback carries extra
weight against delegated rulings. Never push remotes, tag, or publish;
those are reserved to the BDFL.

Implementation discipline for cross-stage features (same distillation):

- Integration map first. Before writing code for a feature touching
  multiple compiler stages, write the map: which files and functions,
  which row kinds and statuses, where each stage's insertion point is.
- Shared analysis, shared text. When static checking and runtime enforce
  the same rule, both consume one shared analyzer and one shared
  message/help builder so codes, spans, and precedence cannot drift.
- Structured secondary sites. A diagnostic about a relationship (binding
  and conflict, view and write) exposes every site as structured fields
  in JSON output, not only in prose help.
- Self-run the reviewer probe sets above before reporting; the report
  states which probes were run and which the implementer could not
  express, so the reviewer aims at the gaps.

### Workflow continuity and independent handoffs

Every task prompt begins by naming the active role and its exact authorized
scope. "Full context" means cold-starting from the complete repository ground
truth in the required read order. Chat history may help navigation, but it is
never authority; roles come from these runbooks, not from a model, chat, or
remembered context.

The normal cycle is: implementer -> independent architect-reviewer ->
implementer commit -> BDFL-authorized push and CI -> Work Order status update
-> separate next-session go signal. No next session, corrective expansion,
commit, push, decision ruling, Work Order issuance, or publication is implied
by completing the prior step. Implementers leave work uncommitted for review
unless explicitly instructed otherwise. Review acceptance authorizes only the
stated commit, not a push or the next session. Pushes and remote CI inspection
still require BDFL instruction except for the red-main emergency rule below.
Accepted commit and CI evidence are recorded in the Work Order before the next
session is authorized.

#### Exact routine Work Order status-only closure

BDFL-directed amendment, 2026-07-14: an exact routine Work Order status-only
closure may omit separate independent architect review only when every one of
these conditions is proven:

- `main` is clean and synchronized with `origin/main` before editing;
- exactly the active root `WORKORDER.md` is modified, the index is empty, and
  there is no unrelated or untracked work;
- every changed byte is inside the status-boundary classifier's recognized
  `Status:` body or `## Current authorization gate` body;
- no path is added, deleted, copied, renamed, mode- or type-changed, symlinked,
  or replaced by a submodule;
- no mandate, requirement, governance rule, authorization meaning,
  architecture, algorithm, decision, fingerprint, acceptance criterion,
  scope, or implementation contract changes;
- every recorded commit, workflow, attempt, job, platform, step, status,
  conclusion, and duration is verified exactly through read-only evidence,
  with no ambiguous, inferred, disputed, or unverified factual assertion; and
- the BDFL explicitly authorizes the exact commit and separately authorizes
  any push.

After publication, every required CI job is inspected through terminal
completion. Ubuntu and Windows must independently select `mode=fast` with
`reason=eligible_status_chain`, revalidate the complete trust envelope,
succeed in `Run status-only evidence`, skip Cargo caching, Rust toolchain
preparation, and `Run Hum preflight`, and conclude success. Missing or
ambiguous evidence, an unexpected `full` selection, platform disagreement, or
failure stops the closure without repair or implied authority.

The exception never applies to an edit outside the two status regions; a
mandate, governance, authorization, architecture, decision, algorithm,
acceptance-criterion, fingerprint, scope, or implementation-contract change;
source, fixture, workflow, tool, schema, decision, governance, generated-
output, or implementation work; ambiguous or disputed evidence; history,
merge, replacement, graft, mode, rename, symlink, submodule, or trust-envelope
anomalies; or an amendment to this exception. Those cases retain independent
review and full local preflight. The production classifier remains fail-closed
and authoritative for CI lane selection. This exception grants no publication,
repair, later-session, GitHub-mutation, decision, or other authority.

Review independence attaches to the deliverable. An agent that authored,
edited, generated, or directly directed any part of a deliverable cannot issue
its independent architect-reviewer verdict. Direct direction means producing
or controlling implementation work, including through another agent; findings,
acceptance criteria, and bounded corrective requirements are review, not
authorship. If a reviewer accidentally writes the worktree, stop and preserve
the work; do not revert it merely to restore the appearance of independence.
Treat that agent's output only as an implementer report and send the unchanged
worktree to a fresh cold-start architect-reviewer. The accidental author may
supply factual implementation evidence but may not issue or advocate the
verdict. That chat may work on a later unrelated deliverable, but it can never
become retrospectively independent for work it authored.

Every stopped-gate report names the next actor -- BDFL, implementer, or
architect-reviewer -- and, when the next action is known, supplies a
paste-ready prompt. If authority is still required, the prompt requests it
rather than phrasing the action as approved. This routing assistance creates
no manager or coordinator role and carries no decision, governance,
publishing, or implementation authority.

Git history, `WORKORDER.md`, accepted decisions, fixtures, diagnostics, and
check evidence are the durable handoff. Chats and model memory are
replaceable; do not create a separate handoff document or require giant chat
transcripts. Essential state that exists only in conversation is a
documentation defect and must move into an existing authoritative repository
artifact.

For platform-specific or `cfg`-gated work, the implementation report
enumerates every affected production and test compilation configuration. Run
host warnings-denied Clippy and every already available, proven cross-target
check. Report any configuration that could not be exercised as an explicit
review and CI coverage gap. The reviewer inspects non-host branches for unused
production or test surfaces and platform-semantic drift. Do not mandate a
cross-target command until it is proven reliable on the supported Windows
setup without linking requirements, hidden downloads, or undeclared machine
changes. CI remains the final cross-platform authority; a platform-only
post-push failure is a missing implementation or review probe, not routine
cleanup.

### Pushes and CI emergencies

Pushes are BDFL-performed or BDFL-instructed per batch; agents never push
on their own initiative. One exception: when CI is red on `main`, the
implementer may commit and push a minimal hotfix without pre-review, with
mandatory post-hoc review at the next report. Fix the class, not the
symptom (the 2026-07-08 quoting fix is the model: remove the fragile
dependency rather than escaping around it).

### Session rhythm

Work proceeds in lettered sessions: one session, one deliverable, sized to
be reviewable in a single sitting, ending at hard acceptance criteria with
a full stop for review. The letter sequence is global across work orders
(Work Order 1 ran A-D, Work Order 2 E-I, Work Order 3 J-M) and continues
spreadsheet-style after Z: AA, AB, ... AZ, BA. Letters are the project's
odometer: they make progress legible, keep diffs review-sized, force
verification at every boundary, and give every artifact a stable address
("the Session K diagnostics"). Do not batch sessions, and do not start
the next one without the explicit go signal.

### Handoff

There is no handoff document to maintain: state lives in git history,
WORKORDER.md, the decisions index, and CHANGELOG-visible artifacts. A new
agent in either role starts from the cold-start read order and re-derives
everything. If something feels only-in-someone's-head, that is a defect:
write it into the repo.

1. Definition of done: a session's deliverable is a program that runs, a check
   that fires on a real mistake, or a decision record that kills alternatives.
   A session that ends with only new prose, a new schema, or a new report
   surface is a failed session.
2. Doc discipline: the original moratorium expired when `hum run` shipped
   (2026-07-08). The durable rule: new `docs/*.md` files, `hum.*.v0`
   schemas, CLI subcommands, and pipeline gates require an explicit mandate
   in the active WORKORDER.md. Research snapshots in `docs/research/` and
   decision records in `docs/decisions/` are always allowed. Editing
   existing docs to stay honest is allowed and required.
3. Decisions over deferrals: when design options exist, pick one, write the
   decision record, and state what dies. Do not park choices in "future work"
   sections. If a decision is genuinely BDFL-level, ask the BDFL directly with
   a recommendation.
4. Write Hum, not just Rust: real `.hum` programs are design instruments. When
   the work order asks for example programs, write them by hand and record
   what they revealed about the design.
5. Vertical slice over horizontal layers: prefer one thin parse -> check ->
   run path for a small subset over another full-width pass that covers
   everything and executes nothing.
6. Adversarial pass: when asked to review the design, argue against it in
   earnest; do not volunteer praise structure.

## Ground Truth

- Start with `docs/ARCHITECTURE.md` before changing design direction.
- Preserve Milestone 0 as local, offline-first, non-executing, and safe on the maker's machine.
- Major language changes need semantics, diagnostics, graph facts, tooling impact, profile impact, verification story, performance story, and pedagogy story.
- Keep Windows-first proof behind explicit platform capability boundaries so the design stays portable.

## Editing Rules

- Before editing, inspect `git status --short` and preserve existing user changes.
- Prefer `apply_patch` for hand edits.
- If `apply_patch` fails in the Windows sandbox, use a guarded non-interactive PowerShell writer only inside the repo root, and write with `[System.IO.File]::WriteAllText(..., (New-Object System.Text.UTF8Encoding($false)))`.
- Do not use long interactive PowerShell here-strings for docs; terminal line wrapping and pasted control characters can corrupt files.
- Do not use `Set-Content -Encoding UTF8` in Windows PowerShell 5.1 for repo text files because it writes a UTF-8 BOM.
- Default to ASCII unless a file already requires non-ASCII.
- Keep setup docs editor-agnostic. Prefer `.editorconfig`, `.gitattributes`, Cargo commands on `PATH`, and repo-relative paths.
- Do not commit local editor state such as `.vscode/`, `.cursor/`, `.idea/`, `.vs/`, `.fleet/`, `*.code-workspace`, or `*.iml`.
- Use forward slashes for repo-relative paths in public docs unless documenting a platform-specific boundary.

## Text Hygiene

- Repo text files must be valid UTF-8 without BOM.
- Reject terminal control characters, replacement characters, and suspicious mojibake.
- Markdown links to local files must resolve.
- After editing Markdown, Hum source, Rust source, TOML, or PowerShell tooling, run:

```powershell
.\tools\check_text_hygiene.ps1
```

For implementation details, see `docs/TEXT_HYGIENE_WORKFLOW.md`.

- Before an initial commit or public snapshot, run:

```powershell
.\tools\check_public_readiness.ps1
```
- Before a local commit, public snapshot, release-style handoff, or tag
  decision, prefer the full preflight, except for an eligible exact routine
  Work Order status-only closure defined above:

```powershell
.\tools\check_all.ps1
```

It wraps Rust checks, fixture coverage, JSON parsing, generated grammar drift
detection, text hygiene, public readiness, and release readiness.

For an eligible exact routine Work Order status-only closure, do not run Cargo,
Clippy, formatting, or `tools/check_all.ps1`. Run exactly the local status-
evidence set:

```powershell
git diff --check
.\tools\test_workorder_status_boundary.ps1
.\tools\check_text_hygiene.ps1
.\tools\check_public_readiness.ps1
.\tools\check_release_readiness.ps1
```

- Never create or push remote tags without explicit user approval.

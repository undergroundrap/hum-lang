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
  decision, prefer the full preflight:

```powershell
.\tools\check_all.ps1
```

It wraps Rust checks, fixture coverage, JSON parsing, generated grammar drift
detection, text hygiene, public readiness, and release readiness.

- Never create or push remote tags without explicit user approval.

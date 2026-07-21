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

### Repeated-rejection loop diagnosis (distilled 2026-07-15 from Work Order 9 Sessions AP-AQ)

When the same finding is rejected twice — the same-shaped rejection, not a
fresh finding each cycle — stop. Do not authorize a third blind implementer
cycle. Past two same-shaped rejections the cycle count measures spec
quality, not implementer quality, and the reviewer's rigor is not the fault:
it is correctly rejecting evidence that an unsatisfiable requirement forced
the implementer to fabricate.

Run a satisfiability audit: read the actual production code — trusting
neither the implementer's report nor the reviewer's finding — and determine
whether the requirement is reachable through the real production path as
written. Loops have three diagnosable causes, each with its own fix:

- Envelope bug (Session AP): the work genuinely needs a file the authorized
  map omitted. Fix: amend the envelope by intent, not by guessed file list.
- Ghost requirement (Session AQ, capability half): the spec demands
  behavioral proof of a transition or state production cannot reach; the
  implementer fakes it because honest evidence does not exist. Fix: re-spec
  to prove structural unreachability or insert-only, not behavior.
- Dodged hard technique (Session AQ, execution-time invariant): the honest
  path exists but is hard, so the implementer ships the easy synthetic fake.
  Fix: name and require the exact technique — there, a test-only corruption
  seam that traverses the real branches.

Fix the spec or the envelope, never the rigor. Re-issue the corrected
criteria to both the implementer and the reviewer, so the reviewer measures
against the corrected bar rather than re-rejecting the old ghost.

### Amendment and evidence discipline (distilled 2026-07-17 from Work Order 10 Increment 10B)

Four rules that sharpen the loop diagnosis above, each proven in 10B:

- Demand the complete inventory before amending, never just the file the
  rejection named. When an envelope bug or an allocation ripple surfaces,
  require the implementer to enumerate the *complete* set of affected files
  or surfaces and authorize all of them in one amendment. Twice in 10B a
  reject named one file (`typed_failure.rs`; three catalog literals) and the
  full inventory found five more, then three more. A second amendment for
  the next surprise file is the same-shaped loop — get the whole set once.
- Prove consumption by corruption. To prove a consumer actually reads an
  authoritative fact rather than re-deriving it, require that corrupting the
  fact changes the consumer's output. A stage that renders-and-reparses, or
  otherwise reconstructs from a projection, is unaffected by corrupting the
  source of truth — so this gate mechanically distinguishes real consumption
  from a shim, where a behavioral or public-projection check cannot. This is
  general: it applies anywhere you must prove "X consumes Y," not just to
  expression trees.
- Over-scoping manufactures fake work. An increment too large to implement
  honestly in one review-sized pass does not merely risk a bad review — it
  pressures the implementer to shim the hard parts to reach green (the dodged
  hard technique, at scale). A twice-rejected increment whose diff dwarfs the
  "review-sized in one sitting" bar is a re-scoping problem, not an
  implementer problem: split it per consumer, each with the corruption gate,
  before authorizing another cycle.
- Verify that selectors actually select. A test selector, filter, or
  classifier can silently match zero cases while every check stays green — a
  hole in the safety net that reports safety. Periodically prove the harness
  is exercising what it claims. When a dead selector is found, establish
  whether any *published* commit passed with it dead, and what it was meant
  to cover.

Identity and reproducibility corollary: an identity that gates an
irreversible action (an archive, a disposal, a promotion) must be
content-addressed and demonstrated reproducible across shells and platforms,
never a shell-piped text hash. Piping `git diff` through a shell varies by
encoding and line-ending conversion; Git-computed object identities
(`git write-tree`, per-file blob hashes, the archive commit SHA) do not.
Demand the identity be reproduced in both shells, not asserted.

### Ceremony proportionality and anti-pathology tripwire (distilled 2026-07-21 from Work Order 10)

Process weight must match the kind of change. This rule changes no technical
acceptance bar: real production-path evidence, complete applicable checks, one
final independent review, scoped commits, separately authorized pushes, and
terminal required CI remain mandatory where the active Work Order requires
them. It decides whether a correction is ordinary implementation work or a
substantive change that must return to the Work Order boundary.

A correction is implementer-inline when all of these are true:

- every changed path is already inside the active accepted envelope;
- semantic scope, accepted architecture, public human/JSON/runtime/schema
  behavior, diagnostic allocation and meaning, evidence meaning, and
  acceptance criteria remain unchanged;
- no dependency, platform surface, or reserved BDFL decision is added or
  changed; and
- the complete corrected deliverable still receives its real checks and final
  independent implementation review.

Missing imports or explicit qualifications; rustfmt ordering or wrapping;
compiler, Clippy, lint, warning, or typo repairs; exact-selector or
deterministic harness plumbing; and small in-envelope line-count changes are
normally implementer-inline. The implementer fixes them in place, discloses
them in the final handoff, and proves them with the complete deliverable. Such
a mechanical correction must not create its own Work Order amendment,
pre-issuance documentation review, publication relay, status record, exception
gate, or separate corrective go signal. It receives no automatic acceptance
credit merely because it is mechanical.

A substantive Work Order amendment is required for any semantic scope or
behavior change; any production, test, fixture, tool, schema, or documentation
path outside the accepted envelope; a new or changed diagnostic meaning or
allocation; public output or schema change; architecture, authority,
ownership, or precedence change; materially changed acceptance evidence; a
new dependency or platform surface; or another reserved BDFL decision. When
classification is genuinely ambiguous, stop and report the concrete ambiguity
directly to the BDFL. Ambiguity grants no edit and does not authorize an agent
to invent a gate, exception, status layer, or workaround.

Size work by dependency coherence, not a line-count quota. The governing unit
is the smallest change that compiles in every applicable configuration, is
testable through its real production path, can be independently reviewed in
one sitting, leaves no deliberately broken intermediate representation, and
does not need a later unit merely to compile, format, or make its acceptance
harness select nonzero evidence. Smaller than compilable is too small. Line
counts are review telemetry, never authority by themselves to stop, split,
reject, or create an exception. A genuinely oversized unit may be split once
at a real producer, authority, validator, or consumer dependency boundary; it
must not be decomposed into sub-sub-units or hidden phases merely to satisfy a
size target.

Any one of these conditions is a ceremony-pathology tripwire:

- two consecutive governance-only commits land without implementation code;
- a compile, formatting, Clippy, lint, warning, typo, selector,
  harness-plumbing, or small in-envelope line-count issue is proposed as a
  Work Order amendment;
- decomposition exceeds two meaningful levels or introduces a sub-sub-unit;
- a line-count target produces a stop, exception, or re-scope amendment; or
- process work consumes more commits or elapsed time than the implementation
  it governs.

When a tripwire fires, stop and return directly to the BDFL with the concrete
evidence. The response is simplification, a coherent re-scope, or a direct
BDFL ruling--never another status layer, exception gate, tripwire amendment,
or process about the process. This stop rule grants no implementation,
acceptance, commit, push, publication, or later-work authority.

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

#### Work Order lifecycle and file freshness (distilled 2026-07-15 from Work Order 9)

A Work Order is a bounded unit with a lifecycle: issued, executed session by
session, then closed. When its authorized session sequence completes, close it
— freeze the file, record the closure, and make no further amendments. Git
preserves the full history; the working file does not need to.

Start new work as a fresh Work Order file, never as more amendments to a
completed one. Over a long Work Order, `WORKORDER.md` accumulates the full
spec, every session record, CI job logs, and layered amendments; left
unbounded, that mass becomes a cold-start tax — every agent must read it to
act, and the signal-to-noise degrades. That degradation is context rot, and it
is a defect, not a badge of thoroughness.

A fresh Work Order carries forward only live, load-bearing context — accepted
decisions, active strategy, and the specific evidence its own sessions need —
not the closed order's session-by-session history, CI logs, or amendment
trail. Those live in git.

Observed live in Work Order 9: its AN-AQ sequence completed, but a post-AQ
hardening addendum kept extending the same file through repeated amendments
until the cold-start cost was itself a concern. The lesson: close on
completion, and give the next order a clean file.

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

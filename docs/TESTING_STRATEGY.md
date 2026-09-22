# Hum Testing Strategy

## Fixed validation profiles: reviewed-policy first version

This increment is separate from unfinished WO25 Unit D. It changes when
existing checks run, not what a Full result means. Local editing uses affected
selectors plus applicable format/check/Clippy and hygiene. A local focused
result is not a CI result. The typed commands are
`hum-dev evidence language|runtime|compiler --pwsh ABSOLUTE_PATH`;
`full`, `exhaustive`, `focused`, and `status` retain their existing meanings.
There are no arbitrary skip switches.

| Profile | Existing work selected |
| --- | --- |
| Language | Short real capture/exit smoke; format; root all-target checking, compiler suite and warnings-denied Clippy; compiler build; example checking; word-count resolve/graph/linked-obligation and explicit-entry checks; hygiene/readiness. |
| Runtime | Language plus shared integer-sign and constant-Text interpreter/native/CLI contract blocks. |
| Compiler | Language/Runtime consumers plus the shared compiler front and corpus blocks: privacy compile-fail checks, WO22/23/24 production-predicate mutations, compiler exact selectors, source audits, all existing CLI/corpus/projection checks (including use-after-move runtime and ownership JSON). |
| Full | Complete preflight, harness/restricted pair, source audits, exact selectors, restoration and infrastructure mutations, plus Ubuntu exhaustive. Existing order is preserved; shared groups are invoked, not copied validators. |

All normal CI profiles also run the existing Ubuntu exhaustive canonical-seal
check. Compiler calls the same front/corpus bodies as Full, in the same order;
it does not replace CLI and projection obligations with root Rust tests.
Language/Runtime omit those additional compiler corpus/source-audit blocks.
All normal profiles omit the complete capture matrix, infrastructure campaign,
Full receipt and ledger. Omissions are recorded,
not credited as passes. Normal rejects a Full-receipt request before work
and emits a distinct terminal marker. Each group's elapsed time encloses
only that group; overlapping job durations are not summed as wall time.

The accepted base's `tools/check_ci_policy.ps1` owns this finite table.
Malformed, duplicate/case-ambiguous, incomplete NUL, or invalid UTF-8
inventories cannot select cheaper work. Unknown paths and empty changes select
Full; strongest wins. Git raw records are completely validated before any
paths are published. Query failures or ambiguous inventories select Full.

| Changed paths | Route |
| --- | --- |
| Exactly `examples/probes/word_count.hum`, `README.md`, `docs/LANGUAGE_REFERENCE.md` | Language |
| `src/run.rs`, including the shipped three-file word-count change | Runtime |
| Literal registered compiler sources, examples and `.hum` fixtures, including `src/parser.rs`, `src/type_check.rs` and the ownership corpus | Compiler |
| Workflows, tools, crates, Cargo files, programs, governance/decisions, testing/release/schema policy, Work Orders, evidence fixtures, anything unlisted | Full |
| Normal paths mixed with infrastructure or unknown ownership | Full |

The registry is a closed list in accepted policy, not glob families or a list
generated from candidate files. New lookalike paths cannot admit themselves.
Only regular `100644`-to-`100644` modifications can use normal profiles.
Additions, deletions, either side of a rename, mode changes, symlinks and
gitlinks select Full, even at registered names. Schema, governance and policy
documents are Full, not ordinary documentation by extension.
Diffs use PR base to the checked-out integration commit, without rename
detection so both names participate. Its parents must match base/head. Record
those SHAs, integration SHA/tree, profile and run/attempt. Base changes require
fresh integration evidence.

### Workflow and trust boundary

`validation.yml` runs on PR updates, nightly schedule, or separately authorized
manual integration validation. It selects using accepted-base policy and
calls the unprivileged shared `ci.yml` executor on both platforms.
A base without policy selects Full bootstrap, never proposed-policy fallback.
Policy/workflow/validator changes select Full; independent review inspects old
and proposed semantics. The separately owner-authorized `hum-full-validation`
label only increases coverage to Full; it cannot waive failure.

Each platform checks actual selected-step outcomes and conclusions. The
always-running `Hum required validation` job requires successful planning
and evaluation, recognized profile, matching integration/run/attempt and PR
provenance. Failed, missing, cancelled or unexpectedly skipped work rejects,
including failure masked by a successful step conclusion.

This is maintainer-reviewed trust, not hostile-workflow attestation. A malicious
PR could rewrite execution and acceptance together. Independent review of
definitions and execution plus owner approval is the accepted boundary.
A badge or App-level restriction is not exclusive workflow identity.
No custom publisher, privileged PR checkout, App, credentials, or checks/status
write permission is introduced. Grants are contents/actions read, with PR
metadata read for planning. Auto-merge remains off; publication and required-
check/settings activation need separate authority.

### Health, rollout and recovery

Normal PR planning and ordinary main-push admission check the latest scheduled
`validation.yml` main run,
never an older green run behind a failure. It must be completed/successful on
both platforms with Full preflight, Ubuntu exhaustive and cleanup completed,
and no more than 30 hours old from creation. Missing, pending, failed,
cancelled, foreign, stale or ambiguous metadata blocks normal admission.
Schedules may be delayed or dropped; a schedule alone proves no freshness.

Immediately before integration the reviewer/owner repeats the same health
check using current GitHub run/attempt/job metadata and checks current PR
base/head/integration provenance. The accepted `Assert-HumIntegrationHealth`
and `Assert-HumIntegrationJobs` functions validate those read-only responses.
This publication check is procedural: already-green checks are not revoked
automatically when health expires or a later nightly fails.

Bootstrap and repair use separately authorized two-platform Full evaluation
of the exact rollout/repair integration candidate, independent of old health.
Infrastructure automatically selects Full; a separately authorized Full label
supports an otherwise-normal repair. Review and owner approval bind that Full
result to integration. No result is renamed nightly and no failed current
Full result is waived. Normal admission resumes only with actual fresh
scheduled health. Tags/releases retain exact-candidate Full/exhaustive checks.

Ordinary main pushes use accepted policy from the actual pre-push revision,
not policy modified by the push. The complete `before..head` range is inspected
edge-by-edge (all merge parents), including edits reverted before the final
tree. Mixed work selects the strongest route. More than 512 commits, shallow,
missing or unresolved history and absent pre-push policy select Full; there is
no last-commit or candidate-policy fallback. Tags remain Full. The existing
authenticated status-only exception takes precedence and is unchanged.
Normal main pushes, PR/nightly/manual evaluations cannot emit a compatible
push summary or Full anchor. Rollout still requires separately authorized
two-platform Full validation, followed by genuine fresh scheduled health.

### Evidence still required before rollout

`tools/test_ci_policy.ps1` exercises real routing and aggregation owners with
owned Git objects and metadata corruptions. Dispatch controls substitute
costly groups and prove routing, not execution of those groups.
The short `-CompilerConsumerOnly -HumPath ABSOLUTE_PATH` control executes the
actual shared use-after-move runtime/projection consumers, not the whole
Compiler profile. Source-body and omission controls protect shared coverage;
they do not establish complete-profile or native Ubuntu execution.
`-ProfileSmokeOnly` uses real contained short children and proves only smoke
obligations. Preserve credited unaffected evidence.

Independent whole-change review, current-candidate two-platform Full rollout,
real PR failure/skip/cancellation/provenance checks, nightly bootstrap/recovery
and publication-time health checks remain necessary. Measure queue, setup/cache,
each group, platform job and aggregate wall time on ordinary PRs, reporting
cold and warm separately. The 5-10-minute target is unmeasured; historical
Full or short control timings demonstrate no speedup. Unit D and I09 remain open.

## Portable shell and hook boundary

New Hum orchestration is owned by `hum-dev` and PowerShell 7. Every production
PowerShell route requires typed `--pwsh <absolute-path>`; ambient executable
discovery is not authority. Process launches bind that authenticated ordinary
executable, repository-rooted script, argument vector,
and explicit `PATH` and `PSModulePath`; malformed UTF-8, CR/LF/NUL, missing
executables, and paths outside the repository fail before process creation.
Commit-message semantics are owned by `hum-dev commit-message check`; shell
utilities such as `sed`, `grep`, and `cat` have no substantive validation role.

The retired Windows PowerShell 5.1 obligations were exact stdout/stderr byte
capture, exit retention, one absolute timeout, containment-before-resume,
descendant quiescence, and owned cleanup. The PowerShell 7 capture matrix keeps
each obligation and runs on Windows and Ubuntu. CI authenticates the selected
PowerShell 7 application before any evidence lane. Full producers bind its
runtime, numeric version, and executable SHA-256 into the canonical V2 summary;
platform-specific identities are authenticated independently.

Date: 2026-07-06

## Thesis

Testing should be part of the language surface, not an afterthought hidden in a
separate framework.

Hum should support two related forms:

```text
tests:
  empty title is rejected
  saved task can be shown
```

and:

```text
test add task rejects empty title {
  does:
    expect add task("") fails with TaskError.empty_title
}
```

`tests:` is an obligation. `test` is executable evidence.

The canonical `integer_sign` corpus is permanent compiler evidence rather than
a demo: positive source/interpreter/native parity covers negative, zero, and
positive inputs, while dedicated fixtures bind layout diagnostics and thirteen
initialized production-predicate mutations. The exact-selector ledger carries
five feature selectors and must remain ordered and exact-once.

## Why This Matters

A senior engineer reads requirements and immediately hears test cases:

- precondition should reject invalid input
- postcondition should be checked after success
- edge case should get a regression test
- security promise should get an adversarial test
- cost claim should get a benchmark or static cost check

Hum should make that translation visible.

## Test Kinds

Hum should eventually support:

- unit tests
- integration tests
- property tests
- fuzz tests
- regression tests
- model tests
- contract-generated tests
- benchmark tests through `benchmarks:`

Benchmarks are related, but separate. A test checks behavior. A benchmark checks
measured performance.

## Top-Level `test`

A `test` is a top-level form.

```text
test add task rejects empty title {
  why:
    empty tasks should never be saved

  uses:
    fake tasks

  covers:
    add task
    TaskError.empty_title

  does:
    expect add task("") fails with TaskError.empty_title
}
```

Tests may use the same intent blocks as tasks where useful:

- `why:` explains the test's purpose
- `uses:` names fixtures or capabilities
- `changes:` names test state that may change
- `covers:` names tasks, branches, contracts, or risks
- `needs:` declares generated input assumptions
- `watch for:` records tricky test hazards
- `cost:` prevents tests from becoming accidentally expensive
- `does:` contains executable expectations

## Property Tests

Property tests should be first-class because they match Hum's contract style.

```text
test add task saves any nonempty title(title: Text) property {
  needs:
    title is not empty
    title is not only spaces

  covers:
    add task ensures new task is saved
    add task ensures new task is not done

  does:
    let result = add task(title)
    expect result is ok
    expect tasks contains task with title
}
```

## Fuzz Tests

Fuzz tests should come naturally from `watch for:` and `protects:`.

```text
test fuzz task titles(title bytes: Bytes) fuzz {
  why:
    title input may contain unusual bytes or whitespace

  covers:
    add task watch for title may be only spaces

  does:
    let title = decode text title bytes or ""
    call add task(title)
    expect no panic
}
```

## Regression Tests

Regression tests should not get their own top-level keyword.

Use the same `test` form with a `regression` kind:

```text
test empty title with spaces is rejected regression {
  why:
    prevent blank-looking tasks from being saved again

  regression:
    found when title "   " was accepted as nonempty

  covers:
    add task watch for title may be only spaces
    TaskError.empty_title

  does:
    expect add task("   ") fails with TaskError.empty_title
}
```

Why not a separate `regression` top-level form?

- It adds another concept for beginners.
- It duplicates `test` behavior.
- It makes tooling branch around syntax instead of metadata.
- It weakens the simple rule: behavior evidence is always `test`.

Regression tests should still be first-class in tooling:

```text
hum test --kind regression
hum test --changed-contracts
hum test --stale
```

A regression test should usually include a `regression:` block that records the
bug, incident, issue, or failure mode it prevents.

## Generated Tests

Hum should generate test skeletons from:

- `needs:` invalid cases
- `ensures:` success checks
- `fails when:` error cases
- `watch for:` edge cases
- `protects:` adversarial cases
- `cost:` static cost checks
- `benchmarks:` measured performance checks

Generated tests should be visible source or generated artifacts, not invisible
magic.

## Test Diagnostics

Test failures should preserve blame:

```text
error[HUM-TEST-004]: expected `add task("")` to fail with `TaskError.empty_title`

blame:
  test add task rejects empty title

related contract:
  add task fails when title is empty
```

The test runner should link failures back to contract blocks.

## Brutal Warning

Do not let `tests:` become a checkbox list that nobody runs.

If a task declares `tests:`, Hum tooling should show whether each obligation is:

```text
missing
generated
implemented
passing
failing
stale
```

A stale test is one whose covered contract changed since the test was written.

## Milestone 0 Scope

Milestone 0 should parse top-level `test` blocks, emit them into the semantic
graph, and print generated Hum `test` skeletons for obligations that do not yet
have exact `covers:` links. It does not need to run them yet.

Milestone 1 can execute basic tests against the interpreter or tiny executable
core.

WO24 permanent evidence binds the canonical constant-Text program to six exact
selectors and N01-N08 initialized mutations. It proves source-literal mutation
changes artifact identity and interpreter/native output, unsupported typed
shapes stop at H0635, deny-first consent reaches neither JIT nor output, v2 live
equality is required, one finalized invocation/store occurs, unsupported
targets cannot earn readiness, and rejected native requests never fall back.
The prior 112-selector order remains intact and the append-only ledger contains
118 case-sensitive unique selectors.

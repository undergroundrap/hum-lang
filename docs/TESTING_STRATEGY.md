# Hum Testing Strategy

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

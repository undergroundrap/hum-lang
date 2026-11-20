# Hum Work Order 31: declaration-time lexical-scope restoration

Date: 2026-11-19
<!-- hum-active-workorder:v1 -->
Status: ACTIVE — Item A only (Session AH), issued 2026-11-19.
Predecessor: [WO29, closed](../closed/WORKORDER_29.md).

## Authorization

Ocean approved WO29 closeout and issuance of WO31 Item A only on 2026-11-19.
[Decision 0014](../../docs/decisions/0014-adopt-ownership-model.md) governs
ownership and borrowing, and [decision 0015](../../docs/decisions/0015-adopt-classified-runtime-contract-policy.md)
provides the evidence vocabulary. Item A uses existing language behavior.

This issuance records the authorized scope. The issuance task stops before
implementation. Item B and every other proposed WO31 item remain unissued,
including change-call admission, final-value transfer, CallAccess invalidation,
ownership-escape changes, and diagnostic catalog changes.

## Mission and evidence base

A shadowing declaration that never executes must not undo a completed write to
the outer variable. Preserve the existing binding, metadata, and control-flow
contracts while restoring a declaration's displaced binding at scope exit.

The independently accepted WO31 planning draft
is PR #64 at `7cca1778b2a085f7cc87b2f15b5716c65138e41d`, with work-order blob
`d092bb45f1fbddfc4796f9e037c21a1a1fa79762`. Item A comes from §4.3 of the
frozen repair map in PR #63
at head `a8b9246699ad5734f6610e4140f2198c79ab91cc`. The scope below carries
that item forward verbatim. The older planning CI result confers no validation
on this issuance or on any implementation.

## Item A — declaration-time lexical-scope restoration

The repair from the map's §4.3, credited and unchanged:

- `eval_binding` saves the displaced complete `RuntimeBinding` after
  initializer evaluation and immediately before an executed declaration
  replaces it. All insertion paths are covered.
- `eval_block` restores only declarations actually executed in that
  scope via `restore_binding`, replacing entry-time
  `block_binding_names` precomputation.
- Loop-body declarations are saved/restored per iteration.
- Completed writes, moved/view metadata, and early-exit behavior are
  preserved.
- Parameter identity through `definition_id` remains required, but
  does not itself repair rollback.

Standalone acceptance controls (effects reachable without Item B;
fixtures observe the effect, not the form): `set` to an outer binding
whose shadowing declaration never executes (write survives);
nested-block shadowing restored on return/fail/contract-violation
paths; for-each per-iteration isolation. Caller-mutating-initializer
coverage lives in Item B's integrated coverage, with explicit `change`
arguments where the permission rules require them.

## Implementation scope and acceptance

- The runtime owner is `src/run.rs` and its existing test module. Limit any
  fixture and existing CLI acceptance assertions to Item A's observable
  cases. Preserve the public syntax, existing diagnostics, and call machinery.
- Verify the actual interpreter and the observable assertions in the existing
  `hum run` CLI path. Report which effects were observed on each tested path.
- Before any later push, use the accepted planning checks for Item A:
  `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --bin hum`, `tools/test_ci_policy.ps1`,
  the exact commands behind added or changed acceptance assertions,
  `tools/check_public_readiness.ps1`, and text hygiene.
- Keep CI routing, workflow definitions, and Hum required validation outside
  this session. Independent review and publication follow Hum's existing
  workflow.

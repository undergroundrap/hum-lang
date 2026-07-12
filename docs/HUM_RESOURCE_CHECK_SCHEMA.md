# Hum Resource Check Schema

Date: 2026-07-08

Current schema: `hum.resource_check.v0`

## Purpose

`hum resource-check` is the first non-executing declared allocation and resource-intent gate after recognized local ownership checking. It consumes `hum.ownership_check.v0` readiness and the `hum.resource_report.v0` source-declared resource inventory, then checks only resource facts the current Core/body grammar can honestly see.

V0 is intentionally conservative. A task with a `does:` body must declare allocation intent with `allocates:` or a `cost:` line beginning with `allocates:`. Allocation-free declarations such as `allocates: nothing` are accepted only as declared-not-proven when the visible body has no recognized allocation risk. Visible construction shapes or call-like expressions under an allocation-free claim become blockers or errors rather than hidden trust.

This command does not execute source, emit Hum IR, prove allocation freedom, prove memory safety, infer all resource behavior, enforce runtime profiles, optimize, or claim backend readiness.

## Command

```powershell
hum resource-check [--format human|json] [--timings] <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- resource-check fixtures/resource_check/simple_pass.hum
cargo run -- resource-check --format json fixtures/resource_check/simple_pass.hum
```

The human output is for terminals. The JSON output is for agents, CI wrappers, compiler-roadmap checks, and future profile/IR verifier work.

## Top-Level Shape

```json
{
  "schema": "hum.resource_check.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "recognized_core_resources_checked_v0",
  "milestone": "0 semantic graph",
  "mode": "recognized_core_resource_gate_v0",
  "ownership_check_schema": "hum.ownership_check.v0",
  "resource_report_schema": "hum.resource_report.v0",
  "dependencies": {},
  "summary": {},
  "resource_items": [],
  "non_claims_v0": []
}
```

## Fields

- `schema`: schema name, currently `hum.resource_check.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: aggregate resource-check status
- `milestone`: current implementation milestone
- `mode`: currently `recognized_core_resource_gate_v0`
- `ownership_check_schema`: prior gate this command consumes
- `resource_report_schema`: source-declared resource inventory this command consumes
- `dependencies`: consumed summary facts, currently `ownership_check` and `resource_report`
- `summary`: counts for source-visible resource facts, blockers, and non-readiness flags
- `resource_items`: per-task allocation/resource declarations and checks
- `non_claims_v0`: claims this command must not make

## Statuses

- `recognized_core_resources_checked_v0`: every recognized V0 resource check passed
- `resource_errors_v0`: recognized V0 resource checks contradicted declared allocation/resource intent
- `blocked_by_unchecked_resource_facts_v0`: at least one visible statement has resource implications V0 cannot check yet
- `blocked_by_ownership_check_errors`: `hum.ownership_check.v0` did not pass
- `blocked_by_source_errors`: source diagnostics include errors

## Summary Shape

`summary` includes:

- `files`, `tasks`, and `resource_items`
- `resource_claims`, `allocation_claims`, and `allocation_free_claims` carried from `hum.resource_report.v0`
- `checks`, `accepted_checks`, `rejected_checks`, and `unchecked_checks`
- `blocking_issues` plus `source_errors`, `ownership_errors`, and `resource_report_errors`
- `proof_ready`, `execution_ready`, and `ir_ready`, all always `0` in V0

## Checked V0 Resource Facts

V0 recognizes and checks:

- tasks with `does:` bodies as needing explicit allocation intent
- allocation declarations under `allocates:`
- allocation declarations in `cost:` lines beginning with `allocates:`
- conservative allocation-free declarations: `nothing`, `none`, `no heap allocation`, `no allocation`, and `zero allocations`
- visible construction or aggregate shapes under allocation-free claims as contradictions or blockers
- call-like expressions under allocation-free claims as unchecked allocation-effect blockers
- prior ownership blockers as hard blockers for this gate

## Resource Item And Check Statuses

`resource_items[].status` may be:

- `recognized_core_resource_facts_checked_v0`
- `resource_errors_v0`
- `blocked_by_unchecked_resource_facts_v0`
- `blocked_by_prior_errors`

`resource_items[].checks[].status` may be:

- `accepted_conservative_allocation_free_claim_v0`
- `accepted_allocation_behavior_declared_v0`
- `rejected_missing_allocation_declaration_v0`
- `rejected_allocation_free_claim_has_visible_allocation_risk_v0`
- `unchecked_call_allocation_effect_v0`
- `not_checked_blocked_by_prior_errors_v0`

Accepted checks are declaration facts, not proofs. `accepted_conservative_allocation_free_claim_v0` means the current body shape does not visibly contradict the declaration; it does not prove allocation freedom.

## Honesty Rules

- `hum resource-check` must not execute code.
- no executable semantics
- It must not emit Core Hum, Hum IR, bytecode, machine code, backend adapter input, proof artifacts, optimized code, or executable behavior.
- no allocation-freedom proof
- It must not claim allocation-freedom proof, complete resource analysis, complete cost analysis, memory safety, profile enforcement, optimization, or backend readiness.
- It may report recognized V0 allocation/resource intent facts and explicit blockers only.
- It must block when `hum.ownership_check.v0` reports blockers.
- It must stay in sync with `hum resource-report --format json`, `hum core-contract --format json`, `hum ir-readiness --format json`, `hum capabilities --format json`, and `hum version --format json`.

## Privacy And Dependency Rules

The command is local-first:

- no network
- no cloud
- no telemetry
- no solver dependency
- no backend dependency
- no generated code execution

## Non-Goals For V0

V0 does not provide a complete resource, allocator, cost, region, profile, or memory-safety system. It is a narrow source-visible gate that lets the compiler honestly move from recognized local ownership facts to declared allocation/resource intent before future profile, IR verification, and backend work.

## Session AL Callable Facts

Participating tasks report `not_applicable_to_al_ordinary_value_v0` because
the runtime value is one nonretained definition handle with zero callable
environment. This is slice evidence, not allocation-freedom proof.

An otherwise nonparticipating exact `UInt -> UInt` task definition in the AL
exam may report `accepted_nonretained_callable_definition_constant_space_v0`
only when its existing `cost:` declares `space: O(1)` and its structured body
contains neither visible allocation construction nor a call. This consumes the
exam's resource claim without adding an `allocates:` declaration or exempting
unrelated tasks that lack that claim.

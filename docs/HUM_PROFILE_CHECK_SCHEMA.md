# Hum Profile Check Schema

Date: 2026-07-08

Current schema: `hum.profile_check.v0`

## Purpose

`hum profile-check` is the first non-executing runtime profile policy gate after declared allocation and resource-intent checking. It consumes `hum.resource_check.v0` readiness and the `hum.runtime_profiles.v0` catalog, then recognizes source-visible profile declarations without pretending profile enforcement exists.

This command exists to make strict profile claims fail closed before Hum IR readiness. A task with no explicit profile declaration defaults to the `normal` profile. Known strict profiles such as `hard realtime`, `embedded no heap`, `safety critical`, `medical class c`, and `automotive asil d` are recognized but block in V0 until enforcement and evidence checks exist.

This command does not execute source, emit Hum IR, enforce runtime profiles, narrow the stdlib, select a target, certify safety, prove memory safety, prove allocation freedom, prove realtime behavior, measure performance, measure footprint, or claim backend readiness.

## Command

```powershell
hum profile-check [--format human|json] [--timings] <file-or-dir>...
```

During the Rust bootstrap:

```powershell
cargo run -- profile-check fixtures/profile_check/simple_pass.hum
cargo run -- profile-check --format json fixtures/profile_check/simple_pass.hum
```

## Top-Level Shape

```json
{
  "schema": "hum.profile_check.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "recognized_profile_policy_checked_v0",
  "milestone": "0 semantic graph",
  "mode": "recognized_profile_policy_gate_v0",
  "resource_check_schema": "hum.resource_check.v0",
  "runtime_profiles_schema": "hum.runtime_profiles.v0",
  "runtime_profile_schema": "hum.runtime_profile.v0",
  "runtime_profile_mode": "contract_only_no_profile_enforcement",
  "dependencies": {},
  "summary": {},
  "profile_items": [],
  "non_claims_v0": []
}
```

## Fields

- `schema`: schema name, currently `hum.profile_check.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: aggregate profile-check status
- `milestone`: current implementation milestone
- `mode`: currently `recognized_profile_policy_gate_v0`
- `resource_check_schema`: dependency schema for the prior gate
- `runtime_profiles_schema`: runtime profile catalog schema consumed by this gate
- `runtime_profile_schema`: individual profile entry schema consumed by this gate
- `runtime_profile_mode`: must remain `contract_only_no_profile_enforcement` in V0
- `dependencies`: compact summaries of `resource_check` and `runtime_profiles` inputs
- `summary`: counts for files, tasks, profile declarations, checks, blockers, and non-readiness fields
- `profile_items`: one row per item with profile declarations, plus body tasks that default to `normal`
- `non_claims_v0`: claims this command must not make

## Summary Shape

`summary` contains:

- `files`
- `tasks`
- `profile_items`
- `declared_profiles`
- `default_profiles`
- `known_profiles`
- `unknown_profiles`
- `strict_profiles`
- `checks`
- `accepted_checks`
- `rejected_checks`
- `unchecked_checks`
- `blocking_issues`
- `source_errors`
- `resource_check_errors`
- `proof_ready`: always `0` in V0
- `execution_ready`: always `0` in V0
- `ir_ready`: always `0` in V0

## Profile Item Shape

Each `profile_items` row has:

- `id`: source-derived profile row ID
- `kind`: owner item kind, such as `app`, `type`, `store`, `task`, or `test`
- `name`: owner item name
- `graph_node_id`: semantic graph node ID for the same item
- `source_span`: owner source span
- `status`: item-local profile status
- `declarations`: source declarations and catalog policy facts
- `checks`: profile checks and blocker rows

`declarations` preserve source section, text, normalized catalog ID, source span, and the matched `hum.runtime_profile.v0` policy entry when the profile is known.

## Status Values

- `recognized_profile_policy_checked_v0`: source parsed, resource check passed, and all visible profile policy declarations are known `normal` policy declarations or default `normal` declarations
- `profile_errors_v0`: one or more visible profile declarations are unknown and fail closed
- `blocked_by_unchecked_profile_policy_v0`: one or more known strict profiles were declared, but V0 cannot enforce the policy or validate required evidence yet
- `blocked_by_resource_check_errors`: the prior `hum.resource_check.v0` gate reported blockers
- `blocked_by_source_errors`: source diagnostics include errors

## Check Status Values

- `accepted_normal_profile_policy_v0`: `normal` profile policy is recognized, but not enforced
- `rejected_unknown_profile_v0`: profile declaration is not present in the runtime profile catalog
- `unchecked_strict_profile_enforcement_v0`: profile is known and strict, but enforcement and evidence checks do not exist yet
- `not_checked_blocked_by_prior_errors_v0`: source or resource-check errors prevented profile checking

## Honesty Rules

- `hum profile-check` must consume `hum.resource_check.v0` before any profile gate claim.
- It must use `hum.runtime_profiles.v0` as the catalog for known profile IDs.
- Unknown profile names fail closed.
- Known strict profiles fail closed until enforcement and evidence checks exist.
- The default profile for body tasks with no explicit declaration is `normal`.
- V0 strict-profile blockers are not proof that the source violates the profile. They are proof that Hum cannot honestly enforce the profile yet.

## Non-Claims

V0 must keep these non-claims visible:

- no profile enforcement
- no stdlib narrowing
- no executable runtime behavior
- no certification claim
- no target selection
- no host probing
- no performance or footprint measurement
- no concurrency-safety proof
- no memory-safety proof
- no Hum IR emission
- no backend lowering
- no proof artifact

## Related Surfaces

- [RUNTIME_PROFILES.md](RUNTIME_PROFILES.md)
- [HUM_RUNTIME_PROFILES_SCHEMA.md](HUM_RUNTIME_PROFILES_SCHEMA.md)
- [HUM_RESOURCE_CHECK_SCHEMA.md](HUM_RESOURCE_CHECK_SCHEMA.md)
- [HUM_IR_READINESS_SCHEMA.md](HUM_IR_READINESS_SCHEMA.md)
- [HUM_CORE_CONTRACT_SCHEMA.md](HUM_CORE_CONTRACT_SCHEMA.md)
- `hum capabilities --format json`
- `hum version --format json`

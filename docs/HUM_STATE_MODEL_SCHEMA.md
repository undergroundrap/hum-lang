# Hum State Model Schema

Date: 2026-07-07

## Purpose

`hum state-model --format json` emits the current state-management contract.

This is a contract surface for language design, tools, agents, future Core Hum
lowering, effect checking, ownership checking, profile checking, and debugging.

It is not a borrow checker or runtime.

## Schema IDs

- Catalog schema: `hum.state_model.v0`
- Permission entry schema: `hum.state_permission.v0`

## V0 Shape

```json
{
  "schema": "hum.state_model.v0",
  "permission_schema": "hum.state_permission.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "milestone": "0 semantic graph",
  "mode": "contract_only_partial_declared_mutation_check",
  "state_kinds": [],
  "permissions": [],
  "gates": [],
  "rules": [],
  "non_goals_v0": []
}
```

## State Kind Shape

```json
{
  "id": "mutable_local",
  "status": "current_preview",
  "surface": "change plus set",
  "role": "task-local state with explicit mutation permission",
  "default_permission": "exclusive_change",
  "profile_pressure": "strict profiles require bounded lifetime, visible allocation, and no hidden sharing"
}
```

## Permission Shape

```json
{
  "schema": "hum.state_permission.v0",
  "id": "change",
  "status": "current_preview",
  "meaning": "exclusive permission to mutate a local place or declared external place",
  "surface": "change, set, changes",
  "rejects": []
}
```

## Gate Shape

```json
{
  "id": "declared_local_mutation",
  "status": "current",
  "requirement": "set name = ... requires local change name: ... or a matching changes entry"
}
```

## Status Values

- `current`: implemented checker behavior exists today.
- `current_preview`: recognized by current syntax or body preview, but not a full semantic checker.
- `current_parse`: parsed and exposed as structure, but not fully checked.
- `reference`: accepted design rule, not yet implemented.
- `planned`: required future implementation.
- `deferred`: intentionally later than the first executable core.

## Honesty Rules

- V0 must stay `contract_only_partial_declared_mutation_check`.
- V0 may name ownership, borrowing, linear resources, and sharing, but must not claim they are enforced.
- A future borrow checker must preserve the source vocabulary: read, own, borrow, change, consume, share.
- Profile restrictions may narrow state permissions but cannot create hidden state authority.
- External state must remain visible through effects, capabilities, profiles, target facts, or artifact evidence.

## Related Surfaces

- [STATE_MODEL.md](STATE_MODEL.md): doctrine and design rules
- [FORMAL_CORE.md](FORMAL_CORE.md): Core Hum places and statements
- [HUM_CORE_CONTRACT_SCHEMA.md](HUM_CORE_CONTRACT_SCHEMA.md): machine-readable Core Hum boundary
- [RESOURCE_REPORT_SCHEMA.md](RESOURCE_REPORT_SCHEMA.md): resource and allocation claims
- [HUM_RUNTIME_PROFILES_SCHEMA.md](HUM_RUNTIME_PROFILES_SCHEMA.md): profile restrictions
- `hum capabilities --format json`: advertises `hum state-model --format json`

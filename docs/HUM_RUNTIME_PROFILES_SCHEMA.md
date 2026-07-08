# Hum Runtime Profiles Schema

Date: 2026-07-07

## Purpose

`hum profiles --format json` emits the current runtime profile policy catalog.

This is a contract surface for tools, agents, documentation checks, `hum profile-check --format json`, and future
package/build metadata. It does not enforce profiles yet.

## Schema IDs

- Catalog schema: `hum.runtime_profiles.v0`
- Profile entry schema: `hum.runtime_profile.v0`

## V0 Shape

```json
{
  "schema": "hum.runtime_profiles.v0",
  "profile_schema": "hum.runtime_profile.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "milestone": "0 semantic graph",
  "mode": "contract_only_no_profile_enforcement",
  "profiles": [],
  "rules": [],
  "non_goals_v0": []
}
```

Each profile entry has this shape:

```json
{
  "schema": "hum.runtime_profile.v0",
  "id": "hard_realtime",
  "source_spelling": "hard realtime",
  "status": "reserved_v0",
  "purpose": "control loops, audio engines, robotics, medical control, and strict-deadline jobs",
  "forbids_by_default": [],
  "requires_evidence": [],
  "allowed_capability_families": [],
  "denied_capability_families": []
}
```

## Field Meanings

- `mode`: must be `contract_only_no_profile_enforcement` in V0.
- `profiles`: named profile policy records.
- `id`: stable machine identifier, using snake_case.
- `source_spelling`: human-facing spelling from [RUNTIME_PROFILES.md](RUNTIME_PROFILES.md).
- `status`: must not imply enforcement in V0; current entries are `reserved_v0`.
- `forbids_by_default`: policy facts the profile intends to reject once enforcement exists.
- `requires_evidence`: evidence categories the profile will require before serious release claims.
- `allowed_capability_families`: capability families the profile may allow when explicitly declared.
- `denied_capability_families`: capability families denied by profile policy even if target facts say available.
- `rules`: catalog-level rules shared by profiles.
- `non_goals_v0`: claims this command must not make.

## Honesty Rules

- The catalog is not a runtime, package manager, verifier, optimizer, or target selector.
- Profile denial is policy-level intent until compiler enforcement exists.
- Unknown or conflicting profile rules must fail closed when enforcement is added.
- No profile can certify safety without source facts, target facts, toolchain identity, and evidence.

## Related Surfaces

- [RUNTIME_PROFILES.md](RUNTIME_PROFILES.md): profile doctrine and candidate profile definitions
- [TARGET_FACTS_SCHEMA.md](TARGET_FACTS_SCHEMA.md): target capability availability before profile policy
- [HUM_PROFILE_CHECK_SCHEMA.md](HUM_PROFILE_CHECK_SCHEMA.md): first non-executing profile policy gate
- [SEMANTIC_GRAPH_SCHEMA.md](SEMANTIC_GRAPH_SCHEMA.md): future graph links for active profiles
- `hum capabilities --format json`: advertises `hum profiles --format json`

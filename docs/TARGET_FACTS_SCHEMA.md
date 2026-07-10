# Hum Target Facts Schema

Date: 2026-07-07

Current schemas: `hum.target_facts.v0`, `hum.target_fact_record.v0`

## Purpose

`hum target-facts` publishes the non-executing portability contract introduced
in [PORTABILITY_BOUNDARY_MODEL.md](PORTABILITY_BOUNDARY_MODEL.md).

The command is a contract surface, not a target probe. It lets tools discover:

- which target fact fields Hum reserves
- which capability families Hum recognizes
- how absent capabilities should fail closed
- which fixture records exist for common target shapes
- which claims V0 explicitly does not make

This prevents editors, CI wrappers, agents, backend adapters, and package tools
from guessing what `target`, `platform`, `WASI`, `embedded`, or `portable`
means.

## Command

```powershell
hum target-facts
hum target-facts --format json
```

During the Rust bootstrap:

```powershell
cargo run -- target-facts
cargo run -- target-facts --format json
```

The human output is for terminals. The JSON output is the adapter and agent
contract.

## Top-Level Shape

```json
{
  "schema": "hum.target_facts.v0",
  "tool": "hum",
  "version": "0.0.1",
  "status": "pre-alpha",
  "milestone": "0 semantic graph",
  "record_schema": "hum.target_fact_record.v0",
  "mode": "contract_only_no_host_probe",
  "boundary_model": "docs/PORTABILITY_BOUNDARY_MODEL.md",
  "default_policy": "unknown_fails_closed",
  "field_catalog": [],
  "capability_families": [],
  "fixture_records": [],
  "non_goals_v0": []
}
```

## Top-Level Fields

- `schema`: report schema, currently `hum.target_facts.v0`
- `tool`: tool name, currently `hum`
- `version`: package version reported by the build
- `status`: maturity label such as `pre-alpha`
- `milestone`: current implementation milestone
- `record_schema`: schema used by individual fixture records
- `mode`: must be `contract_only_no_host_probe` in V0
- `boundary_model`: owning design document
- `default_policy`: unknown facts fail closed
- `field_catalog`: reserved target-fact fields
- `capability_families`: reserved platform and artifact authority families
- `fixture_records`: built-in sample target fact records
- `non_goals_v0`: explicit non-claims

## Field Catalog Entries

Each `field_catalog` entry has:

- `name`: stable field name
- `kind`: `identity`, `layout`, `platform`, `authority`, or `hardware`
- `required`: whether a serious record must answer the field
- `description`: human-facing field meaning

Current required fields include:

- `triple`
- `os`
- `arch`
- `abi`
- `endian`
- `pointer_width_bits`
- `path_kind`
- `newline_policy`
- `filesystem`
- `process`
- `network`
- `clock`
- `random`
- `atomics`
- `simd`

## Capability Family Entries

Each `capability_families` entry has:

- `family`: stable capability family identifier
- `status`: currently `reserved`
- `examples`: examples in that family
- `absence_policy`: what tools should do when a program requires the family
  but a target or profile lacks it

Reserved V0 families include:

- `target.layout`
- `target.cpu`
- `target.memory`
- `target.path`
- `os.filesystem`
- `os.stdio`
- `os.clock`
- `os.random`
- `os.process`
- `os.network`
- `sandbox.host`
- `artifact.release`

## Target Fact Record Shape

Fixture files under [../fixtures/target_facts](../fixtures/target_facts) use
`hum.target_fact_record.v0`:

```json
{
  "schema": "hum.target_fact_record.v0",
  "id": "windows-x86_64-msvc",
  "status": "fixture",
  "absence_policy": "unknown_or_absent_capabilities_fail_closed",
  "facts": {},
  "capabilities": [],
  "non_claims": []
}
```

Each record is illustrative but machine-readable. It is allowed to be useful
before Hum can build that target, because its status is `fixture` and its
`non_claims` explicitly reject artifact support or host probing.

## Record Facts

The `facts` object contains the current required target facts. A record should
prefer specific values such as `available_profile_gated`, `absent_by_default`,
`import_required`, `device_specific`, or `profile_dependent` over vague
phrasing.

## Capability Availability Entries

Each record `capabilities` entry has:

- `family`: one reserved capability family
- `availability`: target availability before source/profile checks
- `note`: short explanation for tools and reviewers

Availability is not permission. A target may have a filesystem, process model,
or network stack while a strict Hum profile denies it. For Milestone 0,
`absent_by_default`, `mostly_absent`, `reserved_mapping_only`, and omitted
capability entries are treated as unavailable for source `requires:` checks.

Session Z adds `os.stdio` to every fixture with
`reserved_mapping_only`. This is a mapping reservation, not target
availability, host probing, permission, profile enforcement, or process-spawn
authority. It does not satisfy a source `requires: os.stdio`; that requirement
fails closed under H1204 until a target record advertises real availability.

## Semantic Graph Link

`hum.semantic_graph.v0` includes a reserved top-level `portability` object that
points back to this schema through `target_facts_schema` and
`target_fact_record_schema`.

In Milestone 0 that graph object records `reserved_v0`,
`source_analysis_only_no_target_selection`, explicit non-claims, and normalized
source declarations from `targets:` sections. Recognized `targets:` lines can
fill `target_fact_records`, `required_capability_families`,
`denied_capability_families`, and `source_target_declarations`. `hum check`
validates those source names against this catalog and emits `H1201`, `H1202`,
`H1203`, `H1204`, or `H1205` for unknown, unsupported, unavailable, or
contradictory declarations, but they do not select a backend target, probe the
host, enforce a profile, or prove an artifact is portable. Future
source/profile analysis should add adapter
identities, profile-granted capabilities, artifact evidence, and deeper
portability diagnostics.

## Adapter Rules

- Query `hum target-facts --format json` before assuming a target fact field or
  capability family exists.
- Treat fixture records as examples, not build support.
- Treat `unknown_fails_closed` as the default portability policy.
- Do not infer OS path semantics from strings.
- Do not infer network, clock, random, filesystem, or process authority from a
  target name.
- Do not claim portable artifacts without release evidence.

## Non-Goals For V0

V0 does not promise:

- host capability probing
- backend target selection
- artifact generation
- runtime profile enforcement
- package or foreign build-script execution
- portability certification

The surface exists so portability can become a checked, versioned contract
instead of a marketing word.

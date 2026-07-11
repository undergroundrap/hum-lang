# 0017: Adopt A Structural App And Explicit Authority Boundary

Date: 2026-07-10
Status: accepted under delegated authority (BDFL veto open)

## Context

Hum needs one program root before it can add external operations. Global task
lookup is sufficient for a bootstrap probe but cannot define application
identity, authority, or operator consent. At the same time, treating source
declarations as permission would recreate ambient process authority under a
different spelling.

Session X proves only the structural half of this proposal. Sessions Y and Z
must independently prove source capability closure, operator grants, and the
first bounded output operation before a delegated ruling is permitted.
Sessions AB and AC later establish a non-Text, fail-closed native path boundary
before any file read.

## Proposed Decision

### Structural Program Root

An executable run input has exactly one top-level `app`. The app has exactly
one meaningful `starts with:` line containing one bare snake_case name of a
task directly nested in that app. The start task returns `Unit` or
`Result Unit, E`; its parameters receive the existing runner arguments.

App-mode lookup is lexical to the app and never falls back to a same-named
external task. Successful app completion adds no automatic `Unit` display.
Typed app failure remains an exit-1 causal failure and is rendered on stderr.
Source diagnostics also use stderr. A file without an app retains legacy
single-task behavior, while explicit `--entry` remains a direct pure-task probe
and does not become app mode.

`starts with:` is not state initialization. App state initialization remains
undesigned.

### Source Capability Vocabulary And Closure

The initial source capability vocabulary is exactly:

| Source capability | Core effect | Runtime/target meaning |
| --- | --- | --- |
| `stdout.write` | `output` | bounded bootstrap stdout adapter; reserved `os.stdio` mapping |
| `clock.replay` | `time` | ordered runner replay input, with no host clock and no `os.clock` requirement |
| `files.read` | `file` | one exact local file through the bootstrap adapter and `os.filesystem` family |

Other `uses:` lines remain ordinary dependencies and grant no runtime
authority. The app declaration is the maximum source authority. Recognized
tasks declare direct and transitive external capability closure, callers cover
callees, and the app covers the start-task closure. Unknown capability-like
IDs and unidentifiable calls in the executable authority subset fail closed.
This is closed direct-call analysis, not effect polymorphism or a general
capability-value system.

Each pinned capability has a typed exact policy tuple: kind, scope, strength,
one-run lifetime, ordinary external-authority severity, Core mapping, and
runtime/target meaning. Existing effect boundary rows preserve a stable policy
ID and the complete app/task/call/declaration route so later operator decisions
and operation exercises can join the source snapshot without reconstructing it
from prose. Sandbox-bypass authority (process launch, FFI, unsafe, or
unrestricted import) is a separate severity tier and cannot be introduced as
an ordinary grant.

### Operator Grant Algebra

Source declaration is never consent. For an operation, effective authority is
the intersection of:

1. the app's declared maximum authority;
2. the reachable task's declared capability closure; and
3. the operator's exact grant;

minus any matching deny. The default operator grant set is empty, exact
duplicates are idempotent, and deny wins. Direct `--entry` cannot acquire
external authority or bypass the app boundary.

Exact grants must preserve kind, object/scope, strength, and lifetime.
Persistence is an explicit separately reviewable action; wildcards, if ever
admitted, are visibly dangerous rather than an implicit widening. Consent is
task-coupled and never a startup prompt. A future audit trail must join the
source policy snapshot, operator decision event, and operation exercise event
through stable request/policy IDs, with the exact effective intersection,
deny, requesting task/package/source route, rationale surface, lifetime, and
decision reason sufficient for forensic replay. Session Y supplies the source
snapshot and policy join ID. Session Z supplies typed in-memory one-run
decision/exercise facts joined to that ID, including the complete
app/start/caller/output route and every call occurrence. Multiple paths to one
leaf retain distinct stable IDs, and runtime selection keys the actual dynamic
lexical call occurrences rather than task names or execution order. Shared
separator-normalized source identity makes equivalent Windows `/` and `\`
input spellings select the same policy without rewriting display spans. H0624
rejects output-reachable recursion until a finite or summarized causal audit
model is separately earned, but only after H0621/H0618 authority coverage is
valid. The exact `stdout_write` name is reserved
against user task declarations, and `reserved_mapping_only` cannot satisfy a
source `requires:` declaration. Session Z deliberately adds no runtime JSON or
persistence surface.

### Path Boundary

Filesystem paths are not `Text` and are not authority. The first `Path` is an
opaque runner-constructed native identity with no source literal, Text
conversion, formatting, comparison, concatenation, traversal API, return, or
storage surface. Native OS text crosses the runner boundary losslessly.

The first lexical slice is Windows-first and accepts only an ordinary
drive-letter-rooted candidate after fail-closed rejection of relative,
drive-relative, traversal, UNC, verbatim/device/NT/volume, alternate-stream,
trailing-dot/space, and normalized DOS-device aliases. Lexical `Prefix::Disk`
evidence is not proof of locality and authorizes no file operation.

Fixed-local classification is isolated in one small audited bootstrap adapter,
outside the main crate that continues to forbid unsafe code. That adapter may
use only the bounded query chain recorded by Session AC over an already
validated drive root and synthesized volume/physical-disk device names. It
performs no network, process, environment, registry, candidate-file, or content
access. Non-Windows classification fails as unsupported in this initial slice.

### 2026-07-11 Amendment: Threat-Scoped `fixed_local_v0`

`fixed_local_v0` means that, under a trusted Windows kernel, trusted storage
drivers, and a non-deceptive hypervisor, the complete observable backing chain
contains no mapped, network, fabric, file-backed, virtual, removable, or
unknown layer. It is not proof against a malicious or deceptive member of that
TCB and is not a filesystem sandbox.

`GetDriveTypeW` plus `QueryDosDeviceW` is insufficient by itself because fixed
media and an ordinary volume-manager mapping do not disclose all backing
transport. The bounded adapter must also find no type-2 VHD/VHDX/ISO storage
dependency, obtain a complete nonempty physical extent list, accept every
backing disk as non-removable with bus type exactly ATA, SATA, or NVMe, and
observe identical drive-type and mapping facts after inspection. All fabric,
virtual, file-backed, removable, partial, failed, changing, unsupported, or
unknown evidence remains `locality_unclassified`.

The only additional calls are zero-desired-access `CreateFileW` for synthesized
volume and physical-disk device names, the two named query-only
`DeviceIoControl` requests, type-2 `GetStorageDependencyInformation`, and
`CloseHandle`. Candidate files are never opened or inspected. Session AD may
rely only on this exact threat-scoped property and remains unauthorized until
Session AC is accepted.

## Consequences

The program root becomes structural before it becomes authoritative. Future IO
must cross three independently visible facts: source maximum, reachable task
closure, and exact operator consent. The path design prevents lossy Text
round-trips and hidden Windows network/device mappings from being mistaken for
local file authority.

Sessions X and Y implement structural selection plus the checked source
capability root. Session Z adds only exact one-run `stdout.write` allow/deny,
deny-wins intersection, bounded immediate output, typed failure, Core `output`,
reserved `os.stdio`, and joined in-memory decision/exercise evidence. It adds
no prompts, persistence, wildcards, runtime JSON, broader IO, target
availability, Path values, host queries, or filesystem access. The proposal
remains open for independent review of Sessions X-Z together.

## Alternatives Rejected By The Proposal

### Global Task Lookup In App Mode

Rejected because an external same-named task would make the program root
depend on input composition rather than app structure.

### Source Declaration As Consent

Rejected because code may describe required authority but cannot grant itself
operator permission.

### Ambient Process Authority

Rejected because inherited filesystem, network, clock, or process access is
not visible in Hum source or runner evidence.

### Authority-Bearing `--entry`

Rejected because a probe flag must not bypass the application authority root.

### Path As Text Or Implicit Current Directory

Rejected because strings lose native identity and current-directory resolution
hides platform and operator policy.

### Lexical Absolute Path As Locality Proof

Rejected because Windows drive letters can denote remote, substituted, device,
or otherwise ambiguous mappings.

### Replay Clock As Host Clock Authority

Rejected because deterministic runner input must not silently become
`os.clock` access.

## BDFL Note

This proposal deliberately sequences identity, source authority, operator
consent, and host adaptation. Sessions X and Y prove identity and the checked
source maximum/closure. Session Z now supplies the operator-grant and
bounded-output evidence. The record remains proposed; only the independent
architect-reviewer may accept it after verifying Sessions X-Z together.

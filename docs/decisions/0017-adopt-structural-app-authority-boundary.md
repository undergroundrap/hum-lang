# 0017: Adopt A Structural App And Explicit Authority Boundary

Date: 2026-07-10
Status: proposed

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

### Operator Grant Algebra

Source declaration is never consent. For an operation, effective authority is
the intersection of:

1. the app's declared maximum authority;
2. the reachable task's declared capability closure; and
3. the operator's exact grant;

minus any matching deny. The default operator grant set is empty, exact
duplicates are idempotent, and deny wins. Direct `--entry` cannot acquire
external authority or bypass the app boundary.

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
use only bounded calls to `GetDriveTypeW` and `QueryDosDeviceW` over an already
validated drive root to reject remote, mapped/substituted, removable, device,
unsupported, API-failure, or unknown mappings before any candidate metadata or
open. It performs no network, process, environment, registry, candidate-file,
or content access. Non-Windows classification fails as unsupported in this
initial slice.

## Consequences

The program root becomes structural before it becomes authoritative. Future IO
must cross three independently visible facts: source maximum, reachable task
closure, and exact operator consent. The path design prevents lossy Text
round-trips and hidden Windows network/device mappings from being mistaken for
local file authority.

Session X implements only structural selection and diagnostics. It does not
implement capability closure or rejection, operator flags, IO built-ins, Core
`output`, `os.stdio`, Path values, host queries, or filesystem access. The
proposal remains open until the scheduled evidence gates land.

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
consent, and host adaptation. Session X proves only identity. The
architect-reviewer may accept that implementation and authorize Session Y, but
must not accept this record until the required capability and executable
evidence gates are independently verified.

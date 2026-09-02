# Hum Evidence Summary Schema

Status: frozen V1 history and canonical V2 production/status consumption.

## Authority

`hum-dev` and this document own the versioned evidence-summary family. V1 is
frozen historical evidence; V2 is normative for Unit C production and status
credit. Hum compiler producers continue to own
language meaning, typed artifacts, diagnostics, selectors, mutations, and
readiness. PowerShell, workflow steps, GitHub artifacts, and human prose may
transport or project a summary; they do not reinterpret it or synthesize
success.

A summary is derived evidence. It cannot override a red, missing, partial,
stale, or inconsistent underlying stage, and it grants no authority to edit,
commit, push, rerun, publish, optimize, or clean an unowned path.

## Canonical bytes

The canonical representation is one compact JSON object encoded as UTF-8
without a BOM. Fields occur in the order listed below. Arrays preserve their
authenticated order. The file contains LF only and ends with one LF.
Platform-neutral strings are not trimmed, case-folded, or Unicode-normalized.
JSON escaping is deterministic.

1. `schema`
2. `policy`
3. `generator`
4. `commit`
5. `tree`
6. `candidate_manifest`
7. `platform`
8. `target`
9. `toolchain`
10. `profile`
11. `invocation_binding`
12. `selectors`
13. `selector_stream_sha256`
14. `mutations`
15. `mutation_stream_sha256`
16. `expected_stages`
17. `stages`
18. `terminal`
19. `exit`
20. `stdout_sha256`
21. `stderr_sha256`
22. `event_sha256`
23. `cleanup`

All SHA-256 values are 64 lowercase hexadecimal characters over the exact
owned bytes. The selector stream is each case-sensitive selector followed by
LF, including the last selector. Mutation records are ordered and bind the
mutation ID, terminal result, and byte-exact restoration identity; the
mutation stream is the LF-terminated sequence of those three fields separated
by ASCII vertical bars.

## Profiles

The closed V1 profile set is `focused`, `status`, `full`, and `exhaustive`.
Unknown profiles fail closed. A profile and policy version select one exact,
canonically ordered expected-stage set. Legacy profile execution remains
authoritative only behind the named `legacy_adapter` boundary while migration
evidence is incomplete.

## Stage closure

Overall success requires exact ordered equality between `expected_stages` and
the emitted stage names. Every expected stage has exactly one terminal record.
A passed stage has no skip reason. A skipped stage has one nonempty reason from
the policy's closed catalog and authenticates its permitting predicate.

Each stage carries the common invocation binding and a SHA-256 identity for
its authenticated underlying record. Missing, duplicate, unknown, reordered,
partial, stale, unexpected, unauthenticated, or mixed-binding stages fail
closed. A success projection over any red or incomplete underlying record
fails closed. Unit A can serialize a successful focused summary only from its
exact six-selector, four-mutation, and four-stage evidence closure. The bare
`hum-dev evidence summarize` command has no such inputs and therefore rejects;
it cannot manufacture green placeholders for Unit B transport.

## Candidate identity

Candidate identity is derived directly from Git object records and raw file
bytes. It includes commit, the complete parent list (empty for a valid root
commit), tree, symbolic or detached HEAD, exact local refs, every index-stage
entry and intent-to-add bit, and distinct HEAD/index/worktree/untracked path
facts. Raw worktree bytes and path kinds are authenticated; Windows does not
invent a prospective executable mode for an untracked file. Raw and
whitespace-insensitive accounting use independent calculations. Shell-piped
text is not a hash authority. Identity inspection writes no object and changes
no repository or Git configuration.

## Process and streams

`terminal` and `exit` name the controlled disposition. Stream hashes bind the
complete stdout, stderr, and normalized event streams. Timing starts before
the operating system is asked to create the exact process and ends after exit,
stream completion, result parsing, and owned controlled-path cleanup.

## Cleanup

Every current-invocation temporary resource has one initialized owner. The
owner records its exact path, residue class, and disposition and closes it on
normal success, handled failure, and catchable panic. Cleanup authenticates an
ordinary owned path beneath the OS temporary root before deletion. Foreign,
ambiguous, sibling, parent, repository, stash, archive, and historical paths
are never inferred as owned.

An uncatchable process or host failure has no impossible same-process cleanup
guarantee. Later stale-resource discovery is a separately versioned WO25
increment. V1 Unit A makes no claim that all historical temporary artifacts
are absent.

## Nonclaims

This schema is not a compiler semantic graph, test framework, package manager,
workflow protocol, cache authority, proof system, credential store, remote
transport, or publication gate. It does not prove language correctness merely
because serialization succeeds. It does not make benchmark observations into
proofs, skipped stages into passed stages, or legacy wrappers into semantic
owners.

Unit A does not retire PowerShell, replace GitHub transport, implement
change-aware reuse, reconcile crash residue, or update the machine-global
commit hook. Those obligations remain assigned to later WO25 units.

## Full-anchor producer

Only a terminal-green `full` job may run `hum-dev evidence summarize --output
<owned-path>`. The producer derives its commit, sole parent, tree, clean
candidate state, Cargo lock hash, running executable hash, and numeric current
job identity from repository bytes and authenticated current-run metadata. It
does not accept a caller-supplied numeric job ID. Fast jobs never generate a
replacement summary.

The Unit B policy is `wo25.unit_b.v1`. In addition to the V1 identity fields,
its ordered canonical object binds `parent`, `cargo_lock_sha256`,
`dependency_closure_sha256`, `producer_executable_sha256`, `workflow`, `event`,
`run_id`, `run_attempt`, `job_id`, `checkout_sha`, classifier mode/reason,
anchor/transitions, selector and mutation ledger hashes/counts, suite count,
readiness, hygiene inventory, claims, release version, duration, and the exact
terminal stage closure. Numeric identities are positive canonical decimal
integers without signs or leading zeroes.

The closed full stage set is, in order: `classifier`, `workspace`, `selectors`,
`mutations`, `backend`, `readiness`, `hygiene`, `claims`, and `release`.
Overall success is invalid if one is absent, duplicated, reordered, unknown,
skipped, red, partial, stale, or bound to another invocation. S01-S06 retain
those failures as distinct dispositions.

## Artifact contract

Each platform uploads exactly one regular file named
`hum-evidence-summary.v1.json` under:

```text
hum-evidence-summary-v1-<run_id>-<run_attempt>-<job_id>-<platform>
```

Each platform separately uploads exactly one `hum-dev` (`ubuntu`) or
`hum-dev.exe` (`windows`) under:

```text
hum-dev-executable-transport-v1-<run_id>-<run_attempt>-<job_id>-<platform>-<sha256>
```

`platform` is exactly `ubuntu` or `windows`; the final field is lowercase
SHA-256 over the unmodified executable. Both artifacts retain for exactly 14
days. The executable artifact is transport only, never evidence, cache,
release, installer, or package authority.

Uploads and downloads use only the immutable action commits frozen by WO25.
The workflow bootstrap selects by exact artifact ID, authenticates repository,
workflow, event, anchor/tree, run, attempt, job, platform, artifact name, size,
GitHub digest, one-entry archive shape, traversal absence, ordinary-file type,
executable name, and embedded executable hash before process creation. Ubuntu
sets and rechecks execute permission only after bytes authenticate. Windows
requires no-reparse identity and post-exit process/handle quiescence.

## Normative V2 orchestration identity

New full producers emit `hum.evidence_summary.v2` under policy
`wo25.unit_c.v2`. V2 preserves every V1 field and inserts exactly these fields
after `producer_executable_sha256`:

1. `orchestration_runtime`, exactly `powershell-core`;
2. `orchestration_version`, the authenticated dotted numeric PowerShell 7
   version;
3. `orchestration_executable_sha256`, lowercase SHA-256 over the independently
   reauthenticated executable.

The three fields participate in canonical bytes, the summary binding, every
stage binding, validation, parsing, corruption evidence, and status
authentication. `configuration_sha256` remains platform-neutral, and
`toolchain` remains exclusively the Rust/Cargo identity. Ubuntu and Windows
authenticate their own runtime versions and executable digests; cross-platform
equality of those two platform facts is not required. Portable summary bytes
never store the machine-local executable path.

Every PowerShell-backed production command requires typed
`--pwsh <absolute-path>`. The producer canonicalizes and authenticates the
ordinary non-reparse executable, hard-link policy, stable identity, bytes, and
PowerShell 7 version, then launches that exact absolute path. Bare names,
relative paths, `HUM_DEV_PWSH`, ambient PATH/profile/cache discovery, and later
translation back to `pwsh` are not authority.

V1 inputs remain byte-for-byte parseable as historical evidence, but a V1 or
mixed V1/V2 pair earns no Unit C status credit. Future load-bearing fields
require a new explicit schema version and migration fixtures; V1 and V2 are
never reinterpreted as one another.

The canonical V2 payload is exactly:

```text
hum-evidence-summary.v2.json
```

Its artifact name is exactly:

```text
hum-evidence-summary-v2-<run_id>-<run_attempt>-<numeric_job_id>-<platform>
```

Each numeric field is a positive canonical decimal integer with no sign or
leading zero, and `platform` is exactly `ubuntu` or `windows`. The artifact
retains for exactly 14 days and contains exactly the one regular canonical V2
payload: no extra entry, directory, link, reparse point, traversal, or
alternate filename is accepted. The V1/V2 payload and artifact grammars cannot
substitute for each other. The separate `hum-dev` executable transport retains
its existing V1 grammar unchanged.

## Typed status consumer

`hum-dev evidence status` owns run/job/artifact metadata queries, exact summary
downloads, canonical parsing, field authentication, platform agreement,
running-self cross-authentication, immutable Work Order projection, terminal
reporting, and summary cleanup. It records the exact GitHub CLI executable,
version, and SHA-256. PowerShell owns no summary meaning.

Ubuntu and Windows must agree on every platform-neutral field, including source
commit, parent, tree, Cargo/dependency closure, workflow/event, run/attempt,
classifier binding, ledgers, stage set, readiness, hygiene, claims, release,
and terminal disposition. Each platform separately binds its job, target,
toolchain, producer executable, streams, timing, and artifact transport. The
running executable must match its own platform summary; it is not required to
match the other platform binary.

Status consumption never downloads full logs and never invokes Cargo, rustc,
the Hum compiler, Fast, Exhaustive, cache restore, local-copy reuse, build,
workflow dispatch, rerun, or PowerShell semantic fallback. Missing, expired,
duplicate, ambiguous, corrupt, mixed, red, incomplete, or cleanup-failed
evidence rejects with its individual predicate. Consecutive recognized status
commits may consume one still-valid exact full anchor.

The bootstrap owns and closes only its executable archive/extraction resource.
`hum-dev` separately owns and closes only its summary resources. Neither may
delete a sibling, foreign, historical, repository, stash, archive, or remote
artifact. GitHub owns remote expiry.

## Work Order projection

`hum-dev workorder status-facts` accepts an exact target, base SHA-256, status
body, current-gate body, and output. It reconstructs and hashes the immutable
projection before and after, writing only when those bytes agree. The command
proposes deterministic facts; human review still owns public claims and
authorization meaning.

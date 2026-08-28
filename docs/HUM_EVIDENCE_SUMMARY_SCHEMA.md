# Hum Evidence Summary Schema

Status: canonical V1 foundation for repository developer evidence.

## Authority

`hum-dev` owns the `hum.evidence_summary.v1` type, validation, canonical
serialization, and cleanup disposition. Hum compiler producers continue to own
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

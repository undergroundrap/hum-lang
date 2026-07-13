# Hum Security Model

Date: 2026-07-06

Exact diagnostic allocations come only from
[`src/diagnostic_catalog.rs`](../src/diagnostic_catalog.rs); the checked human
projection is [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Purpose

Hum should be designed as a fast secure systems language, not as a fast language
with security bolted on later.

This document describes the cybersecurity lens for Hum: what attackers try, what
safe Hum should prevent by construction, what the compiler should make visible,
and what the toolchain must prove before Hum can claim serious security value.

## Brutal Thesis

Memory safety is necessary. It is not enough.

A secure systems language must also make these bug classes harder to write:

- missing authorization
- injection
- path traversal
- unsafe deserialization
- supply-chain compromise
- secret leakage
- resource exhaustion
- confused deputy bugs
- exceptional-condition mishandling
- unsafe/foreign boundary mistakes
- time, randomness, and crypto misuse

Fast insecure code is not impressive. Secure code that is too slow to deploy will
also lose. Hum must make the secure path the fast path.

## Research Signal

Current security guidance points toward language and toolchain responsibility:

- MITRE's 2025 CWE Top 25 still includes injection, missing authorization,
  out-of-bounds writes, path traversal, use-after-free, out-of-bounds reads,
  command injection, null dereference, buffer overflow, deserialization, exposed
  sensitive information, and resource allocation without limits.
- OWASP Top 10:2025 includes broken access control, software supply chain
  failures, cryptographic failures, injection, insecure design, integrity
  failures, logging/alerting failures, and exceptional-condition mishandling.
- NIST SSDF says secure development practices should reduce released
  vulnerabilities, reduce exploitation impact, and address root causes.
- NIST cyber-resiliency guidance treats security as systems engineering: systems
  should anticipate, withstand, recover from, and adapt to attacks or compromise.
- SLSA and OpenSSF Scorecard show that provenance, build integrity, dependency
  posture, signed releases, branch protection, tests, fuzzing, and static analysis
  belong in machine-checkable workflows.

Hum should turn those lessons into compiler and package-manager facts.

## Security Prime Directive

```text
Security-sensitive intent belongs in checked source, graph output, package
metadata, tests, fuzzers, proofs, benchmarks, and release artifacts.
```

Do not leave it in comments, wikis, or tribal memory.

## Threat Model

Hum should assume attackers can:

- control input bytes
- control paths, URLs, headers, IDs, names, and serialized data
- trigger rare error paths
- exhaust CPU, memory, file handles, sockets, and queues
- exploit timing differences
- induce integer overflow, truncation, and parsing confusion
- race concurrent operations
- tamper with dependencies or build scripts
- exploit unsafe or foreign code
- trick agents into generating plausible but insecure code
- inspect logs, traces, crash dumps, and telemetry
- replay, reorder, or delay network messages
- target update channels and package registries

Hum should not assume attackers are polite users with strange input.

## Security Goals

Safe Hum should make these properties natural:

1. Memory safety by default.
2. Capability-visible IO, time, randomness, networking, filesystem, and process
   access.
3. Declared authorization boundaries.
4. Typed untrusted input.
5. Safe parser and protocol patterns.
6. Injection-resistant command, SQL, HTML, URL, and shell APIs.
7. Secret values that do not format, log, compare, or clone casually.
8. Resource budgets for CPU, memory, allocations, locks, queues, and IO.
9. Typed failure and exceptional-condition handling.
10. Supply-chain evidence for packages, builds, tools, and generated artifacts.
11. Small, reviewed unsafe and foreign boundaries.
12. Security diagnostics that produce repairable facts for humans, IDEs, CI, and
    agents.

## Language Mechanisms

### Capabilities

`uses:` and `changes:` are security boundaries.

```text
uses:
  files.read

changes:
  session store
```

A task should not open files, access network, read clocks, generate randomness,
spawn work, or mutate stores unless the boundary says so.

### Protection Claims

`protects:` names what must not break.

```text
protects:
  user cannot read another user's invoice
  token cannot be guessed
  password never appears in logs
```

The compiler may not prove all of these early, but it must preserve them in the
semantic graph and connect them to tests, fuzzers, and review diagnostics.

### Trust Boundaries

`trusts:` names what the task relies on outside itself.

```text
trusts:
  operating system path sandbox is enforced
  hardware random source passes startup health checks
  TLS library validates certificate chains
```

A task with `trusts:` but no `protects:` is suspicious because it names an
assumption without naming the defended property.

### Untrusted Data

Hum should eventually distinguish data by trust state:

```text
Untrusted Text
Validated Email
Trusted UserId
Secret Token
```

Untrusted data should not flow into shell commands, SQL, paths, HTML, dynamic
code, deserializers, or authorization decisions without a validator or typed
builder.

### Authorization

Authorization should be a source-level obligation, not scattered `if` folklore.

Candidate shape:

```text
requires permission:
  user may read invoice invoice.id

protects:
  invoice cannot be read by another account
```

Broken access control and missing authorization remain too common for Hum to
leave authz as an ordinary boolean pattern.

### Injection Resistance

Hum should prefer typed builders over stringly APIs:

```text
sql query { select User where id == user.id }
command git { arg "status"; arg "--short" }
html text user.display_name
path under uploads join safe_file_name
```

Raw string execution should be a security-sensitive boundary requiring
`protects:`, `trusts:`, tests, and likely an unsafe or foreign review packet.

### Secrets

Secret values need special behavior:

- no accidental debug formatting
- no ordinary string interpolation
- no equality operator unless timing policy is named
- zeroization when profile or type requires it
- explicit reveal boundary
- no logging, telemetry, panic message, or crash dump exposure by default

Candidate shape:

```text
Secret Token
reveal token only for network.write under tls session
```

### Resource Exhaustion

Resource limits are security controls.

Hum should make budgets visible:

```text
cost:
  max request bytes: 1 MiB
  max allocations: 8
  max parse time: O(input bytes)
  check: compile
```

A parser that is memory-safe but accepts unbounded nesting is still a security
problem.

### Exceptional Conditions

Failure paths are attack surface.

Hum should force security-sensitive tasks to state:

- what happens when allocation fails
- what happens when input is malformed
- what happens when crypto fails
- what happens when authorization data is missing
- what happens when a dependency returns nonsense
- what happens when a lock, timeout, or queue fails

`fails when:` should become security evidence, not just an error enum note.

## Security Profiles

Candidate profiles:

```text
profile security hardened
profile network exposed
profile parser hardened
profile crypto sensitive
profile supply chain locked
profile sandboxed plugin
```

A profile can forbid:

- raw string command execution
- unchecked deserialization
- hidden allocation in parser hot paths
- secrets in formatting/logging
- ambient filesystem or network access
- dynamic code loading
- dependency build scripts
- unsafe without review packet
- foreign without ABI and ownership contract

## Unsafe And Foreign Boundaries

Unsafe and foreign code are exploit amplifiers.

Every unsafe or foreign boundary must answer:

```text
What memory invariant is the compiler no longer proving?
What external authority is trusted?
What attacker input can reach this boundary?
What profile allows this boundary?
What tests, fuzzers, proofs, or sanitizer runs cover it?
What happens on failure?
```

See [UNSAFE_POLICY.md](UNSAFE_POLICY.md).

## Supply Chain

Nectar should treat package security as first-class.

Future package metadata should include:

- dependency lock
- source provenance
- build provenance
- generated artifact hashes
- unsafe summary
- foreign interface summary
- build script capabilities
- vulnerability status
- maintainer/review status
- license
- SBOM
- test/fuzz/static-analysis evidence
- release signature status
- SLSA/OpenSSF-style posture where available

A dependency is not just code. It is an authority entering the program.

## Agent Security

Agents are useful and untrusted.

Rules:

- Agent-generated code must not bypass compiler checks.
- Agents must repair from diagnostics and graph facts, not vibes.
- Agents should be shown `protects:`, `trusts:`, `uses:`, `changes:`, and profile
  obligations before editing security-sensitive code.
- Agent changes near unsafe, FFI, crypto, authz, parsers, or supply chain require
  stricter review packets.
- The compiler should never accept a natural-language security claim without a
  checked artifact, test, fuzzer, proof, or explicit trust boundary.

## Fast Security

Hum should not make security the slow path.

Design targets:

- bounds checks optimized away when proven
- typed builders with zero or low allocation
- parser combinators that compile to tight loops
- secret wrappers with predictable layout
- capability checks resolved statically where possible
- profile-guided hardening for exposed builds
- fuzzing and sanitizer hooks in CI/release profiles
- benchmark obligations for security-sensitive primitives

If a safe API is too slow, users will reach for unsafe APIs. That is a language
design failure.

## Diagnostics

The active `security_trust` family is `H0400-H0499`; its currently allocated
codes retain the meanings in [DIAGNOSTICS.md](DIAGNOSTICS.md). Package and
supply-chain diagnostics have no assigned family. Existing and reserved family
ownership stays exact:

- `H0800-H0899`: `ownership_borrowing`
- `H0900-H0999`: `nominal_typed_failure`
- `H1000-H1099`: reserved `unsafe_ffi_provenance`
- `H1100-H1199`: reserved `runtime_profile_policy`

Future security diagnostics should still cover typed command construction,
secret logging, authorization obligations, package build authority, and unsafe
provenance. Those ideas are family-level design requirements, not exact code
allocations.

## Semantic Graph Requirements

`hum graph` should eventually expose:

- security-sensitive tasks
- `protects:` claims
- `trusts:` assumptions
- untrusted-to-trusted validation edges
- secret flow edges
- authz obligations
- parser and deserializer boundaries
- unsafe and foreign boundaries
- package trust and provenance
- generated security tests/fuzz targets
- profile gates
- unresolved risks

## Sources

- MITRE 2025 CWE Top 25: https://cwe.mitre.org/top25/archive/2025/2025_cwe_top25.html
- OWASP Top 10:2025: https://owasp.org/Top10/2025/
- NIST SP 800-218, Secure Software Development Framework: https://csrc.nist.gov/pubs/sp/800/218/final
- NIST SP 800-160 Vol. 2 Rev. 1, Cyber-Resilient Systems: https://csrc.nist.gov/pubs/sp/800/160/v2/r1/final
- SLSA v1.2 specification: https://slsa.dev/spec/v1.2/
- OpenSSF Scorecard: https://scorecard.dev/
- CHERI project: https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/
- MSWasm, revised 2026: https://arxiv.org/abs/2208.13583
- Verifying the Rust Standard Library, 2026: https://arxiv.org/abs/2606.17374
- Towards verifying unsafe Rust programs against Rust's pointer-aliasing restrictions, 2026: https://arxiv.org/abs/2603.28326

## Brutal Assessment

Hum's security pitch cannot be "we are memory safe and readable."

It must be:

```text
Hum makes authority, mutation, trust, unsafe, input validation, failure,
resource use, secrets, and supply-chain evidence visible enough for compilers,
tools, reviewers, and agents to enforce.
```

That is the security bar.

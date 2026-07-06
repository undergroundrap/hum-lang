# Computing Lessons Sweep 2026

Date: 2026-07-06

## Purpose

This sweep asks whether Hum is still missing old and new lessons from computing:
hardware, operating systems, containers, cloud deployment, networking, storage,
security, observability, supply chain, and agentic tooling.

Hum should not be designed only as a nicer Rust syntax. It should be designed as
a language that understands the machine, the deployment environment, the package
supply chain, the operator, and the agent at the same time.

## Brutal Conclusion

The missing lesson is deployment reality.

A great systems language is not only:

```text
source -> compiler -> binary
```

It is:

```text
source -> graph -> checks -> package -> image -> runtime profile -> telemetry -> incident -> repair
```

If Hum cannot explain what a program reads, writes, allocates, exposes, listens
on, depends on, deploys as, observes, and lets agents touch, then Hum is not yet
the 2050 language.

## The Old Lessons Still Win

### 1. The Machine Is Layers Of Contracts

Every layer hides authority:

- CPU instructions hide memory ordering and speculation.
- Operating systems hide files, sockets, signals, and process authority.
- Runtimes hide allocation, scheduling, and unwinding.
- Package managers hide code execution and trust.
- Containers hide kernel sharing and host mounts.
- Agents hide tool calls behind natural language.

Hum rule:

```text
hidden authority is a bug class
```

The language must make authority visible in source, graph output, package
metadata, deployment manifests, and diagnostics.

### 2. Memory Hierarchy Beats Pretty Big-O

Big-O is necessary, but real performance is usually decided by:

- allocation count
- cache locality
- branch predictability
- SIMD/vector width
- false sharing
- lock contention
- NUMA placement
- page faults
- TLB pressure
- syscall count
- serialization and copy count
- p95 and p99 latency, not just average speed

Hum rule:

```text
cost: must eventually describe asymptotic shape, allocation shape, locality, and tail latency.
```

A function claiming `O(1)` while causing cache misses, heap churn, or p99 spikes
is not telling the whole truth.

### 3. IO Is Part Of Semantics

Networking, storage, clocks, files, entropy, subprocesses, GPUs, and databases
are not side quests. They are where many production bugs live.

Hum should model:

- timeout
- cancellation
- retry
- backoff
- idempotence
- transaction
- durability
- crash consistency
- flush/fsync expectations
- resource budgets
- observability
- rollback

If those are only library comments, Hum misses the system.

### 4. Deployment Is A Language Surface

Programs today run as:

- local CLIs
- services
- containers
- Kubernetes workloads
- Wasm components
- serverless functions
- embedded firmware
- agent-invoked tools
- CI jobs
- data pipelines

Hum should not hard-code one runtime story. It should make deployment a profile
and evidence problem.

## Docker, OCI, And Kubernetes

Yes: Docker and OCI compatibility matter.

Companies care because containers give a common way to build, ship, isolate,
scan, run, and schedule software. The Open Container Initiative standardizes
container image, runtime, and distribution behavior. Docker and Kubernetes are
how many teams move software through real organizations.

But containers are not a perfect sandbox. Docker documents that containers rely
on kernel namespaces, cgroups, daemon security, capabilities, and hardening
features. It also warns that controlling the Docker daemon is powerful because
host directories can be mounted into containers. Kubernetes has separate Pod
Security Standards because privileged pods, host namespaces, host paths, and
extra capabilities are serious risk.

Hum rule:

```text
container compatible, not container naive
```

Hum should eventually produce or validate:

- OCI image metadata
- Dockerfile or BuildKit plan
- SBOM
- SLSA-style provenance
- runtime profile
- required Linux capabilities
- dropped Linux capabilities
- seccomp/AppArmor/SELinux expectations where applicable
- Kubernetes securityContext expectations
- ports and protocols
- health/readiness checks
- signal handling behavior
- filesystem mount policy
- writable path policy
- environment variable and secret policy
- CPU, memory, file descriptor, and process budgets
- OpenTelemetry schema

Hum should default container services toward:

- non-root user
- read-only root filesystem when practical
- declared writable mounts only
- no host network by default
- no host PID/IPC by default
- no hostPath by default
- drop capabilities unless required
- no privileged container by default
- no Docker socket access by default
- no secret through ordinary environment unless declared
- no unbounded logs
- graceful shutdown on signals

## Wasm And WASI

Wasm/WASI should be Hum's first serious untrusted-code and plugin boundary.

WASI is designed as a secure standard interface for Wasm applications across
browsers, clouds, and embedded devices. WASI applications run with capability
based sandboxing: no ambient authority unless the host grants it.

Hum lesson:

```text
plugins should look like WASI before they look like native dynamic libraries
```

That does not mean Hum is only a Wasm language. It means Hum should prefer
capability-scoped component boundaries before native plugin trust.

## Agentic CLI And MCP Reality

The agent era changes the threat model.

MCP standardizes how hosts, clients, and servers expose resources, prompts, and
tools. The official MCP spec says tools are functions the AI model can execute,
and its security section treats tools as arbitrary code execution requiring user
consent and caution. The MCP security guide calls out real attack classes such as
confused deputy flows, token passthrough, SSRF, session hijacking, local server
compromise, and stdio proxy risk.

Hum rule:

```text
agent tools are untrusted programs with schemas, capabilities, and audit trails
```

Hum should eventually support an `agent tool sandbox` profile for tools exposed
to agents, IDEs, MCP servers, CI bots, or local automation.

That profile should require:

- exact input schema
- exact output schema
- declared read/write capabilities
- no raw shell strings
- no hidden network
- no hidden filesystem write
- dry-run for mutation
- audit log events
- prompt-injection risk notes when tool output returns to an agent
- secret redaction
- consent text generated from actual capabilities
- sandbox recommendation: process, container, or Wasm depending on risk

The compiler must never trust a tool description, agent prompt, or generated
plan as evidence. It should trust checked source, graph facts, test results,
proofs, fuzzing, and signed/reproducible artifacts.

## Observability Is Source-Level Truth

OpenTelemetry exists because modern systems need traces, metrics, logs, baggage,
and profiles that can flow across languages, services, and runtimes.

Hum should not bolt observability on after incidents.

Hum should eventually support:

```text
emits:
  metric session.created count
  trace span session.create
  log security.session_rejected redacts secrets

observes:
  p99 latency
  allocation count
  retry count
```

Telemetry must also be security-aware:

- secrets do not format into logs
- high-cardinality values are controlled
- telemetry cost is budgeted
- security events are named
- traces preserve source graph node identity
- sampling policy is visible

## Networking Lessons

A good network API is not just `connect(host, port)`.

Hum `std.net` should make these facts visible:

- IP, CIDR, port, DNS name, URL, origin, and service identity are different
  types
- DNS can change
- TLS identity is a contract
- timeouts are mandatory for exposed network tasks
- retries require idempotence or compensation
- backpressure is part of API design
- protocol parsers are untrusted-input parsers
- request body size is a security budget
- cancellation must be structured
- connection pooling changes latency and failure modes
- network partitions are ordinary, not weird

Hum should prefer typed protocol builders and Sans I/O parser patterns where
possible: parse bytes and state transitions separately from sockets.

## Storage Lessons

A good storage API is not just `read` and `write`.

Hum `std.storage` should make these facts visible:

- data format version
- endian and layout
- atomicity expectations
- durability expectation
- crash consistency
- checksum or integrity policy
- migration path
- compression/encryption boundary
- mmap risk
- cache policy
- fsync/flush behavior
- partial write behavior
- concurrency and locking
- backup/restore assumptions

A storage system that is memory safe but corrupts data on crash is not safe.

## Performance And Latency Lessons

Hum should optimize for measured truth.

Performance features should distinguish:

- throughput
- p50 latency
- p95 latency
- p99 latency
- cold start
- steady state
- memory footprint
- allocation count
- code size
- compile time
- energy where relevant
- worst-case time for realtime profiles

Hum should make these claims profile-specific. A server, game loop, firmware
control loop, data pipeline, and CLI do not have the same definition of fast.

## Supply Chain Lessons

Package managers became part of the trusted computing base.

Nectar should learn from SLSA, OpenSSF Scorecard, SBOM practice, Docker
attestations, and secure development guidance:

- package install should not mean code execution
- build scripts require explicit capabilities
- generated files need hashes and origin
- dependencies need lockfiles
- releases need provenance
- images need SBOMs
- CI tokens need least privilege
- dangerous workflows should be rejected or flagged
- binary artifacts need trust evidence
- vulnerability and maintainer status are package facts

Hum should treat every dependency as an authority entering the program.

## New Profiles Hum Should Add

### `containerized service`

For Docker, OCI, Kubernetes, and service workloads.

Requires:

- declared ports
- declared health/readiness behavior
- graceful shutdown behavior
- resource budgets
- filesystem mount policy
- user identity policy
- capability/seccomp policy where applicable
- telemetry schema
- SBOM/provenance in release profile

Forbids by default:

- privileged containers
- host filesystem mounts
- host network/PID/IPC
- Docker socket access
- undeclared secret reads
- hidden listening sockets
- unbounded logs

### `agent tool sandbox`

For MCP servers, agent-callable CLIs, IDE tools, CI repair tools, and codegen
helpers.

Requires:

- declared input/output schemas
- declared capabilities
- dry-run for mutation
- audit log event schema
- secret redaction
- prompt-injection risk notes
- sandbox boundary recommendation

Forbids by default:

- token passthrough
- raw shell strings
- hidden network
- hidden repo mutation
- accepting tool descriptions as trusted facts

## Design Locks From This Sweep

1. Docker/OCI support is a product requirement, but not a Milestone 0 feature.
2. Hum should add a `containerized service` runtime profile before serious
   server demos.
3. Hum should add an `agent tool sandbox` profile before exposing Hum tools over
   MCP or agent CLIs.
4. Nectar should eventually emit SBOM, provenance, container metadata, runtime
   profile metadata, and OpenTelemetry schema evidence.
5. `hum graph` must reserve deployment, container, observability, and agent-tool
   facts.
6. Hum should prefer Wasm/WASI for untrusted plugin boundaries before native
   plugin loading.
7. Containers, agents, package scripts, and generated code remain banned from
   Milestone 0 execution.
8. Performance claims must include tail latency, allocation, locality, and
   deployment profile when relevant.
9. Network and storage APIs must be typed around failure, authority, durability,
   and resource budgets.
10. A feature that cannot survive Docker, Kubernetes, observability, supply-chain
    evidence, and agent tooling probably is not production-grade Hum.

## Near-Term Consequence

Before building Nectar networking, Docker image generation, MCP servers, native
plugins, or remote package behavior, write:

1. `docs/DEPLOYMENT_AND_CONTAINER_MODEL.md`
2. `docs/OPERATIONS_MODEL.md`
3. `docs/NETWORK_MODEL.md`
4. `docs/STORAGE_MODEL.md`
5. `docs/AGENT_TOOL_SECURITY_MODEL.md`
6. `docs/PACKAGE_AND_BUILD.md`

Milestone 0 remains local parse/check/graph. This sweep changes the map, not the
safety boundary.

## Sources

- Open Container Initiative overview: https://opencontainers.org/
- OCI runtime specification: https://github.com/opencontainers/runtime-spec
- Docker Engine security: https://docs.docker.com/engine/security/
- Docker Build attestations: https://docs.docker.com/build/metadata/attestations/
- Docker provenance attestations: https://docs.docker.com/build/metadata/attestations/slsa-provenance/
- Kubernetes Pod Security Standards: https://kubernetes.io/docs/concepts/security/pod-security-standards/
- Kubernetes security context: https://kubernetes.io/docs/tasks/configure-pod-container/security-context/
- Kubernetes seccomp tutorial: https://kubernetes.io/docs/tutorials/security/seccomp/
- WASI introduction: https://wasi.dev/
- WebAssembly Component Model: https://component-model.bytecodealliance.org/
- MCP specification 2025-06-18: https://modelcontextprotocol.io/specification/2025-06-18
- MCP security best practices: https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices
- NIST SP 800-218 SSDF: https://csrc.nist.gov/pubs/sp/800/218/final
- SLSA v1.2 specification: https://slsa.dev/spec/v1.2/
- OpenSSF Scorecard: https://scorecard.dev/
- OpenTelemetry overview: https://opentelemetry.io/docs/what-is-opentelemetry/
- OpenTelemetry signals: https://opentelemetry.io/docs/concepts/signals/
- OpenTelemetry semantic conventions: https://opentelemetry.io/docs/specs/semconv/
- Breaking the Protocol: Security Analysis of MCP, 2026: https://arxiv.org/abs/2601.17549
- MCP-DPT defense placement taxonomy, 2026: https://arxiv.org/abs/2604.07551
- Quantifying Frontier LLM Capabilities for Container Sandbox Escape, 2026: https://arxiv.org/abs/2603.02277

## Brutal Assessment

You were right to pause.

Hum had strong language-design instincts, but the project needed a sharper
runtime/deployment lens. The next world-class move is not adding more syntax.
It is making deployment, containers, observability, network/storage failure,
supply-chain evidence, and agent tool authority part of the same checked story
as memory safety and `uses:`/`changes:`.
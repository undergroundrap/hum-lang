# Hum Practitioner Pain Sweep 2026

Date: 2026-07-06

## Purpose

Hum should not be designed only for cybersecurity researchers or application
developers.

If Hum wants to win serious systems people, it must help the people who live in:

- terminals
- production incidents
- deployment pipelines
- routers and switches
- kernels and firmware
- GPUs and accelerators
- data pipelines
- numerical correctness
- reproducible science

This document records what those practitioners complain about, what existing
tools already prove, and what Hum should make first-class before Nectar and the
standard library harden.

## Brutal Conclusion

Winning low-level, ops, network, AI/ML, and math users does not mean adding every
domain to core syntax.

It means Hum must make real-world facts typed, checked, diffable, observable,
and agent-readable:

```text
Path
Host
IpAddr
Cidr
Port
Service
Process
User
Group
Secret
Credential
Duration
Deadline
RetryPolicy
DesiredState
Plan
Transaction
Rollback
Trace
Metric
Tensor
Shape
DType
Device
Unit
ErrorBound
Seed
```

These should mostly be standard-library and toolchain primitives, not new syntax
yet. The language core should provide the contracts that make them enforceable:
effects, capabilities, typed failure, resource budgets, profiles, source spans,
semantic graph facts, and generated tests.

## Practitioner Pain Ledger

### Sysadmins And Shell Automation

Common pain:

- strings are used for paths, command lines, users, process IDs, ports, and
  secrets
- shell quoting and word splitting are easy to get wrong
- errors can be ignored accidentally
- pipelines lose structure
- scripts mutate machines with weak dry-run support
- idempotence is a convention, not a checked property
- logs and diffs often leak secrets
- cross-platform behavior is hard to reason about

Research signal:

- ShellCheck's `SC2086` exists because an unquoted shell variable can silently
  become word splitting plus glob expansion.
- PowerShell's object pipeline shows that sysadmins benefit when commands pass
  typed values instead of only text.
- Ansible popularized desired state, check mode, diff mode, and idempotent
  modules, but the docs still warn that not every module or playbook is
  idempotent.

Hum lesson:

```text
Command is not Text.
Path is not Text.
Secret is not Text.
UserName is not Text.
ProcessId is not Int.
```

Hum should make typed command construction the normal path:

```text
task restart service(service: ServiceName, host: Host) -> Result<()> {
  why:
    apply a requested service restart without shell injection

  uses:
    host.ssh
    service.manager

  changes:
    service state on host

  needs:
    operator may administer host
    service exists on host

  ensures:
    service is running or failure names why not

  protects:
    service name cannot become shell syntax
    credentials never appear in diff or logs

  dry run:
    report exact service action without changing host

  rollback:
    if restart fails after stop, attempt start once

  does:
    connect to host
    ask service manager to restart service
    verify service reached running state
}
```

Design rule:

Hum should have `dry run:` and `rollback:` as serious contract candidates for
ops-facing tasks, but they should start as parsed task sections and graph facts
before becoming executable semantics.

### DevOps And SRE

Common pain:

- toil grows linearly with service count
- config drift causes outages
- YAML is easy to generate and hard to understand at scale
- deployment plans are not always tied to source-level intent
- retries, timeouts, backoff, and circuit breakers are scattered
- observability is added after the incident
- build and runtime environments drift apart
- dependency and generated-artifact trust is hard to audit

Research signal:

- Google's SRE book defines toil as manual, repetitive, automatable, tactical,
  low-enduring-value work that scales linearly with service growth.
- Kubernetes objects are records of intent: users declare desired state in
  `spec`, while the control plane maintains and reports actual `status`.
- Terraform state exists because infrastructure tools need a mapping between
  configuration and real-world resources, plus dependency metadata and cached
  attributes for large environments.
- CUE and Dhall both exist because configuration wants types, validation,
  semantic diffs, reusable constraints, and safe evaluation.

Hum lesson:

Hum should treat production automation as a first-class programming workload.

Core/std/tooling primitives:

- `DesiredState<T>`
- `ObservedState<T>`
- `Drift<T>`
- `Plan<T>`
- `Apply<T>`
- `RollbackPlan`
- `ChangeSet`
- `Diff<T>`
- `Service`
- `Endpoint`
- `HealthCheck`
- `Slo`
- `Trace`
- `Metric`
- `LogField`
- `Span`
- `RetryPolicy`
- `Backoff`
- `Timeout`
- `Deadline`
- `RateLimit`
- `ResourceBudget`

Candidate sections:

```text
observes:
  service health
  deployment latency

emits:
  metric deploy.duration
  trace span deploy.apply

timeouts:
  health check within 30 seconds

retries:
  transient network failure at most 3 times with jittered backoff

idempotent:
  applying the same desired state twice makes no further changes
```

Design rule:

Do not put Kubernetes, Terraform, or Ansible into the core language. Instead,
make the primitives that those tools reveal: desired state, observed state,
diffs, plans, idempotence, rollback, transactions, capabilities, secrets, and
evidence.

### Network Admins And Protocol Engineers

Common pain:

- IP addresses, CIDRs, VLANs, ASN values, ports, MAC addresses, and routes are
  often handled as strings
- vendor configs are parsed with brittle regexes
- network changes need transactions, rollback, and audit trails
- telemetry streams are semantically richer than logs
- protocol parsers need endian, alignment, bounds, and version discipline
- retries can be dangerous when an operation is not idempotent
- partial failure is normal

Research signal:

- YANG models configuration data, state data, RPCs, and notifications for network
  management protocols.
- gNMI exposes capabilities, `Get`, `Set`, and `Subscribe`; its `Set` operations
  are transactional and must roll back on failure across affected trees.
- gNMI also ties telemetry shape to subscriptions and requires TLS for sessions.

Hum lesson:

Networking should not be a stringly afterthought.

Core/std candidates:

- `IpAddr`
- `Ipv4Addr`
- `Ipv6Addr`
- `Cidr`
- `MacAddr`
- `Port`
- `Protocol`
- `SocketAddr`
- `VlanId`
- `Asn`
- `Route`
- `Prefix`
- `Packet`
- `Frame`
- `Endian`
- `Checksum`
- `TelemetryPath`
- `Subscription`
- `NetworkTransaction`

Network tasks should naturally say:

```text
uses:
  router.capabilities
  router.telemetry

changes:
  router.config.interfaces

needs:
  target supports model openconfig-interfaces
  transaction can roll back all touched paths

ensures:
  interface admin state matches requested state
  old state is recoverable from rollback plan

protects:
  management session is mutually authenticated
  partial config update cannot be committed
```

Design rule:

Hum should make typed protocol and network data part of `std.net` early. But
vendor-specific device modeling belongs in packages until proven broadly useful.

### Low-Level, Kernel, Firmware, And Embedded Engineers

Common pain:

- hidden allocation breaks firmware, kernels, realtime code, and hot paths
- layout, alignment, and endian behavior must be explicit
- memory-mapped IO needs volatile semantics without making the whole language
  unsafe
- interrupts, DMA, cache coherency, and atomics require exact rules
- cross-compilation and target features must be visible
- debug info and codegen predictability matter
- C interop cannot be hand-wavy

Research signal:

- Rust embedded practice separates `core`, `alloc`, and `std`, with `no_std`
  builds for firmware, bootloaders, and kernels.
- Embedded Rust material calls out volatile access, `repr(C)`, packed/aligned
  layouts, raw pointers, and the lack of ambient OS/runtime support.

Hum lesson:

Hum must have profiles before it has a big runtime:

```text
profile hosted
profile no std
profile no heap
profile hard realtime
profile kernel
profile firmware
profile engine hot path
```

Core/std candidates:

- `Layout`
- `Align`
- `Endian`
- `Addr`
- `PhysAddr`
- `VirtAddr`
- `Register<T>`
- `Volatile<T>`
- `Mmio<T>`
- `DmaBuffer<T>`
- `Atomic<T>`
- `Interrupt`
- `TargetFeature`
- `StackBudget`
- `CodegenBudget`

Candidate sections:

```text
allocates:
  none

layout:
  repr c
  aligned 64

target:
  x86_64 avx2
  fallback scalar

latency:
  worst case under 50 microseconds
```

Design rule:

No hidden heap, hidden runtime, hidden thread, hidden syscall, hidden panic, or
hidden allocation in restricted profiles. If the compiler cannot explain the
cost and authority, the feature does not belong in low-level Hum.

### AI/ML And Data Scientists

Common pain:

- Python is easy to write but slow outside optimized libraries
- tensor shape, dtype, layout, and device bugs appear late
- CPU/GPU behavior can differ
- reproducibility is fragile across seeds, releases, platforms, and devices
- notebooks are exploratory but weak production artifacts
- copying between array libraries wastes memory and time
- accelerator code mixes performance, numerics, memory layout, and toolchain
  assumptions
- dynamic shapes and graph breaks are hard to diagnose

Research signal:

- PyTorch documentation warns that fully reproducible results are not guaranteed
  across releases, commits, platforms, or CPU/GPU execution, even with identical
  seeds.
- PyTorch numerical accuracy docs describe bitwise differences between batched
  and sliced computations, overflow with extremal values, ill-conditioned linear
  algebra, and accelerator precision modes like TF32.
- The Python Array API standard exists because the ecosystem needs common array
  semantics, dtype/device awareness, zero-copy interchange where possible, and a
  stable ABI.
- MLIR's dialect ecosystem includes tensor, sparse tensor, linalg, vector, GPU,
  shape, quantization, and accelerator-oriented dialects.

Hum lesson:

Hum should not try to beat Python notebooks first. Hum should be the language
that makes production data and ML code reproducible, typed, fast, and auditable.

Core/std/package candidates:

- `Array<T, Shape, Device>`
- `Tensor<T, Shape, Layout, Device>`
- `Shape`
- `Axis`
- `DType`
- `Device`
- `Layout`
- `Strides`
- `SparseTensor`
- `Matrix`
- `Vector`
- `Seed`
- `Rng`
- `DataSet`
- `Batch`
- `DataLineage`
- `Checkpoint`
- `DeterminismPolicy`
- `PrecisionPolicy`
- `Gradient`

Candidate sections:

```text
numeric:
  dtype float32
  device cuda
  determinism required for tests
  tolerance absolute 1e-6

data:
  source hash sha256:...
  schema TrainingRow
  split seed 42

accelerates:
  use gpu when available
  fallback cpu scalar
```

Design rule:

Tensor and numeric primitives should not enter the core language. They should be
designed as stdlib/package APIs with graph visibility and eventual MLIR lowering
hooks. The core must still understand enough about shape, device, allocation,
determinism, and numeric tolerance to produce useful diagnostics.

### Math, Scientific Computing, Quant, And Numerical Engineers

Common pain:

- floats look like real numbers but are not real numbers
- units are mixed accidentally
- overflow, underflow, cancellation, NaN, and infinities leak through APIs
- approximate equality is too often hidden in tests
- sparse and dense algorithms have different cost models
- ill-conditioned problems need warnings and diagnostics
- reproducibility conflicts with fastest available hardware paths
- exact, interval, decimal, rational, and floating-point arithmetic should not be
  confused

Hum lesson:

Numeric correctness must be visible at the type and contract level.

Core/std/package candidates:

- `Int<N>`
- `UInt<N>`
- `Float32`
- `Float64`
- `Decimal`
- `BigInt`
- `Rational`
- `Complex<T>`
- `Interval<T>`
- `ErrorBound<T>`
- `Unit<T, Dimension>`
- `Probability`
- `Tolerance`
- `ConditionNumber`
- `StableSum`
- `Exact<T>`
- `Approx<T>`

Candidate sections:

```text
numeric:
  units meters per second
  rejects NaN
  tolerance relative 1e-9
  warns if condition number exceeds 1e8

cost:
  time O(rows * columns)
  memory O(columns)
```

Design rule:

Hum should distinguish exact, approximate, measured, and bounded numeric values.
That is how the language earns trust from math people and from systems people
who ship physical-world code.

## Cross-Cutting Core Requirements

These are not optional if Hum wants these practitioners.

### Typed Reality

Primitive wrappers must be cheap and honest:

```text
Path != Text
Url != Text
IpAddr != Text
Command != Text
Secret != Text
Tensor<Float32, Shape[Batch, Features], Cpu> != raw bytes
```

The compiler should optimize wrappers away when safe, while keeping graph facts.

### Capability-Aware Effects

`uses:` and `changes:` must cover:

- filesystem
- environment variables
- process spawning
- network
- clocks
- randomness
- services
- host inventory
- deployment state
- telemetry
- GPU/accelerator device access
- data sources

### Plans Before Mutation

Ops, networking, and infrastructure code should naturally support:

- validate
- plan
- diff
- dry run
- apply
- verify
- roll back
- audit

### Idempotence As A Contract

Hum should eventually be able to reject or warn when a task claims idempotence
but performs obvious non-idempotent actions without a guard.

### Secrets And Redaction Everywhere

Secrets must be redacted by type across:

- diffs
- logs
- traces
- panic messages
- build output
- graph output
- agent context
- generated docs

### Observability From Source

`emits:` and `observes:` should feed:

- logs
- metrics
- traces
- profiler labels
- incident summaries
- semantic graph
- generated docs

### Determinism And Reproducibility

Hum should distinguish:

- deterministic by construction
- deterministic for a pinned toolchain/profile/device
- nondeterministic but bounded
- intentionally nondeterministic

This matters for tests, simulations, ML, games, networking, and safety-critical
systems.

### Numeric Honesty

Hum should never pretend that all numeric APIs are alike.

Every serious numeric primitive should say:

- exact or approximate
- overflow behavior
- NaN/infinity policy
- tolerance policy
- deterministic policy
- hardware precision assumptions
- sparse/dense behavior
- units or dimensions if relevant

## What This Changes For Hum

1. Nectar should wait until `PACKAGE_AND_BUILD.md` includes ops/data evidence:
   build plans, generated artifacts, SBOMs, deployment metadata, profile locks,
   benchmark packets, and reproducibility records.
2. `std` should reserve early namespaces for `std.path`, `std.process`,
   `std.net`, `std.time`, `std.secret`, `std.observe`, `std.config`,
   `std.plan`, `std.numeric`, and eventually `std.tensor`.
3. The semantic graph schema needs room for dry runs, rollback plans,
   idempotence, telemetry, desired/observed state, tensor shape, device, dtype,
   numeric tolerance, and reproducibility facts.
4. The compiler should parse new sections only after the docs define their
   meaning: `dry run:`, `rollback:`, `observes:`, `emits:`, `timeouts:`,
   `retries:`, `idempotent:`, `numeric:`, `data:`, and `target:`.
5. Hum must avoid becoming a giant domain-specific language. Most domain power
   belongs in typed libraries plus graph-visible contracts.

## Near-Term Recommendation

Before writing `PACKAGE_AND_BUILD.md`, add these design gates:

1. `docs/OPERATIONS_MODEL.md` for dry-run, rollback, idempotence, desired state,
   drift, telemetry, and deployment plans.
2. `docs/NETWORK_MODEL.md` for typed addresses, protocol parsing, transactions,
   telemetry subscriptions, and management-plane security.
3. `docs/NUMERIC_AND_TENSOR_MODEL.md` for units, exact/approx values, shape,
   dtype, device, tolerance, determinism, and accelerator lowering.
4. Update `docs/SEMANTIC_GRAPH_SCHEMA.md` to reserve graph fields for these
   facts before implementing more syntax.

## Brutal Assessment

Yes, this lens makes Hum stronger.

But the warning is serious: if Hum tries to replace Bash, Ansible, Terraform,
Kubernetes YAML, Python, NumPy, PyTorch, Julia, Rust, and C all at once, it will
become a fantasy language.

The winning move is narrower and more powerful:

```text
Hum should be the systems language where real-world authority, mutation,
configuration, networking, hardware, numeric behavior, and evidence are visible
enough for humans, compilers, tools, and agents to reason about.
```

That is how we win low-level people without becoming every tool they already
use.

## Sources

- ShellCheck SC2086: https://www.shellcheck.net/wiki/SC2086
- PowerShell language specification introduction: https://learn.microsoft.com/en-us/powershell/scripting/lang-spec/chapter-01?view=powershell-7.5
- Google SRE book, Eliminating Toil: https://sre.google/sre-book/eliminating-toil/
- Kubernetes Objects: https://kubernetes.io/docs/concepts/overview/working-with-objects/
- Terraform state purpose: https://developer.hashicorp.com/terraform/language/state/purpose
- Ansible playbooks and idempotency: https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_intro.html
- Ansible check mode and diff mode: https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_checkmode.html
- CUE introduction: https://cuelang.org/docs/introduction/
- Dhall configuration language: https://dhall-lang.org/
- YANG 1.1, RFC 7950: https://www.rfc-editor.org/rfc/rfc7950.html
- gNMI specification: https://www.openconfig.net/docs/gnmi/gnmi-specification/
- Embedded Rust `no_std`: https://docs.rust-embedded.org/book/intro/no-std.html
- Embedded Rust tips for C developers: https://docs.rust-embedded.org/book/c-tips/index.html
- PyTorch reproducibility notes: https://docs.pytorch.org/docs/2.12/notes/randomness.html
- PyTorch numerical accuracy notes: https://docs.pytorch.org/docs/2.12/notes/numerical_accuracy.html
- Python Array API standard: https://data-apis.org/array-api/latest/index.html
- Python Array API data interchange: https://data-apis.org/array-api/latest/design_topics/data_interchange.html
- MLIR dialect list: https://mlir.llvm.org/docs/Dialects/

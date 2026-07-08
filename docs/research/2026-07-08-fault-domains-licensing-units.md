# Fault Containment, Toolchain Licensing, And Units Research

Date: 2026-07-08
Status: distilled snapshot, triaged by external reviewer

## Provenance And Confidence

Source: agent-run deep research commissioned against WORKORDER backlog items
5 (fault containment) and 6 (units), plus the standing BDFL licensing
decision. Citation markers in the raw report were tool-mangled; substance
was cross-checked against reviewer knowledge. The report's own "thin
evidence"/"unknown" flags are preserved.

## 1. Fault Containment And Partial Failure

Findings accepted:

- BEAM's crash containment is structural, not stylistic: per-process heaps,
  copied messages, individually GC'd processes. Supervision restart budgets
  (`intensity`/`period`) exist to stop infinite crash loops. None of this
  transfers automatically to a shared-heap systems language.
- Midori's abandonment model (kill the whole process on a bug, refuse to
  run user code during teardown) worked because applications were already
  decomposed into many small isolated processes with typed message
  passing. Abandonment presupposes small isolation units.
- Safety-critical practice (ARINC 653 time/space partitioning, MPU zones,
  watchdogs) restarts partitions and devices, not stack frames. Watchdogs
  are the last line against hangs, not a recovery mechanism.
- Panic-policy convergence across Rust/Zig/Go: no general-purpose stack
  unwinding as a recovery story; typed errors for expected failure,
  abortive semantics for bugs. Rust's catch_unwind carries known FFI and
  invariant hazards.
- Intra-process isolation without green threads is possible (SFI, Wasm
  module sandboxing, MPU zones) but only with deliberate machinery:
  restricted sharing, revocable capabilities, domain-scoped ownership.
  Mutex-poisoning-writ-large is not enough.

Direction adopted for backlog item 5 (design work still required before
any concurrency syntax):

```text
fault domain: first-class unit with explicit creation, parent supervisor,
  restart budget, and capability set
expected failure: typed error values (existing fail path)
unexpected failure: kills the fault domain, revokes its capabilities,
  reports fault metadata to the supervisor
restart strategies from day one: restart-self, restart-subtree, escalate
no general unwinding in the safe subset; any catch-like fault API is
  supervisor-only and domain-boundary-only
cross-domain state: message passing, immutable snapshots, or capability-
  guarded resources with explicit recovery contracts
profiles: hard-realtime/embedded get watchdog-first abort/reset semantics
honesty rule: Hum never claims crash isolation for ordinary shared state
  without real isolation machinery
```

Test doctrine to adopt when this builds: chaos tests injecting faults into
tasks holding locks, descriptors, buffers, and FFI resources; verify no
leaked capabilities, no zombie resources, no cross-domain memory reuse, no
silent continuation.

## 2. Toolchain Licensing

Findings accepted:

- Every surveyed successful toolchain is permissive: Rust MIT/Apache-2.0
  dual, LLVM Apache-2.0 with LLVM exceptions, Swift Apache-2.0 with
  runtime exception, Go BSD-3, Zig MIT. GCC's GPLv3 needed the Runtime
  Library Exception precisely so proprietary programs could use its
  runtime. The exceptions exist to make one answer obviously safe: "using
  this toolchain does not taint my binary."
- Google publicly bans AGPL code outright. No successful precedent exists
  in the surveyed set for an AGPL general-purpose language toolchain.
- Attribution and anti-fork protection are what trademark policy is for;
  copyleft on the toolchain buys adoption pain, not protection.
- CLA preserves relicensing/commercial options at the cost of contributor
  friction; DCO is frictionless but makes future relicensing effectively
  impossible. The choice must be made before the first external
  contribution.
- Ferrocene's commercial model (qualification, support, certification
  artifacts sold on top of a permissive core; IEC 62304 Class C
  qualification announced) is the proof that regulated-industry revenue
  does not require copyleft.

Status: BDFL decision pending. Reviewer recommendation on record:
MIT/Apache-2.0 dual license, trademark note reserving "Hum" for the
stewarded toolchain, CLA-vs-DCO decided before the first external
contributor. The current AGPL-3.0-plus-attribution license is a known
adoption blocker for the stated target domains.

## 3. Units Of Measure

Findings accepted:

- F# is the working precedent: units participate in type inference, are
  erased at runtime (zero cost), and support practical measure
  polymorphism. Known limits: unit information is lost at
  reflection/serialization boundaries, and same-kind generic code has real
  ergonomic gaps that constrained adoption.
- Library approaches (C++ mp-units heading toward standardization, Rust
  uom) are credible but pay in syntax, diagnostics, and boundary
  coverage; Python Pint shows the runtime-registry model trades proof for
  flexibility.
- Incident record: Mars Climate Orbiter is a textbook typed-interface
  catch (high confidence); Gimli Glider's volume/mass/density confusion is
  partially catchable (process failures were also involved); medical
  mg/mcg errors are catchable only where the entry path is typed
  end-to-end.
- No standard gives named certification credit for static unit checking.
  It strengthens verification evidence; it is not a badge. Hum must never
  claim otherwise.

Direction adopted for backlog item 6:

```text
core-language feature, F#-style: inference on literals and arithmetic,
  runtime erasure, deliberately first-order (base units, products,
  quotients, exponents, explicit same-dimension conversions)
no silent unit drops across serialization or FFI: boundary code either
  preserves unit metadata in the schema or writes an explicit cast
validation corpora before syntax freezes: flight-control, dosage, and
  industrial-conversion examples, plus negative tests modeled on MCO,
  Gimli, and mg/mcg incidents
diagnostics are a product feature: a failed dimension check must name the
  mismatched units and where the conversion belongs
```

## Open Verification Debt

- IEC 62304 normative text remains paywalled; medical-wedge claims still
  require purchasing the standard.
- Large-scale OTP operational evidence on tuning restart budgets is
  thinner than folklore suggests; treat supervision parameters as
  something Hum must make measurable, not something research settles.

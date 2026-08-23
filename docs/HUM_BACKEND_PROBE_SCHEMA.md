# Hum Backend Probe Schema

Date: 2026-08-22

Current schema: `hum.backend_probe.v0`

## Purpose

`hum backend-probe [--format human|json] examples/core/minimal_add.hum` is the
first explicit native lowering surface. It accepts only the immutable canonical
minimal-add source and reaches Cranelift only through the callback-scoped
`VerifiedBackendInput` capability issued by `hum.ir_verify` production logic.

The command is a host-local evidence probe, not a general compiler backend. It
does not accept backend-input bytes, JSON, reports, AST/Core values, source text,
or primitive operands as backend authority.

## Exit Contract

- `0`: `decision=GO`, all B01-B15 rows are GO, `ir_ready=1`, and
  `backend_ready=1` for this host-local probe.
- `3`: evidence-backed `NO_GO`; stdout contains one complete report and
  `backend_ready=0`.
- `2`: invocation misuse; stdout is empty and stderr contains one diagnostic.
- `1`: internal evidence loss or malformed adapter result; no readiness claim.

Ordinary `check`, `ir-readiness`, `backend-input`, and `ir-verify` remain
non-executing. Repository-wide backend readiness additionally requires the
published Ubuntu and Windows full-CI lifecycle.

## Top-Level Fields

- `schema`: `hum.backend_probe.v0`
- `decision`: `GO` or `NO_GO`
- `ir_ready`: `1` only because live verification issued the capability
- `backend_ready`: `1` only when every ordered runtime row is GO
- `target_triple`: native ISA triple used by the retained JIT module
- `cranelift_version`: exact production version, `0.133.1`
- `artifact_id` and `source_revision`: authenticated Unit A identities
- `verified_capability_origin`: `verified_backend_input_callback_v0`
- `rows`: exactly fifteen ordered GO/NO-GO records
- `clif_sha256`, `clif_instruction`, and `source_location`: emitted CLIF identity
- `compile`: declaration, definition, and finalization dispositions
- `probes`: four ordinary and two overflow invocation results

Canonical JSON contains no timing field. Timing belongs to the surrounding
evidence capture, not semantic readiness.

## Verified Mapping And ABI

The adapter maps exactly one verified internal function as follows:

- two ordered, distinct Hum `Int` parameters become two `I64` block parameters;
- the verified checked-add operation becomes exactly one `sadd_overflow`;
- the overflow flag feeds one `brif` to a status-1 return;
- the normal edge stores the sum and returns status 0;
- the verified operation span becomes a non-default `SourceLoc`.

The private ABI is exactly `(i64, i64, *mut i64) -> i32`. Status 0 means the
result slot was written. Status 1 means signed overflow and no result is
semantically present. One retained finalized function runs all six probes.

## Ordered Runtime Rows

| ID | Property | NO-GO class |
| --- | --- | --- |
| B01 | verified capability admission | `verified_capability_admission_unavailable` |
| B02 | pinned Cranelift API | `unsupported_cranelift_api` |
| B03 | internal function declaration plan | `function_declaration_unsupported` |
| B04 | exact ABI and ordered parameters | `abi_construction_failed` |
| B05 | fact-derived checked-add selection | `checked_add_selection_failed` |
| B06 | overflow branch/status/store CFG | `overflow_control_flow_failed` |
| B07 | exact non-default source location | `source_location_mapping_failed` |
| B08 | required native target ISA | `unsupported_or_unavailable_target` |
| B09 | Cranelift function verification | `cranelift_verification_failed` |
| B10 | JIT declaration | `jit_declaration_failed` |
| B11 | JIT definition | `jit_definition_failed` |
| B12 | finalization and owned code pointer | `jit_finalization_failed` |
| B13 | four ordinary executions | `ordinary_execution_mismatch` |
| B14 | two overflow executions | `overflow_execution_mismatch` |
| B15 | complete deterministic evidence | `incomplete_backend_evidence` |

Every primary NO-GO retains earlier GO rows. Dependent later rows become
`NO_GO:blocked_by_<ID>` and receive no property credit. Rows may not be missing,
duplicated, reordered, aggregated, or preselected.

## Exact Probe Matrix

| Left | Right | Status | Result |
| ---: | ---: | ---: | ---: |
| 2 | 3 | 0 | 5 |
| -7 | 11 | 0 | 4 |
| 0 | 0 | 0 | 0 |
| 1,000,000 | 24 | 0 | 1,000,024 |
| `i64::MAX` | 1 | 1 | absent |
| `i64::MIN` | -1 | 1 | absent |

Tests compute the expected values independently with Rust `checked_add`; the
adapter cannot satisfy the matrix with constants or interpreter delegation.

## Target And Unsafe Boundary

Required GO build and execution configurations are x86_64 Windows-MSVC and
x86_64 Linux-GNU. After valid capability admission, a runnable non-required
host, or a required host whose native ISA builder is unavailable or rejects the
exact target, must report B08 NO_GO. Targets outside Hum's compiled and runnable
support envelope may remain explicitly unexercised; failure to compile for such
a Rust target is not a B08 runtime result. No unsupported host may report GO,
silently fall back, route to the interpreter, or set backend readiness.

The only unsafe code is one named finalized-function invocation. Its safety
argument binds module lifetime, finalized ownership, exact C ABI, non-null code
pointer, and live result-slot validity. No parser, verifier, raw allocation,
global state, public FFI, AOT, object, linker, or broader language support is
implied.

## Non-Goals

- no general expression or function lowering
- no source-path special-case as semantic authority
- no unchecked, wrapping, or saturating addition
- no durable verified capability or public backend API
- no fallback interpreter, second backend, optimizer, AOT, object, or linker
- no performance, safety-critical, or release-readiness claim

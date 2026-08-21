# Hum IR Verify Schema

Date: 2026-08-20

Current schema: `hum.ir_verify.v0`

## Purpose

`hum ir-verify` strictly authenticates one complete `hum.backend_input.v0`
artifact. It is a local, non-executing report surface. Accepted report text is
evidence only; it is never a capability or backend input.

## Command And Exit Contract

```text
hum ir-verify [--format human|json] <backend-input-file>
```

Human is the default. Exactly one regular readable file is accepted. Stdin,
directories, zero or multiple files, unknown flags, timings, and unsupported
formats are invocation errors: exit 2, no stdout, one stderr diagnostic.

An accepted artifact exits 0 and writes one deterministic report to stdout.
A readable rejected artifact exits 1 and writes one deterministic rejection
report to stdout. Both cases have no unexpected stderr.

## Terminal Status

- `accepted_canonical_backend_input_v0`
- `rejected_backend_input_v0`

Human and JSON reports carry the same ordered facts: schema, tool version,
status, declared artifact ID, computed payload digest, semantic contract,
compiler version, target context, source revision, task/function/operation
counts, ordered pass count, rejected check, and ordered findings.

Rejected reports populate a fact only after that fact has been decoded and
authenticated before the failing row. An unavailable identity or count is
rendered as the literal `null` in both human and JSON output. Early framing,
canonical-byte, and digest failures therefore do not claim a semantic identity,
one task/function/operation, or fourteen passes. Later semantic failures retain
only the facts established by earlier rows. Accepted output remains fully
populated and byte-stable. A `null` is lack of authenticated evidence, never a
backend-readiness claim.

## Strict Byte Boundary

The verifier rejects empty input, UTF-8 BOM, every CR byte, missing or extra
final framing, invalid UTF-8, malformed JSON, duplicate/unknown/missing/reordered
keys, noncanonical strings or integers, and any parse/re-encode difference.
It hashes the exact payload byte slice with repository-owned SHA-256 and keeps
the decoded declared ID unchanged during canonical re-encoding.

Distinct checks preserve blame:

- A-R01: framing, UTF-8, JSON shape, or canonical bytes;
- A-R02: changed authenticated payload bytes retain a foreign digest;
- A-R03: the declared ID changes over otherwise valid payload bytes;
- A-R04: schema/compiler/semantic/target/source identity;
- A-R05: ordered prerequisite pass selection and binding;
- A-R06: task/function/parameter/definition/operation/result/type/span shape;
- A-R07: profile and checked-empty sets;
- A-R08: signed-64 checked-add overflow edge;
- A-R09: decoded identities disagree with live typed facts; and
- A-R10: raw/report/fixture/fabricated data attempts authority.

A semantic corruption with a newly correct digest reaches its semantic row;
hash validity cannot substitute for semantic completeness.

## Sealed Capability

The file command shares the same byte verifier as the live compiler path but
never issues authority. The live path is:

```text
Program + diagnostics
  -> current canonical typed backend facts
  -> canonical artifact bytes
  -> strict independent byte verification
  -> logical cross-binding against the same live facts
  -> callback-scoped VerifiedBackendInput
```

`VerifiedBackendInput` has private fields and a private constructor. It is not
cloneable, serializable, deserializable, default-constructible, string-parsed,
or convertible from bytes or reports. Its lifetime is bound to the verified
artifact and callback. Equivalent independent parses are portable when all
authenticated logical identities agree; mixed or substituted identities fail.

The canonical producer exposes only artifact bytes outside its private typed
facts boundary. Live cross-binding is driven by a verifier-owned private
request whose fields cannot be named or constructed by a sibling backend.
Neither raw typed facts, the artifact/report surface, nor a fixture can mint or
substitute the callback-scoped capability.

The capability exposes only crate-private typed getters for the future backend
adapter. It does not expose raw JSON as authority.

## Readiness Boundary

Successful live verification sets the canonical minimal-add candidate to:

```text
status=ready_for_ir_with_verified_backend_input_v0
ir_ready=1
ready_for_ir=1
backend_ready=0
backend_blocking_reasons=[backend_adapter_not_implemented]
```

`ready_for_ir` is the V0 parity alias of `ir_ready`. No backend has consumed
the capability, so verification does not claim lowering, execution, native
code, ABI stability, optimization, memory safety, or backend readiness.

## Privacy And Dependencies

The verifier is safe Rust, local-only, deterministic, and uses no network,
cloud, telemetry, solver, backend, unsafe code, or new dependency.

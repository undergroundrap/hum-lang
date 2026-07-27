# Hum Backend Lowering Contract V0

Date: 2026-07-27

Status: experiment result; not a production schema or Work Order amendment

## Decision

A Hum backend must not consume the existing `hum.core_lower.v0`,
`hum.resolve.v0`, or checker JSON reports independently. They are projections
for observation. They are not one canonical artifact, and a caller can edit,
substitute, or combine them without invalidating an authority the adapter can
verify.

The production boundary must be:

```text
checked compiler state
  -> deterministic UnverifiedHumIrArtifact bytes
  -> ir_verify(bytes)
  -> opaque VerifiedBackendInput<'a>
  -> backend adapter
```

The adapter accepts only `VerifiedBackendInput<'a>`. It has no constructor from
JSON, source text, spans, names, parameter order, independent checker reports,
or caller-supplied IDs. The verifier and adapter live in the same process and
the capability immutably borrows the exact artifact bytes it verified.

Canonical JSON may be emitted for inspection and caching, but it carries no
authority by itself. An out-of-process adapter must run the production verifier
over the received bytes inside its own process and obtain the same opaque
capability. It must never trust a serialized `"verified": true` value or a
separate CLI verification report.

## Required Artifact

The dedicated artifact schema is `hum.backend_input.v0`. Its deterministic
UTF-8 envelope contains `schema`, `artifact_id`, and `payload`.
`artifact_id` is
`sha256:<64-lowercase-hex-digits>` over the exact canonical UTF-8 bytes of
`payload`; this avoids a self-referential digest. Array order is semantic where
stated below; map key order and insignificant whitespace are fixed by the
artifact encoder. The verifier rejects a transport whose payload is not in
canonical encoding. The digest is an identity and substitution guard, not a
signature.

The payload contains these fields:

| Field | Required shape | Producer |
| --- | --- | --- |
| `compiler` | version, target-independent IR schema version, feature set | compiler driver |
| `source_revision` | source-blob identity plus semantic file ordinal | parser/source owner |
| `module` | canonical module identity and ordered files | module loader |
| `functions` | ordered function records described below | Core/Hum IR lowering |
| `types` | canonical type-ID table | full type checker |
| `definitions` | canonical definition-ID table | resolver |
| `effects` | canonical effect/authority-ID table | effect/authority checker |
| `resources` | canonical resource/profile outcome table | resource/profile checkers |
| `failure_edges` | typed failure and trap table | type/effect/Core lowering |
| `unsupported` | explicit rejected or weakened facts; never silent fallback | owning pass |

Each function record contains:

- canonical function identity, source identity, and item kind;
- export/linkage identity separate from the source display name;
- exact ABI: ordered parameter value IDs and types, result type, calling
  convention, target-independent integer widths, and trap/status convention;
- ordered blocks and operations;
- a canonical expression table keyed by producer-owned node ID;
- for every expression: kind, operator discriminant where applicable, ordered
  child value/node IDs, result value ID, checked type ID, effect/authority ID,
  failure edge, source provenance, and unsupported/weakened status;
- for every operand use: canonical use-node ID and resolver-owned target
  definition ID;
- checked overflow behavior for integer arithmetic;
- the exact pass set whose successful results are required before lowering.

For `minimal_add.hum`, the minimum honest record is:

- one function with two ordered parameter value IDs;
- each parameter value typed as signed 64-bit `Int`;
- one return operation whose value is a checked-add expression;
- two ordered operand node/value IDs;
- each operand use bound to its distinct resolver definition;
- a checked-add result typed as `Int`;
- overflow represented as a typed runtime-trap edge, not native wraparound;
- a pure/no-external-authority effect fact;
- accepted ownership, resource, and profile outcomes;
- source and semantic identities sufficient for diagnostics and debug
  provenance.

## Verification Contract

`hum.ir_verify.v0` consumes the exact artifact bytes and either returns an
opaque `VerifiedBackendInput<'a>` or a structured rejection. The success
capability is non-`Clone`, non-`Copy`, non-`Default`, non-serializable, and bound
by lifetime to the verified bytes.

Verification must fail for:

- absent, extra, duplicate, unknown, or out-of-order semantic records;
- an artifact digest that does not match the exact bytes;
- source, module, function, block, operation, expression, value, type,
  definition, effect, resource, or failure-edge identity substitution;
- missing or reordered expression children;
- a use bound to a foreign or wrong-scope definition;
- a type/effect/resource result from a different node or source revision;
- any checker report not bound into the same artifact;
- unimplemented, skipped, failed, or zero-selection required passes;
- unsupported facts without an explicit fail-closed outcome;
- backend-specific inference that would reconstruct a missing Hum fact.

The adapter rechecks the artifact schema and target-independent ABI subset
through the capability, rejects unsupported operations, and reports loss before
constructing Cranelift IR. It does not reparse source or expression text.

## Cranelift Mapping for the One-Function Probe

Only after verification, the adapter may map:

| Verified Hum fact | Cranelift use |
| --- | --- |
| two ordered `Int` parameter values | two `I64` block parameters |
| checked `add` operator | `sadd_overflow` |
| typed overflow edge | branch to the failure-status return |
| `Int` result | store the `I64` sum to the result slot |
| internal checked-call ABI | `(i64, i64, *mut i64) -> i32` |
| status `0` | result slot initialized |
| status `1` | checked arithmetic overflow; runtime wrapper emits the accepted trap and exits 2 |

This experiment does not claim that the public Hum ABI is fixed. It records the
smallest backend-kernel ABI already demonstrated by the prior feasibility
experiment and the semantic facts required to select it honestly.

## Gap Mapping

| Prior gap | Current finding | Contract result |
| --- | --- | --- |
| 1. Ordered expression-child identities | Canonical expression nodes exist internally, but `core-lower` exports only text, kind, operator, and node count. | Internal fact: GO. Backend transport: NO-GO until the artifact carries ordered node/value IDs. |
| 2. Operand-to-resolver-definition bindings | Resolution succeeds and private summaries retain canonical node and definition identity, but no verifier-bound Core operand binding table is exported. | Internal fact: GO. Backend transport: NO-GO until bindings share the artifact identity. |
| 3. Checked expression type on the Core node | `full-type-check` accepts the statement as `Int`, while the Core expression remains `not_type_checked_v0` with no type text. | NO-GO. Attach a type ID to the canonical expression/result value in the artifact. |
| 4. IR verification pass | `core-verify` checks a non-executing report; `ir_verify` remains unimplemented and returns no capability. | NO-GO. Implement the byte-bound verifier contract above. |
| 5. IR-ready artifact | `core-lower` and `ir-readiness` both report zero IR-ready artifacts. | NO-GO. Emit `hum.backend_input.v0` only after all required producer facts exist. |

Two additional requirements became concrete:

- effect and ownership checks accept this example, but their facts are not
  bound to the Core expression in one artifact;
- resource checking rejects the example because no allocation intent is
  declared, so the current program cannot become IR-ready even after gaps 1-4
  are transported. The backend must consume the canonical accepted/blocker
  outcome; it must not reinterpret or bypass it.

## Exact Stopping Point

Cranelift 0.133.1 successfully initialized the host ISA. The real source passed
`hum check`. The probe then collected the production Core, resolver, type,
effect, ownership, resource, profile, and readiness outputs.

Conversion stopped before the first CLIF instruction at:

```text
VerifiedBackendInput::try_from(current production outputs)
  -> verified_backend_input_artifact_absent_v0
```

There is no `hum.backend_input.v0` artifact, no `hum.ir_verify.v0` capability,
and no safe way to combine the existing projections. Emitting an `iadd` from
`"text": "a + b"`, parameter order, spans, or separately printed reports would
be a fake lowering and is deliberately absent.

## GO / NO-GO

| Requirement | Result |
| --- | --- |
| Cranelift library and host target initialize | GO |
| Real Hum source passes the current checked front end | GO |
| Canonical expression and resolver facts exist inside the compiler | GO, with no backend transport |
| One verifier-bound backend artifact | NO-GO |
| Ordered expression nodes available to the adapter | NO-GO |
| Operand uses bound to definitions in that artifact | NO-GO |
| Checked type attached to the Core result node | NO-GO |
| Canonical effect/authority/resource outcomes attached to the node/function | NO-GO |
| Implemented byte-bound IR verification | NO-GO |
| Cranelift IR emitted from Hum facts | NO-GO; zero instructions emitted |

The next production design target is therefore finite: emit and verify one
backend input containing the facts above for `minimal_add.hum`. Consumer
convergence unrelated to this artifact is not on the native-lowering critical
path.

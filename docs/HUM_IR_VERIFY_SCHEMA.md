# Hum IR Verify Schema

Date: 2026-08-12

Current schema: `hum.ir_verify.v0`

## Purpose

`hum ir-verify` strictly checks one canonical `hum.backend_input.v0` artifact
without loading Hum source, executing code, selecting a backend, or creating
durable authority. Successful verification lends an opaque, byte-bound
`VerifiedBackendInput<'artifact>` capability only for the duration of an
internal callback.

## Command

```powershell
hum ir-verify <backend-input-file>
hum ir-verify --format json <backend-input-file>
```

Accepted input exits zero. A well-invoked but rejected artifact exits one and
prints the selected report to stdout. Invocation and file-I/O failures exit two
on stderr without a report or capability.

## Ordered JSON Shape

The top-level key order is exactly:

1. `schema`
2. `tool`
3. `version`
4. `status`
5. `artifact_schema`
6. `artifact_id`
7. `summary`
8. `rejections`
9. `non_claims_v0`

Success uses `accepted_canonical_backend_input_v0`; failure uses
`rejected_backend_input_v0`. The summary keys are exactly, in order:

`payload_bytes`, `source_count`, `module_count`, `function_count`,
`block_count`, `operation_count`, `expression_count`, `type_count`,
`definition_count`, `effect_count`, `resource_count`, `failure_edge_count`,
`required_pass_count`, and `unsupported_count`.

The accepted minimal-add counts after `payload_bytes` are
`1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 14, 0`.

Each rejection object has exact key order `code`, `byte_offset`,
`logical_path`, `reason`. V0 rejection codes are:

- `invalid_framing_v0`
- `invalid_utf8_v0`
- `malformed_json_v0`
- `trailing_json_bytes_v0`
- `invalid_escape_v0`
- `invalid_control_v0`
- `invalid_number_v0`
- `duplicate_key_v0`
- `invalid_envelope_v0`
- `unsupported_schema_v0`
- `invalid_artifact_id_v0`
- `semantic_model_mismatch_v0`
- `noncanonical_bytes_v0`
- `digest_unavailable_v0`
- `artifact_id_mismatch_v0`

`non_claims_v0` is exactly:

1. `not_backend_ready_v0`
2. `not_executable_v0`
3. `not_a_signature_v0`
4. `no_durable_authority_v0`

## Verification Order

The verifier's load-bearing order is exactly:

1. exact raw transport and UTF-8 framing;
2. ordered occurrence and raw-span decoding with duplicate retention;
3. closed structural validation, including duplicate, unknown, cardinality,
   and order checks;
4. canonical re-emission through the sole encoder in `src/backend_input.rs`
   using the decoded declared artifact ID unchanged;
5. exact equality between those canonical bytes and the original bytes;
6. SHA-256 validation over the original raw payload range;
7. closed semantic and cross-table validation; and
8. private construction and callback-scoped lending of capability.

Whitespace, alternate escapes, key reordering, unknown or duplicate members,
foreign identities, failed or reordered passes, nonempty checked-empty tables,
and any digest mismatch fail closed.

## Authority Boundary

The public report owns diagnostic values only. It contains no borrowed bytes,
verified ranges, permit, constructor, or conversion to capability.
`VerifiedBackendInput<'artifact>` is non-Clone, non-serializable, has no public
fields, and cannot escape the higher-ranked callback. Backend adapter work is
still not implemented.

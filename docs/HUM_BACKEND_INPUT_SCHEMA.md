# Hum Backend Input Schema

Date: 2026-08-12

Current schemas: `hum.backend_input.v0` and `hum.backend_input.v1`

## Purpose

`hum.backend_input.v0` is the canonical, target-independent byte artifact
produced for Hum's exact supported `Int + Int -> Int` minimal-add program. It
freezes the authenticated compiler facts needed by the future IR verifier. The
bytes are unverified: storing, copying, hashing, parsing, or reproducing them
grants no compiler or backend authority.

The producer consumes the private Work Order 19 facts chain. It never derives
authority from a rendered report, a caller-supplied ID, or source-text search.

The additive `hum.backend_input.v1` variant binds only the canonical
`integer_sign` feature: exact live source/module/app/entry identity, one `Int`
argument, signed branch order, three source literals and spans,
`stdout.write` closure, analysis lineage, target-independent semantics, and
artifact provenance. V0 bytes and meaning remain unchanged. V1 bytes alone
grant no JIT or output authority.

## Command

```powershell
hum backend-input examples/core/minimal_add.hum
```

The command accepts exactly one `.hum` file and no options. Success writes only
the canonical artifact bytes to stdout and exits zero. A valid source that is
not the exact supported shape, or whose authenticated prerequisites fail,
exits one with diagnostics on stderr and empty stdout. Invocation and I/O
errors exit two with an error on stderr and empty stdout.

## Canonical Envelope

The artifact is UTF-8 without a BOM or CR. Its exact framing is:

```text
{"schema":"hum.backend_input.v0","artifact_id":"sha256:<64-lowercase-hex>","payload":<PAYLOAD>}\n
```

The envelope keys occur once in the order `schema`, `artifact_id`, `payload`.
There is no insignificant whitespace. The final LF is the sole byte after the
closing brace. `artifact_id` is SHA-256 over the exact embedded payload bytes,
not the envelope.

For the checked minimal-add fixture:

- total bytes: `8715`;
- payload range: `[131,8713)`;
- payload bytes: `8582`;
- artifact ID:
  `sha256:a37707c23cc20a1720e45de901624e3101183a77ec1b5eb4ed55095b5097b82f`;
- source revision SHA-256:
  `sha256:aeae6ae9de975eee9873c3d9ece891e66bd7d6881b5035c24b1a11f3902a52b6`.

The reviewed golden is
[`fixtures/backend_input/minimal_add.backend_input.v0.json`](../fixtures/backend_input/minimal_add.backend_input.v0.json).
It is inspection evidence, not an authority source.

## Payload Order

The payload is one compact JSON object. Its top-level keys occur exactly once
in this order:

1. `compiler`
2. `source_revision`
3. `module`
4. `functions`
5. `types`
6. `definitions`
7. `effects`
8. `resources`
9. `failure_edges`
10. `unsupported`

No top-level member is optional. Empty required facts are explicit arrays;
`null`, floating-point values, duplicate keys, unknown keys, and reordered
members are outside V0.

## Closed Minimal-Add Model

V0 contains exactly:

- one source revision at semantic file ordinal zero;
- module `examples.core.minimal_add`;
- one internal function `add`;
- two ordered `Int` parameters and one `Int` result;
- one `does` section, block, return operation, and checked-add expression;
- two ordered, distinct resolver and semantic definition bindings;
- one `type:int64` record;
- one checked-empty effect record;
- one checked-empty resource record with `allocation_declaration="nothing"`
  and profile `normal`;
- one signed-64 `checked_add` failure edge with
  `runtime_trap_on_overflow`; and
- an empty `unsupported` array.

IDs assigned by the encoder are deterministic:

- `source:0`
- `module:examples.core.minimal_add`
- `function:0`
- `section:function:0:does:0`
- `block:function:0:0`
- `operation:function:0:block:0:0`
- `effect:function:0:0`
- `resource:function:0:0`
- `failure-edge:function:0:0`

Parser node IDs, Core value IDs, resolver definition IDs, and semantic
definition IDs remain producer-owned strings. Ordering never depends on a
pointer, locale, filesystem canonicalization, map iteration, or display-name
sorting.

## Required Passes

Each record has `status="passed"`, `selected=1`, and its zero-based ordinal.
The exact order is:

1. `parse`
2. `semantic_graph_build`
3. `resolve`
4. `body_grammar`
5. `core_preview`
6. `core_lowering`
7. `core_verify`
8. `type_check`
9. `full_type_check`
10. `effect_check`
11. `ownership_alias_check`
12. `allocation_resource_check`
13. `contract_evidence_linking_checked_empty_for_exact_item`
14. `profile_check`

IR verification is not a prerequisite record inside the artifact. The strict
`ir_verify` consumer independently authenticates these complete bytes before
issuing compiler-owned typed authority.

## Canonical Scalars

Object order is schema-defined and arrays preserve semantic order. Integers
are base-ten ASCII without a sign or leading zero. Booleans are `true` and
`false`. Strings use shortest UTF-8 and only these required escapes: `\"`,
`\\`, `\b`, `\t`, `\n`, `\f`, and `\r`; remaining U+0000 through U+001F
scalars use lowercase `\u00xx`. `/` is not escaped.

Paths use `/` and equal the parser-owned normalized path. Host absolute paths,
drive letters, current directories, case folding, and Unicode normalization do
not enter the bytes.

## Source Provenance

The checked fixture pins authenticated locations:

- task: line 3, column 1;
- left parameter: line 3, column 10;
- right parameter: line 3, column 18;
- return statement: line 8, column 5;
- checked-add expression: line 8, column 12.

Production obtains those values from authenticated spans. It does not search
the source text for matching lines.

## SHA-256 Boundary

The repository-owned SHA-256 implementation is private, safe Rust, and
one-shot. It uses standard SHA-256 padding with checked bit-length arithmetic
and lower-case hexadecimal output. It is not a public crypto API and the
artifact digest is a substitution/identity guard, not a signature or proof of
origin.

## Readiness Boundary

Successful live production and verification append these ordered readiness
facts after the earlier producer facts:

- `canonical_backend_input_bytes_v0`;
- `sha256_payload_identity_verified_v0`;
- `ir_verify_passed_v0`; and
- `verified_backend_input_capability_lent_v0`.

The exact canonical candidate becomes:

```text
status=ready_for_ir_with_verified_backend_input_v0
ir_ready=1
ready_for_ir=1
backend_ready=0
missing_passes=[]
blocking_reasons=[]
backend_blocking_reasons=[backend_adapter_not_implemented]
```

No artifact byte, digest, report field, or fixture can replace the private
Program-bound facts access that produced it.

## Non-Claims

V0 does not claim:

- backend lowering or backend-ready evidence;
- a durable or serializable authority;
- durable, serializable, or forgeable capability authority;
- executable code;
- a target choice;
- a stable external ABI;
- a public cryptographic API; or
- support for programs outside the exact closed minimal-add subset.

## Additive Constant-Text V2

`hum.backend_input.v2` binds one canonical constant-Text-output feature. Its
canonical payload fixes compiler/semantic identity, source SHA-256 and
normalized identity, zero arguments, `Result Unit, OutputError`, the exact
literal plus literal/call/binding/return spans, output authority, normal
profile, and ordered required passes. V0 minimal-add and v1 integer-sign retain
their existing meaning. V2 bytes are evidence, not authority; only exact live
regeneration can issue the callback-scoped verifier capability.

# 0011: Add Checked Resolver Before Execution

Date: 2026-07-07
Status: accepted

## Context

Hum has a parser, semantic graph, body grammar preview, state model, and Core Hum
preview. Those surfaces preserve useful facts, but executable semantics need one
more load-bearing layer first: a checked resolver.

Without checked resolution, later passes would guess what a name means, whether a
`set` target is a mutable local or external state, whether a call points at a
task, and whether a source phrase is a real dependency or only prose.

That path creates the same class of bugs Hum exists to prevent: hidden state,
ambiguous intent, tool disagreement, and agent hallucination around names.

## Decision

Hum adds `hum resolve` as the first checked name, scope, reference, and mutable
place report.

The V0 schema is `hum.resolve.v0`, documented in
[../HUM_RESOLVE_SCHEMA.md](../HUM_RESOLVE_SCHEMA.md).

The checked resolver must happen before executable core lowering, type checking,
effect checking, ownership, borrowing, debugger facts, LSP go-to-definition, and
IR emission claims.

## Consequences

- `hum resolve --format json` becomes an adapter-ready tool surface.
- The resolver owns scope, definition, reference, and mutable-place identity.
- `uses:` and `changes:` references are linked to known source definitions when
  possible and preserved as declared permissions for body resolution.
- `set` targets must resolve to mutable places or declared change permissions.
- Resolver diagnostics use stable `H060x` codes.
- V0 remains honest: no type checking, borrow checking, effect checking, module
  import resolution, executable semantics, optimizer authority, or IR emission.

## Alternatives Rejected

1. Wait for the type checker to own all resolution.

   Rejected because editor tools, state modeling, and Core Hum lowering need
   checked source identity earlier than full type checking.

2. Let `core-preview` quietly become the checked resolver.

   Rejected because `core-preview` is explicitly a non-executing preview. A
   separate resolver report keeps preview facts and checked facts distinct.

3. Rely on agents or prose rules to infer name meaning.

   Rejected because Hum's compiler artifacts, not conversation context, must be
   the authority for source meaning.

## BDFL Note

Checked resolution is the first place Hum stops being a pile of beautiful
intent blocks and starts becoming a compiler.

The standard is simple: if a human, compiler pass, editor, debugger, verifier,
optimizer, and agent cannot point at the same definition for a name, the next
layer is not ready.

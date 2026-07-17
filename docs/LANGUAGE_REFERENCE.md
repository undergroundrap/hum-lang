# Hum Language Reference

Date: 2026-07-08
Version: 0.0.1 pre-alpha
Status: reference spine, not a stable standard

## Purpose

This is the traditional language reference spine for Hum.

It exists so the project does not become only a pile of research notes, doctrine
docs, examples, and implementation details. This document answers the ordinary
language questions first:

- What can a `.hum` file contain?
- What are the top-level forms?
- What do sections mean?
- What does Milestone 0 parse and check today?
- Which deeper document owns each unfinished area?

If this document conflicts with [ARCHITECTURE.md](ARCHITECTURE.md), the
architecture is the design bug detector and the conflict must be resolved.

## Normative Levels

Hum uses explicit stability language:

- `current`: implemented or recognized by the Rust bootstrap today
- `reference`: intended language rule, but not fully implemented yet
- `alpha`: intended for the `offline-tool@0.1` public alpha subset
- `future`: design direction, not a compatibility promise
- `rejected`: should not enter Hum without a new accepted design decision

Milestone 0 is local, offline-first, parser/checker/graph only. Milestone 1 has
begun a narrow local `hum run` interpreter for the explicitly documented core
subset; it does not execute generated code, build scripts, packages, plugins, or
foreign code.

## Reading Map

- [ARCHITECTURE.md](ARCHITECTURE.md): ground-truth map
- [PAVED_ROAD_DOCTRINE.md](PAVED_ROAD_DOCTRINE.md): default-path doctrine
- [LANGUAGE_CONSTITUTION.md](LANGUAGE_CONSTITUTION.md): rules Hum must not violate
- [MILESTONE_0_GRAMMAR.md](MILESTONE_0_GRAMMAR.md): current Rust bootstrap parser grammar contract
- [FORMAL_CORE.md](FORMAL_CORE.md): precise executable core direction
- [HUM_CORE_CONTRACT_SCHEMA.md](HUM_CORE_CONTRACT_SCHEMA.md): machine-readable Core Hum contract
- [HUM_CORE_PREVIEW_SCHEMA.md](HUM_CORE_PREVIEW_SCHEMA.md): non-executing Core Hum candidate preview
- [HUM_RESOLVE_SCHEMA.md](HUM_RESOLVE_SCHEMA.md): checked scope, definition, reference, and mutable-place report
- [LANGUAGE_SUBSET_0_1.md](LANGUAGE_SUBSET_0_1.md): pinned alpha subset
- [SEMANTIC_GRAPH_SCHEMA.md](SEMANTIC_GRAPH_SCHEMA.md): graph JSON emitted today
- [DIAGNOSTICS.md](DIAGNOSTICS.md): stable diagnostic code contract
- [FORMATTER.md](FORMATTER.md): canonical formatting direction
- [COMPILE_TIME_STRATEGY.md](COMPILE_TIME_STRATEGY.md): check/build/comptime discipline
- [INTEROP_AND_PORTABILITY.md](INTEROP_AND_PORTABILITY.md): foreign, platform, and adoption boundaries
- [PORTABILITY_BOUNDARY_MODEL.md](PORTABILITY_BOUNDARY_MODEL.md): target facts, platform adapters, capability absence, and artifact portability evidence
- [PERFORMANCE_CONTRACTS.md](PERFORMANCE_CONTRACTS.md): benchmark and optimization claim discipline
- [MATH_OBLIGATIONS_SCHEMA.md](MATH_OBLIGATIONS_SCHEMA.md): external-validator obligation export surface
- [RESOURCE_REPORT_SCHEMA.md](RESOURCE_REPORT_SCHEMA.md): resource and optimization claim report surface
- [HUM_RESOURCE_CHECK_SCHEMA.md](HUM_RESOURCE_CHECK_SCHEMA.md): declared allocation/resource intent gate
- [HUM_PROFILE_CHECK_SCHEMA.md](HUM_PROFILE_CHECK_SCHEMA.md): runtime profile policy declaration gate
- [STATE_MODEL.md](STATE_MODEL.md): state, mutation, ownership, borrowing, and linear resource doctrine
- [HUM_STATE_MODEL_SCHEMA.md](HUM_STATE_MODEL_SCHEMA.md): machine-readable state model contract

## Checked Reference Fixture

[../examples/reference_surface.hum](../examples/reference_surface.hum) is the
canonical Milestone 0 reference fixture. It is intentionally small and should
stay parseable by the Rust bootstrap CLI.

Use it when changing this reference, the grammar contract, syntax highlighting,
or graph facts. If prose says a current construct exists, the fixture should
show the ordinary spelling unless a smaller focused example owns that case.

A healthy reference fixture passes `hum check` without diagnostics, produces no
unlinked test obligations from `hum test-skeletons`, and has no unlinked
security or trust evidence obligations in `hum graph`.

## Source Files

A Hum source file uses the `.hum` extension.

Current source files are UTF-8 text without BOM. Public docs and tools should
use repo-relative paths and portable examples unless documenting a platform
boundary.

A file may start with a module declaration:

```hum
module examples.task_list
```

Milestone 0 treats the module path as source metadata. Future module and
visibility rules are tracked in [CORE_LANGUAGE_SHAPE.md](CORE_LANGUAGE_SHAPE.md).

## Lexical Shape

Hum is line-oriented and block-structured.

Current rules:

- top-level items use braces
- normal source does not use semicolons
- section headers end in `:`
- section body lines are indented under the section header
- comments may start with `#` or `//` inside sections
- two-space indentation is the formatter direction

Current parser behavior is intentionally small. It captures many section body
lines as text for graph facts and future lowering. It is not the final grammar
engine. See [MILESTONE_0_GRAMMAR.md](MILESTONE_0_GRAMMAR.md) for the exact
Milestone 0 parser contract.

## Top-Level Forms

Current Milestone 0 recognizes these item kinds:

- `app`
- `type`
- `store`
- `task`
- `test`

The `offline-tool@0.1` alpha subset may accept `task`, `type`, `store`, and
`test`. `app` remains parsed for examples but is not yet an alpha execution
boundary.

### `app`

```hum
app reference_surface {
  why:
    explain the application

  starts with:
    run_reference

  task run_reference -> Unit {
    does:
      return
  }
}
```

An `app` groups program-level intent and may contain nested items. The current
executable slice selects exactly one top-level app when `hum run` has no
`--entry`. Its single meaningful `starts with:` line is one bare snake_case
name of a directly nested task returning `Unit` or `Result Unit, E`. App-mode
lookup never falls back to a same-named task outside the app. A successful app
adds no automatic `Unit` output; typed failure retains the causal chain and is
written to stderr. Files without an app keep legacy single-task selection, and
`--entry` remains a direct task probe rather than app execution.

`starts with:` does not initialize state. App state initialization remains
undesigned. Session Y recognizes exactly `stdout.write`, `clock.replay`, and
`files.read` as external source capabilities. App `uses:` is the maximum;
reachable tasks declare direct and transitive capability budgets, callers cover
callee closure, and the app covers the start closure. Stable effect-policy IDs
and structured route sites make that source policy auditable. These declarations
are not operator grants. Session Z adds repeatable exact `--allow stdout.write`
and `--deny stdout.write`; the default is deny, duplicates are idempotent, and
deny wins. The only operation is
`stdout_write(text: Text) -> Result Unit, OutputError`. It writes exact UTF-8
bytes immediately with no newline through the app adapter after source closure,
operator consent, and the 1 MiB rolling limit succeed. Denial, limit rejection,
and adapter failure are typed `OutputError` values. `--entry` remains a pure
probe and rejects an authority-bearing task closure even when allowed.
The exact `stdout_write` name is reserved; a user task declaration with that
name is H0623 rather than a shadowing rule that could differ between stages.
Output-reachable recursive call cycles are H0624 in this bounded slice; use an
explicit bounded loop or non-recursive task chain so the audit route remains
finite and exact. Missing task/app authority remains H0621 and missing caller
closure remains H0618 instead; H0624 is checked only after that route is valid.
This is not a general recursion ban.

Session AA adds repeatable `--replay-tick <UInt>` input, bounded to 1024
values, and exact `--allow clock.replay`/`--deny clock.replay` consent. Values
are ordered input, never authority. The only operation is
`clock_replay_tick() -> Result UInt, ReplayClockError`; each successful call
consumes the next runner value. Default or explicit denial returns
`ReplayClockError.denied` before adapter access, and an empty or exhausted
sequence returns `ReplayClockError.exhausted`. Replay routes use the same
app/task/caller closure and finite forensic identity as output. H0625-H0628 own
missing direct authority, invalid arity, reserved-name collision, and
replay-reachable recursion. This controls one replay input only: it does not
read `os.clock` or claim whole-program determinism or deterministic scheduling.

Session AB adds one opaque runner-owned `Path` parameter for structural app
entry. `hum run` preserves that argument through the native OS string boundary;
it is never reconstructed from `String`. Hum source has no Path literal,
conversion, display, comparison, concatenation, component, join, parent,
storage, return, or general library operation. H0629 rejects Path declarations
outside the one start parameter, and H0630 rejects source construction or use
outside Session AD's exact hardened reader consumption.
Direct `--entry` cannot inject Path. On Windows the invocation accepts only an
ordinary drive-letter-rooted lexical candidate and rejects the namespace,
traversal, ADS, empty-component, trailing-dot/space, and DOS-device classes
pinned by the Work Order before host access. One exact native
`--allow files.read=<path>` payload is retained separately from the Path input;
duplicates are idempotent, distinct payloads reject, and exact
`--deny files.read` wins. Both facts begin `locality_unclassified`. Session AC's
isolated Windows adapter may narrow an internal status to threat-scoped
`fixed_local_v0` only when stable drive/mapping observations, an empty storage
dependency result, a complete bounded extent list, and non-removable
ATA/SATA/NVMe descriptors for every backing disk agree. It opens only
synthesized volume/disk devices with desired access zero and never opens the
candidate path. This supports no candidate metadata, file read, portable
locality, or filesystem-sandbox claim. Non-Windows Path execution remains
unavailable.

Session AD permits the otherwise opaque Path to appear only as the sole
argument of `files_read_text(path: Path) -> Result Text, FileReadError` on the
structural start task. H0631 requires complete `files.read` source closure;
H0632 owns the exact Path signature; and H0633 reserves the builtin name.
Default or exact operator denial returns `FileReadError.denied`, while a
different exact native grant returns `FileReadError.outside_grant`, before
candidate access. Lexical or reparse rejection is `unsafe_path`; unsupported or
unclassified locality is `unavailable`; and candidate outcomes are
`not_found`, `not_file`, `too_large`, `invalid_utf8`, or `io_failed`. The
adapter performs one read-only file open only after the authority and
`fixed_local_v0` gates, reads at most 1 MiB, and accepts strict UTF-8. There is
no directory, write, glob, recursive, current-directory, portable-IO, or
filesystem-sandbox surface.

### `type`

```hum
type Task {
  title: Text
  done: Bool
}
```

A `type` describes data. Milestone 0 parses field names, field types, section
counts, section spans, and section line facts. Rich type checking is future
work.

### `store`

```hum
store tasks: list Task {
  why:
    remember the user's tasks
}
```

A `store` declares persistent or shared data intent. The type after `:` is
currently captured as text. Future stdlib/profile work decides the concrete
strategy, such as a paved-road `Map` with profile-backed internals.

### `task`

```hum
task add_item(title: Text) -> Result Task, TaskError {
  why:
    let the user remember something to do

  needs:
    title is not empty

  ensures:
    new item is saved

  does:
    return item
}
```

A `task` is Hum's function-like unit. It combines a callable header, checked
intent sections, and a `does:` body. Milestone 0 parses and checks task shape
but does not execute the body.

Session AL permits one passed pure task value: a receiver parameter may have
the exact type `task(UInt) -> UInt`, a caller may pass one unqualified same-file
task name, and the receiver may return exactly `transform(value)`. Resolution
precedes this analysis. The row-elided source type is accepted only for an
inferred closed-empty latent row and no failure root. Task values cannot be
stored, returned, displayed, compared, captured, qualified, or constructed.

### `test`

```hum
test add_item rejects empty title regression {
  covers:
    add_item fails when title is empty

  does:
    expect add_item("") fails with TaskError.empty_title
}
```

A `test` is first-class evidence. Milestone 0 parses tests, modifiers, `covers:`
lines, and links exact or canonical-token coverage phrases to generated task obligations.

Known current test modifiers:

- `unit`
- `property`
- `fuzz`
- `regression`
- `integration`
- `model`

## Names

Current Hum identifiers are deterministic tokens, not English phrases.

Value names use `snake_case` and match `[a-z_][a-z0-9_]*`. This covers module
path segments, app names, store names, task names, parameter names, and field
names. Type names use `PascalCase` and match `[A-Z][A-Za-z0-9]*`.

```hum
module examples.task_list

store work_items: list WorkItem {
  why:
    keep work visible
}

task remember_work_item(title: Text) -> Result WorkItem, WorkError {
  why:
    let the user capture work without losing the reason it matters

  does:
    return item
}
```

Spaces are not part of identifiers. A spaced name such as `remember work item`
is a parse error (`H0009`) whose help suggests `remember_work_item`. Human
phrasing belongs in `why:` and other prose-bearing sections. Test names may
remain multi-word phrases until the test grammar is pinned; `snake_case` is the
default candidate for that future rule.

Milestone 0 stores accepted names as parsed source text. `hum resolve --format
json` emits the first checked scope, definition, reference, and mutable-place
report as `hum.resolve.v0`.

See [decisions/0012-adopt-snake-case-identifiers.md](decisions/0012-adopt-snake-case-identifiers.md) and [decisions/0009-adopt-formal-readability-not-english-mimicry.md](decisions/0009-adopt-formal-readability-not-english-mimicry.md).
## Parameters And Results

Task and test parameters use this current shape:

```hum
task name(input: Type, change draft: Type, consume owned: Type) -> Output {
  does:
    return value
}
```

Current parser facts:

- parameter name
- parameter permission: `borrow`, `change`, or `consume`
- parameter type text
- source span
- optional result type text for tasks

Unmarked parameters default to `borrow`. The default keeps ordinary signatures
read-only and follows decision 0014's ownership direction: mutation and ownership
transfer should be visible at the boundary, while the paved road remains small.
Use `change` when the task may write through the parameter. Use `consume` when
the task receives ownership authority and may move or close it.

Parameter and result types are not yet fully type-checked in Milestone 0.

Task result text may carry the first ownership source relationship:

```hum
task echo_view(borrow text: Text) -> Slice Text from text {
  does:
    return text
}
```

In the current V0 checker, `from` sources for returned views must name a task parameter. The executable subset proves two parameter-derived forms: returning the bare parameter, and returning through the closed view-deriving operation `slice_until(source, separator)`. `slice_until` is the first member of the closed set; it returns the text before the separator and carries the provenance of its first argument only. Returning a view that depends on a local is rejected with `H0805`, non-closed derivation chains are rejected instead of guessed, and internal references such as `from parser.buffer` remain explicitly banned until the internal-reference repair from decision 0014 is implemented. `hum graph` exposes the declared dependency fact, while `hum ownership-check` verifies the narrow parameter-derived subset.

### Writable Field Aliases

Milestone 1 recognizes exactly this unannotated local form in a straight-line
task body:

```hum
let alias_to_x = change point.x
set alias_to_x = 9
```

`point` must already have visible mutation authority from a `change` or
`consume` parameter or a local `change` binding. A borrowed parameter cannot
supply the alias and reports H0802. The alias stores the place `point.x`, not a
copy: reads observe the current field and `set alias_to_x` writes through to
the owner.

The recognized lifetime runs from the binding through the alias's last
syntactic use in the same straight-line body. During that interval, another
direct read or write of `point.x`, an owner-wide `point` access, or a second
alias that may name `point.x` reports H0808. Direct access to the definitely
distinct field `point.y` remains accepted, and a new `point.x` alias may begin
after the earlier alias's last use.

H0809 rejects creation or use inside branches/loops, passing, returning,
storing, wrapping the alias in `borrow`/`change`/`consume`, alias-to-alias
binding, nested places, list elements, rebinding either alias or live owner,
and shadowing an already-visible parameter, local, or declared permission name.
Hum does not infer a wider lifetime or support general aliases, internal
references, or broad flow-sensitive borrowing. The checked examples and pinned
misuses are in `examples/probes/writable_field_aliases.hum` and
`fixtures/ownership_check/session_v_*`.

## Sections

Sections are named intent blocks inside items.

```hum
needs:
  title is not empty
```

Milestone 0 graph output includes:

- section name
- section span
- non-empty line count
- `line_items` with text, span, and meaningful/comment status

Comment-only section lines are preserved in graph output and marked
`meaningful: false`.

`hum syntax` emits a machine-readable section catalog with hover text and a
semantic-token legend so editor and LSP adapters can explain and color the
current surface without duplicating language copy.

## Task Sections

Common task sections:

| Section | Status | Meaning |
| --- | --- | --- |
| `why:` | current | human and agent purpose |
| `targets:` | current | target fact records and capability-family declarations |
| `uses:` | current | read dependencies and capabilities |
| `changes:` | current | mutation/write permissions |
| `needs:` | current | preconditions, Predicate v2 runtime entry checks, and generated test obligations |
| `ensures:` | current | postconditions, Predicate v2 runtime exit checks with `old(...)` pre-state capture, and generated test obligations |
| `protects:` | current | safety/security promises and evidence obligations |
| `trusts:` | current | trust assumptions and evidence obligations |
| `fails when:` | current | explicit failure modes |
| `watch for:` | current | edge cases and generated test obligations |
| `cost:` | current | time, space, allocation, and check claims |
| `allocates:` | current | visible allocation expectations and limits |
| `avoids:` | current | implementation shapes to avoid |
| `tradeoffs:` | current | accepted engineering compromises |
| `optimizes:` | current | performance or quality priorities when tradeoffs conflict |
| `tests:` | current | declared test obligations |
| `proves:` | reference | formal or semi-formal proof obligations |
| `does:` | current | captured body text, with a narrow Milestone 1 executable subset under `hum run` |

Additional sections such as `creates:`, `deletes:`, `assumes:`, `keeps:`,
`benchmarks:`, and `calls:` are part of the broader design direction and appear
in design docs. They require more checker and graph work before becoming stable
executable promises.

## Canonical Section Order

Milestone 0 warns when known task and test sections are out of canonical order.

Preferred task order follows [FORMATTER.md](FORMATTER.md). The current checker warning order is narrower and is listed in [DIAGNOSTICS.md](DIAGNOSTICS.md):

```text
why
targets
uses
changes
creates
deletes
needs
assumes
ensures
keeps
protects
trusts
fails when
watch for
cost
allocates
avoids
tradeoffs
optimizes
tests
benchmarks
proves
does
```

## Target Declarations

`targets:` is the current source-visible portability section. It lets source say
which target fact records or capability families matter without selecting a
backend target, probing the host, enforcing a runtime profile, or claiming an
artifact exists.

Milestone 0 recognizes only these formal line keys inside `targets:`:

```hum
targets:
  triple: wasm32-wasi-preview1
  requires: os.filesystem
  denies: os.network
```

Current graph meaning:

- `triple:` adds a source-declared `target_fact_records` value
- `requires:` adds a source-declared `required_capability_families` value
- `denies:` adds a source-declared `denied_capability_families` value
- every recognized line also appears in `source_target_declarations` with a
  source span and `declared_not_enforced_v0` status

Milestone 0 validates named target fact records and capability families against
`hum target-facts`. Unknown record IDs or target triples emit `H1201`; unknown
capability families emit `H1202`; meaningful lines that do not use a current
formal key emit `H1203`. When a known `requires:` family is absent or omitted in
a declared target fact record, Hum emits `H1204` and lists the family under
`unavailable_capability_families` in `hum graph`. If the same `targets:` block
both requires and denies one capability family, Hum emits `H1205`. Milestone 0
still does not select a backend target, enforce runtime profiles, or prove
artifact portability.

## Effects And Capabilities

Hum does not intend to hide IO, mutation, allocation, randomness, time, network,
or unsafe behavior behind innocent-looking calls.

The current state model contract is [STATE_MODEL.md](STATE_MODEL.md), emitted by `hum state-model --format json` as `hum.state_model.v0`.

Current Milestone 0 checks and reports are small:

- save-like mutation in `does:` should refer to declared `changes:` targets
- parameter writes are checked against `borrow`/`change`/`consume` authority by `hum ownership-check`
- known sections should appear in canonical order
- tasks should have important context such as `why:` and `does:`
- contract-like lines should not be obviously hollow, tautological, or placeholder-shaped
- simple cost claims should not contradict visible loop shape
- `hum resolve` should link scopes, definitions, references, and mutable-place targets without claiming type or borrow checking

Future effect reports are tracked in [EFFECT_REPORT_SCHEMA_0_1.md](EFFECT_REPORT_SCHEMA_0_1.md).

## Resources, Layout, And Optimization

Hum treats resources as language facts, not folklore.

Current source-visible resource blocks include:

- `cost:`
- `allocates:`
- `avoids:`
- `tradeoffs:`
- `optimizes:`

Milestone 0 preserves these lines in graph facts, exposes them through
`hum.resource_report.v0`, and performs small honesty checks. It also exports
conservative external-validator math obligations for explicit allocation-free
claims such as `allocates: nothing`. See
[RESOURCE_REPORT_SCHEMA.md](RESOURCE_REPORT_SCHEMA.md),
[HUM_RESOURCE_CHECK_SCHEMA.md](HUM_RESOURCE_CHECK_SCHEMA.md), and
[MATH_OBLIGATIONS_SCHEMA.md](MATH_OBLIGATIONS_SCHEMA.md).

Reference rule: the compiler may optimize from declared intent only when it can
emit evidence for the choice or clearly mark the claim as unverified. Hum should
not claim that every program becomes faster, that a compiler can always find the
optimal algorithm, or that benchmarks are proofs.

Future layout rules should make memory representation inspectable:

- records, arrays, maps, packets, tensors, and FFI structs should expose layout
  assumptions when layout matters
- ABI, alignment, endian, pointer-width, shape, dtype, and device assumptions
  should become semantic graph facts
- changing layout-sensitive code should change the evidence that depends on it

Future space/time strategy rules should distinguish program shapes:

- streaming and sequential code can receive the strongest space guarantees
- tree, DAG, and circuit-like code may support recompute-heavy lowering
- oblivious access patterns may support bounded simulation strategies
- arbitrary pointer mutation, I/O, concurrency, and hardware effects require
  explicit effects and weaker claims

The practical Hum promise is not magic speed. The promise is source-visible
resource intent, compiler classification, generated obligations, measured
benchmarks, and reviewable optimization evidence.

## Contracts And Blame

Contract sections assign responsibility:

- `needs:`: caller or context must satisfy the precondition
- `ensures:`: task must satisfy the postcondition on success
- `fails when:`: task exposes a typed failure condition
- `changes:`: task may mutate only declared targets
- `cost:`: implementation claims must be checkable or explicitly deferred
- `protects:` and `trusts:`: security boundaries must be named

Current executable blame semantics are deliberately small and explicit:
recognized `needs:` predicates run at task entry and blame the caller when
false; recognized `ensures:` predicates run after successful return and blame
the task when false. Predicate v2 remains one comparison and preserves v1
arithmetic, direct eligible places, `old(place)` entry capture in `ensures:`,
and `list_len(place)`. It adds exact Text `==`/`!=`, exact ordered `List Text`
`==`/`!=`, and contract-only `list_count(list_text, text) -> UInt`. It does not
add ordering for Text/lists, arbitrary calls, membership, normalization,
connectives, or general list equality. H0701 marks meaningful prose with no
executable intent; H0704 rejects malformed or semantically invalid executable
candidates before evaluation. H0630 retains precedence for opaque Path use.

The broader reference rule is simple: important claims belong in checked sections, not comments. A checked section line should be specific enough that a future verifier, test, or reviewer could notice when an implementation breaks it.

## Test Obligations And Coverage

Milestone 0 generates task `test_obligations` from meaningful lines in:

- `needs:`
- `ensures:`
- `watch for:`
- `tests:`

`hum graph` links obligations to top-level tests when a meaningful `covers:`
line exactly matches the generated coverage phrase after whitespace
normalization or shares its canonical token key. The canonical key lowercases and
splits on punctuation while preserving identifier tokens such as `add_item`.
It does not absorb filler words, aliases, synonyms, or broad paraphrases.

`hum test-skeletons` prints Hum `test` blocks for unlinked obligations. It does
not execute code or write files.

## Evidence Obligations

Milestone 0 generates task `evidence_obligations` from meaningful lines in:

- `protects:`
- `trusts:`

`protects:` lines become `security_property` obligations with
`security_boundary` blame. `trusts:` lines become `trust_boundary` obligations
with `trust_boundary` blame. Current obligations include source spans,
generated `covers` phrases, canonical `coverage_key` values,
`suggested_evidence`, `verification_status`, and `linked_evidence`.

`hum graph` links evidence obligations to top-level tests when a meaningful
`covers:` line exactly matches the generated coverage phrase after whitespace
normalization or shares its conservative coverage key. `verification_status` is
`linked` when at least one evidence artifact matches and `unverified` when none
do.

This is intentionally not a proof claim. A linked test means the test names the
same coverage target; it does not prove the protection or trust boundary is
fully verified. Future evidence kinds include proof exports, threat-model
notes, review packets, sanitizer runs, and profile evidence.

## Executable Body Status

The `does:` block is executable only for the explicitly interpreted Milestone 1 subset, and remains future surface beyond that subset.

Milestone 1 begins with `hum run <file> [--entry <task>] [--args ...]` over checked source. With no `--entry`, one top-level app selects its directly nested structural start task; with no app, legacy single-task selection remains. Explicit `--entry` remains the direct task probe. The current tree-walking interpreter covers the forms required by the executable core fixtures: Int/Bool literals, arithmetic, comparisons, `let`, `change`, `set`, direct record field reads, direct field-place assignment with `set record.field = value`, direct numeric list element reads such as `items[0]`, local direct field-view bindings of the form `let view = borrow record.field`, local direct element-view bindings of the form `let view = borrow items[0]`, exact local writable field aliases of the form `let alias = change record.field`, `if`, `for each`, `return`, `fail`, task calls, typed failure values, Predicate v2 `needs:`/`ensures:` checks including `old(...)`, `list_len(...)`, exact Text/ordered `List Text` equality, and contract-only `list_count(...)`, the simple list/record values needed by `count_completed`, the minimal list-growth operation `list_append(change list, item)`, the body-reachable list-length read `list_len(list)`, and the closed text view operation `slice_until(text, separator)`; `old(...)` and `list_count(...)` remain contract-only vocabulary and trap with a clear message if called from a body. Arithmetic and contract expressions share the parser-owned tree: `*` and `/` bind before `+` and `-`, each level associates left, comparisons bind outside arithmetic, then `and`, then `or`; parentheses override that order. It also enforces the current narrow ownership subset at runtime: writing through a default `borrow` parameter or one of its direct fields traps with `H0802`, using a local after it was moved by `consume` or by return traps with `H0801`, leaving recognized Transaction-shaped resources unconsumed traps with `H0803`, consuming them twice traps with `H0804`, violating the V0 returned-view `from parameter` rule traps with `H0805`, structurally appending to a list during active iteration traps with `H0806`, using a local field view after that exact field was written traps with `H0807`, and using a local element view after `list_append` grew that list also traps with `H0807`. Writable aliases read and write through the exact owner field; H0808 rejects live overlapping access and H0809 rejects escape or unsupported alias shapes before body mutation. Integer overflow and division by zero trap instead of wrapping; executable contract violations exit as runtime failures with caller/task blame diagnostics.

Session W adds only two explicit direct named-call failure forms: `let value =
try fallible_call()` for equal nominal error roots, and `let value = try
fallible_call() or fail CallerError.context` for caller-root wrapping that
preserves the inner cause. Known fallible calls cannot propagate implicitly in
any currently executable expression position, including nested calls and loop
collections. The runtime prints a stable outer-to-root causal chain with root
origin and all recognized call sites. Permission-bearing call arguments, first-class
`Result`, variant membership, recovery, exceptions, ambient backtraces, and
general call typing remain unsupported.

The interpreter also executes the exact Session AL passed-callable slice using
a private resolved-definition handle. The handle is nonescaping and has no
source conversion or display surface; this does not add general closures or
higher-order runtime values.

The report gates remain non-executing. `hum core-preview` maps recognized lines into Core Hum candidate operations and explicit blockers without executing, type-checking, effect-checking, or emitting IR. `hum resolve` performs the first checked pass over scopes, definitions, references, and mutable-place targets. `hum type-env` records declared type names and annotations with resolver identity. `hum type-check` validates declaration annotation names without expression inference or body checking. `hum full-type-check`, `hum effect-check`, `hum ownership-check`, and `hum resource-check` report recognized facts and blockers without execution or IR emission. `hum ir-readiness` consumes the checked resolver, type, Core verifier, full-type-check, effect-check, ownership-check, and resource-check summaries before any future lowering claim. New executable syntax must still become checkable, lower into [FORMAL_CORE.md](FORMAL_CORE.md), and preserve the non-claims of the report surfaces before it becomes stable.

Starter executable forms are tracked in [CORE_LANGUAGE_SHAPE.md](CORE_LANGUAGE_SHAPE.md).

## Compile-Time Execution

Compile-time execution is `future`, not part of Milestone 0.

Hum should eventually support compile-time constants, assertions,
specialization, generated tests, and bounded code generation. These features
must remain explainable to humans, tools, and agents.

Reference rules:

- compile-time work must be explicit in source
- compile-time work must have effect and resource limits
- imports must not execute arbitrary code
- package metadata must stay declarative and cacheable
- compile-time I/O, network access, process execution, and foreign calls require
  separate profile gates
- compile-time output must preserve source spans, provenance, and graph facts
- expensive compile-time features need timing budgets and diagnostics

Hum should prefer declarative checked sections and compiler-known generation
before user-defined macro systems. A macro or compile-time feature that defeats
diagnostics, formatting, semantic graphs, or agent tooling is not ready.

## Diagnostics

Milestone 0 diagnostics have stable `H####` codes, severity, message, optional
span, and optional help. Diagnostics appear in terminal output and `hum graph`
JSON.

See [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Semantic Graph

The semantic graph is the machine-readable form of what the compiler currently
understands. It is the source for agents, editor tools, future formatters,
coverage checks, and evidence bundles.

Current command:

```powershell
hum graph <file-or-dir>...
```

During bootstrap:

```powershell
cargo run -- graph examples
```

See [SEMANTIC_GRAPH_SCHEMA.md](SEMANTIC_GRAPH_SCHEMA.md).

## Evidence Report

`hum evidence` summarizes the current security and trust evidence obligations
from `protects:` and `trusts:` lines without requiring a human or agent to read
the full semantic graph JSON.

Current command:

```powershell
hum evidence <file-or-dir>...
hum evidence --format json <file-or-dir>...
```

During bootstrap:

```powershell
cargo run -- evidence examples/reference_surface.hum
cargo run -- evidence --format json examples/reference_surface.hum
```

See [EVIDENCE_REPORT_SCHEMA.md](EVIDENCE_REPORT_SCHEMA.md).

## Interop And Adoption Boundaries

Interop is a future adoption requirement, not a Milestone 0 execution feature.

Milestone 0 has no FFI, no generated code execution, no package downloads, no
foreign build scripts, and no network registry access.

Reference rule: foreign code must be source-visible and graph-visible. C, C++,
Rust, Python, Wasm, process boundaries, platform APIs, and accelerator runtimes
must enter Hum through explicit trust, ownership, layout, effect, failure, and
profile contracts. Target-sensitive code must also expose target facts,
capability absence, adapter identity, and artifact evidence. See
[INTEROP_AND_PORTABILITY.md](INTEROP_AND_PORTABILITY.md) and
[PORTABILITY_BOUNDARY_MODEL.md](PORTABILITY_BOUNDARY_MODEL.md).

Hum should adopt existing ecosystems by wrapping them safely, not by pretending
foreign code has Hum's safety model.

## Agent And Tooling Contract

Hum is designed for humans and agents, but agents are not trusted.

Agents should receive compact, current, schema-backed facts:

- stable diagnostics
- syntax surface metadata
- semantic graphs
- capability reports
- evidence reports
- math obligation exports
- source spans and document symbols
- setup health facts
- Core Hum, Hum IR, and backend contract facts

Agents should not scrape terminal prose when a JSON schema exists. Agent-facing
docs should be small, versioned, and tied to the current CLI surface so generated
code follows the language that exists rather than an older memory of Hum.

## Formatting

Hum will have one canonical format. The formatter should be strict, boring, and
low-configuration.

Current direction:

- two-space indentation
- canonical section order
- braces for item bodies
- no semicolons in ordinary source
- preserve comments and intent blocks

See [FORMATTER.md](FORMATTER.md).

## Paved Road Rule

Language syntax and standard-library APIs should follow the
[Paved Road Doctrine](PAVED_ROAD_DOCTRINE.md): one obvious safe default, with
explicit side roads only when evidence and source-visible intent justify them.

This means the language reference should prefer one clear spelling for common
programs. Alternatives need a reason that appears in diagnostics, graph facts,
profiles, or evidence.

## Rejected Or Delayed In Core

Rejected for core Hum unless a future accepted design reverses it:

- C/C++ headers
- C-style `for (init; condition; step)` loops
- implicit null
- hidden exceptions for normal failure
- fallthrough switch statements
- C-style preprocessor macros
- hidden unsafe behavior

Delayed until the formal core, graph, diagnostics, and tooling can explain them:

- user macros
- closures
- async/await surface syntax
- operator overloading
- inheritance
- dynamic dispatch by default
- compile-time reflection

## Current Commands

```powershell
hum check <file-or-dir>...
hum check --format json <file-or-dir>...
hum run <file> [--entry <task>] [--args ...]
hum graph <file-or-dir>...
hum evidence <file-or-dir>...
hum evidence --format json <file-or-dir>...
hum math-obligations <file-or-dir>...
hum math-obligations --format json <file-or-dir>...
hum math-obligations --out-dir <dir> <file-or-dir>...
hum resource-report <file-or-dir>...
hum resource-report --format json <file-or-dir>...
hum ir-readiness <file-or-dir>...
hum ir-readiness --format json <file-or-dir>...
hum core-preview <file-or-dir>...
hum core-preview --format json <file-or-dir>...
hum core-lower <file-or-dir>...
hum core-lower --format json <file-or-dir>...
hum core-verify <file-or-dir>...
hum core-verify --format json <file-or-dir>...
hum resolve <file-or-dir>...
hum resolve --format json <file-or-dir>...
hum type-env <file-or-dir>...
hum type-env --format json <file-or-dir>...
hum type-check <file-or-dir>...
hum type-check --format json <file-or-dir>...
hum full-type-check <file-or-dir>...
hum full-type-check --format json <file-or-dir>...
hum effect-check <file-or-dir>...
hum effect-check --format json <file-or-dir>...
hum ownership-check <file-or-dir>...
hum ownership-check --format json <file-or-dir>...
hum resource-check <file-or-dir>...
hum resource-check --format json <file-or-dir>...
hum profile-check <file-or-dir>...
hum profile-check --format json <file-or-dir>...
hum core-contract
hum core-contract --format json
hum ir-contract
hum ir-contract --format json
hum backend-contract
hum backend-contract --format json
hum profiles
hum profiles --format json
hum state-model
hum state-model --format json
hum test-skeletons <file-or-dir>...
hum syntax
hum syntax --format textmate
hum capabilities
hum capabilities --format json
hum diagnostics
hum diagnostics --format json
hum doctor
hum doctor --format json
hum explain <H####>
hum explain <H####> --format json
hum lsp --capabilities
hum lsp --capabilities --format json
hum version
hum version --format json
```

Bootstrap examples:

```powershell
cargo run -- check examples
cargo run -- check --format json examples
cargo run -- run examples/core/add.hum --entry add --args 2 3
cargo run -- graph examples/reference_surface.hum
cargo run -- graph examples
cargo run -- evidence examples/reference_surface.hum
cargo run -- evidence --format json examples/reference_surface.hum
cargo run -- math-obligations examples/control_flow.hum
cargo run -- math-obligations --format json examples/control_flow.hum
cargo run -- math-obligations --out-dir target/hum-math-obligations examples/control_flow.hum
cargo run -- resource-report examples/control_flow.hum
cargo run -- resource-report --format json examples/control_flow.hum
cargo run -- ir-readiness examples/reference_surface.hum
cargo run -- ir-readiness --format json examples/reference_surface.hum
cargo run -- core-preview examples/reference_surface.hum
cargo run -- core-preview --format json examples/reference_surface.hum
cargo run -- core-lower examples/reference_surface.hum
cargo run -- core-lower --format json examples/reference_surface.hum
cargo run -- core-verify examples/reference_surface.hum
cargo run -- core-verify --format json examples/reference_surface.hum
cargo run -- resolve examples/reference_surface.hum
cargo run -- resolve --format json examples/reference_surface.hum
cargo run -- type-env examples/reference_surface.hum
cargo run -- type-env --format json examples/reference_surface.hum
cargo run -- type-check examples/reference_surface.hum
cargo run -- type-check --format json examples/reference_surface.hum
cargo run -- full-type-check fixtures/full_type_check/simple_pass.hum
cargo run -- full-type-check --format json fixtures/full_type_check/simple_pass.hum
cargo run -- effect-check fixtures/effect_check/simple_pass.hum
cargo run -- effect-check --format json fixtures/effect_check/simple_pass.hum
cargo run -- ownership-check fixtures/ownership_check/simple_pass.hum
cargo run -- ownership-check --format json fixtures/ownership_check/simple_pass.hum
cargo run -- resource-check fixtures/resource_check/simple_pass.hum
cargo run -- resource-check --format json fixtures/resource_check/simple_pass.hum
cargo run -- profile-check fixtures/profile_check/simple_pass.hum
cargo run -- profile-check --format json fixtures/profile_check/simple_pass.hum
cargo run -- core-contract
cargo run -- core-contract --format json
cargo run -- ir-contract
cargo run -- ir-contract --format json
cargo run -- backend-contract
cargo run -- backend-contract --format json
cargo run -- profiles
cargo run -- profiles --format json
cargo run -- state-model
cargo run -- state-model --format json
cargo run -- test-skeletons examples
cargo run -- syntax
cargo run -- syntax --format textmate
cargo run -- capabilities
cargo run -- capabilities --format json
cargo run -- diagnostics
cargo run -- diagnostics --format json
cargo run -- doctor
cargo run -- doctor --format json
cargo run -- explain H0201
cargo run -- explain H0201 --format json
cargo run -- lsp --capabilities
cargo run -- lsp --capabilities --format json
cargo run -- version
cargo run -- version --format json
```

## Open Reference Gaps

This reference is intentionally incomplete. The next gaps to close are:

- exact expression grammar for the first executable subset
- type grammar for records, results, options, lists, and maps
- import and visibility rules
- profile matching and adapter-backed capability grants
- formal lowering from surface constructs into Core Hum
- stable examples for every accepted syntax form beyond `examples/reference_surface.hum`
- generated editor grammar and syntax-highlight keyword list beyond the current TextMate snapshot

Until those are pinned, broad syntax expansion should stay in design docs, not
in stable Hum.
Session AM keeps the same source spelling and infers one local latent row for
the accepted application. A task with checked local `set` operations propagates
their exact `change` occurrences through the callable call. There is no row
syntax, public generalization, handler, callable storage, or callable return.

# Hum Work Order 24: Generalize One Source-Derived Native Text Program

Date: 2026-08-25
<!-- hum-active-workorder:v1 -->
Status: ISSUED AND PUBLISHED; UNIT A IMPLEMENTATION PENDING FRESH BDFL SIGNAL.

WO24 was independently accepted and issued by commit
`d2d78c71e646ec4ad75de1a655b3f43c9832a661` with parent
`d393116d74b0e64a81f0f1329113a2b27e203e7b`, tree
`4eef8c1badeec92c65f2939fe4b55da6b09e1c2d`, and subject
`docs(workorder): issue work order 24`. The exact committed topology inventory
was:

- delete `workorders/active/WORKORDER_23.md`, whose parent mode/blob was
  `100644 522366a5e5845aef7301d93cab70b1cd6efd4ab7`;
- add `workorders/active/WORKORDER_24.md` at
  `100644 2a892677f611f36326a569ea78a4ee915bf72339`; and
- add `workorders/closed/WORKORDER_23.md` at
  `100644 294ca87a3bcd822659867ad66397ea94ab9861d8`.

The issuance package accounted for exactly `+672/-16` raw and
whitespace-insensitive. It was published by exactly one successful normal,
non-force push advancing only `refs/heads/main` across the exact authorized
parent-to-child range; the remote ref count, archives, and unrelated refs
remained unchanged.

Terminal publication evidence is workflow `ci` run `32919030984`, attempt `1`,
event `push`, tested SHA
`d2d78c71e646ec4ad75de1a655b3f43c9832a661`, conclusion `success`. Ubuntu job
`98028669716` succeeded in `15m24s`; Windows job `98028669784` succeeded in
`33m49s`. Both selected exactly:

```text
mode=full
reason=no_status_transition
anchor=
run_id=0
run_attempt=0
ubuntu_job_id=0
windows_job_id=0
transitions=
```

Both platforms passed all 151 ordered, case-sensitive classifier cases twice
deterministically; the required Ubuntu `pwsh` and Windows PS5.1/`pwsh` capture
matrices; formatting; Cargo checks; warnings-denied Clippy; compile-fail
boundaries; and root and subsidiary Rust suites. The published implementation
ledger remained exactly 112 ordered selectors. B01-B15, both B12 enforcement
sites, M01-M13, `integer_sign` interpreter/native parity and authority
evidence, exactly six backend probes, and ordered readiness evidence with
`ir_ready=1` and `backend_ready=1` all passed. Text hygiene and public
readiness passed for 559 files; alpha claims and release readiness `0.0.1`
passed.
Exactly one terminal full-preflight success marker appeared per platform.

Ubuntu Exhaustive passed F1 `630`, F2 `4,950`, and F3/F4 `8,646`, totaling all
`14,226` pairs with seed `0x48554D5F5345414C`. Windows skipped only the
duplicate Exhaustive producer. Both platforms correctly skipped status-only
evidence under full mode.

This issuance and its CI implement no WO24 program or compiler obligation and
earn no WO24 implementation credit. `programs/hello_world.hum`, backend-input
v2, H0635, selectors 113-118, and N01-N08 remain unimplemented. Unit A remains
unauthorized pending acceptance, local commit, publication through required
fast CI, and a fresh explicit BDFL implementation signal.

Owner: BDFL (Ocean).

## Objective

Work Order 24 proposes Hum's second canonical user-facing native program:
`programs/hello_world.hum`. The program writes exactly the source Text literal
`Hello, world!` through the existing explicit `stdout.write` authority and
bounded output adapter, with exact interpreter/native parity on required
Windows and Linux configurations.

The point is not to add a demonstration or another filename exception. WO24
must establish one reusable native-program feature-discrimination seam and one
sealed constant-Text-output feature. The accepted source must cross the real
parse, resolver, type, full-type, effect, ownership, resource, profile, Core,
backend-input, IR-verifier, Cranelift, runner, and output boundaries. Changing
the source literal must change the canonical artifact and observable native
output without changing a Rust output table.

WO24 has one implementation unit because the discriminator, verified Text
fact, additive artifact, backend consumer, runner integration, stable refusal
ownership, and end-to-end evidence are not independently user-complete. The
unit may begin only after this package is independently accepted, committed,
published through terminal-green full CI, status-recorded if required, and
followed by a fresh explicit BDFL implementation signal.

If implementation inspection shows that `hello_world` can be delivered only
by matching its filename, module, app, path, literal bytes, or a Rust-side
answer table, implementation must stop and return for architectural amendment.
That outcome earns no partial implementation credit.

## Issuance baseline and predecessor preservation

This successor package is authored from published terminal WO23 state:

- branch, `HEAD`, local `main`, cached `origin/main`, and live `origin/main`:
  `d393116d74b0e64a81f0f1329113a2b27e203e7b`;
- ahead/behind: `0/0`;
- published active WO23 mode/blob:
  `100644 522366a5e5845aef7301d93cab70b1cd6efd4ab7`;
- worktree and index clean, with no untracked artifact;
- canonical topology: one active, fourteen closed, zero root Work Orders;
- sole standalone active marker: active WO23 line 4; and
- eight stashes, archives, remote refs, unrelated refs, credentials,
  configuration, and environment unchanged.

The combined package performs exactly these topology changes:

1. update only WO23's mutable Status body and current authorization gate with
   its terminal closeout and published status-CI evidence;
2. remove exactly WO23's standalone marker line;
3. move the resulting record to
   `workorders/closed/WORKORDER_23.md`; and
4. create this file with the sole standalone active marker at line 4.

Every WO23 byte outside the mutable Status/current-gate regions and removed
marker must remain identical to the published predecessor. Reconstructing the
published mutable regions and reinserting the marker must reproduce blob
`522366a5e5845aef7301d93cab70b1cd6efd4ab7` exactly.

WO23's final status workflow is `ci` run `32911725130`, attempt `1`, event
`push`, tested SHA `d393116d74b0e64a81f0f1329113a2b27e203e7b`, and concluded
`success`. Ubuntu job `98007090418` and Windows job `98007090526` both
succeeded. Both reproduced the exact fast binding to full anchor run
`32891079181`, passed all 151 ordered classifier cases twice
deterministically, status-only evidence, 558-file hygiene/readiness, alpha
claims, and release readiness `0.0.1`, while correctly skipping Cargo
preparation, full preflight, and Exhaustive.

## Repository-grounded selection

The selected program is architecturally useful because the published native
path is currently feature-specific in exactly the places WO24 must improve:

- `src/main.rs` sends every accepted native layout to
  `run_native_integer_sign`;
- `src/type_check.rs` owns an integer-sign-only typed authority;
- the full-type/effect/ownership/resource/profile wrappers carry only that
  feature;
- `src/backend_input.rs` and `src/ir_verify.rs` own an integer-sign-only v1
  artifact and capability;
- `src/backend_cranelift.rs` lowers only the integer-sign branch/tag shape; and
- the exact selector and mutation ledger proves only minimal-add and
  integer-sign backend consumers.

The canonical layout validator in `src/app_entry.rs` is already generic over
normalized `programs/<name>.hum`, `module programs.<name>`, `app <name>`, the
final app, and its first direct-child start task. WO24 must reuse it unchanged.
No new layout rule, path exception, parser representation, or H0634 meaning is
authorized.

The interpreter already owns exact Text literals, `stdout_write(Text)`, typed
`OutputError`, explicit source closure, operator consent, exact no-newline
UTF-8 output, and the bounded output adapter. WO24 reuses those semantics. It
adds no string library, formatting, concatenation, interpolation, allocation
claim, or general Text lowering.

## Frozen representative program

The canonical source shape is:

```hum
module programs.hello_world

app hello_world {
  why:
    prove one source-derived constant Text output through native execution

  uses:
    stdout.write

  starts with:
    run_tool

  task run_tool -> Result Unit, OutputError {
    why:
      write one checked source literal without hidden fallback

    uses:
      stdout.write

    fails when:
      bounded output is denied or the output adapter rejects the write

    allocates:
      callee-defined allocation behavior

    does:
      let written = try stdout_write("Hello, world!")
      return written
  }
}
```

The supported feature has exactly one zero-argument structural start task,
result `Result Unit, OutputError`, one recognized direct Text-literal
`stdout_write` call through explicit `try`, one return of that binding, exact
source/app/task `stdout.write` closure, and truthful callee-defined allocation
intent. Comments and blank lines are irrelevant; semantic reordering,
additional executable statements, a nonliteral output expression, a second
write, a helper call, an input parameter, another result root, or missing
authority is not the feature.

The canonical invocation is:

```text
hum run --native --allow stdout.write programs/hello_world.hum
```

Success writes exactly 13 UTF-8 bytes, `Hello, world!`, with no newline,
empty stderr, and exit zero. The same invocation without `--native` is the
interpreter oracle. `--args` is empty for both routes.

## Program-specific and reusable ownership

| Class | Permitted ownership |
| --- | --- |
| Program-specific | `programs/hello_world.hum`, its dedicated unsupported-shape fixtures, and the canonical v2 golden may name `hello_world` or freeze `Hello, world!`; no production selector may consume those names or bytes as authority |
| Reusable compiler path | typed constant-Text-output recognition, sealed feature discrimination, H0635 refusal ownership, canonical v2 encoding, live verification, native tag/store lowering, and no-fallback dispatch must be expressed in program-neutral facts |
| Existing runtime primitive | `stdout_write(Text)`, source/operator authority intersection, the 1 MiB bound, typed `OutputError`, and the injectable output adapter are reused unchanged; WO24 adds no competing output operation |
| Future standard library | Text construction, formatting, console APIs, writers, buffering, and reusable greeting helpers remain candidates only after multiple later programs prove stable reuse; WO24 promotes none of them |
| Future Nectar package | no package identity, manifest, dependency, import, publication, or package-level API exists or is implied by this program |

Production code may retain a feature ID such as
`canonical_constant_text_output_app_v0` only after typed recognition. It may
not retain `hello_world`, the canonical path/module/app name, or the greeting
literal as a dispatch key or output source.

## Living intent-enforcement matrix

`docs/LANGUAGE_REFERENCE.md` must gain one living matrix with exactly these
columns in this order:

| Intent | Parsed/retained | Machine-readable fact | Statically checked | Runtime enforced | Formally verified | Current limitation | Evidence owner |
| --- | --- | --- | --- | --- | --- | --- | --- |

The initial rows must cover at least:

- normalized source/module/app identity and structural entry identity;
- the exact source Text literal and its span;
- app/task `stdout.write` closure;
- explicit operator consent and deny-first behavior;
- canonical backend-input artifact identity and live source revision;
- opaque callback-scoped verified capability;
- feature discriminator and Cranelift tag/store/invocation facts; and
- exact output bytes, no fallback, readiness, and supported target evidence.

Each cell must say an exact implemented fact, `not yet`, or a specific bounded
limitation. A prose claim cannot substitute for a machine-readable fact or an
evidence owner. Any later Work Order that changes one of these intents must
update the row or prove it unchanged. The matrix is a public truth surface for
humans and coding agents, not an implementation checklist, proof by table, or
claim of full formal verification.

## Supported and unexercised configurations

Required publication configurations are:

| Host | Target | Required evidence |
| --- | --- | --- |
| Windows | `x86_64-pc-windows-msvc` | full preflight, interpreter/native parity, exact output and refusals, Rust suites, selectors, mutations, Clippy, claims, and release checks |
| Ubuntu | `x86_64-unknown-linux-gnu` | the same evidence plus the sole Exhaustive producer |

The local implementation platform may supply focused Windows evidence, but
only terminal-green publication CI establishes both required targets. Windows
skips only duplicate Exhaustive. macOS, non-x86_64 hosts, non-MSVC Windows,
non-GNU Linux, cross-compilation, Wasm, and every other target are unexercised
and unsupported by WO24. They must reject before successful native output or
backend readiness; no fallback is permitted.

## Unit A authorization and exact path envelope

After the complete issuance lifecycle and a fresh BDFL implementation signal,
Unit A authorizes exactly these forty-one implementation paths. Deletions are
ceilings, not goals. Every path and category is non-borrowable, and the same
ceilings apply to raw and whitespace-insensitive statistics.

| Path | Max + | Max - | Purpose |
| --- | ---: | ---: | --- |
| `README.md` | 18 | 6 | second native-program claim and exact invocation |
| `docs/ARCHITECTURE.md` | 40 | 15 | reusable feature-discrimination seam and narrow Text slice |
| `docs/BACKEND_CONTRACT_SCHEMA.md` | 35 | 12 | additive verified feature and no-fallback contract |
| `docs/BACKEND_STRATEGY.md` | 28 | 10 | bounded constant-output Cranelift rung |
| `docs/CAPABILITIES_SCHEMA.md` | 25 | 8 | honest second-feature readiness surface |
| `docs/DIAGNOSTICS.md` | 35 | 12 | H0635 allocation, reasons, ownership, and precedence |
| `docs/HUM_BACKEND_INPUT_SCHEMA.md` | 50 | 18 | additive v2 constant-Text artifact contract |
| `docs/HUM_CORE_LOWER_SCHEMA.md` | 30 | 10 | source literal/output-operation lowering facts |
| `docs/HUM_CORE_VERIFY_SCHEMA.md` | 30 | 10 | exact constant-output verification facts |
| `docs/HUM_IR_CONTRACT_SCHEMA.md` | 35 | 12 | bounded second native-program feature |
| `docs/HUM_IR_READINESS_SCHEMA.md` | 30 | 10 | verified constant-output readiness claim |
| `docs/HUM_IR_VERIFY_SCHEMA.md` | 35 | 12 | v0/v1/v2 verifier and rejection boundary |
| `docs/LANGUAGE_REFERENCE.md` | 100 | 35 | canonical hello-world semantics and living intent matrix |
| `docs/TESTING_STRATEGY.md` | 40 | 15 | source-derived Text and no-special-case evidence |
| `fixtures/backend_input/hello_world.backend_input.v2.json` | 150 | 0 | canonical serialized verifier fixture |
| `fixtures/programs/hello_world/unsupported_helper_call_fail.hum` | 55 | 0 | layout-valid helper-mediated output remains outside the feature |
| `fixtures/programs/hello_world/unsupported_nonliteral_output_fail.hum` | 55 | 0 | layout-valid nonliteral Text output refusal |
| `fixtures/programs/hello_world/unsupported_two_writes_fail.hum` | 55 | 0 | layout-valid second-write refusal |
| `programs/hello_world.hum` | 50 | 0 | second canonical user-facing native program |
| `src/backend_contract.rs` | 60 | 15 | exact second feature and retained non-general claims |
| `src/backend_cranelift.rs` | 420 | 70 | verified constant tag/store lowering and one invocation |
| `src/backend_input.rs` | 420 | 70 | sealed v2 producer and canonical encoding |
| `src/capabilities.rs` | 50 | 15 | truthful feature/readiness reporting |
| `src/core_lower.rs` | 220 | 35 | source Text literal and output-operation facts |
| `src/core_verify.rs` | 200 | 35 | exact literal, call, binding, return, and span verification |
| `src/diagnostic_catalog.rs` | 100 | 25 | H0635 code/cause allocation and public catalog order |
| `src/diagnostics.rs` | 8 | 6 | complete 90-code text/JSON catalog projection evidence |
| `src/effect_check.rs` | 90 | 15 | sealed stdout-effect lineage wrapper |
| `src/full_type_check.rs` | 110 | 20 | sealed constant-Text type lineage wrapper |
| `src/ir_contract.rs` | 50 | 15 | current native features and exact non-goals |
| `src/ir_readiness.rs` | 130 | 25 | live verified-program readiness evidence |
| `src/ir_verify.rs` | 280 | 45 | v2 artifact validation and typed capability issuance |
| `src/main.rs` | 180 | 35 | sealed native-feature dispatch and exact CLI evidence |
| `src/native_program.rs` | 380 | 0 | new sealed typed feature discriminator and H0635 producer |
| `src/ownership_check.rs` | 80 | 15 | sealed ownership lineage wrapper |
| `src/profile_check.rs` | 90 | 15 | final profile lineage and live identity |
| `src/resource_check.rs` | 80 | 15 | sealed resource lineage wrapper |
| `src/run.rs` | 240 | 40 | zero-argument native route, consent, and output bridge |
| `src/type_check.rs` | 260 | 45 | source-derived constant-Text type authority producer |
| `src/version.rs` | 20 | 6 | honest milestone/public non-claims |
| `tools/check_all.ps1` | 480 | 120 | exact selectors, mutations, corpus, and readiness evidence |
| **Unit A total** | **4,844** | **867** | **no path borrowing** |

Non-borrowable category ceilings are:

| Category | Paths | Max + | Max - |
| --- | ---: | ---: | ---: |
| Production Rust, including colocated tests | 21 | 3,468 | 562 |
| Canonical program and permanent fixtures | 5 | 365 | 0 |
| Documentation and schemas | 14 | 531 | 185 |
| PowerShell integration and mutation proof | 1 | 480 | 120 |
| **Unit A category total** | **41** | **4,844** | **867** |

Path arithmetic is `21 + 5 + 14 + 1 = 41`. No category or path may borrow
from another. No Cargo manifest, lockfile, dependency, workflow, parser, AST,
app-entry, diagnostic-renderer, unsafe-policy, output-adapter, grant-policy,
capture, classifier, package, stdlib, Nectar, macOS, LLVM, release, or tag path
is authorized. Every unlisted path is excluded.

## Production integration map

The implementation must preserve this auditable chain:

| Boundary | Required result |
| --- | --- |
| generic layout | unchanged H0634 analysis authenticates one normalized source/module/app identity, final app, first direct-child entry, and existing authority precedence without recognizing a feature |
| typed feature facts | exact live type/Core facts recognize integer-sign v0 or constant-Text-output v0; path, module, app, entry spelling, and literal bytes are unavailable as feature selectors |
| discriminator | `src/native_program.rs` issues exactly one sealed variant from disjoint typed facts; zero recognized variants is H0635 unsupported and more than one is H0635 ambiguous |
| stage lineage | full type, effect, ownership, resource, and profile wrappers carry the same Program, layout, item, statement, diagnostics, and source identities to the producer |
| backend input | deterministic additive `hum.backend_input.v2` bytes bind the constant Text literal/span, output call/binding/return, authority closure, required passes, feature ID, source revision, and target-independent semantics |
| IR verification | strict canonical decoding and live-fact equality issue one callback-scoped opaque v2 capability; bytes, fixture, report, digest, path, and name grant no authority |
| Cranelift | typed getters alone create one zero-input semantic tag, one initialized result-slot store, one verified function, and one finalized invocation through the existing uniform ABI |
| runner | exact feature dispatch reuses deny-first operator consent and the bounded output adapter; successful native execution selects only the verified source literal |
| evidence | artifact, target, invocation, output, refusal, `ir_ready=1`, and `backend_ready=1` facts are observable without exposing capability construction or JIT pointers |

The discriminator is a closed compiler-internal enum, not a registry, plugin,
callback framework, filename map, string dispatch table, or public API. Adding
a future feature requires another accepted Work Order and a new sealed fact
producer; unrecognized programs fail closed.

## Feature admission and stable refusal ownership

Allocate exactly one new active public code:

```text
H0635 unsupported native program feature
```

It belongs to the front-end semantics family, semantic owner
`native_program`, and owning stage `native_admission`. It has exactly two
producer-owned causes, in order:

1. `native_feature_not_supported_v0`; and
2. `native_feature_ambiguous_v0`.

The active public catalog advances mechanically from 89 to 90 codes and the
cause registry from 180 to 182 entries. Text begins `Hum diagnostics (90
codes)` and JSON reports `"count": 90`; every active entry appears exactly
once. No runtime sorting, filtering, compatibility alias, hidden omission, or
derived-count weakening earns credit.

H0635 runs only after existing parser/checker/app-authority errors and H0634
layout admission are green. It runs before backend-input production, IR
capability issuance, JIT, output, or readiness. Missing source capability stays
H0621; malformed `stdout_write` and typed-failure forms retain their existing
owners; allocation/profile failures retain their existing stages. H0635 does
not make an interpreter-valid program invalid Hum.

After typed feature admission, backend and runner refusals use closed internal
reason IDs owned by their existing stages: artifact framing/live mismatch,
unsupported target, unavailable ISA, verification/finalization failure,
invalid status/tag or unchanged result slot, deny-first output, output limit,
and adapter rejection. Each real refusal has one exact public disposition and
zero fallback. Generic `failed`, a panic, or another stage's message cannot
earn its evidence credit.

## Backend-input v2 and verified authority

V0 minimal-add and v1 integer-sign bytes and meaning remain byte-identical.
WO24 adds only `hum.backend_input.v2` for feature
`canonical_constant_text_output_app_v0`.

The v2 payload binds exactly:

- schema, compiler version, semantic contract, feature, and target context;
- SHA-256 source revision and normalized source identity;
- module, app, direct-child entry, zero arguments, and result/error roots;
- the sole Text literal bytes and source span;
- the exact `stdout_write` call, `written` binding, direct `try`, and return;
- app/task source-authority closure and operator-consent prerequisite;
- final profile and live Program identity;
- ordered required-pass identities and exact selected counts;
- target-independent one-tag semantics; and
- canonical field order, encoding, payload digest, artifact identity, empty
  unsupported list, and no unknown fields.

The verifier rereads persisted fixture bytes, rejects noncanonical framing,
unknown/duplicate/reordered fields, mixed v0/v1/v2 facts, stale compiler or
source revision, literal/call/span substitution, foreign Program identity, and
wrong feature lineage. Only equality with fresh live facts may issue the
non-constructible, non-cloneable, non-serializable callback-scoped
`VerifiedConstantTextBackendInput`.

## Native lowering and observable execution

The existing single reviewed unsafe invocation boundary and uniform ABI remain
unchanged:

```text
unsafe extern "C" fn(i64, i64, *mut i64) -> i32
```

For constant-Text execution both integer inputs are authenticated reserved
zeroes. Verified facts produce one function, one source-located constant tag,
one result-slot store, and status zero. The initialized result slot must change
exactly once to the sole accepted tag. The retained JIT module owns the
non-null finalized code pointer for the complete call.

Only after successful native invocation may the runner resolve that tag to the
capability's verified source literal and call the existing output adapter once.
No raw Text pointer, host symbol, Rust literal table, program-name match,
interpreter callback, object file, subprocess compiler, second unsafe block,
fallback, or side channel is permitted.

Changing `"Hello, world!"` to another valid nonempty Text literal in an
initialized source mutation must change the v2 artifact ID and both interpreter
and native output without any Rust edit. The canonical fixture remains frozen
to exactly `Hello, world!` and 13 output bytes.

## Permanent evidence

Evidence must be persisted before execution and fail closed.

### Positive and end-to-end evidence

1. The unchanged generic layout accepts both `integer_sign` and `hello_world`
   using only their own internally consistent path/module/app identities.
2. The feature discriminator selects integer-sign from its typed signed-branch
   facts and constant-Text output from its typed zero-argument literal-write
   facts, exactly once each, with no name/path/literal lookup.
3. The canonical program passes parse, resolve, type, full type, effect,
   ownership, resource, profile, Core lower/verify, backend-input, and IR
   verification with exact live lineage.
4. The v2 artifact is deterministic, byte-identical to its golden, and useful
   only as verifier input.
5. Interpreter and native execution produce exactly `Hello, world!`, empty
   stderr, exit zero, one output adapter call, `ir_ready=1`, and
   `backend_ready=1`.
6. Cranelift evidence authenticates required target, version, CLIF shape,
   source location, verification, declaration, definition, finalization,
   non-null pointer, one store, one invocation, valid status/tag, and zero
   survivors.
7. Required Windows and Ubuntu publication jobs execute the exact committed
   program through both interpreter and native routes.

### Negative and precedence evidence

- default deny and explicit deny call neither JIT nor output adapter;
- missing app/task source authority retains its existing diagnostic before
  H0635;
- every existing H0634 layout fixture retains its exact code/reason and zero
  feature/backend/output evidence;
- each dedicated hello-world fixture is valid Hum and valid canonical layout
  but produces exactly one H0635 unsupported-feature occurrence before
  backend input;
- one argument, malformed CLI authority, a corrupt/mixed/stale v2 artifact,
  foreign live facts, unsupported target, failed JIT row, invalid status/tag,
  unchanged result slot, output overflow, and adapter failure reject at their
  exact owners with zero successful output/readiness;
- `integer_sign` remains byte-for-byte behaviorally compatible; and
- an ordinary interpreter-supported program never falls back from rejected
  `--native` execution.

Combined-cause controls repair one earlier blocker at a time and require the
next owning diagnostic or refusal. H0635 cannot mask H0634, semantic-stage, or
authority errors, and backend errors cannot mask H0635.

### Initialized mutation evidence

Extend the existing exact mutation framework without redesigning it. Each row
must alter one authenticated production predicate, run its exact selector,
fail at the intended disposition with zero compile errors, and restore bytes:

| ID | Initialized weakening | Required escaped disposition |
| --- | --- | --- |
| N01 | select a native feature from path/module/app spelling | a layout-valid foreign identity reaches the wrong feature assertion |
| N02 | bypass exact constant-Text body/type admission | one dedicated unsupported fixture reaches artifact issuance |
| N03 | weaken v2 live artifact equality | stale or foreign bytes reach the capability callback |
| N04 | replace the verified literal getter with a Rust literal | source-literal mutation no longer changes artifact/output parity |
| N05 | bypass or duplicate finalized native invocation | exact-one invocation or initialized-slot evidence fails |
| N06 | weaken deny-first operator consent | denied execution reaches JIT or output |
| N07 | route H0635/backend refusal into the interpreter | rejected native input produces forbidden fallback output |
| N08 | weaken required target admission | an unsupported target earns backend-ready evidence |

N01-N08 are separate from and preserve WO23 M01-M13 and WO22 B01-B15. No
mutation may earn credit from compilation failure, panic, unrelated assertion,
missing fixture, no-op replacement, or non-restoration.

### Exact selectors and integrated evidence

Preserve the published ordered 112-selector ledger byte-for-byte and append
exactly these six case-sensitive selectors at ordinals 113 through 118:

1. `native_program::tests::native_feature_discrimination_is_typed_and_load_bearing`
2. `backend_input::tests::canonical_hello_world_backend_input_is_exact_and_nonforgeable`
3. `ir_verify::tests::hello_world_artifact_rejection_matrix_is_complete`
4. `backend_cranelift::tests::hello_world_lowering_is_source_driven_and_load_bearing`
5. `diagnostic_catalog::tests::unsupported_native_feature_catalog_projection_is_exact`
6. `main::tests::native_hello_world_run_is_authority_bound_and_platform_exact`

The final ledger contains exactly 118 ordered, case-sensitive unique names,
runs each exactly once through the integrated route, and rejects deletion,
duplication, reordering, fabricated replacement, casing change, stale 112
count, or credit from broad Cargo suites.

Integrated evidence also requires formatting, all-target checking,
warnings-denied Clippy, compile-fail opacity, root and subsidiary suites,
B01-B15, M01-M13, N01-N08, the six fixed minimal-add probes, fifteen ordered
GO rows, deterministic readiness/backend projections, raw and
whitespace-insensitive accounting, all untracked no-index checks, hygiene,
public readiness, alpha claims, and release readiness `0.0.1`.

One independently adjudicated Fast is permitted only after the complete
candidate and focused evidence are green and the reviewer explicitly consumes
the allowance. Publication CI remains the Linux-GNU authority and the final
cross-platform integrated authority. Exhaustive runs only in authorized full
publication CI, once on Ubuntu.

## Public claims and explicit non-claims

After terminal-green publication, Hum may claim only:

- two canonical source-driven native program features are proven:
  integer-sign v0 and constant-Text-output v0;
- `programs/hello_world.hum` is checked from source, verified through an
  additive v2 artifact/capability, lowered with Cranelift, executed natively,
  and observed on required Windows and Linux CI;
- the exact output literal is source-derived and changes under source mutation;
- explicit output authority is deny-first and no rejected native case falls
  back to the interpreter; and
- `ir_ready=1` and `backend_ready=1` apply only to the admitted sealed feature
  on the two required targets.

WO24 does not claim arbitrary-program native compilation, a general Text
backend, general calls or control flow, string formatting/interpolation,
general application dispatch, optimization, AOT/object/linker output, debug
information, package/import support, a standard library or Nectar API, macOS,
another architecture, self-hosting, LLVM, another backend, production
readiness, release, or tag.

## Explicit deferrals

The following remain outside WO24 and receive no path, budget, or implied
authority:

- packages, imports, dependency resolution, package identity, build contracts,
  Nectar publication, and crater-like downstream testing;
- standard-library promotion of Text/output operations until repeated programs
  prove a reusable API rather than a compiler-internal primitive;
- macOS or any additional supported target;
- loops, recursion, containers, concurrency, generics, closures, async,
  networking, files, randomness, clocks, or broader IO;
- optimization, AOT/object production, linking, debug information,
  self-hosting, LLVM migration, or a second backend;
- release, tag, installer, distribution, or version change;
- a general formatter or program scaffold generator; and
- PowerShell capture, Fast adapter, classifier, workflow, or evidence-framework
  redesign absent a concrete new risk and separate amendment.

Cranelift remains the bounded, replaceable current backend. WO24 neither
promises its removal nor commits Hum to an undecided LLVM migration.

## Review, commit, publication, and status lifecycle

The package and one implementation unit follow this exact gated sequence:

1. fresh independent pre-issuance review of the complete WO23 closeout and
   WO24 planning package;
2. only unqualified ACCEPT may recommend a separately authorized local commit
   with exact subject `docs(workorder): issue work order 24`;
3. BDFL separately authorizes one normal non-force push and terminal full
   Ubuntu/Windows CI for the issuance tree;
4. a routine status-only issuance record and its fast publication occur only
   if separately required and authorized;
5. a fresh BDFL signal authorizes Unit A implementation;
6. the implementer leaves one complete forty-one-path candidate unstaged with
   empty index and no artifact after proportional focused evidence;
7. a fresh independent reviewer reviews the complete producer/verifier/backend
   chain and may adjudicate the sole Fast allowance only when prerequisites are
   green;
8. only unqualified ACCEPT and terminal-green Fast may recommend the exact
   implementation commit subject
   `feat(program): run canonical hello world natively`;
9. BDFL separately authorizes publication and terminal full Ubuntu/Windows CI;
10. mandatory post-hoc review follows any bypassed or emergency repair;
11. BDFL separately authorizes a WO24 status record and its fast publication;
    and
12. closeout, successor planning, or later work requires a fresh BDFL signal.

Any unexpected path, budget breach, semantic ambiguity, feature special case,
fallback, forged authority, mutation weakness, platform mismatch, red Fast/CI,
evidence loss, or state drift stops at its actual boundary and grants no retry,
repair, amendment, commit, push, or later authority.

## Planning-package validation

Authorship and independent pre-issuance review run only proportional planning
evidence:

- `git diff --check` and an independently parsed whitespace-insensitive diff
  check;
- exact changed-region and immutable-projection authentication for closed
  WO23;
- reconstruction of published WO23 after restoring its authorized mutable
  regions and standalone marker;
- fail-closed raw and whitespace-insensitive no-index checks for both untracked
  topology endpoints;
- exact-one-marker, line-4, regular-file, UTF-8/no-BOM/LF/final-LF, and
  canonical one-active/fifteen-closed/zero-root topology checks;
- exact 41-path uniqueness, category membership, per-row sums, aggregate
  `4,844/-867`, and non-borrowable arithmetic;
- exact H0635 allocation/cause/count, v2 boundary, six-selector appendix,
  N01-N08 ownership, exclusions, supported configurations, living-matrix
  columns, and ordered gate anchors;
- the complete 151-case classifier suite twice deterministically as planning
  evidence, including canonical successor issuance;
- text hygiene and public readiness for the resulting inventory;
- alpha claims; and
- release readiness `0.0.1`.

No Cargo, Rust selector, compiler command, interpreter/native execution,
backend probe, JIT, Fast, full preflight, Exhaustive, production-classifier
prediction, CI run, archive code, or stash operation is authorized during
planning authorship or review.

## Current authorization gate

WO24 is issued, published, and terminal-green. This status candidate records
that issuance lifecycle and grants no implementation authority.

The sole next action is fresh independent review of this exact WO24
issuance-publication status candidate. Only an unqualified `ACCEPT` may
recommend, but does not execute, a separately authorized local documentation
commit with the frozen subject:

```text
docs(workorder): record work order 24 publication
```

Review acceptance authorizes no commit or publication by itself. Staging,
commit, push, CI, Unit A implementation, `programs/hello_world.hum`,
backend-input v2, H0635, selectors 113-118, N01-N08, Fast, native execution,
another program, closeout, successor work, packages, stdlib/Nectar, macOS,
crater-like testing, optimization, AOT, another backend, LLVM, release/tag
work, stash/archive operations, and every later action remain separately
unauthorized. Unit A may begin only after this status record is independently
accepted, separately committed and published through terminal required fast
CI, and followed by a fresh explicit BDFL implementation signal.

<!-- workorder-current-authorization-gate:end -->

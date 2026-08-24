# Hum Work Order 23: Run The First Canonical Hum Program Natively

Date: 2026-08-23
<!-- hum-active-workorder:v1 -->
Status: PROPOSED. This successor package authorizes independent pre-issuance
review only. It grants no implementation, commit, publication, or later-work
authority.

Owner: BDFL (Ocean).

## Objective

Work Order 23 advances Hum from the fixed `minimal_add` backend proof to one
canonical user-facing program that is checked from source, verified through the
existing authority chain, lowered with Cranelift, executed as native code, and
observed through Hum's existing bounded output adapter on required Windows and
Linux configurations.

The representative program is `programs/integer_sign.hum`. It accepts one
`Int` argument and writes exactly `negative`, `zero`, or `positive` according
to the checked source conditions. This is intentionally more than a demo and
less than a general compiler claim. It forces reusable growth in structural app
entry, checked argument binding, signed comparison, conditional control-flow
lowering, source-derived Text selection, capability admission, diagnostics,
and native/interpreter parity without importing loops, files, packages,
generic containers, optimization, or a standard library.

WO23 has one implementation unit. The verified source producer, sealed backend
input, Cranelift consumer, runner integration, and permanent evidence form one
end-to-end feature: none is independently user-complete, and splitting them
would create an intentionally unusable intermediate tree. Every implementation
session still ends with a compiling candidate and meaningful focused evidence.

## Issuance baseline and predecessor preservation

This planning package is authored from the published terminal WO22 tree:

- branch, `HEAD`, local `main`, cached `origin/main`, and live `main`:
  `4bdc50e39c254cfa630fb12316b27e62be5bb519`;
- ahead/behind: `0/0`;
- subject: `docs(workorder): close work order 22`;
- clean worktree, empty index, and no untracked files before authorship;
- closeout workflow `ci`, run `32673940961`, attempt `1`, event `push`, exact
  SHA `4bdc50e39c254cfa630fb12316b27e62be5bb519`, conclusion `success`;
- Ubuntu job `97278901712`, success; and
- Windows job `97278901574`, success.

The issuance package performs exactly these topology changes:

1. move `workorders/active/WORKORDER_22.md` to
   `workorders/closed/WORKORDER_22.md`;
2. delete exactly its sole marker line
   `<!-- hum-active-workorder:v1 -->`, preserving every other WO22 byte; and
3. create this regular `100644` UTF-8 LF-only successor with the sole marker at
   line 4.

The eight existing stashes, local and published archive refs, unrelated refs,
credentials, configuration, and environment remain preservation-only and
unchanged. They confer no WO23 authority.

## Repository-grounded selection

The planning inventory covered every tracked `.hum` example and fixture family,
the `hum run` CLI, structural app analysis, capability closure, interpreter,
core lowering and verification, final profile lineage, backend-input verifier,
Cranelift adapter, public architecture, language reference, paved-road and
stdlib doctrine, and accepted decisions 0001-0004, 0007-0018, and 0020.

The alternatives were rejected for concrete scope reasons:

- a file processor would require portable file authority and Text/list/runtime
  work that Hum does not yet support on Linux;
- word count, task-list processing, Fibonacci, or GCD would pull in loops and
  WO20's still-unimplemented termination-measure design;
- `hello world` would exercise output but add almost no compiler semantics;
- another direct `--entry` probe would not establish canonical structural app
  identity; and
- another fixed arithmetic answer would merely rename the WO22 proof.

`integer_sign` is the smallest recognizable program that makes a real native
control decision from user input, exercises an already-decided authority
boundary, and produces observable source-derived output on both required
platforms. The three-way result makes constant-answer, swapped-branch, and
interpreter-fallback defects independently falsifiable.

## Canonical program-file framework

Until imports and package manifests are separately designed, canonical native
programs use this predictable single-file layout:

```text
programs/<snake_case_name>.hum
fixtures/programs/<snake_case_name>/...
```

Canonical-program V0 is a mechanically enforced identity and semantic order:

1. the native CLI source spelling is one repository-relative path whose `/`
   and `\` separators normalize to `/` and whose normalized form has exactly
   two components: `programs` and `<snake_case_name>.hum`;
2. the sole module is exactly `module programs.<snake_case_name>` and the
   module suffix, filename stem, and sole app name are the same exact identity;
3. exactly one module declaration exists;
4. that module is the first top-level semantic item;
5. zero or more program-local `type` declarations may follow the module;
6. before the app, `Store`, `Task`, `Test`, and every unknown or newly added
   top-level item kind are forbidden;
7. no type declaration may appear after the app begins;
8. exactly one top-level `app` follows the module and types;
9. the app is the final top-level semantic item;
10. the task named by the app's one valid `starts with:` section is its first
    direct-child task;
11. zero or more direct-child helper tasks may follow the entry task in lexical
    source order;
12. helpers are not required to be sorted by call graph; and
13. canonical-program V0 defines no top-level evidence-item section.

The path predicate is lexical and bounded. Empty components, repeated
separators, `.`, `..`, absolute paths, drive roots, UNC paths, alternate
directory case, filename case changes, alternate extensions, and stems outside
Hum's exact snake_case identifier rule reject. Comparison is ordinal and
case-sensitive after separator normalization. The validator never resolves the
path against the host filesystem and never emits a host absolute path.

Comments and blank lines are not semantic items, do not change that order, and
must not make the retained ordering ambiguous. Source-level evidence placement
remains deferred until Hum has a real evidence form with semantic value.

The parser retains every top-level module occurrence with its name and span in
addition to the existing ordered item vectors and spans. A native-only
canonical-layout validator in `src/app_entry.rs` consumes the already checked
CLI source spelling supplied by the runner, the retained source path, module
occurrences, ordered item kinds and spans, and the accepted app identity. It
runs only after the existing parse, resolution, structural app-entry,
app-authority, and required semantic-stage producers are green. It validates
the generic path/module/app relationship, module cardinality and position, the
closed allowed-item sequence, app cardinality and finality, and direct-child
entry position. Its Rust match over `Item` is exhaustive with no wildcard that
could silently admit a future variant. It neither reparses text nor
special-cases `integer_sign` or source bytes. Ordinary checking and interpreter
execution continue to use the existing legacy rules unless `--native` requests
canonical-program admission. No formatter or generalized path abstraction is
required or authorized.

Authority belongs only in the existing `uses:` sections on the app and each
reachable task. Operator consent remains a runner fact, never source authority.
Program-specific types, literals, branching policy, and helper tasks remain in
the program file. Reusable compiler semantics remain in `src/`; permanent
rejection corpus belongs under the matching fixture directory; harness
orchestration remains in `tools/check_all.ps1`.

This framework is deliberately not a package format. A first consumer never
creates a standard-library or Nectar API. A primitive may be proposed for an
experimental package only after two unrelated programs demonstrate the same
need, and standard-library promotion requires several representative programs
plus an explicit compatibility and ownership review.

## Frozen representative program

Unit A creates exactly this semantic source at
`programs/integer_sign.hum`; formatting-only differences are permitted, but
comments and blank lines may not alter semantic ordering. The exact-one module,
module-first position, no-type-after-app order, sole final app, first direct
entry task, authority, entry signature, three conditions/outcomes, and typed
output failure contract are load-bearing:

```hum
module programs.integer_sign

app integer_sign {
  why:
    classify one checked integer through the first source-driven native app

  uses:
    stdout.write

  starts with:
    run_tool

  task run_tool(value: Int) -> Result Unit, OutputError {
    why:
      write the source-selected sign without hidden fallback

    uses:
      stdout.write

    fails when:
      bounded output is denied or the output adapter rejects the write

    allocates:
      callee-defined allocation behavior

    does:
      if value < 0 {
        let written = try stdout_write("negative")
        return written
      }
      if value == 0 {
        let written = try stdout_write("zero")
        return written
      }
      let written = try stdout_write("positive")
      return written
  }
}
```

The source introduces no new builtin, library type, package, implicit output,
or app-state model. It reuses `Int`, `Text`, `Result`, `OutputError`, signed
comparison, structural app entry, `stdout.write`, and the existing bounded
adapter. The chosen literal is read from authenticated source facts; neither
the host runner nor backend may contain a path/name-to-answer table.

## Supported configurations

Required GO build and execution configurations remain exactly:

- `x86_64-pc-windows-msvc` on the Windows publication job; and
- `x86_64-unknown-linux-gnu` on the Ubuntu publication job.

Both must execute negative, zero, and positive cases through the real native
path. Local Windows evidence is useful but cannot substitute for hosted Linux
execution. macOS is an explicit future supported-platform addition, not an
implied claim. Other Rust targets and hosts may remain unexercised when Hum or
its JIT dependencies do not compile or run there. A runnable non-required host
or unavailable/rejected required-host ISA must fail closed before JIT and must
never report GO, set backend readiness, invoke the interpreter, or select an
alternate backend.

## Unit A authorization and exact path envelope

Unit A may begin only after this issuance package is independently accepted,
committed, published, terminal-green on the full lane, status-recorded if
required by the classifier lifecycle, and followed by a fresh explicit BDFL
implementation signal.

It authorizes exactly these fifty-four implementation paths. A missing path is
a stop and amendment request, not permission to expand. Deletions are ceilings,
not goals. Every path and category is non-borrowable, and the same ceilings
apply to raw and `git diff -w` statistics.

| Path | Max + | Max - | Purpose |
| --- | ---: | ---: | --- |
| `README.md` | 18 | 6 | narrow first-program claim and invocation |
| `docs/ARCHITECTURE.md` | 38 | 12 | source-to-native app path and boundary |
| `docs/BACKEND_CONTRACT_SCHEMA.md` | 36 | 12 | second verified feature and non-fallback contract |
| `docs/BACKEND_STRATEGY.md` | 28 | 10 | bounded Cranelift program slice |
| `docs/CAPABILITIES_SCHEMA.md` | 24 | 8 | honest program readiness surface |
| `docs/DIAGNOSTICS.md` | 34 | 12 | stable canonical-layout diagnostic and precedence |
| `docs/HUM_BACKEND_INPUT_SCHEMA.md` | 45 | 15 | additive v1 integer-sign artifact contract |
| `docs/HUM_CORE_LOWER_SCHEMA.md` | 35 | 12 | conditional and literal-selection lowering facts |
| `docs/HUM_CORE_VERIFY_SCHEMA.md` | 35 | 12 | exact control-flow verification facts |
| `docs/HUM_IR_CONTRACT_SCHEMA.md` | 35 | 12 | bounded integer-sign IR feature |
| `docs/HUM_IR_READINESS_SCHEMA.md` | 30 | 10 | verified program readiness claim |
| `docs/HUM_IR_VERIFY_SCHEMA.md` | 40 | 14 | v0/v1 verifier and rejection boundary |
| `docs/LANGUAGE_REFERENCE.md` | 70 | 20 | canonical program layout and `run --native` semantics |
| `docs/TESTING_STRATEGY.md` | 30 | 10 | program corpus and native parity discipline |
| `fixtures/backend_input/integer_sign.backend_input.v1.json` | 180 | 0 | canonical serialized verifier fixture |
| `fixtures/programs/integer_sign/duplicate_app_fail.hum` | 70 | 0 | existing multiple-app precedence on canonical-shaped input |
| `fixtures/programs/integer_sign/duplicate_module_fail.hum` | 45 | 0 | exact-one-module rejection |
| `fixtures/programs/integer_sign/helper_before_start_fail.hum` | 75 | 0 | called helper still cannot precede entry lexically |
| `fixtures/programs/integer_sign/illegal_pre_app_store_fail.hum` | 55 | 0 | closed-set rejection of a top-level store before the app |
| `fixtures/programs/integer_sign/illegal_pre_app_task_fail.hum` | 55 | 0 | closed-set rejection of a top-level task before the app |
| `fixtures/programs/integer_sign/illegal_pre_app_test_fail.hum` | 55 | 0 | closed-set rejection of a top-level test before the app |
| `fixtures/programs/integer_sign/late_module_fail.hum` | 45 | 0 | module-first rejection after a semantic item |
| `fixtures/programs/integer_sign/layout_valid_pass.hum` | 90 | 0 | module, types, app, entry, and helpers positive order |
| `fixtures/programs/integer_sign/missing_app_fail.hum` | 35 | 0 | exact-one-app rejection |
| `fixtures/programs/integer_sign/missing_module_fail.hum` | 35 | 0 | exact-one-module rejection |
| `fixtures/programs/integer_sign/module_path_identity_fail.hum` | 55 | 0 | real source mismatch against the supplied logical native path |
| `fixtures/programs/integer_sign/semantic_after_app_fail.hum` | 60 | 0 | app-finality rejection for a later semantic item |
| `fixtures/programs/integer_sign/start_not_first_fail.hum` | 65 | 0 | named entry is not the first direct-child task |
| `fixtures/programs/integer_sign/type_after_app_fail.hum` | 55 | 0 | type/app ordering rejection |
| `fixtures/programs/integer_sign/unsupported_shape_fail.hum` | 60 | 0 | layout-valid backend-admission rejection |
| `programs/integer_sign.hum` | 70 | 0 | first canonical user-facing program |
| `src/app_entry.rs` | 480 | 80 | native-only identity/layout validator and focused evidence |
| `src/ast.rs` | 100 | 20 | module-occurrence name/span representation |
| `src/backend_contract.rs` | 60 | 15 | bounded feature and target contract |
| `src/backend_cranelift.rs` | 520 | 80 | source-driven conditional lowering and execution |
| `src/backend_input.rs` | 480 | 70 | sealed v1 producer and canonical encoding |
| `src/capabilities.rs` | 50 | 15 | truthful feature/readiness reporting |
| `src/core_lower.rs` | 260 | 40 | checked comparisons, blocks, and output-selection facts |
| `src/core_verify.rs` | 260 | 40 | exact conditional/source provenance verification |
| `src/diagnostic_catalog.rs` | 100 | 25 | H0634 allocation and producer-owned layout reasons |
| `src/effect_check.rs` | 120 | 20 | sealed stdout-effect lineage wrapper |
| `src/full_type_check.rs` | 140 | 25 | sealed integer-sign type lineage wrapper |
| `src/ir_contract.rs` | 60 | 15 | current native-program feature/non-goals |
| `src/ir_readiness.rs` | 180 | 30 | live verified-program readiness evidence |
| `src/ir_verify.rs` | 300 | 45 | v1 artifact validation and typed capability issuance |
| `src/main.rs` | 260 | 45 | exact `hum run --native` CLI and diagnostics |
| `src/ownership_check.rs` | 110 | 20 | sealed ownership lineage wrapper |
| `src/parser.rs` | 180 | 35 | retain every module occurrence and span |
| `src/profile_check.rs` | 120 | 20 | final profile lineage and live identity |
| `src/resource_check.rs` | 110 | 20 | sealed resource lineage wrapper |
| `src/run.rs` | 280 | 45 | checked arguments, consent, and output adapter bridge |
| `src/type_check.rs` | 300 | 45 | canonical integer-sign type authority producer |
| `src/version.rs` | 24 | 8 | honest milestone/public non-claims |
| `tools/check_all.ps1` | 620 | 145 | exact selectors, mutations, corpus, and readiness evidence |
| **Unit A total** | **6,717** | **1,068** | **no path borrowing** |

Non-borrowable category ceilings are:

| Category | Max + | Max - |
| --- | ---: | ---: |
| Production Rust, including colocated Rust tests | 4,494 | 758 |
| Canonical program and permanent fixtures | 1,105 | 0 |
| Documentation and schemas | 498 | 165 |
| PowerShell integration and mutation proof | 620 | 145 |
| **Unit A category total** | **6,717** | **1,068** |

The complete correction adds exactly `+860/-160` production Rust, `+795/-0`
fixtures, `+34/-12` documentation, and `+120/-25` PowerShell to the original
thirty-five-path envelope. Thus `4,908 + 1,809 = 6,717` insertions and
`871 + 197 = 1,068` deletions, with no cross-category borrowing.

No dependency-manifest or lockfile path is authorized. The five direct
Cranelift pins and locked transitive graph remain exact. Path arithmetic is
`35 + 19 = 54`: the additions are four production Rust owners, one diagnostic
document, and fourteen dedicated layout fixtures; no prior path was removed. No
unsafe-policy, workflow, capture, classifier, capability-root,
operator-grant, `src/syntax.rs`, `src/diagnostic.rs`, `src/diagnostics.rs`,
diagnostic-schema, interpreter grammar, package, stdlib, or Nectar path may
change. Every path not listed above remains unauthorized.

## Production integration map

The implementation must preserve one auditable producer-to-consumer chain:

| Boundary | Required result |
| --- | --- |
| parser/AST | parser-owned module occurrences retain exact names and spans without replacing the legacy module projection; existing item vectors and spans remain the ordered top-level and direct-child authority |
| canonical layout | after existing source, name, app-entry, app-authority, and semantic-stage gates, the native-only validator accepts only the bounded normalized path whose stem equals the module suffix and app name, then exactly module, zero-or-more types, final app, first entry task, and later helpers in lexical order; exhaustive item-kind admission rejects H0634 before backend input, JIT, output, or readiness |
| structural source | one layout-valid `integer_sign` app, one direct start, one `Int` argument, truthful callee-defined allocation intent, three exact literal-output branches, and complete `stdout.write` source closure |
| type/core | existing parsing and app facts feed new sealed integer-sign type authority; core lowering and verification authenticate signed `< 0`, `== 0`, fallthrough, exact branch order, exact literals, and source spans |
| effect/ownership/resource/profile | sibling non-forgeable wrappers carry the same exact item, app, statements, diagnostics, and program identity through every already-required stage |
| backend input | a deterministic `hum.backend_input.v1` artifact encodes only the verified integer-sign feature, exact source revision, app/entry identity, argument ABI, CFG, literal table, effect route, output authority, required passes, target context, and provenance |
| IR verification | the public verifier accepts unchanged minimal-add v0 plus exact integer-sign v1, rejects mixed/foreign facts, and issues a callback-scoped `VerifiedIntegerSignBackendInput` that cannot escape, clone, copy, construct, deserialize, or be replaced by bytes |
| Cranelift | typed getters alone build signed compare/branch blocks and one result-tag store; the interpreter, raw AST, artifact bytes, file path, and program name are unavailable as lowering authority |
| runner | `hum run --native` reuses checked app selection, canonical-layout admission, argument binding, capability closure, operator consent, and the existing bounded output adapter; it writes only the verified source literal selected by successful native execution |
| public evidence | exact output, exit, no-fallback, readiness, target, and provenance facts are observable without exposing typed authority or JIT pointers |

The wrapper chain is feature-specific, not a new generic callback framework.
Existing minimal-add wrappers and evidence remain valid. Shared code may be
factored only when it reduces duplication without widening construction or
authority visibility.

The representation is intentionally minimal. `src/ast.rs` adds only the
parser-issued module occurrence facts that current `SourceFile.module` loses;
the existing `SourceFile.path`, top-level and app-child vectors, item variants,
and spans remain the path, order, kind, and blame authority. `src/syntax.rs`,
the generic diagnostic renderer, the interpreter grammar, and any formatter
remain unchanged. Every V0 predicate is a bounded lexical identity,
cardinality, exhaustive item-kind, relative-order, or first-task question over
those retained facts.

## Backend-input and verifier contract

`hum.backend_input.v0` and its canonical minimal-add fixture remain accepted
and byte-identical. WO23 adds an additive `hum.backend_input.v1` integer-sign
variant; it does not reinterpret v0.

The v1 artifact must bind at least:

- schema and compiler version;
- feature `canonical_integer_sign_app_v0`;
- complete source revision and normalized source identity;
- module, app, direct-child entry, argument, result, and OutputError roots;
- one `Int` input and the existing uniform backend ABI;
- three ordered control-flow outcomes: `< 0`, `== 0`, and fallthrough;
- three exact source Text literals and their source spans;
- exact `stdout.write` app/task closure and checked operator-consent
  prerequisite;
- all required analysis-stage identities and selected counts;
- final profile lineage and live Program identity;
- target-independent signed-i64 semantics; and
- canonical encoding, payload digest, artifact identity, and no unknown fields.

The verifier must reread persisted bytes, validate canonical encoding, bind the
artifact to the live compiler-produced facts, and only then issue the opaque
typed capability. Artifact validity alone never authorizes JIT or output.
Corruptions, reordered fields, duplicate fields, alternate casing, mixed v0/v1
facts, source/literal/CFG substitution, foreign lineage, wrong compiler
version, and stale live identity fail closed before backend entry.

## Native lowering and observable execution

The only new public surface is the `--native` flag on `hum run`:

```text
hum run --native --allow stdout.write programs/integer_sign.hum --args -7
```

It requires exactly one source file, successful existing source/name/app and
authority analysis, canonical V0 layout admission, no `--entry`, one parseable
`Int` argument, complete source authority, and explicit operator consent.
`--native` on another command, missing/extra arguments, an invalid integer,
direct-entry mode, source errors, malformed canonical layout, unsupported
program shape, denied or missing output consent, unsupported target,
unavailable ISA, failed verifier, or failed backend row returns a stable
nonzero disposition before observable output. Deny wins. No case falls back to
the interpreter.

The JIT keeps the single reviewed WO22 unsafe invocation boundary. The uniform
ABI remains exactly:

```text
unsafe extern "C" fn(i64, i64, *mut i64) -> i32
```

For integer-sign execution the first input is the checked source argument, the
second is an authenticated reserved zero, and the initialized result slot
receives exactly one tag: negative, zero, or positive. Cranelift emits signed
comparison and branch instructions derived from verified getters. The host
accepts status zero only with one valid tag and an initialized slot change,
maps that tag through the verified source literal table, and invokes the
existing checked output adapter exactly once. An invalid status/tag, unchanged
slot, duplicate store, finalization failure, null pointer, adapter failure, or
denial yields no successful result and no hidden second write.

The implementation may rename the existing minimal-add invocation helper only
to reflect its now-uniform ABI ownership. It may not add another unsafe block,
local allow attribute, FFI callback, raw Text pointer, external symbol, object
file, subprocess compiler, interpreter route, alternate backend, or side
channel.

Expected successful output is exact UTF-8 with no automatic newline:

| Argument | stdout | stderr | exit |
| ---: | --- | --- | ---: |
| `-7` | `negative` | empty | 0 |
| `0` | `zero` | empty | 0 |
| `9` | `positive` | empty | 0 |

Changing any of the three source literals must change the corresponding native
output and artifact identity without editing Rust. Changing either condition
must either change the verified native result consistently or be rejected as
outside the bounded feature. A source path, module name, or input value must
never select a precomputed answer.

## Diagnostics and fail-closed boundary

Allocate exactly one new stable public code, `H0634 canonical native program
layout`, in the existing front-end semantics family. Its producer-owned
reasons distinguish path/module/app identity, module count, module-first
position, illegal top-level item kind, missing app, type-after-app order, app
finality, and first-entry-task position. The identity reason exposes the
normalized expected path/module/app identity, observed module and app names,
their retained spans where available, and concise repair guidance without a
host absolute path. The illegal-item reason exposes the offending real `Item`
kind and span. Comments, strings, prose, source bytes, or fixture expectations
cannot satisfy the validator. The generic diagnostic renderer and schema do
not change.

H0634 is native canonical-admission evidence, not a claim that otherwise valid
legacy Hum is globally malformed. It rejects before backend-input capability
issuance, JIT, output, or readiness. Existing parser/checker errors, H0602
duplicate-name errors, H0614 unknown-start errors, H0615 multiple-app errors,
existing app-authority failures, and existing type/effect/ownership/resource/
profile failures retain precedence. Native layout analysis runs only after
their authoritative producers are green; it cannot mask or renumber them.
Multiple apps therefore remain H0615, a missing allocation declaration remains
the existing resource-stage failure, and a missing app in an otherwise
parseable canonical candidate is the H0634 missing-app reason.

Within H0634, fail-closed reason priority is module count, module-first,
missing app, path/module/app identity, illegal pre-app item kind,
type-after-app, app finality, then first-entry-task position. This order is
permanent combined-cause evidence: after repairing the earlier property, the
next applicable reason must appear with exactly one H0634 occurrence.

After layout admission, a separate stable backend-admission class still owns a
layout-valid program whose semantics exceed the exact integer-sign feature. It
must not describe the program as invalid Hum when the interpreter can run it.
The permanent unsupported-shape fixture is layout-valid, changes one semantic
shape beyond the admitted three-way program, and proves rejection before
capability issuance, JIT, output, or readiness.

## Permanent evidence

Focused evidence must be persisted before interpretation and must cover:

### Canonical-layout corpus and precedence evidence

The dedicated corpus is initialized, non-degenerate, and checked through the
real parser/AST/app-entry native-admission path:

| Fixture | Required result |
| --- | --- |
| `layout_valid_pass.hum` | module, two local types, final app, first entry task, and two later helpers are accepted in lexical order |
| `missing_module_fail.hum` | H0634 module-count reason |
| `duplicate_module_fail.hum` | H0634 module-count reason with both retained spans |
| `late_module_fail.hum` | H0634 module-first reason after an earlier semantic item |
| `missing_app_fail.hum` | H0634 missing-app reason |
| `duplicate_app_fail.hum` | existing H0615, not H0634 |
| `module_path_identity_fail.hum` | otherwise-valid real source bound to logical `programs/integer_sign.hum` rejects for a mismatching module identity |
| `illegal_pre_app_store_fail.hum` | H0634 illegal-item reason on the retained top-level `Store` and span |
| `illegal_pre_app_task_fail.hum` | H0634 illegal-item reason on the retained top-level `Task` and span |
| `illegal_pre_app_test_fail.hum` | H0634 illegal-item reason on the retained top-level `Test` and span |
| `type_after_app_fail.hum` | H0634 type-after-app reason |
| `semantic_after_app_fail.hum` | H0634 app-finality reason |
| `start_not_first_fail.hum` | H0634 first-entry-task reason |
| `helper_before_start_fail.hum` | the same H0634 reason even though the earlier helper is called by the entry |
| `unsupported_shape_fail.hum` | layout passes and the later backend-feature admission rejects |

All paths in this table are relative to
`fixtures/programs/integer_sign/`. The positive fixture proves optional types,
multiple helpers, and the same truthful allocation declaration as the frozen
program instead of merely copying it. Every negative fixture differs from a
valid layout only in its named property. The three pre-app fixtures retain one
otherwise-valid final app and independently insert exactly one `Store`, `Task`,
or `Test` between module/types and app. Comments and blank-line insertion
controls leave all results unchanged.

The existing `app_entry` selector also owns one permanent table over logical
native path spellings. `programs/integer_sign.hum` and
`programs\integer_sign.hum` normalize to the same accepted identity. Separate
initialized rows reject a module-suffix mismatch, filename-stem mismatch,
app-name mismatch, case mismatch, wrong directory, wrong extension, absolute
path, drive-rooted path, UNC path, `.`, `..`, empty component, and repeated
separator. Inline/table variants may share source structure, but the permanent
`module_path_identity_fail.hum` bytes must independently prove that real parsed
module/app facts disagree with the supplied logical path. No row may depend on
the fixture's own filename or a host filesystem lookup.

Precedence evidence also reuses the existing permanent H0602 duplicate-child,
H0614 unknown-start, H0615 multiple-app, and app-capability-mismatch fixtures,
plus initialized malformed parser inputs. Each is combined with a potential
layout issue and must retain its original exact diagnostic with zero H0634.
Only after those blockers are repaired may the corresponding layout diagnostic
appear.

### Positive and parity evidence

1. The parser retains one exact module occurrence and source span, the
   canonical-layout validator accepts the frozen program's generic
   path/module/app identity and closed item order, and its retained app child
   order identifies `run_tool` as the first task without reordering.
2. The frozen program passes parse, resolve, type, full-type, effect,
   ownership, resource, profile, core-lower, core-verify, backend-input, and
   IR-verify stages with exact live identities.
3. The v1 artifact is deterministic, canonical, byte-identical to its fixture,
   and round-trips only through the public verifier.
4. Interpreter and native executions agree on exact stdout, empty stderr, and
   success for at least `-7`, `-1`, `0`, `1`, and `9`.
5. Native evidence authenticates Cranelift verification, declaration,
   definition, finalization, one non-null module-owned code pointer, exact
   signed branch shape, source locations, one store, one invocation, one
   output, `ir_ready=1`, and `backend_ready=1`.
6. The exact program is executed on required Windows and Linux publication
   jobs.

The frozen program and the output-writing valid-layout control contain exactly
`allocates: callee-defined allocation behavior` in canonical task-section
order. Removing it or replacing it with a false allocation-free claim must
produce the existing resource-stage rejection before native layout or backend
admission. WO23 allocates no new diagnostic for that established rule; the
declaration describes the source task and does not claim that JIT code allocates
`Text` or performs output directly.

### Negative evidence

- default deny and explicit deny produce typed output failure, zero output,
  and zero backend invocation;
- `--entry`, wrong arity, invalid Int, duplicate/conflicting consent, source
  error, each canonical-layout violation, unsupported layout-valid shape,
  corrupt artifact, mixed lineage, unavailable ISA, invalid native status/tag,
  and adapter failure all reject at their owning boundary;
- an ordinary non-WO23 Hum program remains interpreter-runnable without
  `--native` but is never silently accepted by the native slice; and
- no rejected case sets backend readiness or reports a successful native
  program result.

### Mutation evidence

Use the existing exact-selector and initialized source-mutation machinery; add
no new capture, classifier, or generalized parsing framework. Each mutation
must prove its source changed, execute the exact guarded selector, fail at the
intended property with zero compile errors, and restore bytes exactly:

| ID | Initialized weakening | Required escaped disposition |
| --- | --- | --- |
| M01 | bypass final live-factory/profile identity admission | foreign or stale source reaches verifier/backend assertion |
| M02 | replace verified signed-condition lowering with a constant or swapped branch | at least one of negative/zero/positive differs from the source oracle |
| M03 | replace verified literal getter use with a Rust hard-coded literal/table | source-literal mutation fails to change native output |
| M04 | bypass app/task `stdout.write` closure or operator deny before JIT/output | denied run reaches backend or writes bytes |
| M05 | remove the required target/ISA predicate or add interpreter fallback | unsupported/native failure reports success or interpreter-derived output |
| M06 | accept native status/tag before finalization, valid sentinel change, and exact-one store | malformed or incomplete JIT evidence reaches output/readiness |
| M07 | ignore the real module-occurrence count predicate | missing- or duplicate-module fixture escapes H0634 |
| M08 | ignore the real module-first predicate | late-module fixture escapes H0634 |
| M09 | ignore the real type-before-app predicate | type-after-app fixture escapes H0634 |
| M10 | ignore the real app-finality predicate | semantic-after-app fixture escapes H0634 |
| M11 | ignore the real first-entry-task predicate | both entry-order fixtures escape H0634 |
| M12 | ignore the real normalized path/module/app identity predicate | an initialized logical-path or name mismatch escapes the H0634 identity reason |
| M13 | ignore the real exhaustive allowed-item-kind/sequence predicate | each pre-app Store, Task, and Test fixture escapes the H0634 illegal-item reason |

Test-only fault values are supplemental controls and receive no load-bearing
mutation credit by themselves. A later rejection cannot mask the row-specific
escaped disposition. M07-M13 must mutate the actual production validator
predicate, prove initialized source change, cause the named otherwise-valid
fixture to be accepted by layout analysis, and restore the source bytes
exactly. M12 must weaken the generic lexical identity comparison, not replace
the path with `integer_sign`; M13 must weaken the exhaustive production match
without changing M09 type ordering or M10 app finality. Source-text counting,
expected-value edits, fixture-name special-casing, and synthetic-only switches
earn no credit.

### Exact selector and integrated evidence

Add exactly five guarded Rust selectors, each listed and executed exactly once:

```text
app_entry::tests::canonical_native_program_layout_is_ordered_and_load_bearing
backend_input::tests::canonical_integer_sign_backend_input_is_exact_and_nonforgeable
ir_verify::tests::integer_sign_artifact_rejection_matrix_is_complete
backend_cranelift::tests::integer_sign_lowering_is_source_driven_and_load_bearing
main::tests::native_integer_sign_run_is_authority_bound_and_platform_exact
```

The existing `app_entry` selector owns the complete H0634 reason/precedence
matrix, the path-normalization table, and M07-M13. No sixth selector is added.
The ordered selector ledger advances from 107 to exactly 112 invocations and
112 case-sensitive unique names with a newly authenticated digest. Existing
selectors retain order and exact-once credit. Full preflight also owns fmt,
all-target check, warnings-denied Clippy, root and subsidiary Rust suites,
compile-fail authority boundaries, program corpus, CLI misuse, human/JSON
contract determinism, text hygiene, public readiness, alpha claims, and release
readiness 0.0.1.

The complete focused set must pass before an independent reviewer may launch
the single Unit A Fast child through the existing persistence-first adapter.
No second Fast, local Exhaustive, or local CI is implied by a launcher or test
failure. Ubuntu Exhaustive remains the publication-CI producer; Windows skips
only its duplicate when the full lane is selected.

## Public claims and explicit non-claims

After terminal-green publication, Hum may claim only:

- one canonical structural `integer_sign` source program is checked through
  the existing semantic/authority chain, lowered with Cranelift, and executed
  natively on required x86_64 Windows-MSVC and Linux-GNU configurations;
- its native program file is mechanically admitted only when its bounded
  repository-relative path stem, module suffix, and app name match exactly and
  its closed top-level sequence is one module, optional local types, one final
  app, its entry task first, and later helpers retained in lexical order;
- its native control decision is input-driven and its observable Text is
  source-derived through the existing checked output adapter;
- the path reaches `ir_ready=1` and `backend_ready=1`; and
- the public `hum run --native` surface fails closed outside the exact bounded
  program feature.

WO23 does not claim arbitrary-program compilation, general conditional or Text
lowering, automatic native support for existing examples, macOS, portable
filesystem access, loops or termination checking, recursion, user-defined
types in native code, optimization, AOT/object output, linking, a runtime or
standard library, Nectar packaging, dependency resolution, self-hosting,
debugger maturity, another backend, LLVM migration, release readiness beyond
the existing 0.0.1 checks, or a tag/release.

WO23 also does not claim a general package grammar, automatic formatting,
call-graph-sorted declarations, a source evidence section, or global rejection
of legacy Hum files that do not use the canonical native V0 layout.

Cranelift remains the bounded, replaceable first backend. WO23 neither promises
its removal nor commits Hum to LLVM. Hum retains language semantics,
verification, capability authority, backend admission, and public claims.

## Explicit deferrals

The following remain separately planned future work:

- a second representative program and any reuse-based library extraction;
- stdlib or Nectar promotion;
- imports, package identity, manifests, dependency resolution, builds, and a
  downstream corpus;
- canonical formatting, call-graph declaration sorting, and any source-level
  evidence-item section;
- crater-like ecosystem testing until those foundations exist;
- macOS CI and a documented supported-platform contract;
- loops, termination measures, containers, files, concurrency, generics,
  optimization, AOT/object emission, debug information, self-hosting, another
  backend, LLVM work, releases, and tags; and
- capture/classifier/harness redesign absent a concrete newly discovered risk.

No advisory or deferral reserves a path, budget, session, or implementation
hook.

## Review, commit, publication, and status lifecycle

The one implementation unit follows this exact sequence with no implicit next
step:

1. the complete corrected fifty-four-path issuance candidate, including the
   mechanically enforceable layout design, receives fresh independent
   pre-issuance review;
2. only unqualified ACCEPT may recommend a separately authorized issuance
   commit with subject `docs(workorder): issue work order 23`;
3. BDFL separately authorizes one normal non-force push and terminal full
   Ubuntu/Windows CI for the issuance tree;
4. a fresh BDFL signal authorizes Unit A implementation;
5. implementer leaves the complete candidate unstaged with empty index and no
   artifacts after proportional focused evidence;
6. a fresh independent reviewer reviews the whole producer/verifier/consumer
   chain and owns the single Fast allowance only after focused green evidence;
7. only unqualified ACCEPT may recommend the exact implementation commit:
   `feat(program): run canonical integer sign natively`;
8. BDFL separately authorizes one push and terminal full Ubuntu/Windows CI;
9. BDFL separately authorizes a status-only WO23 record and local commit;
10. BDFL separately authorizes that status commit's push and terminal fast
    Ubuntu/Windows CI; and
11. closeout or any successor requires a fresh BDFL signal.

Any unexpected path, budget breach, semantic failure, platform mismatch,
forged authority, fallback, mutation weakness, red Fast/CI, evidence loss,
stash/archive drift, or review finding stops at its actual boundary. It grants
no repair, retry, amendment, commit, push, status edit, or later action.

## Planning-package validation

Authorship and independent pre-issuance review run only:

- `git diff --check`;
- fail-closed raw and whitespace-insensitive no-index checks covering the
  moved predecessor and untracked successor;
- proof that closed WO22 reconstructs its published blob after reinserting
  exactly the deleted marker line;
- exact-one-marker, line-4, regular-file, encoding, and canonical topology
  checks;
- exact fifty-four-path inventory, non-borrowable category arithmetic, thirteen
  unique ordered V0 layout rules, fourteen dedicated fixture additions, H0634
  reasons, and M07-M13 real-predicate ownership checks;
- the complete published 151-case classifier suite twice deterministically as
  planning evidence, including the canonical successor-issuance case;
- text hygiene and public readiness for the resulting repository inventory;
- alpha claims; and
- release readiness 0.0.1.

No Cargo, Rust selector, compiler/backend probe, native execution, JIT, Fast,
full preflight, Exhaustive, production-classifier prediction, CI run, archive
code, or stash operation is authorized during planning authorship or review.

## Current authorization gate

The sole next action is a fresh independent WO23 pre-issuance re-review of this
complete corrected uncommitted successor package. The reviewer must review the
fifty-four-path envelope, H0634 ownership/precedence, layout corpus, mutation
coverage, budgets, and preserved topology in addition to the unchanged
producer/verifier/consumer plan. It may inspect repository and published CI
truth and run only the planning-package validation above. It may not edit or
begin Unit A.

Only unqualified `ACCEPT` may recommend, without executing, the separately
authorized exact issuance commit:

```text
docs(workorder): issue work order 23
```

Issuance commit, push, CI, status recording, Unit A implementation, Fast,
native execution, macOS, another program, stdlib/Nectar work, packages,
crater-like testing, optimization, AOT, another backend, LLVM work, release or
tag work, WO23 closeout, and every later action remain unauthorized.

<!-- workorder-current-authorization-gate:end -->

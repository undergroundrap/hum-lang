# Hum Work Order 6: Overlapping Places And The First Local IO Slice

Date: 2026-07-09
Status: active; Sessions V-AA accepted and committed; decision 0017 accepted
under delegated authority, BDFL veto open; Session AB implementation is
complete and uncommitted pending architect-reviewer review; Session AC is
forbidden; issued under delegated authority (`docs/GOVERNANCE.md`)
Owner: BDFL (Ocean). Reviewer/ruler: architect-reviewer. Implementer: agent
sessions.
Predecessor: Work Order 5, Sessions R-U. Commit `8a6dd1c` was the initially
reported Session U snapshot, not an accepted closure point. The BDFL accepted
the corrective Session U verdict, decision 0015, and this Work Order 6 stack;
commit `6d7ccb7` is the approved Work Order 5 closure and Work Order 6 issue
point.

## Corrected predecessor state

The historical Session R mandate called its fixtures "programs 8 and 11
misuses." That label was inaccurate. Session R delivered local stale
field-view invalidation and H0807 evidence. It did not express or reject the
pinned Program 8 case: a live writable alias to `point.x` overlapping a second
write path to the same storage. Corrective Session U therefore records:

- 4/12 corpus programs fully running: 9, 10, 11, and 12;
- Program 3 partial only for ordinary same-list mutation rejection, with the
  two-list odd-filter positive, retain positive, and stale retained-item-view
  misuse still absent;
- Program 8 positive-only partial, because the direct field swap runs but the
  overlapping writable-alias misuse is absent;
- ownership at exactly three active records and triggered under the
  three-strike rule;
- contracts at four active records and triggered under the same rule.

H0802 borrowed-parameter permission and H0807 stale-view invalidation are
adjacent evidence, not substitutes for Program 8's required overlap check.
This correction is the baseline for every count in this order.

## Decision and sequence

Work Order 6 takes one path, in this order:

```text
V  overlapping-place repair for Program 8
W  explicit causal typed-failure slice and proposed decision 0016
X  structural executable app entry and proposed decision 0017
Y  checked source capability root under proposed decision 0017
Z  bounded stdout, operator grants, and decision 0017 evidence gate
AA runner-provided replay clock
AB opaque native Path boundary, with no host file read
AC audited Windows fixed-local drive classification, with no file read
AD hardened exact-file text read
AE integrated app, ledger, and retrospective
```

This order is mandatory.

1. Session V pays the renewed ownership trigger and decision 0014's
   disjoint-field repair before internal references.
2. Session W settles explicit propagation and causal wrapping before any IO
   operation exists. IO must not create unstructured string errors first and
   repair them later.
3. Session X makes the structural app entry executable and records the complete
   proposed app/capability decision without yet implementing capability
   closure. Session Y supplies the checked source-authority evidence and
   Session Z supplies the operator-grant/first-IO evidence before the proposal
   may be ruled accepted. Entry is therefore designed with the IO vocabulary,
   not as a generic `main()` added earlier.
4. Sessions Z, AA, AB, AC, and AD add one independently reviewable boundary at
   a time. Session AE proves composition and closes the order.

Do not authorize a later session merely because an earlier implementation is
present. Every session ends at its gate and requires an independent reviewer
verdict plus an explicit go signal for the next session.

## Contract-policy ruling carried into this order

[Decision 0015](docs/decisions/0015-adopt-classified-runtime-contract-policy.md)
settles the Session U mandated contract-check-mode question under delegated
authority, with the BDFL veto open. Current Hum still checks every executable
predicate contract and assigns no classification. This order does not
implement a classifier, release/debug profile, proof evidence, or elision.

Decision 0015 resolves only the `divide` check-mode record. The three active
Predicate v2 vocabulary records remain: conditional content/count for
`word_count`, list content for `builder_demo`, and text-literal equality for
`element_views`. They are deliberately deferred through Session AE so this
order remains one ownership repair followed by one coherent adoption slice.
Session AE must reapply the three-strike rule; this paragraph must not be used
to call contracts de-triggered.

## Ground-truth honesty locks

These are facts about the implementation at issue time, not aspirations:

- `Result T, E` is currently an annotated task return channel, not a
  first-class runtime `Result` value. A failed call propagates implicitly,
  without nominal compatibility checking.
- Failure values currently discard their origin and call sites. Error variant
  membership and exhaustiveness are not checked.
- `app` is parsed and graphed but has no execution semantics. `SPEC.md` uses
  `starts with:` as draft initial-state prose; Session X deliberately retires
  that never-implemented interpretation.
- `hum run --entry` recursively selects a task and prints direct-task returned
  values and failures to stdout. That direct-task behavior is a probe surface,
  not program stdout semantics.
- `hum effect-check` is a narrow task-local recognized-body gate. It does not
  prove general transitive effects or effect polymorphism.
- There are no runtime IO built-ins, operator grants, executable profiles, or
  OS sandbox today.
- `hum capabilities` reports toolchain discovery surfaces. It is not runtime
  authority and must not be presented as such.

Each session may narrow only the lock it proves. No output may silently rewrite
these baseline limitations into broad language claims.

## Global safety and scope boundaries

All sessions are local and offline-first. The following are banned throughout
this order unless a session below names an exact exception:

- internal references, nested/self-referential stored views, general alias
  inference, alias-to-alias propagation, element aliases, broad flow-sensitive
  borrowing, and a full ownership or borrow-soundness claim;
- closures, tasks-as-values, callbacks, effect polymorphism, `retain`,
  higher-order standard-library work, and general transitive-effect claims;
- concurrency, threads, scheduling, network, process execution, environment
  reads, randomness, host wall/monotonic clocks, plugins, dynamic loading,
  unsafe, FFI, backends, and package behavior. The sole bootstrap exception is
  Session AC's isolated, audited Windows drive-locality adapter; it is not Hum
  language unsafe/FFI and may expose no general foreign-call surface;
- filesystem writes, directory reads, globs, recursive traversal, current
  working-directory authority, implicit path conversion, and arbitrary roots;
- exceptions, unwind, catch/recovery syntax, ambient backtraces, erased
  any-error values, string-only context, and panic as ordinary failure;
- a general capability-value system, linear-capability proof, general Path or
  OsText API, complete stdlib IO, process sandbox, portable-IO claim, runtime
  profile enforcement, or target availability claim;
- implementation of decision 0015's contract classifier, build profiles,
  evidence schema, or check elision;
- public-alpha, full sandbox, deterministic-mode, complete effect-safety,
  memory-safety-completeness, or safety-critical-readiness claims.

No new schema identifier, CLI subcommand, report surface, pipeline gate, or
runtime profile is authorized. The following narrow changes are explicitly
authorized and no others:

- additive statement/check/status values inside existing human and JSON report
  shapes when a session below requires a machine-readable fact;
- the existing Core effect catalog gains `output` in Session Z;
- the existing target-facts family catalog gains reserved `os.stdio` in
  Session Z, synchronized with its documentation and fixtures, without any
  availability or permission claim;
- `hum run` gains only the `--allow`, `--deny`, and replay-sequence flags named
  in Sessions Z and AA. No new subcommand or `hum run --format json` is implied.

Tests may read only checked-in repository fixtures or isolated temporary files.
They must not modify source files, user files, or paths outside their isolated
test directory. No test may use the network, host clock, environment as input,
or shell/process execution from Hum code.

## Standing semantic and diagnostic gate

Every session below must satisfy all of these rules in addition to its local
acceptance criteria:

1. Every new source rejection rule has a non-degenerate misuse fixture and a
   stable H-code. Help names every relevant source site, the violated
   relationship, and an exact repair. A single primary span is insufficient
   when blame also belongs to a binding, declaration, call, or conflicting
   site; those secondary sites must appear in rendered help and structured
   facts.
2. Human and JSON output agree wherever an existing JSON surface exists.
   Runtime-only output has no invented JSON analogue. Static and runtime
   enforcement consume the same checked fact and diagnostic identity when both
   surfaces implement the same rule.
3. A source declaration mismatch is a checked diagnostic and prevents host
   action. A missing or denied operator grant is a typed operation failure with
   exit 1 and performs no host action. A malformed CLI invocation remains a CLI
   usage error; do not mislabel it as a source diagnostic.
4. Existing report/schema IDs remain unchanged. Additive status values are
   documented and golden outputs stay synchronized. Do not add fields merely
   to make the diff look evidence-native when existing section, statement, or
   boundary-check rows carry the fact honestly.
5. New surface syntax updates `SPEC.md`, the language reference, grammar and
   syntax metadata, TextMate output, README/showcase material, and Formal Core
   only where actually affected. Examples longer than five lines remain
   extracted from checked fixtures under preflight discipline.
6. The interpreter, resolver, full type check, effect check, ownership check,
   resource check, Core preview/lower/verify, graph, and docs must not disagree
   about whether a recognized form is supported. A report may remain explicitly
   blocked, but it may not call an executed form pure, unsupported, or checked
   under a broader rule than exists.
7. Run the session's targeted commands, `cargo test`,
   `.\tools\check_all.ps1`, and `git diff --check`. Leave the session
   uncommitted and stop for review.

## Session V: narrow writable field aliases (Program 8)

Implementation status: accepted and committed as `acfb36f`. Session W was
accepted and committed as `2af02ae`.

Purpose: pay the exact Program 8 overlap record before any other ownership
work.

Scope:

1. Recognize exactly this local direct-field form in a straight-line task body:

   ```hum
   let alias_to_x = change point.x
   ```

   The owner must already carry visible mutation authority. The binding is a
   writable view of that exact direct field, not a copied value. `set
   alias_to_x = value` writes through to `point.x`.
2. The alias lifetime begins at the binding and ends after its last syntactic
   use in the same straight-line body. While live, a direct read, direct write,
   owner-wide access, or second writable alias that may overlap `point.x`
   rejects. Access to the definitely distinct direct field `point.y` remains
   accepted.
3. Alias creation or use inside a branch/loop, passing, returning, storing,
   alias-to-alias binding, nested places, list elements, and rebinding stay
   outside the recognized slice and fail closed with one stable
   unsupported-writable-alias diagnostic and representative misuse fixtures.
   Do not infer broader lifetimes from this straight-line last-use rule.
4. Allocate H0808 to the overlap rule. Its diagnostic names the alias binding,
   the conflicting site, the overlapping source place, why the writes are not
   known independent, and a concrete fix such as using a distinct field or
   ending the alias's last use before the access. Use one additional stable
   ownership code if the unsupported/escape rule cannot honestly share H0808.
5. `hum ownership-check` human and existing JSON rows expose the same alias,
   place, lifetime, conflict, and status facts used by runtime enforcement. No
   new schema ID or ownership report surface.
6. Add hand-written fixtures proving: write-through changes the owner;
   distinct-field access survives; overlapping direct read rejects;
   overlapping direct write rejects; a second alias rejects; and at least one
   escape/unsupported form rejects. The pinned Program 8 fixture must use a
   live alias to `point.x`; H0802 and H0807 do not count.
7. Update `docs/bakeoff/SCORECARD.md` only after the exact positive and pinned
   misuse pass. Then Program 8 becomes fully running, the corrected count is
   5/12 full plus Program 3 partial, and the ledger annotates only this overlap
   record resolved. Narrow the decision 0014 lock to this exact local form.

Acceptance criteria:

- The positive write-through fixture observes the changed owner value; it is
  not a declaration-only or unused-alias proof.
- The pinned misuse fires H0808 and names both source sites plus the fix in
  human output, ownership JSON, and runtime evidence where those surfaces
  apply.
- The distinct-field fixture passes, and the read/second-alias/escape fixtures
  prove the stated fail-closed boundary.
- Program 8 alone moves from partial to full; no internal-reference or general
  alias claim appears.
- Standing checks pass. Stop. Do not begin Session W.

Implementation evidence:

- `examples/probes/writable_field_aliases.hum` observes real write-through as
  `{x: 9, y: 2}`, swaps through two distinct live aliases as `{x: 2, y: 1}`,
  accepts direct `point.y` access while the `point.x` alias is live as
  `{x: 2, y: 7}`, and accepts a sequential same-field alias after last use as
  `{x: 7, y: 2}`.
- `fixtures/ownership_check/session_v_program8_overlap_write_fail.hum` is the
  pinned non-degenerate Program 8 misuse. Human ownership output, ownership
  JSON, and runtime all report H0808 with binding, conflict, and last-use sites
  plus the distinct-field/end-last-use repairs.
- The direct-read and second-alias fixtures also report H0808. Escape,
  control-flow, permission-wrapper, owner-rebinding, and visible-name-collision
  fixtures report H0809. Acquiring the alias from a borrowed owner remains the separate H0802
  permission failure and takes precedence over a body that would otherwise
  contain an overlap.
- Resolver, full type, effect, ownership, resource, Core preview/lower/verify,
  graph, runtime, `cargo test`, and `tools/check_all.ps1` agree on the exact
  slice. No schema ID, command, report surface, or pipeline gate was added.

## Session W: explicit causal typed failure

Purpose: make the first IO errors composable before IO exists.

Scope:

1. Add only these recognized direct named-call forms, with ordinary value
   arguments:

   ```hum
   let value = try fallible_call()
   let value = try fallible_call() or fail OuterError.context
   ```

   `try call()` propagates only when the callee and caller declare the same
   nominal error root. `try call() or fail OuterError.context` wraps an inner
   failure in a variant whose root matches the caller's declared error root and
   preserves the inner cause.
2. A known fallible call without `try`, incompatible unwrapped propagation, a
   wrapper with the wrong caller root, `try` on an infallible call, direct
   `fail WrongRoot.case`, and an unsupported `try` expression shape each reject
   under stable diagnostics with a dedicated misuse fixture. Help names the
   call or fail site, callee header/result root when relevant, caller
   header/result root, and exact repair.
3. Replace the runtime's origin-free failure payload for this slice with a
   causal carrier that preserves the nominal root/variant string, root origin
   span, and every propagation or wrapping span. Render a stable outer-to-root
   chain with source locations and exit 1. A typed failure is not a runtime
   trap.
4. The ordinary same-root positive and a wrapping positive must execute. A
   root failure wrapped across at least two calls must display the outer error,
   both call/wrap sites, and root origin without dropping or reversing blame.
5. Effect checking treats recognized propagation/wrapping as typed failure and
   requires the existing `fails when:` declaration. Existing full-type/effect
   report rows and diagnostic JSON expose nominal compatibility and sites; no
   new schema or run JSON.
6. Do not claim first-class Result values, checked variant membership,
   exhaustiveness, recovery, narrowing, exceptions, backtraces, or general
   call typing. Permission-bearing `borrow`/`change`/`consume` call arguments
   remain outside this `try` slice.
7. Author proposed decision 0016 recording these semantics and killing:
   implicit fallible calls, incompatible implicit propagation, string-only
   context, exceptions/unwind, erased any-error propagation, and ambient
   backtraces. The implementer leaves its status `proposed`.

Acceptance criteria:

- All named positive and misuse fixtures pass with human/JSON agreement where
  JSON exists, and the causal runtime chain contains every required source
  site.
- Core/report surfaces either recognize the forms consistently or state an
  exact blocker; none silently call the accepted path pure or unsupported.
- The reviewer independently inspects the executable evidence. Only the
  architect-reviewer may change decision 0016 to
  `accepted under delegated authority (BDFL veto open)` and issue its BDFL
  brief.
- Standing checks pass. Stop. Session X is forbidden until decision 0016 is
  accepted or the BDFL gives a contrary ruling.

Accepted corrective implementation evidence (2026-07-10):

- `examples/probes/causal_failures.hum` executes same-root and wrapping success
  as `7`. Its two-wrap failure exits 1 and renders `OuterError.context`, both
  wrapping sites, `MiddleError.context`, `RootError.origin`, and the root
  origin in outer-to-root order. Direct root and same-root propagation failures
  also render the preserved root origin.
- H0901-H0906 each have a dedicated full-type misuse fixture with human/JSON
  root and call/callee/caller site facts plus precise repair help. H0907 pins
  the effect-owned meaningful-`fails when:` rejection.
- H0901 additionally rejects known fallible calls nested in an operator, a
  call argument, and a `for each` collection. Token-bounded recognition keeps
  ordinary `trying()` valid and diagnoses an implicit fallible `try_value()`
  as H0901 rather than H0906.
- Nine adversarial unsupported-`try` shapes produce H0906 in full type and
  remain explicit blockers through Core preview/lower/verify. Relationship
  diagnostics H0901, H0902, and H0906 take precedence over H0907 in the pinned
  combined fixture.
- Effect checking rejects both propagation and wrapping when a task declares
  `avoids: failure`.
- The shared failure-declaration quality rule rejects `fails when: todo` with
  exactly H0907 for direct failure, same-root propagation, and wrapping;
  runtime rejects each fixture during preflight before executing the failure.
- Resolver, full type, effect, ownership, resource, Core preview/lower/verify,
  graph, and runtime exercise the positive surface. Existing schema IDs and CLI
  surface are unchanged.
- Decision 0016 is accepted under delegated authority with the BDFL veto open.
  At Session W closure, Session X was the next unfinished session and required
  a separate BDFL go signal; that authorization was later issued for X only.

## Session X: structural executable app entry

Purpose: execute one pure app root without also implementing capability
analysis or IO.

Scope:

1. The executable form is exactly one top-level `app` in the run input. It has
   one meaningful `starts with:` line containing one bare snake_case name of a
   task directly nested in that app:

   ```hum
   app local_tool {
     starts with:
       run_tool
   }
   ```

   The start task returns `Unit` or `Result Unit, E`. Existing `--args` values
   bind its parameters. App-mode name resolution stays inside that app and
   cannot select a same-named task outside it.
2. With no `--entry`, a valid executable app is selected. Files without an app
   preserve legacy single-task behavior. `--entry` remains a direct pure-task
   probe and does not become app mode.
3. In app mode, a pure Unit success prints nothing. A typed failure exits 1 and
   renders Session W's causal chain to stderr. Source diagnostics go to stderr.
   Legacy direct pure-task returned-value/failure display remains unchanged and
   is regression-tested.
4. Add stable misuse fixtures for missing, empty, or duplicate `starts with:`,
   unknown or non-child start, multiple executable apps, invalid start result,
   and a same-named task outside the app. Relevant app, task, and start sites
   appear in blame/help and existing diagnostic JSON.
5. Explicitly retire `SPEC.md`'s old draft interpretation of `starts with:` as
   state initialization. State initialization remains undesigned; do not add a
   replacement feature.
6. Author proposed decision 0017 with both halves of the intended ruling: the
   structural app semantics proved here and the exact capability vocabulary,
   grant algebra scheduled for Session Y, and Path boundary scheduled for
   Sessions AB-AC, including the isolated audited bootstrap locality adapter
   needed to reject hidden Windows network/device mappings.
   It remains `proposed` after X. The reviewer may accept the X implementation
   and explicitly authorize Y, but must not issue the delegated ruling until Y
   supplies the capability-root evidence.
7. Do not add capability closure, capability rejection rules, operator grant
   flags, IO built-ins, Core `output`, or `os.stdio` in this session.

Acceptance criteria:

- A hand-written pure app runs by its nested start task, emits no automatic
  `Unit`, and a same-named external task cannot be selected through app mode.
- Every structural misuse above fires one stable blamed diagnostic; human and
  existing JSON agree.
- Legacy no-app and direct pure `--entry` behavior remains compatible.
- Decision 0017 is complete enough to constrain Session Y but remains
  `proposed`; no delegated acceptance is recorded yet.
- Standing checks pass. Stop. Session Y requires an explicit review go signal.

Accepted implementation evidence (2026-07-10):

- `examples/probes/pure_app_entry.hum` selects its directly nested `run_tool`,
  binds the ordinary runner argument, exits 0, and leaves both output channels
  empty. `examples/probes/fallible_app_entry.hum` proves `Result Unit, E`
  success and renders `LaunchError.requested` with its preserved origin on
  stderr at exit 1.
- H0610-H0616 cover missing, empty, duplicate, invalid-name, non-child,
  multiple-app, and invalid-result structure. Each permanent misuse fixture
  emits exactly one source diagnostic in human and `hum.check.v0` JSON;
  relationship diagnostics carry labeled app, declaration, and task spans.
- The same-named external-task fixture is H0614 in app mode and remains
  directly runnable through explicit `--entry`, demonstrating that app lookup
  is lexical without changing the legacy probe. Separate passing fixtures pin
  both direct-child entry shadowing and app-local helper-call lookup against
  same-named failing external tasks across resolver, full type, effect,
  ownership, resource, Core, graph, and runtime. External-only calls stop at
  the app boundary under H0601.
- App execution now consumes the existing resolver, declaration-type, and
  recognized full-type gates. Permanent fixtures prove H0602 duplicate child,
  H0605 unknown result root, H0606 Unit-return mismatch, and a body-only
  full-type mismatch all stop before execution. Runtime independently refuses
  to suppress a non-Unit value if a future checker regression reaches it.
- The nested non-child fixture reports H0614 with the start declaration as
  primary blame and labeled outer-app and candidate-task spans in human and
  JSON output.
- Decision 0017 remains proposed. No capability analysis or rejection,
  operator grant, IO built-in, Core `output`, `os.stdio`, Path, or Session Y
  behavior is present.

## Session Y: checked capability root

Purpose: make the X app the maximum source authority declaration before any
external operation exists.

Scope:

1. Pin exactly these source capability IDs and mappings for this order:

   | Source capability | Core effect | Runtime/target meaning |
   | --- | --- | --- |
   | `stdout.write` | `output` | bounded bootstrap stdout adapter; reserved `os.stdio` mapping lands in Z |
   | `clock.replay` | `time` | ordered runner-provided replay input; explicitly no `os.clock` requirement or host-clock access |
   | `files.read` | `file` | one exact local file through the bootstrap adapter and existing `os.filesystem` family |

   Other `uses:` lines remain ordinary dependencies and grant no runtime
   authority. Dotted capability-like names outside this pinned set reject in
   executable app/task authority positions. Reconcile the lone
   `filesystem.read` example in `docs/SAFETY_OF_MAKER_AND_USER.md` to the pinned
   `files.read` spelling already used by `SPEC.md`, `docs/SECURITY_MODEL.md`,
   and `docs/LANGUAGE_SUBSET_0_1.md`.
2. App `uses:` is the program's maximum source authority, never operator
   consent. For recognized fixed calls, every task declares its direct and
   transitive external capabilities, every caller covers its callee closure,
   and the app covers the start-task closure. An unidentifiable call in this
   executable subset fails closed. This is closed direct-call capability
   analysis, not effect polymorphism or a complete effect system.
3. A pinned capability declared on a task is an authority budget even before
   its operation ships; over-declaration is security-relevant and must be
   covered by callers and the app. A positive nested helper fixture with a
   declared capability proves transitive closure while remaining host-effect
   free. Do not claim that the unused declaration proves an IO operation.
4. Pin the later grant algebra: effective operation authority is the
   intersection of the app declaration, the reachable task declaration, and
   the operator's exact grant, minus any matching deny. The default operator
   grant set is empty and deny wins. Source declaration is never consent.
5. `--entry` remains a pure probe. A task whose recognized direct-call closure
   requires any pinned external capability rejects under `--entry`, even after
   operator flags exist.
6. Add stable misuse fixtures for unknown capability ID, missing caller
   transitive declaration, app/task mismatch, and `--entry` authority bypass.
   Relevant app, task, call, declaration, and entry sites appear in blame/help
   and existing diagnostic/effect JSON. Existing effect boundary rows expose
   the minimal closure facts; no new schema ID or report surface.
7. Add the source-authority evidence to proposed decision 0017 without changing
   its status. It
   kills ambient process authority, authority-bearing `--entry`,
   declaration-as-consent, implicit current directory, path-as-Text, global
   task lookup in app mode, and replay-to-`os.clock` mapping.
8. Do not add operator grant flags or any IO built-in in this session.

Acceptance criteria:

- The positive nested-helper app runs without host effects and existing human/
  JSON effect facts show the exact capability closure without calling it IO.
- Every capability mismatch above fires a stable blamed diagnostic with all
  relevant sites; human and existing JSON agree.
- Direct pure `--entry` remains compatible; authority-bearing `--entry` is
  blocked.
- The reviewer independently inspects X and Y evidence together. Decision 0017
  remains `proposed`; the reviewer may explicitly authorize Session Z as the
  final operator-grant/IO evidence slice but must not rule yet.
- Standing checks pass. Stop. Session Z requires an explicit review go signal.

Implementation evidence awaiting review (2026-07-10):

- `examples/probes/capability_root.hum` executes a real nested helper call and
  exits with no output or host operation. App, caller, and helper each declare
  exact `stdout.write`, `clock.replay`, and `files.read` source budgets.
- Existing effect boundary rows expose task budgets, caller closure, and app
  maximum/start closure under stable source-policy IDs. Each row carries the
  exact capability's kind, scope, strength, one-run lifetime, ordinary
  severity tier, reserved mapping status, and structured app/task/call/
  declaration route sites. These are source policy snapshots, not grants or
  IO evidence.
- H0617 rejects `process.run` and classifies it as the separate
  `sandbox_bypass_authority` tier. H0618 blames caller, callee, call, and origin
  declaration for missing transitive coverage. H0619 blames app, start, entry,
  and origin for an incomplete maximum. H0620 blocks an authority-bearing
  `--entry` while the same fixture remains valid in structural app mode.
- The shared analyzer reuses the Session W executable-expression call scanner,
  so nested/operator call positions cannot silently escape closure analysis.
  Existing resolver H0601 remains the fail-closed owner for an unidentifiable
  app-local callee.
- Corrective fixtures prove `--entry` traverses reachable callees and rejects
  transitive `process.run` and `stdout.*` declarations with routed H0617 before
  execution. A three-call single-statement fixture, including a repeated
  callee, proves source-policy IDs are unique per lexical occurrence and stable
  across repeated effect reports.
- Proposed decision 0017 now pins exact grant dimensions, a separate
  sandbox-bypass tier, task-coupled/no-startup-prompt consent, explicit
  persistence and dangerous-wildcard treatment, and the future forensic join
  of policy snapshot, decision event, and exercise event. Session Y implements
  only the source snapshot and policy join ID; it adds no operator flags,
  grants, prompts, persistence, IO, Core `output`, or `os.stdio`.

## Session Z: bounded stdout capability

Purpose: add the first real external effect under the Y authority root.

Scope:

1. Add repeatable `hum run --allow <grant>` and `--deny <capability>` flags.
   In this session the only valid grant is `stdout.write`; it carries no
   payload. The default is deny and an exact deny overrides an allow. Exact
   duplicate flags are idempotent; allow plus deny is valid and deny wins.
   Unknown capabilities, forbidden payloads, or malformed invocations are
   stable CLI usage errors.
2. Add exactly:

   ```hum
   stdout_write(text: Text) -> Result Unit, OutputError
   ```

   It immediately writes the exact UTF-8 bytes of `text` through the app output
   adapter, with no implicit newline. A rolling per-run budget is checked before
   each call and is bounded to 1 MiB; exceeding it is
   `OutputError.limit_exceeded` with no adapter call. Adapter write failure is
   exactly `OutputError.write_failed`, never a trap. Raw host error strings are
   not the typed identity or a stable diagnostic key.
3. The call requires `stdout.write` in the task/caller/app closure and an
   operator allow. Missing source declarations are static blamed diagnostics.
   Missing or denied operator grants return a typed `OutputError.denied` before
   the adapter is called. `--entry` cannot bypass the app even when allowed.
4. Separate the app program-output channel from legacy direct-task result
   display. Each accepted `stdout_write` is immediate; a later typed failure
   does not retract bytes written by earlier successful calls. Successful app
   completion adds no `Unit` or return-value line. Typed app failure and
   diagnostics go to stderr. Do not describe stdout as transactional or
   buffered-until-success, and do not add stdin, stderr-writing, formatting,
   interpolation, a separate flush phase, terminal detection, or atomic-output
   claims.
5. Add Core `output` as the one mandated additive effect catalog value. Add
   reserved target family `os.stdio` to the existing target-facts catalog,
   schema documentation, and all target fixtures. It means stdio mapping only;
   it grants no process-spawn authority, claims no target availability, and
   does not make target facts or profiles runtime-enforcing.
6. Use an injectable output adapter. Unit tests cover success, denial with zero
   adapter calls, deterministic `OutputError.write_failed`, the byte bound, and
   prior successful bytes remaining visible when a later call fails. A
   hand-written Hum app proves exact output and W-style call-site wrapping of an
   output failure.
7. Complete proposed decision 0017's executable evidence. The implementer
   leaves its status `proposed`; the reviewer applies any delegated ruling only
   after independently verifying Sessions X-Z together.

Acceptance criteria:

- The app prints exact bytes only with app/task declarations and the operator
  allow; default deny and explicit deny fail as typed errors without calling
  the adapter.
- Missing declaration fixtures identify call, task, and app sites in human and
  existing JSON. No runtime JSON surface is claimed.
- The failing-adapter and output-limit tests return typed causal failures; no
  host write occurs after denial or limit rejection.
- Core, target-facts, docs, fixtures, and help text stay synchronized under
  their existing schema IDs.
- Only the architect-reviewer may now change decision 0017 to
  `accepted under delegated authority (BDFL veto open)` and issue its BDFL
  brief.
- Standing checks pass. Stop. Session AA is forbidden until decision 0017 is
  accepted or the BDFL gives a contrary ruling.

Implementation evidence awaiting review (2026-07-10):

- `examples/probes/bounded_stdout.hum` prints its exact UTF-8 argument with no
  newline only when source closure and exact `--allow stdout.write` intersect.
  Default deny, explicit deny, and allow-plus-deny return the same typed
  `OutputError.denied` cause under the app's W-style wrapper; direct `--entry`
  remains H0620 even when allowed.
- The runtime owns an injectable adapter and a 1 MiB rolling successful-byte
  budget. Unit probes prove zero adapter calls on denial and limit rejection,
  one opaque failing call for `OutputError.write_failed`, and preservation of
  prior successful bytes when a later write fails.
- H0621 exposes the output call, calling task, and structural app sites when
  source authority is missing. H0622 rejects arity/type drift from the exact
  `stdout_write(Text)` signature. Resolver, full type, effect, ownership,
  resource, Core, graph, and runtime agree on the positive fixture.
- An implicit-call fixture proves `stdout_write` remains a nominal fallible
  operation under Session W: H0901 blocks execution before any output adapter
  call unless the caller uses `try` or explicit causal wrapping.
- A legacy task with `stdout.write` but no structural app is also rejected by
  H0621 before runtime; operator allow cannot manufacture the missing app root.
- A helper-backed output fixture proves H0618 still prevents authority
  laundering when the helper and app declare `stdout.write` but their caller
  omits the transitive budget.
- Each request records typed in-memory operator-decision and operation-exercise
  facts joined to Session Y's lexical source-policy ID, including exact grant
  dimensions, the complete app/start/caller/output route and every call
  occurrence, one-run lifetime, effective deny/allow, byte count, adapter-call
  status, and stable result reason. Multi-hop and multiple-caller tests prove
  distinct stable route IDs across repeated runs. A skipped-first-branch test
  proves runtime selects the executed lexical call occurrence rather than a
  task-name/execution-order cursor. Shared separator-normalized source identity
  makes `/` and `\` Windows input spellings select the same policy ID and route
  spans while original spans remain available for display. No runtime JSON or
  persistence surface is claimed.
- H0623 reserves the exact `stdout_write` callable name across all stages and
  blocks a collision before runtime can substitute either body.
- H0624 rejects output-reachable recursion with the recursive edge and complete
  finite route-to-cycle evidence before execution. This bounded fail-closed
  rule makes no general recursion claim. Authority remains more fundamental:
  permanent combined-cause fixtures prove missing task/app coverage produces
  exactly H0621 and missing caller coverage produces exactly H0618 across
  human, JSON, effect, graph, and runtime preflight, with no H0624 duplicate.
- `reserved_mapping_only` is non-satisfying for `requires:`; a pinned embedded
  target fixture proves H1204 human/JSON/graph agreement for `os.stdio` without
  claiming host probing or availability.
- Core gains only `output`; target facts gain reserved `os.stdio` in every
  fixture as `reserved_mapping_only`, explicitly not target availability.
  No stdin, stderr writer, formatting, buffering-until-success, terminal,
  process, clock, file, Path, prompt, wildcard, or Session AA surface is added.

## Session AA: runner-provided replay clock

Purpose: add deterministic replay input without importing host time.

Scope:

1. Add exactly:

   ```hum
   clock_replay_tick() -> Result UInt, ReplayClockError
   ```

   Each call consumes the next UInt from an ordered runner-provided sequence.
   Add repeatable `hum run --replay-tick <UInt>` with a maximum of 1024 values.
   `--allow clock.replay` is separately required; replay values are input, not
   authority.
2. The call requires `clock.replay` through task/caller/app declarations and
   operator allow. Default/explicit denial returns
   `ReplayClockError.denied`; an empty or exhausted sequence returns
   `ReplayClockError.exhausted`. Neither path reads a host clock or calls the
   replay adapter after rejection.
3. Lower this operation to the existing Core `time` effect with an explicit
   runner-replay status/reason in existing effect rows. It does not require
   `os.clock`, and no code path may use `SystemTime`, monotonic host time,
   environment input, scheduling state, or randomness.
4. Add the declaration/grant/deny misuse fixtures required by the standing
   gate and an exhaustion fixture whose error is wrapped through Session W's
   causal chain.
5. A fixture selects literal stdout text by comparing the tick. Do not add
   UInt-to-Text conversion, interpolation, or text concatenation. Two runs with
   identical source, ordinary args, grants, tick sequence, and no `--timings`
   must have byte-identical stdout, stderr, and exit code.

Acceptance criteria:

- Ordered consumption, exhaustion, default deny, explicit deny, maximum length,
  and zero host-clock access have direct tests.
- Identical runner replay inputs reproduce exact process output, while a
  changed tick predictably changes the selected literal.
- Docs say only that replay-clock input is controlled. They do not claim whole
  program determinism, deterministic scheduling, or `os.clock` support.
- Standing checks pass. Stop. Do not begin Session AB.

Implementation evidence awaiting review (2026-07-10):

- `examples/probes/runner_replay_clock.hum` consumes two runner ticks through a
  nested helper. Inputs `1, 7` select exact stdout bytes `seven`; `1, 8`
  selects `other`; identical complete inputs reproduce stdout, stderr, and exit
  status.
- Exact `clock.replay` source closure and operator allow are independent of the
  tick sequence. Default and explicit denial return
  `ReplayClockError.denied` with zero replay-adapter calls; a one-value sequence
  returns `ReplayClockError.exhausted` on the second call and preserves the
  `ReplayAppError.replay` wrapper, helper call, propagation site, and root
  origin.
- H0625-H0628 pin missing task/app replay authority, invalid zero-argument
  signature, reserved built-in identity, and replay-reachable recursion.
  Permanent combined-cause fixtures preserve H0625/H0618 precedence before
  recursion, and H0626 precedes the effect-owned missing-`fails when:` rule.
- Existing effect boundary rows map route-specific replay operations to Core
  `time` with `implemented_runner_replay_input_v0_no_os.clock`, complete
  app/start/caller/operation spans, and stable policy IDs. In-memory decision
  and exercise facts join those IDs to the ordered sequence index and consumed
  tick. No runtime JSON surface is added.
- Unit and CLI tests cover ordered consumption, repeat stability, changed-tick
  selection, default/explicit denial, exhaustion, the 1024-value limit,
  separator-normalized policy identity, zero host-clock symbols, and replay
  adapter call counts.

## Session AB: opaque native Path boundary, no host read

Purpose: make path identity non-Text and lossless before file contents can be
observed.

Scope:

1. Add an opaque runtime `Path` value constructed only by the app runner for at
   most one `Path` parameter of the nested start task. Preserve native OS text
   through an `args_os`/`OsString` boundary. Converting the current String CLI
   value back into a path is not lossless evidence.
2. `Path` has no source literal, Text conversion, display, comparison,
   concatenation, component, join, parent, return, storage, or general stdlib
   API in this order. It is an inert path identity, not authority. Direct
   `--entry` cannot construct it; only app entry can.
3. The lexical host path candidate in this order is Windows-first and narrower
   than "absolute": accept only an ordinary drive-letter-rooted spelling
   (`Prefix::Disk`). Reject before metadata/open: relative and drive-relative
   forms; empty, `.` or `..` components; UNC; verbatim, device, `GLOBALROOT`,
   volume-GUID, and NT namespace prefixes; alternate data streams or any colon
   after the drive prefix; components ending in a dot or space; and the full
   case-insensitive Win32 DOS-device normalization set, including `CON`, `PRN`,
   `AUX`, `NUL`, `CLOCK$`, `CONIN$`, `CONOUT$`, `COM1`-`COM9`,
   `LPT1`-`LPT9`, and the recognized superscript-digit COM/LPT aliases, with
   extension, trailing-dot, or trailing-space aliases. More than one Path
   argument rejects.

   `Prefix::Disk` is only lexical evidence. The Path and exact grant remain
   `locality_unclassified` until Session AC; neither may authorize a file
   operation. Non-Windows host file access remains unavailable in this order.
   These are invocation-boundary errors with no host access, not fake source
   diagnostics.
4. Extend the existing `--allow` payload parser to represent at most one exact
   native `files.read=<path>` grant and exact `--deny files.read`, still without
   reading metadata or contents. Preserve the grant as native path identity.
   Apply the same lexical-path rejection to grant payloads. Two distinct
   `files.read` payloads are a CLI usage error; exact duplicate grants are
   idempotent and deny still wins. The Path argument need not be granted until
   an operation uses it in AD;
   source spelling and operator consent remain separate facts.
5. Add a source misuse fixture for a Text literal where Path is required and
   any other new source rejection rule. Add platform-native unit tests proving
   non-Text path code units round-trip without loss. A nonexistent absolute
   Path and nonexistent exact grant must parse and reach the no-read test
   harness, proving this session made no filesystem probe.
6. A small app may accept the Path and emit a fixed stdout literal, but must not
   inspect or stringify it. The observable proof is entry construction plus
   the internal lossless round-trip and zero-host-access adapter test, not a
   degenerate path-as-Text echo.

Acceptance criteria:

- Native path round-trip tests pass on the supported platform, including a
  value not representable as ordinary Rust `String` where the platform API can
  construct one.
- Relative/traversal/multiple/literal/direct-entry misuse and every lexically
  banned Windows UNC/device/volume/ADS/DOS-device/normalized-alias path class
  rejects before host access with stable applicable diagnostics or invocation
  errors.
- A nonexistent clean absolute path proves no metadata or content read occurs.
- No docs claim fixed-local classification, a general Path API, portable path
  semantics, or file-read support yet.
- Standing checks pass. Stop. Do not begin Session AC.

## Session AC: audited Windows fixed-local drive classification

Purpose: determine whether an AB drive-letter candidate is truly fixed local
without opening the candidate file or hiding network authority.

Scope:

1. This session is the sole exception to the global bootstrap unsafe/FFI ban.
   Add one small path dependency crate dedicated to Windows drive locality. The
   main Hum binary crate retains `#![forbid(unsafe_code)]`; unsafe code cannot be
   moved into the main crate, widened, or exposed to Hum source.
2. The adapter may call only the Windows `GetDriveTypeW` and `QueryDosDeviceW`
   APIs needed to distinguish fixed local disks from remote, substituted,
   removable, device, or unknown mappings. Use no third-party download, child
   process, environment input, registry access, candidate metadata/open, or
   network call. The adapter's safe public API accepts only an already-lexically
   validated drive root and returns a closed enum such as `FixedLocal`,
   `Remote`, `Substituted`, `Unsupported`, or `Unknown`.
3. Accept `FixedLocal` only when both API results support an ordinary fixed
   local volume. Remote/MUP, SUBST or other indirection, removable/media,
   unsupported, malformed, API-failure, and unknown results fail closed. Do not
   open first and classify later.
4. Keep every unsafe block inside the adapter crate, minimize and comment its
   invariant, use bounded NUL-terminated UTF-16 buffers, check all return values
   and lengths, and enable `unsafe_op_in_unsafe_fn` denial. No general Win32
   binding, raw handle, or arbitrary symbol lookup is authorized.
5. This work order explicitly mandates one new review packet:
   `docs/SESSION_AC_WINDOWS_LOCALITY_REVIEW.md`. It records the exact foreign
   signatures, why safe Rust std is insufficient, pre/postconditions, buffer
   bounds, encoding, failure mapping, TCB impact, forbidden calls, test matrix,
   and rollback. This packet describes bootstrap implementation risk; it does
   not create Hum unsafe/FFI syntax or profile support.
6. Inject the two OS-query results for exhaustive unit tests. A Windows smoke
   test classifies the already-used repository drive without reading a
   candidate file. Non-Windows builds return `Unsupported` without compiling or
   executing foreign calls. Tests prove every non-fixed or ambiguous result
   blocks and no file adapter is invoked.
7. Existing report/schema IDs, CLI flags, Hum grammar, and capability names do
   not change in this session. Only the internal Path/grant status may narrow
   from `locality_unclassified` to `fixed_local`.

Acceptance criteria:

- The main crate still forbids unsafe code; the isolated adapter and mandated
  packet enumerate every foreign call and unsafe invariant.
- Windows fixed-local classification passes for the repository drive; remote,
  mapped/substituted, removable, malformed, API-failure, and unknown cases fail
  closed in injected tests.
- Classification performs no candidate metadata/open, network, process,
  environment, registry, or file-content access.
- No Hum output claims general FFI, unsafe, target availability, portable paths,
  or a filesystem sandbox.
- Standing checks pass. Stop. Do not begin Session AD.

## Session AD: hardened exact-file text read

Purpose: read one explicitly granted local file through the AB Path boundary.

Scope:

1. Add exactly:

   ```hum
   files_read_text(path: Path) -> Result Text, FileReadError
   ```

   It requires `files.read` through task/caller/app declarations and an exact
   operator grant whose native path identity matches `path`.
2. A missing task, caller, or app `files.read` declaration is a static blamed
   source diagnostic and prevents runtime. Only after source checks pass does
   operator authority apply: default deny or exact deny returns
   `FileReadError.denied`, while a different exact grant returns
   `FileReadError.outside_grant`. Both occur before host metadata/open and unit
   tests prove zero adapter calls. Deny wins.
3. Revalidate AB's lexical path class and require AC's `fixed_local` result
   before the file adapter call. Non-Windows, unclassified, or non-fixed
   locality returns `FileReadError.unavailable` with no candidate probe. After
   authority succeeds, fail closed on any directory, symlink, junction, or
   Windows reparse component. Deterministic adapter/attribute tests cover
   Windows reparse behavior and every banned UNC/device/volume/ADS/DOS-device
   class; do not rely on silently skipped privileged symlink tests or
   open-then-decide classification.
4. Open only the one file, read at most 1 MiB, and decode strict UTF-8. Return
   exactly named typed variants `FileReadError.denied`,
   `FileReadError.outside_grant`, `FileReadError.unsafe_path`,
   `FileReadError.unavailable`, `FileReadError.not_found`,
   `FileReadError.not_file`, `FileReadError.too_large`,
   `FileReadError.invalid_utf8`, and `FileReadError.io_failed`. Preserve the
   root origin and any W wrapper sites. Map host error kinds deterministically;
   raw host error strings are not the typed identity or a stable diagnostic
   key.
5. Map `files.read` to Core `file` and the already reserved
   `os.filesystem` family. The implementation is the local bootstrap adapter;
   it is not a portable-IO, target-availability, profile-enforcement, or OS
   sandbox claim.
6. No directory enumeration, root grant, glob, recursive read, file write,
   create/delete, current-directory fallback, source path literal, or path
   convenience API. Concurrent attackers or processes mutating a granted path
   during the run remain explicitly outside the alpha threat model; therefore
   this is not a complete filesystem sandbox.
7. Hand-written fixtures cover a successful checked-in UTF-8 file and source
   declaration misuse. Isolated adapter/temp tests cover every typed runtime
   failure without touching user files.

Acceptance criteria:

- The exact checked-in file reads only under all three authority layers; a
  different Path, default deny, and explicit deny perform zero host opens.
- Every typed failure is tested, bounded, causal, and exit 1 rather than a trap.
- Traversal/reparse/symlink/junction, directory, UNC/network, device namespace,
  volume, ADS, DOS-device, mapped-drive, and unknown-locality cases fail closed
  under deterministic tests. No write occurs in any path.
- Human and existing JSON agree for source checks; runtime output makes no JSON
  claim.
- Standing checks pass. Stop. Do not begin Session AE.

## Session AE: integrated local app and retrospective four

Purpose: prove the three capability slices compose without widening their
claims, then close Work Order 6.

Scope:

1. Write one hand-authored executable app whose nested start task:
   - receives one opaque Path from the runner;
   - reads that one exact granted UTF-8 fixture;
   - consumes one runner replay tick;
   - selects a fixed Text literal from the tick rather than adding formatting;
   - writes the file Text and selected literal through bounded stdout; and
   - wraps a missing-file failure in an app error while retaining the complete
     file origin and wrapper/call trail.
2. Identical source, ordinary args, exact file bytes, app/task declarations,
   operator grants/denies, and replay sequence must produce byte-identical
   stdout, stderr, and exit code. A path string alone is not the replay input;
   changed file bytes are a changed input.
3. Keep output composition within shipped surface. Multiple stdout writes and
   tick-selected literals are allowed; Text concatenation, formatting,
   interpolation, UInt conversion, JSON IO, hashing, and evidence-directory
   output are not.
4. Update `docs/bakeoff/SCORECARD.md` and
   `docs/CORE_LANGUAGE_SHAPE.md` from actual evidence:
   - Program 8 is full only if Session V's exact alias misuse remains green;
   - the expected corpus count is 5/12 full plus Program 3 partial;
   - this IO app is adoption evidence, not ownership-corpus coverage;
   - reapply the three-strike rule, including the exact-three Predicate v2
     vocabulary remainder;
   - review every decision 0014 and 0015 honesty lock; and
   - recommend the next work decisively with concrete deferral costs.
5. Record new friction without laundering it into a shipped claim. The
   retrospective must distinguish language gaps, bootstrap-adapter limits,
   threat-model exclusions, and conveniences deliberately deferred.

Acceptance criteria:

- The integrated success runs twice with byte-identical results under identical
  complete inputs. The missing-file run exits 1 and shows the full outer-to-root
  causal trail with relevant source sites.
- Denied stdout, replay, and file grants each fail before the corresponding
  host adapter action; deny continues to win.
- Corpus and ledger counts are exact, no TBD or degenerate substitute is
  counted, and no IO result is mislabeled as ownership coverage.
- The three-strike rule and honesty locks are visibly reapplied.
- Standing checks pass. Stop: Work Order 6 ends. Session AF requires a new work
  order and explicit authorization.

## Deliberate deferrals and concrete cost

- Internal references follow decision 0014's disjoint-field repair but are not
  inside this IO slice. Program 5 and natural self-referential parser state stay
  blocked for all of Work Order 6, at least until Work Order 7.
- Effect polymorphism remains deferred. Programs 3 and 4, retain, closures,
  callbacks, tasks-as-values, and higher-order stdlib remain blocked for all of
  this order, at least until Work Order 7.
- Predicate v2 remains deferred despite its exact-three active vocabulary
  records. Content-conditional count, list content, and text-literal equality
  remain unchecked through AE; AE must decide the next mandated response.
- The Transaction-shaped linear marker remains non-general. Capabilities in
  this order are explicit declarations/grants, not evidence that general linear
  resources exist.
- Replay ticks defer deterministic run mode: no virtual wall clock, seeded
  random, fixed scheduler, stable map iteration, or full trace/replay proof.
- Runner grants defer a real process/OS sandbox, runtime profile enforcement,
  and evidence bundles. Denied code paths are tested, but hostile OS/process
  containment is not claimed.
- Exact UTF-8 file read defers Bytes IO, directories, writes, output
  directories, JSON, hashing, manifests, SBOM/provenance, and the HumGate demo.
  The first real-tool adoption artifact remains incomplete after this order.
- App entry defers modules/packages, multi-file app linkage, service entry,
  cancellation, signals, stdin, environment, and process exit APIs.

## Preserved backlog after this order

1. Full deterministic run mode: virtual clock, seeded random, fixed schedule,
   stable promised iteration, trace, and bit-for-bit replay evidence.
2. Semantic diff (`hum diff`) for effect/contract/capability deltas.
3. Machine-applicable fixes (`hum fix --apply`), still gating the public
   agent-native repair claim. The eval that gates the broader
   agent-native claim now has a researched design (see
   research/2026-07-10-overnight-research-triage.md item 9): staged
   claims, sealed post-cutoff task sets, hidden tests and grader,
   cheat-labeled trajectories, causal diagnostic-utilization metrics,
   ~384-task statistics, Windows-native primary track.
4. Process/OS sandbox enforcement beyond the narrow runner grant flags.
5. Fault containment design before concurrency syntax.
6. Units of measure after type-system maturity.
7. Language editions before public-alpha stability promises.
8. Decision 0015 implementation: contract classifier, build-mode policy,
   proof/evidence fields, and elision only after proof. No global toggle.
9. Predicate v2, grown only from the three active vocabulary records and future
   ledger evidence.
10. List surface beyond append: retain after effect polymorphism, capacity, and
    profile behavior.
11. Numerics policy: checked defaults, explicit families, benchmark gate,
    floating-point regimes, and decimal library-first.
12. Text tiers: Bytes, Text, and OsText, with the AB Path slice not mistaken for
    that full model.
13. Source hardening: bidi-control rejection and explicit ASCII identifier
    policy.
14. Ownership after V: internal references, then broader flow-sensitive
    borrowing; the general linear marker remains ledger-driven.
15. Effect-polymorphism decision for closures, tasks-as-values, retain,
    callbacks, and higher-order stdlib. Bake-off corpus research is
    complete (triage item 2): Effekt-style second-class computations are
    eliminated (stored callbacks and returned closures inexpressible by
    design); the candidates are Koka rows vs Flix formulas vs
    Scala-style capture checking, with capture checking possibly
    underweighted given decision 0017's authority-as-values direction;
    gates are the ten-program corpus, one-diagnostic rejections,
    near-principal inference, and closure capture of owned resources.
16. Canonical spec-of-record demo using existing graph/contract facts, without
    claiming exported enforcement.
17. Stdlib-labs admission when built-ins reach the constitution's evidence
    threshold.
18. Error recovery/narrowing beyond W's propagation and wrapping slice.
19. Entry expansion beyond X's one-file, one-app, direct-child task root.
20. Module paths bound to file paths when multi-file programs land; imports
    remain non-executing.
21. The free canonical Hum book, with chapters only for shipped features and
    all examples extracted from checked fixtures.
22. IO after AD: Bytes, directories, bounded evidence writes, canonical JSON,
    hashing, manifests, and only then the HumGate wedge. The wedge now
    has a researched identity (triage item 3): the air-gapped update
    validator — one command, one verdict, one evidence dossier — whose
    v1 primitive set is nearly what Sessions Z-AD build, whose first
    customer is Hum's own release kit per the toolchain report, and
    whose launch must pass the seven killer-demo tests (under a minute,
    zero network, crisp tamper failure, evidence proving its own limited
    effects, incumbent comparison).
23. Docs anti-drift hardening: extend the preflight so documented claims
    are machine-checked where cheap — backtick-quoted fixture paths in
    docs must exist on disk; the DIAGNOSTICS.md table must stay in sync
    with the diagnostic catalog; scorecard "Runs" rows must name
    fixtures that exist. Prose drift episodes (the Session R mislabel,
    the Program 8 accounting) motivate this; the docs-claims sweep probe
    covers wording, this item covers referents.

## Current authorization gate

Sessions V and W were accepted and committed as `acfb36f` and `2af02ae`.
Decision 0016 is accepted under delegated authority with the BDFL veto open.
Session X was accepted by both reviewers independently (the Codex
architect with zero findings; the cross-family reviewer on behavioral
re-verification of the final hardened tree) and committed as `1605332`.
Session Y was accepted, committed as `d30107d`, pushed by BDFL instruction,
and passed both CI platforms. Session Z was accepted and committed as
`fdffd43`. Decision 0017 is accepted under delegated authority with the BDFL
veto open. CI portability repair `b168a60` passed Ubuntu and Windows CI.
Session AA was accepted, committed as `77f5f88`, and passed Ubuntu and Windows
CI in workflow `29131490535`. Sessions V-AA are therefore accepted and
committed. Session AB was authorized by a separate BDFL go signal; its
implementation is complete and uncommitted pending architect-reviewer review.
Session AC is forbidden.
Publishing remains a BDFL-reserved action under `docs/GOVERNANCE.md`.

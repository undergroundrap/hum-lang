# Ownership and Effect-Polymorphism Bake-Off Scorecard

Date: 2026-07-11
Status: Session AK comparison independently reviewed; decision 0018 accepted
under delegated authority with the BDFL veto open
Corpora: [Ownership corpus](CORPUS.md) and
[effect-polymorphism corpus](EFFECT_POLYMORPHISM_CORPUS.md)

## Purpose

This cumulative scorecard records two separately frozen bake-offs. The
ownership comparison against the twelve-program ownership corpus is evidence
for accepted decision
[0014](../decisions/0014-adopt-ownership-model.md). The effect-polymorphism
comparison against the twelve-case, 29-variant higher-order corpus is evidence
for accepted decision
[0018](../decisions/0018-adopt-effect-polymorphism-model.md). Neither design
comparison by itself claims production implementation.

## Score Key

- Native: accepted by the candidate's ordinary model.
- Repair: accepted only after a named unimplemented repair.
- Restructure: accepted only by changing the shape the user writes.
- Escape: accepted through a declared escape hatch with a named cost.
- Effect gate: depends on tasks-as-values and effect polymorphism.

"Proven today" in this document means design-proven by the candidate rules
without a named future repair, model-specific restructuring, or declared escape
hatch. It does not claim the Rust bootstrap compiler implements the checker.

## Pattern Frequency

| Frequency | Corpus programs |
| --- | --- |
| Pervasive | 3. collection iteration and retain, 9. returned view, 11. record field update |
| Common | 4. callbacks, 5. parser slice, 6. worker handoff, 7. memoizing cache, 8. field swap, 10. transaction, 12. builder finish |
| Occasional | 2. cyclic graph |
| Rare | 1. hand-written doubly linked list |

The frequency weighting matters. A model that is elegant on linked lists but
awkward on returned views and record updates is paying in the wrong place.

## Summary Counts

| Candidate | Proven today clean | With repairs or restructuring, no declared escape | With declared escapes/imported mechanisms | Gate result |
| --- | ---: | ---: | ---: | --- |
| A: ownership and borrowing | 5/12 | 8/12 after effect polymorphism and internal references | 12/12 | Clears the original eight-program gate only as a target model with planned repairs; Session V raises proven-today maturity by implementing Program 8's exact local overlap gate. |
| B: mutable value semantics | 5/12 | 8/12 after effect polymorphism and value restructuring | 12/12 | Clears only by accepting range/copy restructuring as ordinary. That is implementable, but it taxes pervasive view-returning code. |
| C: region-first ownership | 4/12 | 7/12 after effect polymorphism, region tables, and region transfer | 12/12 | Does not clear eight without escape hatches. It is strongest on natural views but pays pervasive signature plumbing. |

Effect-polymorphism note: programs 3 and 4 depend on tasks-as-values. The
counts above apply that interaction equally. Program 3's retain predicate and
program 4's stored callback should not be treated as fully settled until the
effect-polymorphism decision lands.

## 36-Cell Matrix

| # | Frequency | Candidate A | A repair maturity | Candidate B | B repair maturity | Candidate C | C repair maturity |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1. Doubly linked list | Rare | Escape: explicit arena with node ids. Misuse diagnostic blames use of a removed node view. Beginner story is clear, but the user writes container machinery. | Source-visible arena escape; stale-id proof or generation checks still need container design. | Escape: owner table with value handles. Misuse diagnostic blames a consumed or stale handle. Beginner story is clear: handles are tickets, not pointers. | Source-visible owner-table escape; generation checks are runtime or API proof cost. | Escape: region table with node refs. Misuse diagnostic blames removed generation in the list region. Beginner story is workable but more region vocabulary appears. | Source-visible region-table escape; not pure lexical region. |
| 2. Cyclic graph | Occasional | Escape: explicit graph arena. Misuse diagnostic blames node view escaping graph release. Beginner story is clear. | Source-visible arena escape; good fit for graph APIs, not ordinary field references. | Escape: owner table with inert handles. Misuse diagnostic blames using a node handle after graph owner is gone. Beginner story is clear but indirect. | Source-visible owner-table escape. | Native: lexical graph region. Misuse diagnostic blames returning a ref past the region block. Beginner story is excellent: graph region owns all nodes. | Proven by ordinary lexical-region model; one of C's strongest cells. |
| 3. Collection iteration and retain | Pervasive | Native ownership rule for ordinary iteration plus checked retain. Misuse diagnostic blames active iterator or stale retained item view. Beginner story is excellent. | Effect gate applies because retain uses a predicate task value; otherwise no ownership repair. | Native value rule for ordinary iteration plus checked retain. Misuse diagnostic blames conflicting change access during loop. Beginner story is excellent. | Effect gate applies; otherwise proven today. | Native traversal-view rule plus checked retain. Misuse diagnostic blames active traversal view. Beginner story is excellent. | Effect gate applies; otherwise proven today. |
| 4. Callback registry | Common | Escape: linear registration tied to captured state. Misuse diagnostic blames callback outliving `counter`. Beginner story is clear but the token is ceremony. | Linear-resource escape plus effect gate for stored task. | Escape: move mutable state into registry-owned table and consume registration to get it back. Misuse diagnostic rejects storing caller reference. Beginner story is clear but user restructures state ownership. | Owner-table escape plus linear registration plus effect gate. | Escape: dynamic callback region or longer-lived captured region. Misuse diagnostic blames short region capture. Beginner story is clear after regions are learned. | Dynamic-region escape plus effect gate; imported from known region escape hatches. |
| 5. Parser holding slice into own buffer | Common | Native-looking: parser stores `Slice Text from buffer`. Misuse diagnostic blames buffer replacement while token is live. Beginner story is excellent. | Depends on unimplemented internal-reference repair. No lifetime elision is credited. | Restructure: store `TextRange`, re-present the buffer or copy text. Misuse diagnostic rejects storing `Slice Text` in `Parser`. Beginner story is clear, but this is not the natural parser program. | Implementable by value rules; counted as restructuring, not native. Hidden copy-on-write is not credited. | Native region story: parser buffer and token are `in r`. Misuse diagnostic blames replacing with a different region value. Beginner story is good. | Proven only with explicit `in r` signature/type plumbing; any region elision is unimplemented design work. |
| 6. Producer/consumer handoff | Common | Native ownership transfer with `consume buffer`. Misuse diagnostic blames use after send. Beginner story is excellent. | Proven today clean. | Native whole-value move with `consume buffer`. Misuse diagnostic blames use after send. Beginner story is excellent. | Proven today clean. | Escape: unique buffer transferred into worker region. Misuse diagnostic blames use after transfer. Beginner story is clear but destination-region machinery appears. | Unique pointer plus dynamic worker region escape; visible allocation/transfer cost. |
| 7. Memoizing cache | Common | Native `change cache` on miss, return borrow from cache. Misuse diagnostic blames eviction while entry view lives. Beginner story is clear, and mutation is visible. | Proven today clean for single-threaded cache; concurrency not settled. | Native value read if `get` returns owned text. Misuse diagnostic rejects escaping entry handle. Beginner story is clear, but returning values may allocate. | Proven only if allocation is visible. Hidden copy-on-write or hidden reference counts are rejected. | Native cache-region slice. Misuse diagnostic blames evicting while an entry slice is live. Beginner story is good. | Proven with explicit region signatures; concurrent cache sharing remains a future decision. |
| 8. Swapping two fields | Common | Native-looking field-place update. Misuse diagnostic blames overlapping write paths. Beginner story is clear. | Proven for Session V's exact local direct-field writable aliases, straight-line last-use lifetime, distinct direct fields, and H0808 overlap rejection; broader projection remains unimplemented. | Native whole-record exclusive update. Misuse diagnostic rejects stored field alias across `change point`. Beginner story is clear. | Proven today clean, but less precise if unrelated field views should remain live. | Native whole-record update for the swap. Misuse diagnostic rejects stale `point.x` view. Beginner story is clear. | Proven for whole-record update; keeping unrelated field views live needs field-projection repair. |
| 9. Returning view from parameter | Pervasive | Native returned dependency: `Slice Text from text`. Misuse diagnostic rejects returning a view into a local buffer. Beginner story is excellent. | Proven only if explicit `from` dependencies are required; lifetime elision is not credited. | Restructure: return `TextRange` or owned `Text`. Misuse diagnostic rejects returning `Slice Text`. Beginner story is clear, but callers must re-present text or allocate. | Implementable by value rules; counted as restructuring. Hidden copy-on-write is rejected. | Native region-tagged view: input and result are `Slice Text in r`. Misuse diagnostic rejects local-region escape. Beginner story is good. | Proven with explicit `in r` plumbing; region elision is unimplemented design work. |
| 10. Commit or rollback exactly once | Common | Escape: linear transaction consumed by commit or rollback. Misuse diagnostic blames missing or double close path. Beginner story is excellent. | Linear-resource escape; the right tool, but not the base borrow model. | Escape: linear protocol value. Misuse diagnostic blames unconsumed transaction or double consume. Beginner story is excellent. | Linear-protocol escape. | Escape: unique protocol value. Misuse diagnostic blames unconsumed unique transaction. Beginner story is excellent. | Unique-pointer/linear escape; visible proof obligation. |
| 11. Record field update | Pervasive | Native-looking field update preserving rest. Misuse diagnostic blames stale `item.done` view. Beginner story is excellent. | Proven for direct field update, exact local field-view invalidation, and Session V's narrow distinct direct-field handling; nested places and mature general projection remain unimplemented. | Native whole-record exclusive update. Misuse diagnostic blames stale field view after `change item`. Beginner story is clear. | Proven today clean, but syntax sugar for record update is not settled. | Native direct field update for the changed field. Misuse diagnostic blames stale field view. Beginner story is clear. | Proven for direct update; unrelated live field views need field-projection repair. |
| 12. Builder finish | Common | Native append through builder, then `consume builder` on finish. Misuse diagnostic blames use after finish or stale element view after growth. Beginner story is excellent. | Proven today clean; list growth API still needs stdlib design. | Native builder value, append, then consume finish. Misuse diagnostic blames use after finish. Beginner story is excellent. | Proven today clean. | Escape/restructure: build region, then transfer or copy list out. Misuse diagnostic blames use after finish. Beginner story is workable, but return policy is visible. | Region transfer or visible copy is imported mechanism; not pure lexical region. |

## Repair And Signature Maturity

Candidate A's attractive cells are exactly the ones Hum wants long term:
internal parser slices, returned views, and precise field updates. The problem
is maturity. Internal references and broader disjoint-field projections must
become real checker features before Hum can claim A is mature. Session V earns
only the exact local direct-field slice. This scorecard
does not credit lifetime elision; the user writes explicit `from` relationships
until an accepted elision rule exists.

Candidate B is the least speculative checker. Its cost is paid by users in
program shape: ranges instead of views, owned copies instead of returned slices,
and owner tables where direct relationships were desired. That cost lands on
program 9, one of the most frequent patterns in ordinary systems code.

Candidate C makes parser and returned-view programs feel natural without A's
internal-reference repair, but it pays in signatures: `in r` appears on types,
parameters, results, and region operations. This scorecard does not credit
region elision; if C later proposes elision, that is unimplemented design work
and must be judged like A's lifetime repairs.

No candidate may rely on hidden copy-on-write or hidden reference counts. A
copy, allocation, dynamic region, unique pointer, or RC handle must be
source-visible, effect-visible, profile-aware, and allowed by the applicable
standard-library tier. Candidate B's possible `Text` copy-on-write optimization
is therefore not a semantic point in its favor. Candidate C's RC escape remains
acceptable only because it is explicit and profile-gated.

## Frequency-Weighted Takeaway

Candidate B has the best near-term implementation story, but it makes the most
common view-returning APIs feel like coordinate bookkeeping. That is a deep tax
on parsers, string processing, protocol decoders, slices, spans, and debugging
views.

Candidate C has the strongest story for cyclic region-owned data and natural
views, but the signature plumbing would become part of everyday Hum. That risks
making Hum feel like a lifetime-annotation language with friendlier spelling.

Candidate A is the best long-term fit for Hum if the project is willing to
fund the repairs before making safety claims. It keeps the common code shape
natural, preserves explicit ownership and mutation, uses linear resources for
exactly-once protocols, and keeps arenas/regions as source-visible opt-in power
instead of making regions the default mental model.

## Recommendation

Recommend Candidate A as the proposed ownership model for decision 0014:
Rust-like ownership and borrowing designed around Hum's explicit `borrow`,
`change`, `consume`, and `from` relationships, plus linear resources for
exactly-once protocols and explicit arenas/regions as opt-in escape hatches.

This recommendation deliberately moves the proven-today maturity gate. Candidate
A clears eight of twelve only as a target model with planned repairs; today it
has five clean cells under the stricter maturity count. The concession is real:
Hum must not claim full ownership safety, internal references, broad field projection,
or memory-safety completeness until those repairs are implemented and checked by
probe programs. The reason to accept that debt is frequency-weighted ergonomics:
the alternative is making pervasive view and parser code less natural forever.

## Implementation status

Session V updates what the accepted Candidate A path actually implements after
Work Order 5 and the first Work Order 6 repair. "Runs" means the named corpus behavior
exists as a real Hum fixture under `hum run` and, when ownership is involved,
under the relevant checker and pinned misuse fixture. "Partial" means exactly
one required half is implemented: either the positive program runs while its
pinned misuse remains blocked, or the misuse rejects while the positive program
remains blocked. Degenerate, weaker, or restructured fixtures are named as
evidence only; they do not count the corpus behavior as implemented.

Corrected burn-down: Session M counted 1/12 running. Session Q previously
reported 5/12 running plus 1/12 partial, but Program 8 never satisfied its
pinned misuse gate. The corrected Session Q and Session U count is 4/12 fully
running plus 2/12 partial: programs 9, 10, 11, and 12 fully run; program 3 is
misuse-only partial because retain-with-predicate stays blocked by the
effect-polymorphism gate; program 8 is positive-only partial because the direct
swap runs while the overlapping-alias/two-write misuse remains unimplemented.
Work Order 5 added checked stale field/element-view failures and stronger
contracts, but H0802 permission failure and H0807 stale-view failure are not
the Program 8 overlapping-write misuse and do not promote it to fully running.

Session V adds the missing independent evidence. Program 8 alone moves from
positive-only partial to fully running, so the current burn-down is 5/12 fully
running plus Program 3 partial. Programs 8, 9, 10, 11, and 12 fully run;
Program 3 remains misuse-only partial under the unchanged
effect-polymorphism/retain boundary.

| # | Corpus program | Session V status | Evidence today | Missing feature or ban |
| --- | --- | --- | --- | --- |
| 1 | Doubly linked list with back-pointers | Blocked by missing feature | No runnable fixture. | Explicit arena/container API, stale-handle invalidation, and cyclic mutable structure are not implemented. |
| 2 | Cyclic graph freed as a unit | Blocked by missing feature | No runnable fixture. | Graph arena or region lifetime, cyclic handles, and unit release are not implemented. |
| 3 | Mutating a collection while iterating it | Partial: ordinary same-collection mutation rejects; corpus positives and retained-view misuse blocked | `fixtures/ownership_check/session_p_append_during_iteration_fail.hum` rejects ordinary structural append to the same list during active `for each` with H0806. `fixtures/ownership_check/session_s_append_iteration_view_overlap_fail.hum` proves H0806 diagnostic precedence for append with an active element view; it does not prove deletion or stale retained-item-view rejection. `examples/probes/task_list_flow.hum` contains read-only iteration only and is not an exact two-list odd-filter fixture. | No exact two-list odd-filter positive fixture; no retain-style positive deletion; no stale retained-item-view rejection after deletion. Retain's predicate task remains blocked by the effect-polymorphism gate, and deletion/invalidation semantics are unimplemented. |
| 4 | Callback registry capturing caller state | Blocked by ban | No runnable fixture. | Closures, tasks-as-values, stored callbacks, and effect polymorphism are banned until the effect-polymorphism decision lands. |
| 5 | Parser holding a slice into its own buffer | Blocked by missing feature | `fixtures/ownership_check/session_l_return_view_internal_fail.hum` rejects `from parser.buffer` with H0805. | Internal references and buffer/token invalidation are not implemented. |
| 6 | Producer/consumer handoff between workers | Blocked by ban; local transfer subset runs | Session J consume fixtures prove local authority transfer and reject local use after consume. | Worker/concurrency syntax is banned, so no send/receive fixture exists. |
| 7 | Memoizing cache read through a shared path | Blocked by missing feature | No runnable fixture. | Cache/map stdlib, source-visible internal mutation behind read-shaped APIs, and entry-view invalidation are not implemented. |
| 8 | Swapping two fields of one record | Runs | `examples/probes/field_places.hum` still runs the direct swap. `examples/probes/writable_field_aliases.hum` proves real write-through (`{x:9,y:2}`), a two-live-alias swap (`{x:2,y:1}`), direct distinct-field access while the other alias is live (`{x:2,y:7}`), and sequential same-field aliases after last use (`{x:7,y:2}`). The pinned `fixtures/ownership_check/session_v_program8_overlap_write_fail.hum` reports H0808 for the overlapping second write with binding/conflict/last-use facts in human, JSON, and runtime output; direct-read and second-alias fixtures also report H0808, while escape reports H0809. | Only the exact unannotated local `let alias = change owner.field` straight-line slice is implemented. General aliases, nested/element aliases, stored or passed aliases, internal references, and broad flow-sensitive borrowing remain blocked. |
| 9 | Returning a view derived from a parameter | Runs | `examples/probes/first_word.hum` runs `first_word("hum language") -> hum`; graph and ownership JSON expose the dependency from result to `text`; local and non-closed derivations reject with H0805. | Internal references such as `from parser.buffer` remain blocked; the closed derivation set is intentionally tiny. |
| 10 | Transaction that must commit or roll back exactly once | Runs | `examples/probes/transaction_once.hum` returns `ok`; Session K misuse fixtures reject missing close, double close, and one-branch close with H0803/H0804. | The linear-resource marker is still Transaction-shaped rather than source-visible and general. |
| 11 | Updating one record field while preserving the rest | Runs | `examples/probes/field_places.hum` accepts direct `set item.done = true` in `complete_item` under the checked preservation contract `result.title == old(item.title)`, which now runs with zero warnings; the run-only fixture `fixtures/run/session_o_complete_item_field_place.hum` prints `{done: true, title: hum}`; `fixtures/ownership_check/session_r_stale_item_field_view_fail.hum` rejects stale `item.done` views with H0807. | Nested places and general aliases are not implemented; record-update expression syntax remains future ergonomics. |
| 12 | Builder accumulating a growing list then handing it away | Runs | `examples/probes/list_builder.hum` runs `builder_demo() -> [parse, check, run]` under both checked `list_len(result) == 3` and exact ordered-content Predicate v2 contracts; `builder_contract_demo() -> 3` fires a checked predicate contract; add-after-finish rejects with H0801; `fixtures/ownership_check/session_s_stale_element_view_fail.hum` rejects stale `items[0]` views after `list_append` with H0807; `examples/probes/element_views.hum` proves copying the element value survives growth under exact Text equality. | Retain and the broader list stdlib surface are not implemented. |

### Honesty locks after Session Q

Hum now has narrow checked ownership facts for local moves, parameter
permissions, Transaction-shaped exactly-once resources, parameter-derived
returned views through bare returns and closed `slice_until` derivations, direct
field-place mutation, minimal `list_append(change list, item)`, consume-finish
builder handoff, and active-iteration append rejection. These are real, but they
are still narrow slices.

All broad 0014 honesty locks remain. The toolchain must still not claim full
ownership safety, borrow soundness, memory safety, safety-critical readiness,
internal-reference support, general stale field-view invalidation, general stale element-view
invalidation beyond direct list growth, broad disjoint-field projection, broad flow-sensitive borrowing,
concurrency ownership, a mature list standard library, or general linear
resources.

### Repair recommendation after Session Q

Between internal references and stale-view machinery, fund stale-view machinery
first. This is evidence-driven: Work Order 4 made direct field mutation and
minimal list growth real, and the new unresolved ownership records now cluster
around the same missing check: a view of a field or element cannot yet be
created, tracked, invalidated, or blamed after the underlying record/list
changes. That gap touches programs 8, 11, and 12; programs 8 and 11 are common
or pervasive, and program 12 is common. Internal references for parser state
(program 5) remain important, but they are one active corpus blocker and should
build on the same provenance/invalidation discipline rather than precede it.

### Honesty locks after Session U

Two Session Q locks narrow, each by a shipped, fixture-checked feature and
by nothing else: stale field-view invalidation is now checked for local
direct field views (Session R, H0807), and stale element-view invalidation
is now checked for local direct element views across `list_append` growth
(Session S, H0807). The narrowed locks keep their honest remainder: nested
places, general aliases — including Program 8's pinned overlapping-write
case — and views stored beyond the local task remain unchecked and unclaimed.
Predicate v1 (Session T) additionally makes
pre-state (`old(...)`) and length (`list_len`) contracts real, retiring the
golden-value workaround.

Everything else in the Session Q lock list stands unchanged: no full
ownership safety, borrow soundness, memory safety, safety-critical
readiness, internal-reference support, broad disjoint-field projection,
broad flow-sensitive borrowing, concurrency ownership, mature list standard
library, or general linear resources.

### Work Order 6 recommendation after Session U

The three candidates, with what each defers:

1. Internal references (program 5). Ownership is triggered at three active
   records, so its support has not weakened below the threshold. Funding
   internal references would retire one common parser-state blocker and end
   the range-restructuring fallback, but it would not close Program 8's
   overlapping-alias/two-write gate or generalize the Transaction-shaped
   linear marker. Deferral keeps natural self-referential parser state blocked
   for at least one more work order.
2. The effect-polymorphism decision record (programs 3 and 4, closures,
   higher-order stdlib). No three-strike area directly mandates it. Deferral
   keeps the standard library first-order and leaves the Program 3 positive
   retain path and Program 4 callbacks blocked for at least one more work
   order.
3. The first IO capability slice. No friction record can demand absent IO,
   because no current program can attempt it. Adoption evidence nevertheless
   gives it the strongest product pull: the offline-tool wedge, evidence-bundle
   alpha, real utilities, sandboxing flags, deterministic-mode groundwork, and
   backlog items 18 (error chains) and 19 (entry as capability root) all depend
   on it. Deferral keeps every real-tool adoption artifact hypothetical for one
   more work order.

Recommendation: Work Order 6 planning must begin with the narrow Program 8
overlapping-place/two-write alias repair. It is the missing accepted-decision
0014 disjoint-field gate, it blocks a common otherwise-running corpus program,
and the combination of corrected accounting and newly recognized unresolved
evidence restores ownership to the three-strike threshold. Implementing the
repair pays that active record down. H0802 and H0807 may remain adjacent
evidence but cannot substitute for a pinned misuse fixture and a stable
diagnostic explaining why the two writes are not known independent.

After that mandatory ownership gate, the adoption destination remains the first
IO capability slice, with error chaining designed first and entry as the
capability root designed with the slice. Work Order 6 issuance carries the
triggered contract-policy item as
[decision 0015](../decisions/0015-adopt-classified-runtime-contract-policy.md).
That ruling changes no current runtime behavior and leaves the three active
Predicate v2 vocabulary records deliberately deferred through the Work Order 6
Session AE retrospective, where the three-strike rule must be reapplied.
Internal references remain the next ownership repair after Program 8; effect
polymorphism remains an explicit deferral.

### Session V burn-down, ledger effect, and honesty lock

The exact positive and pinned misuse now pass, so Program 8 is fully running.
No other corpus row changes: current coverage is 5/12 full plus Program 3
partial.

Session V resolves only the active Program 8 overlapping-alias/two-write
ownership record. Reapplying the three-strike rule leaves ownership at two
active records—the Transaction-shaped general linear marker and internal
references—so ownership is below the trigger threshold after this repair.
Contracts remain triggered at exactly three active Predicate v2 vocabulary
records. Decision 0015 resolved the separate check-mode policy record without
implementing its future classifier or erasing those vocabulary records; the
classifier stays an honesty lock and backlog item, not a fourth friction record.
The historical Session U trigger and recommendation above remain the evidence
that authorized Session V; they are not the current count.

Decision 0014's lock narrows only for the exact Session V form. Hum may claim
real local direct-field write-through, straight-line last-use enforcement,
definitely distinct direct-field acceptance, H0808 overlap rejection, and H0809
fail-closed unsupported/escape rejection. Hum still may not claim full
ownership safety, borrow soundness, memory-safety completeness, internal
references, general aliases, stored aliases, nested or element aliases, broad
disjoint-field projection, broad flow-sensitive borrowing, concurrency
ownership, mature list semantics, or general linear resources.

### Session AE composition evidence and final burn-down

`examples/probes/integrated_local_app.hum` composes the accepted app boundary,
one runner-owned opaque `Path`, one exact UTF-8 file read, one replay tick, and
bounded stdout. Injected adapter evidence runs the complete app twice with
identical source, native path identity, file bytes, declarations, exact grants,
denies, and replay input; both runs produce the same two output writes, empty
diagnostics, `AppSuccess`, and repeat-stable authority events. A changed tick
selects a different fixed literal, and changed file bytes change the first
write. Missing-file evidence preserves `IntegratedAppError.file` outside
`FileReadError.not_found`, with the wrapper call and root-origin source site.
Exact file, replay, and stdout denies each win over their matching allow and
record zero calls to the denied adapter.

This is adoption evidence, not a thirteenth ownership program and not a repair
for any blocked corpus row. The final Work Order 6 burn-down remains exactly
5/12 fully running plus Program 3 partial. Programs 8, 9, 10, 11, and 12 are
full. Program 8 remains full only because
`fixtures/ownership_check/session_v_program8_overlap_write_fail.hum` still
rejects the pinned overlapping alias/two-write misuse with H0808; neither IO
composition nor a weaker H0802/H0807 fixture can substitute for that evidence.
Program 3 still has ordinary same-collection mutation rejection through H0806,
but still lacks the exact two-list odd-filter positive, retain-style positive
deletion, and stale retained-item-view rejection after deletion. No TBD,
degenerate fixture, hosted-adapter success, or IO behavior changes those counts.

Reapplying the three-strike rule leaves ownership at two active records and
therefore below the trigger: the Transaction-shaped general linear marker and
internal references. Contracts remain triggered at exactly three active
Predicate v2 vocabulary records: conditional content/count for `word_count`,
list content for `builder_demo`, and text-literal equality for `element_views`.
Decision 0015 paid the separate check-mode policy record; it did not implement
or erase these three vocabulary demands.

Decision 0014's honesty lock narrows no further in Session AE. The integrated
app proves capability composition, not general aliases, stored aliases,
internal references, nested or element aliases, broad disjoint-field
projection, broad flow-sensitive borrowing, general linear resources, full
ownership safety, borrow soundness, or memory-safety completeness. Decision
0015's lock also remains complete: every recognized executable predicate still
runs, and Hum has no contract classifier, build-mode contract policy, proof
evidence, elision, or basis for calling a defensive guard unreachable.

### Recommendation after Session AE

Issue a new work order whose first implementation item is the exact Predicate
v2 repair demanded by the triggered ledger: conditional content/count,
list-content predicates, and Text-literal equality, each admitted only with
typed operands and non-degenerate rejection fixtures. Deferring this again
would leave `word_count`'s intended content-conditional relation represented
only by the weaker hard-coded checked equality `result == 2`, while
`builder_demo`'s list-content intent and `element_views`' Text equality remain
unchecked prose. It would also ignore the repository's three-strike rule at
the point specifically reserved for reapplication.

After that bounded payment, run the effect-polymorphism bake-off as the next
architectural decision, using the pinned higher-order corpus rather than
selecting a model from prose. Deferring it keeps Program 3's retain positive,
Program 4, closures, callbacks, tasks-as-values, and higher-order standard
library design blocked. Internal references follow as the next ownership
repair; deferring them keeps Program 5 and natural parser-held views blocked.
The researched air-gapped update-validator wedge should then drive the next
adoption slice, but starting it immediately would either fake or pre-decide
Bytes, directory input, hashing, manifests, JSON evidence, and evidence-output
authority. Deferring that wedge costs a flagship real-tool proof; forcing it
now costs architectural honesty. Predicate v2 first, then the effect bake-off,
is the decisive next sequence.

### Session AF Predicate v2 payment (accepted)

Accepted Session AF, committed as `7991ef6`, paid the exact three triggered
records without changing the ownership corpus score: `word_count` checks its
content-conditional count with contract-only `list_count`, `builder_demo`
checks exact ordered `List Text` content, and `element_views` checks exact Text
equality. Wrong implementations fail H0703, and malformed or ill-typed
executable candidates fail H0704 rather than becoming H0701 or a trap. This
payment adds no effect-polymorphism or ownership credit.

## Session AK effect-polymorphism bake-off

Session AK freezes the accepted Session AG corpus and harness plus the accepted
AH-AJ candidate code. No candidate module, advocate document, corpus row,
relationship origin, neutral result field, normalization rule, inventory, or
cost rule changed during scoring. Each harness-owned factory ran twice from
fresh state; every candidate produced byte-identical canonical output and then
passed the same closed result validator.

### Common exam result

The exam remains exactly 12 cases and 29 variants: 13 positives and 16
misuses. All three candidates accept all 13 positives, reject all 16 misuses,
and produce zero unsupported or incomplete results. Every rejection has one
primary diagnostic, covers all 36 required relationship sites across the 16
misuses, preserves the model-neutral reason and repair, and exposes allocation,
resource, authority, ownership, and unimplemented-machinery facts through the
same contract.

| Case | Positive variants | Misuse variants | Row | Formula | Capture |
| --- | ---: | ---: | --- | --- | --- |
| Pure map | 1 | 1 | complete | complete | complete |
| Effectful map | 1 | 1 | complete | complete | complete |
| Filter/retain | 2 | 2 | complete | complete | complete |
| Fold | 1 | 1 | complete | complete | complete |
| Retry | 1 | 1 | complete | complete | complete |
| Timeout, type-only | 1 | 1 | complete; runtime uncredited | complete; runtime uncredited | complete; runtime uncredited |
| Parallel map, type-only | 1 | 1 | complete; runtime uncredited | complete; runtime uncredited | complete; runtime uncredited |
| Callback registry | 1 | 1 | complete | complete | complete |
| Event-handler factory | 1 | 1 | complete | complete | complete |
| Memoizing wrapper | 1 | 1 | complete | complete | complete |
| Logging middleware | 1 | 1 | complete | complete | complete |
| Linear capture | 1 | 4 | complete with separate guards | complete with separate guards | complete with retention plus separate guards |
| **Total** | **13** | **16** | **13 accepted / 16 rejected** | **13 accepted / 16 rejected** | **13 accepted / 16 rejected** |

Pure Effekt-style second-class computations remain ineligible without
restructuring for the stored callback, returned handler, returned memoizer,
returned logging wrapper, and returned/stored resource-bearing callable. An
around-call helper does not satisfy those frozen relationships, so this family
does not enter the three-way score.

### Exact measured cost

All implementation counts come from the harness-owned checked-in source
inventory. Every candidate has zero dependencies. Analysis totals aggregate
the same 29 results; ranges are per result. Canonical byte count is audit size,
not a weighted score.

| Measurement | Open rows | Boolean formulas | Capture-oriented |
| --- | ---: | ---: | ---: |
| Nonblank/noncomment implementation lines | 1,568 | 1,692 | 2,030 |
| Canonical result bytes | 110,106 | 121,621 | 131,914 |
| Visited nodes, total (range) | 138 (2-7) | 166 (2-9) | 206 (2-11) |
| Generated facts, total (range) | 65 (1-3) | 75 (1-4) | 108 (1-6) |
| Generated constraints, total (range) | 109 (1-6) | 137 (1-8) | 177 (1-10) |
| Normalization steps, total (range) | 98 (2-5) | 116 (2-6) | 119 (2-6) |
| Maximum live items | 1 | 1 | 1 |
| Rejected diagnostic UTF-8 bytes | 4,193 | 4,791 | 4,816 |
| Required/covered misuse sites | 36/36 | 36/36 | 36/36 |
| Positive source annotations | 13 inferred | 13 inferred | 13 inferred |

The bounded stress probes are not treated as a common throughput benchmark.
Rows deterministically normalize 256 labels. Formulas evaluate 4,096 truth
assignments for 12 atoms and fail closed above a 16-atom shared domain.
Capture checking normalizes 12 atoms in the synthetic probe, has a 64-atom
fail-closed bound on every solver path, and measures strict/lazy result
retention. No candidate has production wall-time, whole-program scaling, or a
medium-project throughput claim.

### Allocation, resources, and machinery

Every candidate reports the same neutral allocation domains:
`callable_environment`, `registry_storage`, `cache_storage`, and explicit
`none_explicit`. Every candidate reports the same resource domains, including
iteration/view state, cache state, registration, owned/linear/transaction
resources, and explicit none. Callable environments, timeout execution, and
parallel execution remain unimplemented and uncreditable for all three.

Creditable candidate machinery is deliberately narrow:

- rows: open-row inference/application/handling;
- formulas: the Boolean solver plus associated-requirement constraints; and
- capture: retained-environment solving, result-retention analysis, and hybrid
  requirement facts.

Callable-shape, exact-authority-retention, allocation-visibility, place/view,
registration-lifetime, machinery-credit, and linear-resource guards are
implemented but uncreditable companion checks. The formula candidate also
prices an uncreditable associated-requirement guard. The capture candidate
also prices an uncreditable hybrid-requirement guard. No candidate receives
ownership or authority credit merely for naming an effect, formula, or capture.

### Eleven-criterion comparison

`Clears` means the bounded prototype passes the issued evidence gate. It does
not mean production support exists. `Pressure` records a real implementation
or pedagogy cost without turning it into a hidden numerical weight.

| Criterion | Open rows | Boolean formulas | Capture-oriented | Finding |
| --- | --- | --- | --- | --- |
| 1. Exact coverage without restructuring | Clears 13/16 | Clears 13/16 | Clears 13/16 | Tie; pure Effekt is separately ineligible. |
| 2. Near-principal inference and stability | Clears bounded alpha/order/application tests; 256-label deterministic probe | Clears modulo exact Boolean equivalence only inside the 16-atom bound | Clears bounded substitution/subcapture and strict/lazy tests inside 64 atoms | Rows provide the broadest direct effect evidence with the least bounded machinery; none proves global principal inference. |
| 3. Caller and public-signature burden | All 13 positive annotations inferred; public higher-order boundaries need rows/aliases | All 13 inferred; public boundaries also carry formulas and associated requirements | All 13 inferred; stored/returned boundaries expose captures plus hybrid effects | Ordinary caller tie; rows have the smallest library-author vocabulary. |
| 4. Diagnostic blame and repair | 16 single-cause diagnostics, 36/36 sites, 4,193 bytes; alias-preserving snapshot | 16, 36/36, 4,791 bytes; raw truth vectors excluded | 16, 36/36, 4,816 bytes; raw capture sets excluded | All clear; rows are smallest and retain written aliases. |
| 5. Returned/stored closure expressiveness | Clears every actual frozen shape | Clears every actual frozen shape | Clears every actual frozen shape with explicit result retention | Tie on type-model shape; runtime callable environments remain unimplemented for all. |
| 6. Owned/linear capture safety | Clears only with separate uncreditable ownership/lifetime guards | Same, plus associated pressure | Native retention is strongest, but ownership/lifetime guards still remain separate and uncreditable | No candidate proves ownership soundness; capture supplies the best companion mechanism. |
| 7. Decision 0017 authority integrity | Exact identity and the three policy roles remain separate through a companion guard | Same | Exact retained identity is native, while consent/exercise remain outside captures | All clear the bounded gate; none proves production authority safety. |
| 8. Allocation/resource cost | Complete neutral facts; closure runtime unimplemented | Same | Same, with strict/lazy environment evidence | Tie on required visible cost; no hidden allocation receives credit. |
| 9. Checker complexity and bounded cost | Lowest source and all trace totals; linear 256-label normalization evidence | Higher totals; exact truth-function normalization is exponential and associated machinery is required | Highest source and trace totals; capture plus hybrid effect machinery are both required | Rows win the bounded cost comparison. |
| 10. Core/graph/tooling and migration fit | Direct latent row on callable types, application propagation, exact handling, and preserved aliases fit existing Core effect vocabulary | Requires Boolean formula equivalence and associated requirements across Core/graph/tooling | Requires both retained-environment facts and a separate hybrid effect channel | Rows are the smallest migration spine; all production integration remains unimplemented. |
| 11. Beginner/library pedagogy | One question: what may this callable do? Exact capture/authority remains a separate advanced relationship | Formula algebra is powerful but unnecessary for this corpus and hardest to explain | The useful `does` versus `keeps` distinction is clear, but capture plus hybrid effects is a two-system story | Rows are the paved-road core; capture vocabulary survives for escape/retention diagnostics. No user study exists. |

### Gate result and recommendation

All three candidates clear the mechanical corpus, diagnostic, bounded
near-principal, ownership/resource-capture, and authority-integrity gates only
with their explicitly priced companion guards. Open rows win the actual model
choice because they express every non-capability effect directly, have the
lowest implementation and analysis cost, preserve effect aliases in blame,
and require neither unused Boolean algebra nor a second hybrid effect system.

Recommend an open row-polymorphic latent-effect core: callable application
propagates an open multiset row, and a handler removes exactly one matching
label while preserving unrelated labels and the open tail. This is a proposed
language rule, not production authorization.

Salvage from capture checking:

- explicit retained-environment facts for stored/returned callables;
- strict/lazy result-retention analysis;
- exact authority identity retention without consent inference; and
- ownership/linear-resource escape and lifetime diagnostics.

These are companion checks, not effects laundered into captures, and decision
0014/0017 do not provide them for free.

Salvage from Boolean formulas:

- exact equivalence tests as an experimental verifier;
- domain-language diagnostics rather than solver dumps; and
- targeted exclusion only if a future pinned real API proves that rows plus
  exact handling cannot express the requirement.

Full complement/intersection/difference and associated-effect machinery are
rejected from the paved-road core because the frozen corpus supplies no payoff
for their complexity. Capture-only effects are rejected because retry,
failure, timeout, parallel, state, and wrapped requirements still need a
separate effect system. Pure second-class computations remain rejected as the
core because they cannot represent the frozen first-class callable shapes.

The recommendation does not implement higher-order values, closures, effect
rows, handlers, capture checking, Core/graph facts, runtime environments, or
stdlib APIs. Decision 0018 remains subject to independent review and delegated
ruling; even acceptance would authorize no production implementation.

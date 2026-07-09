# Ownership Bake-Off Scorecard

Date: 2026-07-08
Status: Session Q implementation retrospective appended after Work Order 4
Corpus: [CORPUS.md](CORPUS.md)

## Purpose

This scorecard compares the three ownership candidates against the pinned
twelve-program corpus. It is evidence for decision record
[0014](../decisions/0014-adopt-ownership-model.md), which is now accepted
under delegated authority with the BDFL veto open.

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
| A: ownership and borrowing | 4/12 | 8/12 after effect polymorphism, internal references, and disjoint-field projection | 12/12 | Clears the original eight-program gate only as a target model with planned repairs; does not clear a proven-today maturity gate. |
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
| 8. Swapping two fields | Common | Native-looking field-place update. Misuse diagnostic blames overlapping write paths. Beginner story is clear. | Depends on unimplemented disjoint-field projection. | Native whole-record exclusive update. Misuse diagnostic rejects stored field alias across `change point`. Beginner story is clear. | Proven today clean, but less precise if unrelated field views should remain live. | Native whole-record update for the swap. Misuse diagnostic rejects stale `point.x` view. Beginner story is clear. | Proven for whole-record update; keeping unrelated field views live needs field-projection repair. |
| 9. Returning view from parameter | Pervasive | Native returned dependency: `Slice Text from text`. Misuse diagnostic rejects returning a view into a local buffer. Beginner story is excellent. | Proven only if explicit `from` dependencies are required; lifetime elision is not credited. | Restructure: return `TextRange` or owned `Text`. Misuse diagnostic rejects returning `Slice Text`. Beginner story is clear, but callers must re-present text or allocate. | Implementable by value rules; counted as restructuring. Hidden copy-on-write is rejected. | Native region-tagged view: input and result are `Slice Text in r`. Misuse diagnostic rejects local-region escape. Beginner story is good. | Proven with explicit `in r` plumbing; region elision is unimplemented design work. |
| 10. Commit or rollback exactly once | Common | Escape: linear transaction consumed by commit or rollback. Misuse diagnostic blames missing or double close path. Beginner story is excellent. | Linear-resource escape; the right tool, but not the base borrow model. | Escape: linear protocol value. Misuse diagnostic blames unconsumed transaction or double consume. Beginner story is excellent. | Linear-protocol escape. | Escape: unique protocol value. Misuse diagnostic blames unconsumed unique transaction. Beginner story is excellent. | Unique-pointer/linear escape; visible proof obligation. |
| 11. Record field update | Pervasive | Native-looking field update preserving rest. Misuse diagnostic blames stale `item.done` view. Beginner story is excellent. | Depends on unimplemented disjoint-field projection for mature unrelated-field handling. | Native whole-record exclusive update. Misuse diagnostic blames stale field view after `change item`. Beginner story is clear. | Proven today clean, but syntax sugar for record update is not settled. | Native direct field update for the changed field. Misuse diagnostic blames stale field view. Beginner story is clear. | Proven for direct update; unrelated live field views need field-projection repair. |
| 12. Builder finish | Common | Native append through builder, then `consume builder` on finish. Misuse diagnostic blames use after finish or stale element view after growth. Beginner story is excellent. | Proven today clean; list growth API still needs stdlib design. | Native builder value, append, then consume finish. Misuse diagnostic blames use after finish. Beginner story is excellent. | Proven today clean. | Escape/restructure: build region, then transfer or copy list out. Misuse diagnostic blames use after finish. Beginner story is workable, but return policy is visible. | Region transfer or visible copy is imported mechanism; not pure lexical region. |

## Repair And Signature Maturity

Candidate A's attractive cells are exactly the ones Hum wants long term:
internal parser slices, returned views, and precise field updates. The problem
is maturity. Internal references and disjoint-field projections must become
real checker features before Hum can claim A is implemented. This scorecard
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
has four clean cells under the stricter maturity count. The concession is real:
Hum must not claim full ownership safety, internal references, field projection,
or memory-safety completeness until those repairs are implemented and checked by
probe programs. The reason to accept that debt is frequency-weighted ergonomics:
the alternative is making pervasive view and parser code less natural forever.

## Implementation status

Session Q records what the accepted Candidate A path actually implements after
Work Order 4, through Sessions J-P. "Runs" means the named corpus behavior
exists as a real Hum fixture under `hum run` and, when ownership is involved,
under the relevant checker or misuse fixture. "Partial" means a real corpus
misuse is rejected, but the positive program is still blocked. Degenerate,
weaker, or restructured fixtures are named as evidence only; they do not count
the corpus behavior as implemented.

Burn-down: Session M counted 1/12 running. Session Q counts 5/12 running, plus
1/12 partial. The running programs are 8, 9, 10, 11, and 12. Program 3 is
partial because Hum now rejects the iterator-invalidation misuse, but the
retain-with-predicate positive form remains blocked by the effect-polymorphism
gate.

| # | Corpus program | Session Q status | Evidence today | Missing feature or ban |
| --- | --- | --- | --- | --- |
| 1 | Doubly linked list with back-pointers | Blocked by missing feature | No runnable fixture. | Explicit arena/container API, stale-handle invalidation, and cyclic mutable structure are not implemented. |
| 2 | Cyclic graph freed as a unit | Blocked by missing feature | No runnable fixture. | Graph arena or region lifetime, cyclic handles, and unit release are not implemented. |
| 3 | Mutating a collection while iterating it | Partial: misuse rejected; retain positive path blocked | `fixtures/ownership_check/session_p_append_during_iteration_fail.hum` rejects structural append during active `for each` with H0806; `fixtures/ownership_check/session_s_append_iteration_view_overlap_fail.hum` proves H0806 wins over element-view invalidation on the append line; `examples/probes/task_list_flow.hum` still proves only read-only list iteration. | Retain-style in-place deletion with predicate tasks is blocked by the effect-polymorphism gate. |
| 4 | Callback registry capturing caller state | Blocked by ban | No runnable fixture. | Closures, tasks-as-values, stored callbacks, and effect polymorphism are banned until the effect-polymorphism decision lands. |
| 5 | Parser holding a slice into its own buffer | Blocked by missing feature | `fixtures/ownership_check/session_l_return_view_internal_fail.hum` rejects `from parser.buffer` with H0805. | Internal references and buffer/token invalidation are not implemented. |
| 6 | Producer/consumer handoff between workers | Blocked by ban; local transfer subset runs | Session J consume fixtures prove local authority transfer and reject local use after consume. | Worker/concurrency syntax is banned, so no send/receive fixture exists. |
| 7 | Memoizing cache read through a shared path | Blocked by missing feature | No runnable fixture. | Cache/map stdlib, source-visible internal mutation behind read-shaped APIs, and entry-view invalidation are not implemented. |
| 8 | Swapping two fields of one record | Runs | `examples/probes/field_places.hum` runs `swap_xy({x:1,y:2}) -> {x:2,y:1}`; ownership JSON accepts distinct `point.x` and `point.y` field writes; borrowed field write rejects with H0802; `fixtures/ownership_check/session_r_stale_point_field_view_fail.hum` rejects stale `point.x` views with H0807. | Nested places and general aliases are not implemented. |
| 9 | Returning a view derived from a parameter | Runs | `examples/probes/first_word.hum` runs `first_word("hum language") -> hum`; graph and ownership JSON expose the dependency from result to `text`; local and non-closed derivations reject with H0805. | Internal references such as `from parser.buffer` remain blocked; the closed derivation set is intentionally tiny. |
| 10 | Transaction that must commit or roll back exactly once | Runs | `examples/probes/transaction_once.hum` returns `ok`; Session K misuse fixtures reject missing close, double close, and one-branch close with H0803/H0804. | The linear-resource marker is still Transaction-shaped rather than source-visible and general. |
| 11 | Updating one record field while preserving the rest | Runs | `examples/probes/field_places.hum` accepts direct `set item.done = true` in `complete_item`; the run-only fixture `fixtures/run/session_o_complete_item_field_place.hum` prints `{done: true, title: hum}`; `fixtures/ownership_check/session_r_stale_item_field_view_fail.hum` rejects stale `item.done` views with H0807; H0701 keeps the prose preservation contract visible as unchecked. | Checked pre-state predicates remain unimplemented. |
| 12 | Builder accumulating a growing list then handing it away | Runs | `examples/probes/list_builder.hum` runs `builder_demo() -> [parse, check, run]`; `builder_contract_demo() -> 3` fires a checked predicate contract; add-after-finish rejects with H0801; `fixtures/ownership_check/session_s_stale_element_view_fail.hum` rejects stale `items[0]` views after `list_append` with H0807; `examples/probes/element_views.hum` proves copying the element value survives growth. | Retain and the broader list stdlib surface are not implemented. |

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

The recommended next ownership repair is therefore a narrow stale-view slice:
field and element view provenance plus invalidation on direct field writes and
`list_append`, with misuse fixtures that reject stale use and diagnostics that
name both the view and the invalidating mutation. Internal references should
follow after that slice is green.

Session R update: the field half of that slice is now green for local direct field views.

Session S update: the element half of that slice is now green for local direct numeric element views. `let view = borrow list[0]` is accepted, `list_append` invalidates outstanding element views for that list with H0807, value copies survive growth, and the overlap fixture proves H0806 wins when active iteration and view invalidation meet on the append line. Internal references still follow after the stale-view slice is complete.

The mandated contracts item should be Predicate v1, not contract-check-mode
first. The full friction ledger now has three predicate-expressiveness demands
(collection counts in `word_count`, pre-state references for field preservation,
and list-content/list-shape contracts in `builder_demo`) against one check-mode
demand from `divide`. Predicate v1 should start with pre-state references such
as `old(item.title)` and small collection/list predicates needed by the probes,
while keeping prose contracts visible as H0701 until they enter the checked
grammar. The contract-check-mode ADR remains backlog, but the next work order
must carry the Predicate v1 item explicitly.

# Hum Ownership Bake-Off Corpus

Date: 2026-07-08
Status: pinned for BDFL review

## Purpose

This corpus fixes the twelve small programs used to evaluate Hum's ownership
model candidates. Each entry specifies behavior and required rejections without
choosing how a candidate proves them safe.

Each program should remain small enough to sketch in under about 60 lines of
Hum. Candidate documents may choose different safe formulations, but they must
answer the same behavior, misuse, and success criteria.

## 1. Doubly Linked List With Back-Pointers

Behavior specification:

Build a list of integers. Starting from an empty list, append `10`, append `20`,
insert `15` after the first item, then remove `15`. Forward traversal returns
`[10, 20]`; backward traversal returns `[20, 10]`. Removing an item updates both
neighbor links and the list endpoints when needed.

Why it is hard:

The program isolates connected mutable structure with two-way links. Updating a
middle item touches the removed item, its predecessor, its successor, and
possibly the list head or tail. The model must keep traversal handles from
silently becoming invalid after structural mutation.

Misuse that must be rejected:

Keep a handle to the removed `15` item, remove it from the list, then use that
handle to read or mutate through the old links. A checker must reject the stale
handle use rather than letting a removed item keep authority over the list.

Success criteria for a candidate:

The candidate can express insertion, removal, and forward/backward traversal
without hidden unsafe code in the ordinary path. It rejects stale item handles
after removal and gives a diagnostic that points at the mutation that invalidated
the later use.

## 2. Cyclic Graph Freed As A Unit

Behavior specification:

Create a graph containing nodes `a`, `b`, and `c`. Add edges `a -> b`, `b -> c`,
and `c -> a`. A reachability walk from `a` visits all three nodes once. Releasing
the graph releases all nodes and edges together.

Why it is hard:

The program isolates cycles plus group lifetime. Nodes refer to each other, and
the graph has a single release point even though no node is an obvious leaf.
Handles to nodes must not outlive the graph that makes them meaningful.

Misuse that must be rejected:

Return node `b` from a task that releases the graph before returning, then try to
read `b.name` at the caller. A checker must reject the escaping node handle.

Success criteria for a candidate:

The candidate can express cyclic edges and unit release without per-node manual
teardown in the common case. It rejects node handles that survive the graph's
release point and explains which lifetime boundary was crossed.

## 3. Mutating A Collection While Iterating It

Behavior specification:

Given tasks `[1, 2, 3, 4]`, compute a new list of odd tasks by scanning the input
and adding `1` and `3` to an output list. The original list remains unchanged
during the scan. Also support a retain-style formulation: given the same list,
walk it in place and delete matching even tasks so the final list is `[1, 3]`.

Why it is hard:

The program isolates simultaneous read progress and structural mutation. A loop
cursor depends on the collection's shape, while insertion or removal can change
indexes, item identity, or traversal order.

Misuse that must be rejected:

For the two-list formulation, inside `for each task in tasks`, remove `task`
from `tasks` or append a new item to `tasks`. A checker must reject ordinary
mutation of the collection being iterated.

For the retain-style formulation, keep a view of an item that may be deleted,
delete matching items during traversal, then use the old view after deletion. A
checker must reject the stale item view.

Success criteria for a candidate:

The candidate accepts the two-list formulation and rejects same-collection
mutation during ordinary iteration. It also accepts an explicit retain-style
formulation that deletes matching items in place while preserving traversal
correctness. Diagnostics name either the active ordinary iteration and the
conflicting mutation, or the in-place deletion that invalidated the later item
view.

## 4. Callback Registry Capturing Caller State

Behavior specification:

Create a callback registry. A caller registers a callback that increments its
`count` when the registry fires event `"tick"`. While the caller's state and the
registration are both alive, firing `"tick"` twice changes `count` from `0` to
`2`. Unregistering the callback stops future increments.

Why it is hard:

The program isolates stored behavior that depends on state owned by the caller.
The registry may fire later than registration, so the model must prove the
captured state is still valid whenever the callback can run.

Misuse that must be rejected:

Register a callback that touches a local `count`, return from the task without
unregistering, then fire the registry later. A checker must reject storing a
callback that can outlive the state it touches.

Success criteria for a candidate:

The candidate can express a safe registration whose callable state remains valid
until unregister or registry release. It rejects an escaping callback and reports
that the callback may run after the captured state is gone.

## 5. Parser Holding A Slice Into Its Own Buffer

Behavior specification:

Load text `"kind:42 body"` into a parser. After parsing the header, the parser's
current token is the slice `"kind"` and the remaining input starts at `"42 body"`.
The token view stays valid while the parser keeps the same buffer.

Why it is hard:

The program isolates an object that stores data and also stores a view into that
same data. Moving, replacing, or growing the buffer can invalidate the stored
view unless the model accounts for that relationship.

Misuse that must be rejected:

Keep `current_token`, replace the parser buffer with new input, then read the old
token. A checker must reject the read because the token view depends on the old
buffer.

Success criteria for a candidate:

The candidate can express a parser state or a clear safe restructuring of it. It
rejects buffer replacement while a dependent token view is still usable, with a
diagnostic that names the buffer update and the later token use.

## 6. Producer/Consumer Handoff Between Workers

Behavior specification:

A producer creates a byte buffer `[1, 2, 3]` and sends it to a consumer worker.
The consumer computes checksum `6` and then releases the buffer. After the send,
the producer can report that the send happened but cannot read or write the
buffer contents.

Why it is hard:

The program isolates authority transfer across worker boundaries. Once a buffer
is handed off, the sender and receiver must not both be able to mutate or release
the same storage.

Misuse that must be rejected:

After sending the buffer to the consumer, the producer writes `buffer[0] = 9` or
releases the buffer. A checker must reject use after handoff.

Success criteria for a candidate:

The candidate expresses the send and receive path with exactly one worker able to
touch the buffer at each point. It rejects sender use after handoff and gives a
diagnostic that names the send as the point where authority moved.

## 7. Memoizing Cache Read Through A Shared Path

Behavior specification:

Create a cache whose loader maps `"hum"` to `"language"`. Calling
`get("hum")` the first time computes and stores `"language"`. Calling it a
second time returns the cached `"language"` without calling the loader again.
Two read-only callers can ask for `"hum"` and both observe the same text.

Why it is hard:

The program isolates internal mutation behind a read-shaped API. A cache lookup
looks like observation, but a miss changes the cache. The model must make that
change visible enough for checking without making ordinary memoized reads
unusable. This is the single-threaded proxy for shared-state design; concurrency
sharing rules are decided later.

Misuse that must be rejected:

Return a mutable handle to the cached entry from `get`, keep that handle, then
let another caller evict or replace the same entry. A checker must reject the
conflicting mutable access.

Success criteria for a candidate:

The candidate can express memoization with source-visible authority for the
cache update on misses. It rejects mutable cached-entry handles that can conflict
with later cache updates and reports the shared path that caused the conflict.

## 8. Swapping Two Fields Of One Record

Behavior specification:

Given `Point { x: 1, y: 2 }`, run `swap_xy` and produce
`Point { x: 2, y: 1 }`. The operation changes only `x` and `y`; other fields,
if present in a larger record version, are preserved.

Why it is hard:

The program isolates simultaneous access to two places inside one aggregate. The
model must distinguish distinct fields from overlapping paths into the same
record.

Misuse that must be rejected:

Try to swap `point.x` with another path that can name the same storage at the
same time, such as an alias to `point.x`. A checker must reject overlapping
write access.

Success criteria for a candidate:

The candidate accepts swapping definitely distinct fields of the same record. It
rejects overlapping field paths and explains that the two writes are not known to
be independent.

## 9. Returning A View Derived From A Parameter

Behavior specification:

`first_word("hum language")` returns the view `"hum"` derived from its input
text. The returned view is valid as long as the caller keeps the input text valid
and unchanged in ways that would invalidate the view.

Why it is hard:

The program isolates an output whose validity depends on an input. The task does
not create independent text; it returns a view tied to caller-provided data.

Misuse that must be rejected:

Build a temporary text buffer inside `first_word`, return a view into that local
buffer, then use the view at the caller. A checker must reject returning a view
into data that ends when the task returns.

Success criteria for a candidate:

The candidate can express that the output depends on the input when that is
safe. It rejects returning views into expired local data and reports the local
source that cannot survive the return.

## 10. Transaction That Must Commit Or Roll Back Exactly Once

Behavior specification:

`transfer(10)` begins a transaction, records a debit and a credit, then commits
once and returns `ok`. If recording the credit fails, the transaction rolls back
once and returns the failure. Every path closes the transaction exactly one time.

Why it is hard:

The program isolates exactly-once protocol state across success, failure, and
early return paths. Closing too many times and closing zero times are both bugs.

Misuse that must be rejected:

Start a transaction and return early on an error path without either commit or
rollback. Also reject a path that commits and then rolls back the same
transaction.

Success criteria for a candidate:

The candidate expresses the success and failure paths with a checker-visible
transaction state. It rejects missing close and double close, with diagnostics
that name the path and the transaction action already taken or still required.

## 11. Updating One Record Field While Preserving The Rest

Behavior specification:

Given `WorkItem { title: "write hum", done: false }`, run `complete_item` and
produce `WorkItem { title: "write hum", done: true }`. The title and any future
metadata fields are preserved unless the task explicitly changes them.

Why it is hard:

The program isolates partial update of an aggregate. The language should let the
author state the one intended field change while ensuring the rest of the record
is neither lost nor exposed through conflicting access.

Misuse that must be rejected:

Keep an active view of `item.done`, update `item.done` through the record, then
use the old view as if it still described the current field. A checker must
reject the stale field view.

Success criteria for a candidate:

The candidate can express changing exactly one field while preserving the rest
of the record. It rejects stale field views across the update and gives a
diagnostic that points at the field update.

## 12. Builder Accumulating A Growing List Then Handing It Away

Behavior specification:

Create a builder, add `"parse"`, add `"check"`, add `"run"`, then finish it.
`finish` returns the list `["parse", "check", "run"]`. After finish, the builder
cannot accept more items.

Why it is hard:

The program isolates append growth plus final handoff. Adding items may move the
list's storage, invalidating element views. Finishing transfers the completed
list out of the builder and ends the builder's authority to mutate it.

Misuse that must be rejected:

Keep a view of the first element, append many more items, then use the old view
after growth. Also reject calling `add` after `finish`.

Success criteria for a candidate:

The candidate expresses repeated append and final handoff without hidden unsafe
code in the ordinary path. It rejects stale element views across growth and
rejects builder use after finish, with diagnostics that name the append or
finish operation that invalidated the later use.

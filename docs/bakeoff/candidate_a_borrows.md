# Candidate A: Ownership And Borrowing

Date: 2026-07-08
Status: Session F advocate draft
Corpus: [CORPUS.md](CORPUS.md)

## Core Rules

Candidate A uses owned values as the default story. A value has one owner, moving a value transfers cleanup responsibility, `borrow` observes for a bounded scope, and `change` gives exclusive mutable access for a bounded scope. `shared` is reserved for checked shared-state forms; this document does not settle concurrency sharing rules.

The checker is place based. It reasons about `record.field`, checked indexes, list elements, returned views, and task parameters as places with permissions. A task may return a view derived from an input only when the output records that dependency and the caller keeps the source valid.

This candidate starts with three planned repairs to the classic rough edges: flow-sensitive borrows for conditional returns and lending iterators, disjoint-field projections so separate fields can be changed together, and internal references for values that own storage plus views into that storage. These repairs are part of the model, not later sugar.

Escape valve 1: linear resources. `consume` closes, commits, rolls back, sends, or finishes a resource exactly once. Cost: every path carries a proof obligation, copied handles are forbidden, and diagnostics must explain the missing or duplicate consume.

Escape valve 2: explicit arenas. An `owned Arena T` gives stable node identity for cyclic or back-linked structures and releases the group as a unit. Cost: allocation is explicit, escaping node ids or views carry proof obligations, reusable slots may need runtime generation checks, and any unsafe arena implementation has a named unsafe responsibility hidden behind the checked API.

## 1. Doubly Linked List With Back-Pointers

Code sketch:

```hum
module bakeoff.candidate_a.doubly_linked_list

type NodeId { value: UInt }

type ListNode {
  value: Int
  prev: Maybe NodeId
  next: Maybe NodeId
}

type LinkedList {
  nodes: owned Arena ListNode
  head: Maybe NodeId
  tail: Maybe NodeId
}

task list_demo() -> List Int {
  does:
    change list: LinkedList = linked_list_empty()
    let first: NodeId = linked_list_append(change list, 10)
    let second: NodeId = linked_list_append(change list, 20)
    let middle: NodeId = linked_list_insert_after(change list, first, 15)

    linked_list_remove(change list, middle)

    return linked_list_forward(borrow list)
}
```

Rejecting rule:

Removing a node invalidates outstanding views to that node and its link fields. A `NodeId` may be copied as identity, but reading through it requires a fresh borrow from the current list state.

Diagnostic sketch:

`ownership error: removed node middle cannot be used here; linked_list_remove(change list, middle) invalidated views into that node`

Beginner explanation:

A removed list item is no longer part of the list. You can keep its old number as an id, but you must ask the list again before reading through it.

Escape hatch:

Uses explicit arena. Cost: node allocation and node-id validity proof.

## 2. Cyclic Graph Freed As A Unit

Code sketch:

```hum
module bakeoff.candidate_a.cyclic_graph

type NodeId { value: UInt }

type GraphNode {
  name: Text
  edges: List NodeId
}

type Graph {
  nodes: owned Arena GraphNode
}

task cyclic_graph_demo() -> UInt {
  does:
    change graph: Graph = graph_empty()
    let a: NodeId = graph_add_node(change graph, "a")
    let b: NodeId = graph_add_node(change graph, "b")
    let c: NodeId = graph_add_node(change graph, "c")

    graph_add_edge(change graph, a, b)
    graph_add_edge(change graph, b, c)
    graph_add_edge(change graph, c, a)

    return graph_reachable_count(borrow graph, a)
}
```

Rejecting rule:

A node view is valid only while the owning graph is live. Releasing the graph ends every node view derived from it.

Diagnostic sketch:

`ownership error: node b escapes graph; graph is released before the returned node view is used`

Beginner explanation:

The graph owns its nodes. When the graph goes away, none of its node views can still be trusted.

Escape hatch:

Uses explicit arena. Cost: group allocation and proof that node views do not escape release.

## 3. Mutating A Collection While Iterating It

Code sketch:

```hum
module bakeoff.candidate_a.collection_iteration

task odd_tasks_copy(tasks: borrow List Int) -> List Int {
  does:
    change odds: List Int = []

    for each task in tasks {
      if task % 2 == 1 {
        list_append(change odds, task)
      }
    }

    return odds
}

task keep_odd_task(task: Int) -> Bool {
  does:
    return task % 2 == 1
}

task retain_odd_tasks(change tasks: List Int) {
  does:
    list_retain(change tasks, keep_odd_task)
}
```

Rejecting rule:

Ordinary iteration gives read access to the collection shape and forbids structural mutation of that same collection. Retain-style traversal is a distinct checked operation that owns the cursor and invalidates item views when it deletes.

Diagnostic sketch:

`ownership error: tasks is being iterated here, so append or remove through tasks conflicts with the active iterator`

Beginner explanation:

A normal loop trusts that the collection shape stays stable. If you want to delete while walking, use the checked retain operation that owns that job.

Escape hatch:

None.

## 4. Callback Registry Capturing Caller State

Code sketch:

```hum
module bakeoff.candidate_a.callback_registry

type Counter {
  value: UInt
}

type CallbackRegistry {
  callbacks: List CallbackSlot
}

type Registration {
  slot: CallbackId
}

task increment_counter(change counter: Counter) {
  does:
    set counter.value = counter.value + 1
}

task callback_demo(change registry: CallbackRegistry) -> UInt {
  does:
    change counter: Counter = {value: 0}
    let registration: owned Registration = registry_on_tick(change registry, change counter, increment_counter)

    registry_fire(change registry, "tick")
    registry_fire(change registry, "tick")
    unregister(consume registration, change registry)

    return counter.value
}
```

Rejecting rule:

A stored callback may capture caller state only when the registration cannot outlive that state. The registration must be consumed by unregister before the captured state leaves scope, or the registry must be released first.

Diagnostic sketch:

`ownership error: callback registration may outlive counter; unregister the registration or move owned state into the callback`

Beginner explanation:

The registry is allowed to call back later. Hum needs proof that the state touched by the callback is still alive at that later time.

Escape hatch:

Uses a linear registration resource. Cost: unregister proof obligation on every path.

## 5. Parser Holding A Slice Into Its Own Buffer

Code sketch:

```hum
module bakeoff.candidate_a.parser_slice

type Parser {
  buffer: owned Text
  cursor: UInt
  current_token: Maybe Slice Text from buffer
}

task load_parser(text: owned Text) -> Parser {
  does:
    return {buffer: text, cursor: 0, current_token: none}
}

task parse_header(change parser: Parser) -> Slice Text from parser.buffer {
  does:
    let token: Slice Text from parser.buffer = slice_until(borrow parser.buffer, ":")
    set parser.current_token = some token
    set parser.cursor = token.end + 1
    return token
}
```

Rejecting rule:

Internal views record which owned field they come from. Replacing or moving that field is rejected while a derived view is still usable.

Diagnostic sketch:

`ownership error: parser.buffer cannot be replaced while current_token still borrows from parser.buffer`

Beginner explanation:

The token is a window into the parser buffer. If the buffer changes, the old window no longer points at the same text.

Escape hatch:

None; this uses the planned internal-reference repair.

## 6. Producer/Consumer Handoff Between Workers

Code sketch:

```hum
module bakeoff.candidate_a.worker_handoff

type ByteBuffer {
  bytes: List UInt8
}

type Worker {
  inbox: Queue ByteBuffer
}

task producer_consumer_demo(change consumer: Worker) -> UInt {
  does:
    let buffer: owned ByteBuffer = {bytes: [1, 2, 3]}
    send_to_worker(consume buffer, change consumer)

    return consumer_checksum(change consumer)
}
```

Rejecting rule:

`consume` moves authority to the receiver. After the send, the producer has no readable or writable place for the buffer.

Diagnostic sketch:

`ownership error: buffer was consumed by send_to_worker, so the producer cannot write buffer.bytes[0] here`

Beginner explanation:

Sending the buffer gives it away. The sender can remember that it sent something, but it cannot keep using the sent bytes.

Escape hatch:

None.

## 7. Memoizing Cache Read Through A Shared Path

Code sketch:

```hum
module bakeoff.candidate_a.memo_cache

type Cache {
  entries: Map Text Text
  loads: UInt
}

task get(change cache: Cache, key: Text) -> borrow Text from cache {
  does:
    if map_missing(borrow cache.entries, key) {
      let loaded: Text = load_value(key)
      map_insert(change cache.entries, key, loaded)
      set cache.loads = cache.loads + 1
    }

    return map_borrow(borrow cache.entries, key)
}

task memo_demo(change cache: Cache) -> Bool {
  does:
    let first: borrow Text from cache = get(change cache, "hum")
    let second: borrow Text from cache = map_borrow(borrow cache.entries, "hum")

    return first == second
}
```

Rejecting rule:

A mutable cached-entry handle excludes eviction or replacement of that entry until the handle ends. Shared reads may coexist, but cache mutation must wait for entry views to end.

Diagnostic sketch:

`ownership error: cache entry hum is still borrowed, so evict(change cache, "hum") would invalidate that view`

Beginner explanation:

You may read a cached value while the cache keeps it stable. The cache cannot throw that value away while somebody is still reading through it.

Escape hatch:

None; concurrency sharing remains out of scope for this program.

## 8. Swapping Two Fields Of One Record

Code sketch:

```hum
module bakeoff.candidate_a.swap_fields

type Point {
  x: Int
  y: Int
}

task swap_xy(change point: Point) -> Point {
  does:
    let old_x: Int = point.x
    set point.x = point.y
    set point.y = old_x
    return point
}
```

Rejecting rule:

Disjoint-field projection permits simultaneous access to `point.x` and `point.y` because they are distinct places. Any path that may overlap one of those fields cannot be written at the same time.

Diagnostic sketch:

`ownership error: point.x and alias_to_x may name the same place, so they cannot both be changed in this swap`

Beginner explanation:

Hum lets you change two different fields of one record. It stops you when two names might secretly be the same field.

Escape hatch:

None; this uses the planned disjoint-field repair.

## 9. Returning A View Derived From A Parameter

Code sketch:

```hum
module bakeoff.candidate_a.returned_view

type TextView {
  start: UInt
  end: UInt
}

task first_word(text: borrow Text) -> Slice Text from text {
  does:
    return slice_until(text, " ")
}

task first_word_demo(input: borrow Text) -> Slice Text from input {
  does:
    return first_word(input)
}
```

Rejecting rule:

A returned view may depend on an input parameter, but it may not depend on a local value that ends when the task returns.

Diagnostic sketch:

`ownership error: result borrows local buffer, but buffer ends when first_word returns`

Beginner explanation:

A slice is a window into someone else's text. You can return that window only if the original text will still be there.

Escape hatch:

None.

## 10. Transaction That Must Commit Or Roll Back Exactly Once

Code sketch:

```hum
module bakeoff.candidate_a.transaction_once

type Transaction {
  id: TransactionId
}

type TransferError {
  code: Text
}

task transfer(amount: UInt) -> Result Unit, TransferError {
  does:
    let txn: owned Transaction = begin_transaction()

    if record_debit(change txn, amount) == false {
      rollback(consume txn)
      fail TransferError.debit_failed
    }

    if record_credit(change txn, amount) == false {
      rollback(consume txn)
      fail TransferError.credit_failed
    }

    commit(consume txn)
    return ok unit
}
```

Rejecting rule:

A linear transaction must be consumed exactly once on every path. After commit or rollback, the transaction has no remaining usable state.

Diagnostic sketch:

`ownership error: txn may return from transfer without commit or rollback on this error path`

Beginner explanation:

A transaction is an unfinished promise to the outside world. Hum requires every path to finish it once, either by commit or rollback.

Escape hatch:

Uses linear resources. Cost: every control-flow path carries an exactly-once proof obligation.

## 11. Updating One Record Field While Preserving The Rest

Code sketch:

```hum
module bakeoff.candidate_a.record_update

type WorkItem {
  title: Text
  done: Bool
}

task complete_item(change item: WorkItem) -> WorkItem {
  does:
    set item.done = true
    return item
}
```

Rejecting rule:

Changing a field requires exclusive access to that field and invalidates views of that field. Views of disjoint fields can remain valid if their places are known not to overlap.

Diagnostic sketch:

`ownership error: old_done borrows item.done, so set item.done = true would invalidate old_done`

Beginner explanation:

Changing one field does not erase the rest of the record. It does make old views of that changed field stale.

Escape hatch:

None; this uses disjoint-field projection.

## 12. Builder Accumulating A Growing List Then Handing It Away

Code sketch:

```hum
module bakeoff.candidate_a.builder_finish

type ItemBuilder {
  items: owned List Text
}

task builder_demo() -> List Text {
  does:
    change builder: ItemBuilder = builder_empty()
    builder_add(change builder, "parse")
    builder_add(change builder, "check")
    builder_add(change builder, "run")

    return builder_finish(consume builder)
}
```

Rejecting rule:

Appending may invalidate element views because growth may move storage. Finishing consumes the builder and moves the finished list out, so the builder cannot be used again.

Diagnostic sketch:

`ownership error: builder was consumed by builder_finish, so builder_add cannot use it here`

Beginner explanation:

The builder owns the growing list until finish. After finish, the list has moved out and the old builder is empty authority.

Escape hatch:

None; this is ordinary ownership transfer.

## Self-Score

Legend: clear means the ordinary formulation is accepted under the candidate rules. Clear with escape means the safe formulation uses one of the two declared escape valves and pays the stated cost.

| Program | Safe expression | User writes | Misuse diagnostic | Beginner explanation | Escape hatch | Score |
| --- | --- | --- | --- | --- | --- | --- |
| 1. Doubly linked list | Yes | Arena-backed list with node ids | Clear stale-node blame | Clear | Explicit arena | Clear with escape |
| 2. Cyclic graph | Yes | Arena-backed graph with node ids | Clear graph-release blame | Clear | Explicit arena | Clear with escape |
| 3. Collection iteration | Yes | Ordinary loop or checked retain | Clear iterator or stale-view blame | Clear | None | Clear |
| 4. Callback registry | Yes | Registration token tied to captured state | Clear escaping-callback blame | Clear | Linear registration | Clear with escape |
| 5. Parser slice | Yes | Internal view from owned buffer | Clear buffer-replacement blame | Clear | None | Clear |
| 6. Worker handoff | Yes | `consume` send | Clear use-after-handoff blame | Clear | None | Clear |
| 7. Memoizing cache | Yes | Visible `change cache` on miss | Clear entry-view invalidation blame | Clear | None | Clear |
| 8. Swap fields | Yes | Disjoint field updates | Clear overlapping-place blame | Clear | None | Clear |
| 9. Returned view | Yes | Return view from input | Clear local-view escape blame | Clear | None | Clear |
| 10. Transaction | Yes | Linear transaction consume | Clear missing or double close blame | Clear | Linear resource | Clear with escape |
| 11. Record field update | Yes | Set one field through `change` | Clear stale-field-view blame | Clear | None | Clear |
| 12. Builder finish | Yes | Append through builder, then consume finish | Clear use-after-finish blame | Clear | None | Clear |

Escape-hatch usage count:

- Explicit arena: 2 programs, programs 1 and 2.
- Linear resources: 2 programs, programs 4 and 10.
- No escape hatch: 8 programs, programs 3, 5, 6, 7, 8, 9, 11, and 12.

Candidate A therefore clears 8 of 12 without escape hatches and clears all 12 with declared costs.

## Friction Records

friction:
  program: docs/bakeoff/CORPUS.md:16
  wanted: direct back-pointers that read like ordinary fields
  forced: explicit arena ownership plus node ids for stable linked structure
  severity: awkward
  indicts: ownership
  proposal: provide checked list or graph containers that hide node-id bookkeeping behind safe APIs

friction:
  program: docs/bakeoff/CORPUS.md:45
  wanted: cyclic node handles that feel as direct as ordinary records
  forced: explicit graph arena boundary and proof that node views do not escape release
  severity: awkward
  indicts: ownership
  proposal: make arena-backed graph patterns first-class in docs before exposing lower-level arena details

friction:
  program: docs/bakeoff/CORPUS.md:101
  wanted: store a callback that mutates caller state with no visible lifetime token
  forced: linear registration token that must be unregistered before captured state ends
  severity: verbose
  indicts: ownership
  proposal: design a small subscription pattern that makes callback lifetime rules obvious

friction:
  program: docs/bakeoff/CORPUS.md:129
  wanted: parser owns a buffer and stores a token view into itself directly
  forced: explicit `from buffer` internal-view relationship and field-update restrictions
  severity: awkward
  indicts: ownership
  proposal: keep internal-reference syntax narrow and teach it through parser examples

friction:
  program: docs/bakeoff/CORPUS.md:181
  wanted: memoized `get` to look like a plain read
  forced: source-visible `change cache` because a miss mutates entries and load count
  severity: awkward
  indicts: ownership
  proposal: cache APIs should surface read-or-insert behavior without hiding mutation

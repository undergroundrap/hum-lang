# Candidate B: Mutable Value Semantics

Date: 2026-07-08
Status: Session G advocate draft
Corpus: [CORPUS.md](CORPUS.md)

## Reviewer Notes For Session I

(1) the scorecard gains a repair-maturity column — any "clear" that depends on an unimplemented repair (internal references, disjoint-field projections, flow-sensitive borrows) is marked, and a proven-today clean count is reported alongside the with-repairs count, applied equally to all candidates;
(2) corpus programs 3 and 4 depend on tasks-as-values, which is gated behind the effect-polymorphism decision — each candidate document from Session G onward must include a short closure-capture-rules statement, and Session I must record this corpus interaction.

## Core Rules

Candidate B treats mutable values as the main unit of reasoning. A record, list, builder, cache, or parser is changed through one exclusive `change` parameter at a time. The checker reasons about whole values first, then permits field and element updates only while that whole value is exclusively available.

References are second-class. A `borrow` or `change` access can be passed into a call, used during a loop step, or used to make a temporary view, but it cannot be stored in a record, returned as an owned value, or captured by a stored callback. Long-lived relationships are represented as values: indexes, ranges, handles, registration tokens, or completed owned values.

This model prefers restructuring over lifetime plumbing. Back-pointers become node indexes in an owner value. Self-referential parser state stores ranges, not slices. A callback registry owns callback state slots instead of storing references to caller locals. These are not evasions; they are the model doing its job by keeping values movable and mutation local.

Closure-capture rules: tasks-as-values are admitted only with effect-polymorphism. A stored task may capture immutable copyable data by value, may move owned state into a registry-owned slot, and may receive second-class `change` access only for the dynamic call; it may not store a first-class reference to caller-owned mutable state. The callback effect includes the effects of the captured task and the registry state it can change.

Escape valve 1: owner tables with value handles. An `owned Table T` stores values and returns small handles with index and generation. Cost: allocation is explicit, stale handle rejection needs either static token consumption or a runtime generation check, and APIs must keep the owner value present for every dereference.

Escape valve 2: linear protocol values. `consume` ends a registration, transaction, builder, or transfer exactly once. Cost: every path carries a proof obligation, and APIs must distinguish ordinary copyable values from protocol values.

## 1. Doubly Linked List With Back-Pointers

Code sketch:

```hum
module bakeoff.candidate_b.doubly_linked_list

type NodeHandle {
  index: UInt
  generation: UInt
}

type ListNode {
  value: Int
  prev: Maybe NodeHandle
  next: Maybe NodeHandle
}

type LinkedList {
  nodes: owned Table ListNode
  head: Maybe NodeHandle
  tail: Maybe NodeHandle
}

task list_demo() -> List Int {
  does:
    change list: LinkedList = linked_list_empty()
    let first: NodeHandle = linked_list_append(change list, 10)
    let second: NodeHandle = linked_list_append(change list, 20)
    let middle: owned NodeHandle = linked_list_insert_after(change list, first, 15)

    linked_list_remove(change list, consume middle)

    return linked_list_forward(borrow list)
}
```

Rejecting rule:

Removing a node consumes the handle that authorized that node update and bumps the owner table generation. Any later read must present a current handle and the current list value.

Diagnostic sketch:

`ownership error: middle was consumed by linked_list_remove, so it cannot authorize a later node read`

Beginner explanation:

The list owns all node storage. A handle is a ticket to ask the list for a node, and removing the node spends that ticket.

Escape hatch:

Uses an owner table with value handles. Cost: allocation plus generation checks for stale copied handles.

## 2. Cyclic Graph Freed As A Unit

Code sketch:

```hum
module bakeoff.candidate_b.cyclic_graph

type NodeHandle {
  index: UInt
  generation: UInt
}

type GraphNode {
  name: Text
  edges: List NodeHandle
}

type Graph {
  nodes: owned Table GraphNode
}

task cyclic_graph_demo() -> UInt {
  does:
    change graph: Graph = graph_empty()
    let a: NodeHandle = graph_add_node(change graph, "a")
    let b: NodeHandle = graph_add_node(change graph, "b")
    let c: NodeHandle = graph_add_node(change graph, "c")

    graph_add_edge(change graph, a, b)
    graph_add_edge(change graph, b, c)
    graph_add_edge(change graph, c, a)

    return graph_reachable_count(borrow graph, a)
}
```

Rejecting rule:

A node handle is inert without the graph value that owns the table. Releasing or consuming the graph removes the only authority that can turn handles into node access.

Diagnostic sketch:

`ownership error: b cannot be read after graph is consumed; node handles require the owning graph value`

Beginner explanation:

The handle is not a pointer to memory by itself. It is a label that only works while the graph value exists.

Escape hatch:

Uses an owner table with value handles. Cost: allocation and a generation check when a handle is resolved.

## 3. Mutating A Collection While Iterating It

Code sketch:

```hum
module bakeoff.candidate_b.collection_iteration

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

Ordinary iteration borrows the list shape for the loop body, so changing the same list conflicts. Retain is a library primitive that owns the traversal and invalidates per-item temporary views as it deletes.

Diagnostic sketch:

`ownership error: tasks is borrowed by the for-each loop, so remove(change tasks, task) needs conflicting change access`

Beginner explanation:

A normal loop reads a stable list. If the job is to delete while walking, call the operation designed for that exact mutation.

Escape hatch:

None, but the predicate task depends on the effect-polymorphism gate for tasks-as-values.

## 4. Callback Registry Capturing Caller State

Code sketch:

```hum
module bakeoff.candidate_b.callback_registry

type Counter {
  value: UInt
}

type CallbackRegistry {
  counters: owned Table Counter
  callbacks: List CallbackSlot
}

type CounterRegistration {
  counter: StateHandle
  callback: CallbackId
}

task increment_registered_counter(change counter: Counter) {
  does:
    set counter.value = counter.value + 1
}

task callback_demo(change registry: CallbackRegistry) -> UInt {
  does:
    let counter: owned Counter = {value: 0}
    let registration: owned CounterRegistration = registry_add_counter(change registry, consume counter, increment_registered_counter)

    registry_fire(change registry, "tick")
    registry_fire(change registry, "tick")
    let returned_counter: owned Counter = registry_remove_counter(change registry, consume registration)

    return returned_counter.value
}
```

Rejecting rule:

The registry may store a task and a state handle, but it may not store a reference to a caller local. Mutable captured state is moved into the registry owner table and later moved back by consuming the registration.

Diagnostic sketch:

`ownership error: stored callback cannot capture change counter; move counter into registry state or unregister before counter ends`

Beginner explanation:

The registry can run the callback later, so the state it changes must live where the registry can keep it safe. The caller gets the counter back when it unregisters.

Escape hatch:

Uses an owner table plus a linear registration. Cost: allocation, unregister proof, and tasks-as-values effect tracking.

## 5. Parser Holding A Slice Into Its Own Buffer

Code sketch:

```hum
module bakeoff.candidate_b.parser_range

type TextRange {
  start: UInt
  end: UInt
}

type Parser {
  buffer: owned Text
  cursor: UInt
  current_token: Maybe TextRange
}

task load_parser(text: owned Text) -> Parser {
  does:
    return {buffer: text, cursor: 0, current_token: none}
}

task parse_header(change parser: Parser) -> TextRange {
  does:
    let token: TextRange = range_until(borrow parser.buffer, parser.cursor, ":")
    set parser.current_token = some token
    set parser.cursor = token.end + 1
    return token
}

task current_token_text(parser: borrow Parser) -> Text {
  does:
    return text_copy_range(borrow parser.buffer, parser.current_token)
}
```

Rejecting rule:

The parser stores a range value, not a slice into itself. A second-class slice may be made only during a call that also borrows the parser buffer, and replacing the buffer clears token ranges or requires recomputing them.

Diagnostic sketch:

`ownership error: Slice Text cannot be stored inside Parser; store a TextRange and borrow parser.buffer when text is needed`

Beginner explanation:

The parser keeps coordinates, not a live window into itself. When you need the text, you ask the current parser buffer to copy or view that range.

Escape hatch:

Uses value restructuring from slice to range. Cost: token access must pass through the parser buffer and may copy text at boundaries.

## 6. Producer/Consumer Handoff Between Workers

Code sketch:

```hum
module bakeoff.candidate_b.worker_handoff

type ByteBuffer {
  bytes: List UInt8
}

type Worker {
  inbox: Queue ByteBuffer
}

task producer_consumer_demo(change consumer: Worker) -> UInt {
  does:
    let buffer: owned ByteBuffer = {bytes: [1, 2, 3]}
    worker_send(change consumer, consume buffer)

    return worker_drain_checksum(change consumer)
}
```

Rejecting rule:

Sending consumes the whole buffer value and appends it to the worker inbox. The sender has no remaining variable that can name the bytes.

Diagnostic sketch:

`ownership error: buffer was consumed by worker_send, so buffer.bytes cannot be changed here`

Beginner explanation:

The buffer moves as a whole value. After the send, the consumer owns that value.

Escape hatch:

None.

## 7. Memoizing Cache Read Through A Shared Path

Code sketch:

```hum
module bakeoff.candidate_b.memo_cache

type Cache {
  entries: Map Text Text
  loads: UInt
}

task get(change cache: Cache, key: Text) -> Text {
  does:
    if map_missing(borrow cache.entries, key) {
      let loaded: Text = load_value(key)
      map_insert(change cache.entries, key, loaded)
      set cache.loads = cache.loads + 1
    }

    return map_copy(borrow cache.entries, key)
}

task memo_demo(change cache: Cache) -> Bool {
  does:
    let first: Text = get(change cache, "hum")
    let second: Text = get(change cache, "hum")

    return first == second
}
```

Rejecting rule:

Cache entries are not exposed as first-class mutable references. A read returns a value, and mutation of the cache requires exclusive `change cache` access.

Diagnostic sketch:

`ownership error: cache entry handle cannot escape get; return a value copy or keep mutation inside change cache`

Beginner explanation:

The cache may update itself on a miss, but callers do not get a live handle into its storage. They get a value they can use safely.

Escape hatch:

None; this is the value-semantics paved road, with possible copy-on-write implementation behind `Text`.

## 8. Swapping Two Fields Of One Record

Code sketch:

```hum
module bakeoff.candidate_b.swap_fields

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

Exclusive access to the whole point permits changing both fields. A second-class field access cannot be stored and reused as an alias during the swap.

Diagnostic sketch:

`ownership error: field access to point.x cannot be stored across change point; use the field within the exclusive update`

Beginner explanation:

If you are changing the whole record, Hum knows nobody else can change a field at the same time. Temporary field access ends before the next statement.

Escape hatch:

None.

## 9. Returning A View Derived From A Parameter

Code sketch:

```hum
module bakeoff.candidate_b.returned_range

type TextRange {
  start: UInt
  end: UInt
}

task first_word_range(text: borrow Text) -> TextRange {
  does:
    return range_until(text, 0, " ")
}

task first_word_text(text: borrow Text) -> Text {
  does:
    let range: TextRange = first_word_range(text)
    return text_copy_range(text, range)
}
```

Rejecting rule:

A returned value may describe a range, but it cannot be a stored reference to text. To read the view, the caller must present the source text again, or choose an owned copy.

Diagnostic sketch:

`ownership error: Slice Text is second-class and cannot be returned; return TextRange or owned Text instead`

Beginner explanation:

The range says where the word is. The original text is still needed to read those characters safely.

Escape hatch:

Uses value restructuring from returned slice to returned range or copy. Cost: extra source parameter at use sites or allocation for owned text.

## 10. Transaction That Must Commit Or Roll Back Exactly Once

Code sketch:

```hum
module bakeoff.candidate_b.transaction_once

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

The transaction is a linear value. Every path must consume it exactly once through commit or rollback, and no path may use it after consumption.

Diagnostic sketch:

`ownership error: txn is still owned on this return path; commit or rollback it before returning`

Beginner explanation:

A transaction is a value that must be finished. Hum checks that every route through the task finishes it once.

Escape hatch:

Uses linear protocol values. Cost: path proof for exactly-once consumption.

## 11. Updating One Record Field While Preserving The Rest

Code sketch:

```hum
module bakeoff.candidate_b.record_update

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

Changing the whole item excludes stored field aliases. A field view is second-class and cannot survive across a later `change item` statement.

Diagnostic sketch:

`ownership error: item.done view cannot be used after change item updated that field`

Beginner explanation:

The update keeps the rest of the value because the whole record is still the same value. Old peeks at the changed field are no longer valid.

Escape hatch:

None.

## 12. Builder Accumulating A Growing List Then Handing It Away

Code sketch:

```hum
module bakeoff.candidate_b.builder_finish

type ItemBuilder {
  items: owned List Text
}

task builder_empty() -> ItemBuilder {
  does:
    return {items: []}
}

task builder_add(change builder: ItemBuilder, item: Text) {
  does:
    list_append(change builder.items, item)
}

task builder_finish(consume builder: ItemBuilder) -> List Text {
  does:
    return builder.items
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

Appending requires exclusive access to the builder value and may invalidate temporary element access. `finish` consumes the builder value and returns the completed list.

Diagnostic sketch:

`ownership error: builder was consumed by builder_finish, so builder_add cannot change it here`

Beginner explanation:

The builder is the one value allowed to grow the list. Finishing moves the list out and ends the builder.

Escape hatch:

None.

## Self-Score

Legend: clear means the ordinary formulation is accepted under the candidate rules. Clear with restructure means the program is expressed by value handles, ranges, or owned copies instead of long-lived references. Clear with escape means it uses a declared escape valve with a named cost.

| Program | Safe expression | User writes | Misuse diagnostic | Beginner explanation | Escape hatch | Score |
| --- | --- | --- | --- | --- | --- | --- |
| 1. Doubly linked list | Yes | Owner table plus node handles | Clear consumed-handle or stale-generation blame | Clear | Owner table | Clear with escape |
| 2. Cyclic graph | Yes | Owner table plus node handles | Clear graph-owner-required blame | Clear | Owner table | Clear with escape |
| 3. Collection iteration | Yes | Ordinary loop or checked retain | Clear loop-borrow conflict | Clear | None | Clear |
| 4. Callback registry | Yes | Move state into registry slot | Clear no stored caller-reference blame | Clear | Owner table plus registration | Clear with escape |
| 5. Parser state | Yes | Store range, not internal slice | Clear no stored slice blame | Clear | Range restructuring | Clear with restructure |
| 6. Worker handoff | Yes | Consume whole buffer value | Clear use-after-send blame | Clear | None | Clear |
| 7. Memoizing cache | Yes | Return value from cache get | Clear no escaping entry handle blame | Clear | None | Clear |
| 8. Swap fields | Yes | Exclusive whole-record update | Clear second-class alias blame | Clear | None | Clear |
| 9. Returned view | Yes | Return range or owned copy | Clear no returned slice blame | Clear | Range or copy restructuring | Clear with restructure |
| 10. Transaction | Yes | Linear transaction value | Clear missing or double consume blame | Clear | Linear protocol | Clear with escape |
| 11. Record field update | Yes | Exclusive whole-record update | Clear stale field view blame | Clear | None | Clear |
| 12. Builder finish | Yes | Exclusive builder, consume finish | Clear use-after-finish blame | Clear | None | Clear |

Escape-hatch usage count:

- Owner table with value handles: 3 programs, programs 1, 2, and 4.
- Linear protocol values: 2 programs, programs 4 and 10.
- Value restructuring without declared escape hatch: 2 programs, programs 5 and 9.
- No escape hatch or restructuring: 6 programs, programs 3, 6, 7, 8, 11, and 12.

Candidate B clears 6 of 12 with the pure paved road, 8 of 12 if value restructuring is counted as ordinary model use, and all 12 with declared escape costs. Programs 3 and 4 also depend on the effect-polymorphism gate for tasks-as-values.

## Friction Records

friction:
  program: docs/bakeoff/CORPUS.md:16
  wanted: direct linked nodes with back-pointers
  forced: owner table with handles and generation checks
  severity: verbose
  indicts: ownership
  proposal: provide a standard linked structure API if this model wins

friction:
  program: docs/bakeoff/CORPUS.md:45
  wanted: graph nodes that refer to each other directly
  forced: graph-owned table and inert node handles
  severity: awkward
  indicts: ownership
  proposal: make graph ownership patterns library-first, not user-rebuilt

friction:
  program: docs/bakeoff/CORPUS.md:101
  wanted: callback mutates caller local state while registry stores the callback
  forced: move mutable state into registry-owned storage and later move it back
  severity: awkward
  indicts: ownership
  proposal: design subscription/state-slot sugar only after effect polymorphism is settled

friction:
  program: docs/bakeoff/CORPUS.md:129
  wanted: parser stores a live slice into its own buffer
  forced: store a range value and rematerialize or copy text on demand
  severity: awkward
  indicts: ownership
  proposal: parser APIs should make range-plus-source ergonomics excellent if this model wins

friction:
  program: docs/bakeoff/CORPUS.md:228
  wanted: return a live view derived from a parameter
  forced: return a range descriptor or allocate an owned copy
  severity: awkward
  indicts: ownership
  proposal: decide whether range values are acceptable as the normal slice-return story

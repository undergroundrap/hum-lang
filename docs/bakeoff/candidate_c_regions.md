# Candidate C: Region-First Ownership

Date: 2026-07-08
Status: Session H advocate draft
Corpus: [CORPUS.md](CORPUS.md)

## Reviewer Notes For Session I

(1) the scorecard gains a repair-maturity column — any "clear" that depends on an unimplemented repair (internal references, disjoint-field projections, flow-sensitive borrows) is marked, and a proven-today clean count is reported alongside the with-repairs count, applied equally to all candidates;
(2) corpus programs 3 and 4 depend on tasks-as-values, which is gated behind the effect-polymorphism decision — each candidate document from Session G onward must include a short closure-capture-rules statement, and Session I must record this corpus interaction.
(3) the scorecard must present proven-today and with-repairs/with-restructure clean counts side by side, and the ADR must explicitly answer the philosophical axis this exposes — does Hum prefer growing language complexity so natural programs check (candidate A), or restructuring programs so the language stays simple (candidate B);
(4) add a pattern-frequency row to the scorecard — how often each program's shape occurs in ordinary systems code — because returning-a-view (program 9) and record update (11) are pervasive while linked lists (1) are rare, and the recommendation must weigh frequency, not just the 12-program count;
(5) any candidate mechanism relying on copy-on-write or hidden reference counts must be reconciled with the constitution's no-invisible-allocation rule or rejected.
(6) candidate C's region parameters (in r) appear in type and task signatures pervasively — the "user writes" column must count signature plumbing for all candidates, and any assumed elision rules (region elision for C, lifetime elision for A) must be marked as unimplemented design work in the repair-maturity column.

## Core Rules

Candidate C makes named regions the primary ownership fact. Allocation happens in a region, values and views carry the region they belong to, and a region can be released only after every value or view tied to it has ended. The ordinary rule is simple: if a value mentions `in r`, it cannot outlive `r`.

The checker reasons about region containment before per-value ownership. A task that returns a view says which input region it comes from. A task that builds a cyclic structure places all nodes in one named region and releases the region as a unit.

This candidate deliberately absorbs the Cyclone and MLKit postmortem. Pure lexical regions were not enough: Cyclone needed dynamic regions, unique pointers, and RC, while MLKit had to combine inference with GC-like escape paths and corrected earlier GC-safety claims. Hum therefore does not pretend lexical regions alone are the ergonomic answer; it imports the escape hatches explicitly and prices them.

Escape hatch 1: dynamic regions. A `dynamic region` is a runtime-checked region handle for lifetimes not expressible as a simple lexical block. Cost: allocation is visible, each access may need a liveness check, release timing can add jitter, and strict profiles may forbid it.

Escape hatch 2: unique pointers. `unique T` owns one heap value outside a bulk region and can be consumed or moved exactly once. Cost: allocation and deallocation are explicit, every path carries a proof obligation, and the type is less compositional than a region-owned value.

Escape hatch 3: explicit RC. `rc T` is an explicit reference-counted owner, not hidden copy-on-write. Cost: count increments and decrements are visible effects, deallocation timing is data dependent, atomic RC has synchronization cost, and `std.core` or no-heap profiles may reject it under the no-hidden-allocation rule.

Closure-capture rules: tasks-as-values are admitted only with effect-polymorphism. A stored task may capture values whose regions outlive the registration, may move state into a registry-owned region, or may use explicit `rc` state; it may not capture a reference into a region that can end before the callback registration. The callback effect includes its captured task effects and the region or RC state it can touch.

## 1. Doubly Linked List With Back-Pointers

Code sketch:

```hum
module bakeoff.candidate_c.doubly_linked_list

type NodeRef in r {
  slot: UInt
  generation: UInt
}

type ListNode in r {
  value: Int
  prev: Maybe NodeRef in r
  next: Maybe NodeRef in r
}

type LinkedList in r {
  nodes: RegionTable ListNode in r
  head: Maybe NodeRef in r
  tail: Maybe NodeRef in r
}

task list_demo() -> List Int {
  does:
    region list_region {
      change list: LinkedList in list_region = linked_list_empty(in list_region)
      let first: NodeRef in list_region = linked_list_append(change list, 10)
      let second: NodeRef in list_region = linked_list_append(change list, 20)
      let middle: NodeRef in list_region = linked_list_insert_after(change list, first, 15)

      linked_list_remove(change list, middle)

      return linked_list_forward(borrow list)
    }
}
```

Rejecting rule:

A node reference cannot outlive its region, and removal invalidates the node generation inside that region. Reading through an old `NodeRef` after removal requires a current generation proof and is rejected if the slot was removed.

Diagnostic sketch:

`ownership error: middle names a removed node in list_region; linked_list_remove invalidated that generation`

Beginner explanation:

The list region owns every node. A node reference only works while the region and the node generation are still valid.

Escape hatch:

Uses region table generations. Cost: region allocation plus runtime or checker-visible generation validation for removed nodes.

## 2. Cyclic Graph Freed As A Unit

Code sketch:

```hum
module bakeoff.candidate_c.cyclic_graph

type NodeRef in r {
  slot: UInt
}

type GraphNode in r {
  name: Text
  edges: List NodeRef in r
}

type Graph in r {
  nodes: RegionTable GraphNode in r
}

task cyclic_graph_demo() -> UInt {
  does:
    region graph_region {
      change graph: Graph in graph_region = graph_empty(in graph_region)
      let a: NodeRef in graph_region = graph_add_node(change graph, "a")
      let b: NodeRef in graph_region = graph_add_node(change graph, "b")
      let c: NodeRef in graph_region = graph_add_node(change graph, "c")

      graph_add_edge(change graph, a, b)
      graph_add_edge(change graph, b, c)
      graph_add_edge(change graph, c, a)

      return graph_reachable_count(borrow graph, a)
    }
}
```

Rejecting rule:

All node references are tagged with `graph_region`. Returning a node reference beyond the region block is rejected because the region release frees every node at once.

Diagnostic sketch:

`ownership error: node b is in graph_region, but graph_region ends before b can be used by the caller`

Beginner explanation:

The graph region is the box that contains all graph nodes. When the box is thrown away, no node label from it can be used.

Escape hatch:

None for lexical-region graphs; cycles are the native strength of this candidate.

## 3. Mutating A Collection While Iterating It

Code sketch:

```hum
module bakeoff.candidate_c.collection_iteration

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

Ordinary iteration creates a traversal view tied to the collection shape. Structural mutation of that same collection is rejected unless the mutation is inside a checked retain traversal that owns the cursor.

Diagnostic sketch:

`ownership error: tasks has an active traversal view, so remove(change tasks, task) cannot change its shape here`

Beginner explanation:

A normal loop expects the list shape to stay put. Retain is the explicit operation for changing the list while walking it.

Escape hatch:

None, but the predicate task depends on the effect-polymorphism gate for tasks-as-values.

## 4. Callback Registry Capturing Caller State

Code sketch:

```hum
module bakeoff.candidate_c.callback_registry

type Counter {
  value: UInt
}

type CallbackRegistry in r {
  callbacks: List CallbackSlot in r
  counters: RegionTable Counter in r
}

type Registration in r {
  slot: CallbackId
  counter: RegionRef Counter in r
}

task increment_counter(change counter: Counter) {
  does:
    set counter.value = counter.value + 1
}

task callback_demo() -> UInt {
  does:
    dynamic region callback_region {
      change registry: CallbackRegistry in callback_region = registry_empty(in callback_region)
      let registration: Registration in callback_region = registry_add_counter(change registry, {value: 0}, increment_counter)

      registry_fire(change registry, "tick")
      registry_fire(change registry, "tick")
      let counter: Counter = registry_take_counter(change registry, registration)

      return counter.value
    }
}
```

Rejecting rule:

A stored callback may capture only state whose region outlives the registration. Caller-local mutable state cannot be captured unless it is moved into the callback region, or the registration is proven to end first.

Diagnostic sketch:

`ownership error: callback captures counter from caller region, but registry may fire after that region ends`

Beginner explanation:

The registry can call later, so the state it changes must live in the registry region or another longer-lived owner. A local variable is too short-lived unless you unregister before it ends.

Escape hatch:

Uses dynamic regions. Cost: explicit allocation, possible runtime liveness checks, and effect-polymorphic tasks-as-values.

## 5. Parser Holding A Slice Into Its Own Buffer

Code sketch:

```hum
module bakeoff.candidate_c.parser_slice

type Parser in r {
  buffer: Text in r
  cursor: UInt
  current_token: Maybe Slice Text in r
}

task load_parser(in parser_region: region, text: Text) -> Parser in parser_region {
  does:
    return {buffer: region_copy(parser_region, text), cursor: 0, current_token: none}
}

task parse_header(change parser: Parser in r) -> Slice Text in r {
  does:
    let token: Slice Text in r = slice_until(borrow parser.buffer, ":")
    set parser.current_token = some token
    set parser.cursor = token.end + 1
    return token
}
```

Rejecting rule:

The token slice and parser buffer share the same region. Replacing the buffer with text from another region is rejected while `current_token` still points into the old region allocation.

Diagnostic sketch:

`ownership error: current_token is a slice in parser_region, so parser.buffer cannot be replaced with a different region value here`

Beginner explanation:

The token and buffer live in the same region. If you swap the buffer out, the old token would point at the wrong storage.

Escape hatch:

None for a parser-owned lexical region; dynamic parser buffers would use a dynamic region with liveness checks.

## 6. Producer/Consumer Handoff Between Workers

Code sketch:

```hum
module bakeoff.candidate_c.worker_handoff

type ByteBuffer in r {
  bytes: List UInt8 in r
}

type Worker {
  inbox_region: dynamic region
}

task producer_consumer_demo(change consumer: Worker) -> UInt {
  does:
    region producer_region {
      let buffer: unique ByteBuffer in producer_region = make_buffer(producer_region, [1, 2, 3])
      worker_send(change consumer, consume buffer)
    }

    return worker_drain_checksum(change consumer)
}
```

Rejecting rule:

Sending consumes the unique buffer authority and transfers its storage into the worker inbox region. The producer region no longer contains a usable buffer after the send.

Diagnostic sketch:

`ownership error: buffer was consumed by worker_send, so producer code cannot write buffer.bytes here`

Beginner explanation:

The buffer moves from the producer to the worker. After the move, only the worker region can touch it.

Escape hatch:

Uses unique pointers plus a dynamic worker region. Cost: explicit transfer proof, visible allocation, and runtime liveness policy for the worker inbox.

## 7. Memoizing Cache Read Through A Shared Path

Code sketch:

```hum
module bakeoff.candidate_c.memo_cache

type Cache in r {
  entries: Map Text Text in r
  loads: UInt
}

task get(change cache: Cache in r, key: Text) -> Slice Text in r {
  does:
    if map_missing(borrow cache.entries, key) {
      let loaded: Text = load_value(key)
      map_insert(change cache.entries, key, region_copy(r, loaded))
      set cache.loads = cache.loads + 1
    }

    return map_slice(borrow cache.entries, key)
}

task memo_demo(change cache: Cache in r) -> Bool {
  does:
    let first: Slice Text in r = get(change cache, "hum")
    let second: Slice Text in r = map_slice(borrow cache.entries, "hum")

    return first == second
}
```

Rejecting rule:

Entry views are tied to the cache region and the cache map version. Eviction or replacement of an entry is rejected while an entry slice from the same map version is still live.

Diagnostic sketch:

`ownership error: cache entry hum has a live slice in cache_region, so evict(change cache, "hum") would invalidate it`

Beginner explanation:

The cache can hand out a view into its region, but it must keep that entry stable while the view is alive. Deleting the entry first would make the view stale.

Escape hatch:

None for a single-threaded region cache; shared concurrent caches would need explicit RC or a future sharing decision.

## 8. Swapping Two Fields Of One Record

Code sketch:

```hum
module bakeoff.candidate_c.swap_fields

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

Exclusive access to the record permits changing both fields. A field view into the record cannot be used after `change point` mutates the record unless the checker proves the view names an unaffected field.

Diagnostic sketch:

`ownership error: alias_to_x is a live view of point.x, so changing point.x during swap would invalidate it`

Beginner explanation:

The whole point is being changed, so old views of its fields are suspicious. Hum allows the swap when it owns the record update.

Escape hatch:

None for whole-record update; preserving unrelated field views would need a field-projection repair tracked in Session I.

## 9. Returning A View Derived From A Parameter

Code sketch:

```hum
module bakeoff.candidate_c.returned_view

task first_word(text: Slice Text in r) -> Slice Text in r {
  does:
    return slice_until(text, " ")
}

task first_word_demo(input: Slice Text in r) -> Slice Text in r {
  does:
    return first_word(input)
}
```

Rejecting rule:

A returned slice carries the same region as its input. Returning a slice into a local region is rejected because that region ends when the task returns.

Diagnostic sketch:

`ownership error: result is in local_region, but local_region ends before the caller can use the returned slice`

Beginner explanation:

The returned word view is safe when it points into text the caller still owns. It is not safe when it points into temporary text made inside the task.

Escape hatch:

None; region-tagged views are the native strength of this candidate.
## 10. Transaction That Must Commit Or Roll Back Exactly Once

Code sketch:

```hum
module bakeoff.candidate_c.transaction_once

type Transaction {
  id: TransactionId
}

type TransferError {
  code: Text
}

task transfer(amount: UInt) -> Result Unit, TransferError {
  does:
    let txn: unique Transaction = begin_transaction()

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

A transaction is a unique protocol value, not merely region memory. Every path must consume it exactly once through commit or rollback before its scope ends.

Diagnostic sketch:

`ownership error: txn reaches return still unique and unconsumed; commit or rollback it on this path`

Beginner explanation:

The transaction has to be finished. Hum checks that each path spends it exactly once.

Escape hatch:

Uses unique pointers as a linear protocol escape hatch. Cost: path proof and explicit resource finalization.

## 11. Updating One Record Field While Preserving The Rest

Code sketch:

```hum
module bakeoff.candidate_c.record_update

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

Changing a field invalidates views of that field. Region membership does not make a stale field view current after mutation.

Diagnostic sketch:

`ownership error: old_done views item.done before set item.done = true, so it cannot be used as the current field value`

Beginner explanation:

The item stays in the same place, and its title is preserved. The old view of the field that changed is no longer current.

Escape hatch:

None for direct field update; keeping unrelated field views live would need field-projection maturity tracked in Session I.

## 12. Builder Accumulating A Growing List Then Handing It Away

Code sketch:

```hum
module bakeoff.candidate_c.builder_finish

type ItemBuilder in r {
  items: List Text in r
}

task builder_demo() -> List Text {
  does:
    region build_region {
      change builder: ItemBuilder in build_region = builder_empty(in build_region)
      builder_add(change builder, region_copy(build_region, "parse"))
      builder_add(change builder, region_copy(build_region, "check"))
      builder_add(change builder, region_copy(build_region, "run"))

      return builder_finish(consume builder)
    }
}
```

Rejecting rule:

Appending may move list storage inside the build region, so element views are invalidated across growth. Finishing consumes the builder and transfers or copies the completed list out of the build region according to the return policy.

Diagnostic sketch:

`ownership error: builder was consumed by builder_finish, so builder_add cannot use build_region storage through it here`

Beginner explanation:

The builder owns the growing list inside a build region. Finish is the moment that hands the completed list away.

Escape hatch:

Uses region transfer on finish. Cost: either region ownership moves to the caller, or the list is copied into the caller region with visible allocation.

## Self-Score

Legend: clear means the ordinary formulation is accepted under the candidate rules. Clear with escape means the program uses dynamic regions, unique pointers, or explicit RC with a named cost. Clear with repair means the sketch depends on an unimplemented checker repair that Session I must mark in the repair-maturity column.

| Program | Safe expression | User writes | Misuse diagnostic | Beginner explanation | Escape hatch | Score |
| --- | --- | --- | --- | --- | --- | --- |
| 1. Doubly linked list | Yes | Region table with node refs | Clear removed-generation blame | Clear | Region table generations | Clear with escape |
| 2. Cyclic graph | Yes | Lexical graph region | Clear region-escape blame | Clear | None | Clear |
| 3. Collection iteration | Yes | Ordinary loop or checked retain | Clear traversal-view conflict | Clear | None | Clear |
| 4. Callback registry | Yes | Dynamic callback region | Clear short-region capture blame | Clear | Dynamic region | Clear with escape |
| 5. Parser slice | Yes | Parser-owned region and same-region slice | Clear region replacement blame | Clear | None | Clear |
| 6. Worker handoff | Yes | Unique buffer into worker region | Clear use-after-transfer blame | Clear | Unique plus dynamic region | Clear with escape |
| 7. Memoizing cache | Yes | Cache-region entry slices | Clear live-entry-slice blame | Clear | None | Clear |
| 8. Swap fields | Yes | Whole-record update | Clear stale field view blame | Clear | Field projection for unrelated views | Clear with repair |
| 9. Returned view | Yes | Region-tagged returned slice | Clear local-region escape blame | Clear | None | Clear |
| 10. Transaction | Yes | Unique protocol value | Clear unconsumed transaction blame | Clear | Unique pointer | Clear with escape |
| 11. Record field update | Yes | Direct field update | Clear stale field view blame | Clear | Field projection for unrelated views | Clear with repair |
| 12. Builder finish | Yes | Build region, consume finish | Clear use-after-finish blame | Clear | Region transfer or copy | Clear with escape |

Escape-hatch usage count:

- Dynamic regions: 2 programs, programs 4 and 6.
- Unique pointers: 2 programs, programs 6 and 10.
- Explicit RC: 0 programs in this corpus; imported only for future shared-state or long-lived shared ownership cases, and not allowed to hide allocation or reference count effects.
- Region transfer/copy: 1 program, program 12.
- Region table generations: 1 program, program 1.
- Unimplemented repair marker: 2 programs, programs 8 and 11, for keeping unrelated field views live across whole-record mutation.
- No escape hatch: 5 programs, programs 2, 3, 5, 7, and 9 are native region wins; program 3 still depends on the effect-polymorphism gate for tasks-as-values.

Candidate C clears 5 of 12 with the pure lexical-region paved road if program 3's task predicate is counted after effect polymorphism, 7 of 12 with ordinary region-table and region-transfer mechanisms, and all 12 with imported escape hatches or repairs. It must not rely on hidden copy-on-write or hidden reference counts; any RC use is an explicit type with visible allocation and count effects.

## Friction Records

friction:
  program: docs/bakeoff/CORPUS.md:16
  wanted: direct linked nodes with simple references
  forced: region table refs plus generation validation to reject removed nodes
  severity: awkward
  indicts: ownership
  proposal: if region-first wins, standard region containers need first-class stale-handle diagnostics

friction:
  program: docs/bakeoff/CORPUS.md:101
  wanted: callback stores caller-local mutable state
  forced: dynamic callback region or longer-lived captured region
  severity: awkward
  indicts: ownership
  proposal: defer callback ergonomics until effect polymorphism and region capture rules are designed together

friction:
  program: docs/bakeoff/CORPUS.md:148
  wanted: send a producer buffer without thinking about destination region lifetime
  forced: unique transfer into a dynamic worker region
  severity: verbose
  indicts: ownership
  proposal: worker/channel APIs must hide region transfer machinery behind checked source-visible calls

friction:
  program: docs/bakeoff/CORPUS.md:304
  wanted: finish a builder and return the list without choosing copy or region move
  forced: explicit region transfer or visible allocation into caller region
  severity: awkward
  indicts: ownership
  proposal: decide whether region-moving return values are beginner-visible or library-internal

friction:
  program: docs/bakeoff/CORPUS.md:181
  wanted: shared cache behavior without settling shared-state ownership
  forced: single-threaded cache-region slices only; concurrent sharing would need explicit RC or a future sharing model
  severity: awkward
  indicts: ownership
  proposal: route shared caches to the future concurrency sharing decision, not the ownership ADR alone

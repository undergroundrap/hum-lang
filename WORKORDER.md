# Hum Work Order: Milestone 1 Vertical Slice

Date: 2026-07-08
Status: active, BDFL-approved
Owner: BDFL (Ocean). Reviewer: external architect pass. Implementer: agent sessions.

## Why this document exists

The design corpus is finished enough. The project's own gate in
`docs/FORMAL_CORE.md` ("First Executable Slice") is now the only work that
matters. This work order turns that gate into concrete sessions with
acceptance criteria. Rules of engagement are in `AGENTS.md` under
"Operating Rules"; read those first. Newly accepted decisions:
`docs/decisions/0012-adopt-snake-case-identifiers.md` and
`docs/decisions/0013-remove-number-type.md`.

Relationship to decision 0003 (Milestone 0 non-executing): that decision
governs Milestone 0 surfaces and still stands. This work order begins
Milestone 1. `hum run <file>` executing a user-named local `.hum` file is in
scope. Executing build scripts, packages, generated code, network access, or
foreign code remains out of scope.

## Global bans (all sessions)

- No new `docs/*.md` files. Edit existing docs where they become stale.
- No new `hum.*.v0` schemas, report subcommands, or pipeline gates.
- No new "readiness"/"preview"/"contract" surfaces of any kind.
- No async, generics, closures, borrowing, FFI, backends, or profiles work.
- Do not refactor the 13-gate pipeline yet; leave it alone unless a session
  below requires touching it.

## Session A: identifier migration (decision 0012) and Number removal (decision 0013)

Scope:

1. Pin the identifier grammar in the lexer/parser: value names
   `[a-z_][a-z0-9_]*`, type names `[A-Z][A-Za-z0-9]*`. A spaced name becomes a
   parse error with a new diagnostic code whose help text suggests the
   snake_case spelling. Test names may keep multi-word phrasing for now per
   decision 0012 rule 4.
2. Migrate every `.hum` file in `examples/` and `fixtures/` to snake_case
   names (`remember_work_item`, `work_items`, ...). Prose sections (`why:`,
   `watch for:`, ...) keep natural language.
3. Simplify `covers:` canonical-key matching to exact token matching where the
   spaced-name absorption logic is no longer needed. Do not expand matching.
4. Remove `Number` from README.md, SPEC.md, and any example or fixture;
   replace with `Int` or `UInt` as the example intends.
5. Update MILESTONE_0_GRAMMAR.md, LANGUAGE_REFERENCE.md "Names" section, and
   the TextMate grammar to match. Regenerate anything generated.
6. Recalibrate section-nag diagnostics against progressive disclosure
   (decision 0007). Writing the first `examples/core/` fixtures showed the
   checker pushes vacuous sections: a pure four-line task warns for missing
   `needs:` (H0107) even when no precondition exists, and a test without
   `why:` is an error (H0105). Rule: a missing section may only be flagged
   when its absence is suspicious (task has effects, mutations, failure
   modes, or nontrivial length). A pure small task with no preconditions must
   pass `hum check` clean, because nagging breeds vacuous contracts (see the
   Eiffel entry in CROSS_LANGUAGE_REGRET_LEDGER.md). The two intentional
   H0107 warnings currently emitted for `examples/core/add.hum` and
   `examples/core/count_completed.hum` must be gone after recalibration,
   without adding `needs:` lines to those files.

Acceptance criteria:

- `.\tools\check_all.ps1` passes.
- `cargo run -- check examples` is clean.
- `rg 'task [a-z]+ [a-z]+' examples fixtures` finds no spaced task names.
- `rg 'Number' README.md SPEC.md examples fixtures` finds no type usages.
- A fixture with a spaced name produces the new diagnostic, with a test.
- README.md and SPEC.md contain no spaced-name or `Number` examples, and the
  README flagship example follows the showcase discipline below: a minimal
  form and a full-contract form, both extracted from checked fixtures under
  `examples/`, with a preflight check that the extraction matches.

## Session B: `hum run` interprets the three FORMAL_CORE programs

Scope:

1. The fixtures already exist: `examples/core/add.hum`,
   `examples/core/divide.hum`, `examples/core/count_completed.hum`
   (snake_case, decision 0012, written 2026-07-08 and passing `hum check`
   with zero errors). Treat them as the target surface; adjust only if the
   interpreter work reveals a real defect, and record why.
2. Implement a tree-walking interpreter over the existing AST for exactly the
   subset those three programs need: Int/UInt/Bool literals and arithmetic,
   comparisons, `let`, `change`/`set`, `if`, `for each`, `return`, `fail`,
   task calls, `Result` success/failure values, and simple record/list values
   as needed by `count_completed`. Nothing more.
3. Add `hum run <file> [--entry <task>] [--args ...]` that parses, checks,
   and interprets. Integer overflow and division by zero must trap with a
   clear diagnostic, not wrap silently.
4. Update ARCHITECTURE.md "Current Build Order" and the README status section
   to reflect that Milestone 1 execution has started.

Acceptance criteria:

- `cargo run -- run examples/core/add.hum --entry add --args 2 3` prints `5`.
- `divide` with `b = 0` produces the declared typed failure value, exit code
  distinguishes failure from crash.
- `count_completed` iterates a literal list and returns the right count.
- Rust unit tests cover each program plus one overflow and one div-by-zero
  trap.
- `.\tools\check_all.ps1` passes.

## Session C: first executable contracts

Scope:

1. Define predicate grammar v0 for `needs:` and `ensures:` lines: a boolean
   expression using the body expression grammar, with `result` bound in
   `ensures:`. Examples: `needs: b != 0`, `ensures: result == a + b`.
   One canonical spelling: operators, not words (`==` not `equals`), per
   decision 0009.
2. In `hum run`, evaluate parseable `needs:` lines at task entry and
   `ensures:` lines at successful exit. A failed `needs:` blames the caller;
   a failed `ensures:` blames the task, in the diagnostic text.
3. A `needs:`/`ensures:` line that does not parse as a predicate is not an
   error; it is recorded as an unchecked prose contract with a new warning-
   level diagnostic so the honest split (checked vs prose) is visible.
4. Migrate the three core programs' contracts to predicate form where
   expressible (`divide` gets `needs: b != 0`; `add` gets
   `ensures: result == a + b`).
5. Update FORMAL_CORE.md and LANGUAGE_REFERENCE.md contract sections to
   record predicate v0 as `current` for these two sections only.

Acceptance criteria:

- Calling `divide` with `b = 0` under `hum run` reports a `needs:` violation
  that blames the caller, with the source span of the predicate line.
- A deliberately wrong `add` body (`return a - b`) fails its `ensures:` at
  runtime with task blame. Keep this as a fixture and test.
- Prose contract lines still pass with the visibility warning, with a test.
- `.\tools\check_all.ps1` passes.

## Session D: design feedback from real programs

Scope:

1. Write two real programs by hand in the current executable subset, extending
   the interpreter only where genuinely small: a word-count over a literal
   text list, and a task-list manipulation program (add, complete, count) in
   one file.
2. Record every friction using the friction-record format from the design
   probe system below, into the existing `docs/CORE_LANGUAGE_SHAPE.md` as a
   new "Friction ledger" section. No new files.

Acceptance criteria:

- Both programs run under `hum run` with at least one firing `needs:` or
  `ensures:` each.
- The friction ledger contains at least three records in the required format
  with the source lines that exposed them.

## Design probe system

Probe programs are to language design what property tests are to code. They
are not written on inspiration; they are generated from four sources, and
every one produces structured feedback. This system governs Session D and
every future program-writing work order, including the ownership bake-off.

### Probe sources

1. Regret-ledger probes. Every "Regrets To Avoid" entry in
   CROSS_LANGUAGE_REGRET_LEDGER.md becomes a probe: write the program that
   caused that regret in its source language, in Hum, and observe whether Hum
   dodges it, reproduces it, or cannot express it. The ledger stops being
   prose and becomes executable evidence.
2. Construct-pair probes. Language constructs break in combination, not in
   isolation. Track a simple matrix of FORMAL_CORE construct pairs
   (loop x mutation, match x failure, record x ownership, call x contract,
   ...) and write probes for uncovered pairs. A construct pair no probe has
   exercised is unproven, whatever the docs say.
3. Misuse probes. Every diagnostic code gets at least one deliberately wrong
   program that triggers it, kept as a fixture. The probe scores the message:
   would the diagnostic alone get a newcomer to the fix? Every "must be
   rejected" rule in any doc needs a rejection fixture or the rule is
   folklore.
4. Domain-slice probes. One small representative program per adoption wedge:
   a CLI tool, a parser/codec (WUFFS-shaped), a state machine
   (embedded-shaped), a service skeleton with mocked IO. Each wedge stresses
   different constructs and keeps the design honest about its stated markets.

### Friction records

Every probe session appends records in this exact shape:

```text
friction:
  program: <file and line>
  wanted: <what the author tried to write>
  forced: <what the language required instead>
  severity: blocked | wrong-by-default | awkward | verbose
  indicts: <contracts | ownership | loops | types | diagnostics | stdlib | checker>
  proposal: <optional one-line fix direction>
```

Rules:

- Three or more records indicting the same area is a design bug: it triggers
  a decision record or a work-order item, not a shrug.
- A `blocked` or `wrong-by-default` record is triaged before the next
  session starts.
- Contract wishlist: every `needs:`/`ensures:` line that had to stay prose is
  a friction record indicting `contracts`. Frequency-rank these to decide
  what predicate grammar v1 must add. The grammar grows from measured demand,
  not speculation.

### Agent round-trip metric

Periodically, an agent is given only a program spec and compiler output (no
human hints) and must reach a green `hum run`. The number of round-trips and
the diagnostics that failed to help are recorded as friction records
indicting `diagnostics`. This is the measurable form of "agent-friendly."

## Showcase discipline

The README flagship example is a living artifact with rules, so it updates
as probes change the design instead of rotting:

1. Every code example in README.md and SPEC.md longer than five lines must be
   extracted from a checked fixture under `examples/`, so it cannot drift
   from the real language. Preflight should verify the extraction matches.
2. The README shows two forms, honestly labeled: the minimal form (what most
   code looks like: header, types, `does:`) and the full-contract form (what
   safety-critical code looks like). Leading only with the maximal form
   misrepresents the daily experience and contradicts progressive disclosure
   (decision 0007).
3. When a design decision changes the surface (0012, 0013, predicate
   contracts), updating the showcase fixtures and README extraction is part
   of that session's definition of done, not a follow-up.

## After Session D

Stop and return to the BDFL with the lessons section. The next work order
(ownership-model bake-off on paper against ten hard programs) gets written
then, informed by what execution taught.

## Backlog: accepted taste, not scheduled

Do not work on these during Sessions A-D. Do not create docs for them. Each
becomes its own work order after Session D, in BDFL-chosen order. They are
recorded here so the ideas survive between sessions without growing the doc
corpus.

1. Deterministic run mode. Because time, randomness, and IO are declared
   capabilities, `hum run --deterministic` (virtual clock, seeded random,
   fixed schedule) is structural, not a mocking framework. Payoff:
   reproducible tests, golden-output tests, replay debugging. This is the
   practical selling point of the capability system and should be built soon
   after the interpreter exists.
2. Semantic diff. `hum diff <old> <new>` reports changes as graph facts:
   effects added or removed, contracts strengthened or weakened, capabilities
   gained, unsafe introduced. Evidence-native applied to changes, not just
   states. This is the code-review killer demo.
3. Machine-applicable fixes. Diagnostics carry structured edits (span plus
   replacement) so `hum fix --apply` and agents repair mechanically. Extends
   the existing repair-hint plan; design it into the diagnostic schema before
   the catalog grows further.
4. Sandboxed execution flags. When IO capabilities reach the interpreter,
   they arrive as run-time policy too: `hum run --allow clock --deny
   network,filesystem`. Makes Hum a safe substrate for executing untrusted
   or agent-generated code. Aligns with the agent-tool adoption wedge.
5. Fault containment doctrine. Typed errors cover expected failure; mission-
   critical systems need a story for unexpected failure: crash isolation
   boundaries, supervision, restart budgets, watchdogs. Research sweep on
   Erlang/OTP lessons is missing from the corpus and should precede any
   concurrency design.
6. Units of measure. Typed physical quantities (duration, mass, force,
   voltage) for the aerospace/medical wedges; the Mars Climate Orbiter class
   of bug should be a compile error. Research item first; may become a
   library-plus-checker feature rather than core syntax.
7. Language editions. A Rust-editions-style mechanism so source written today
   compiles for decades while the language evolves. Belongs in
   RELEASE_AND_VERSIONING.md before any public alpha promises stability.

## Appendix: ownership bake-off program suite

This is the benchmark suite for the post-Session-D work order that decides
Hum's ownership model. Candidate models to score: (a) Rust-style borrows
with better diagnostics, (b) mutable value semantics with second-class
references (Hylo direction), (c) region/arena-first ownership (fits
profiles: embedded and realtime want arenas anyway). Pony's reference
capabilities inform the shared-state parts (see the regret ledger).

Each candidate is scored per program on: can it express the program safely,
what the user actually writes, what diagnostic appears on misuse, and
whether a beginner can explain why the rule exists. The winner must clear
all ten or explicitly name its escape hatch.

1. Doubly linked list with back-pointers (the classic Rust wall).
2. Arena-allocated graph with cycles, freed as a unit.
3. Mutating a collection while iterating it (must be rejected; the diagnostic
   text is part of the score).
4. Callback registry that stores references to caller-owned state
   (lifetime-in-struct pain).
5. Parser holding a slice into the buffer it owns (self-reference).
6. Producer/consumer ownership handoff between two workers.
7. Memoizing cache read through a shared path (interior mutability).
8. Swapping two fields of one record (disjoint-borrow ergonomics).
9. Task returning a reference derived from one of its parameters (lifetime
   in return position).
10. Transaction that must commit or roll back exactly once (linear
    resource; ties to STATE_MODEL.md).

Rule for the bake-off: programs are written on paper in each candidate's
surface syntax before any checker is built. The lessons section from
Session D feeds this suite; add frictions discovered there as programs 11+.

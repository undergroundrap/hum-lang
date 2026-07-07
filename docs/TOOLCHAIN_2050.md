# Hum Toolchain 2050 Strategy

Date: 2026-07-06

## Purpose

Hum is not being designed as a personal syntax toy.

Hum is being designed as a systems language someone should still want to use in
2050. That means the language is the whole environment:

```text
syntax -> parser -> semantic graph -> LSP -> formatter -> linter -> debugger -> profiler -> package graph -> agents
```

A language that feels great only in plain text is not enough.

## Brutal Premise

Every serious Hum feature needs an editor story and a debugger story before it
stabilizes.

If a feature cannot be highlighted, completed, renamed, refactored, explained,
debugged, profiled, and represented in the semantic graph, it is not ready for
stable Hum.

## Crafting Interpreters Lessons

Crafting Interpreters is useful because it teaches the language implementation
map clearly:

- scanning turns source text into tokens
- parsing turns tokens into syntax trees
- static analysis resolves names, scope, and types
- intermediate representations make semantics easier to transform
- optimization must preserve meaning
- bytecode and virtual machines trade peak speed for portability and simplicity
- tree-walk interpreters are good for learning and semantics, but not a final general-purpose runtime

Hum should use the book as a pedagogy and implementation sanity check, not as a
production architecture template.

Hum response:

- build a small interpreter early to prove executable semantics
- keep the Rust bootstrap as the trusted implementation while Hum is young
- create Hum IR before serious backend work
- use bytecode or Cranelift-style execution for developer loops where useful
- keep LLVM or another mature backend for optimized AOT builds
- never confuse a teaching interpreter with the final systems runtime

## The 2050 Toolchain Rule

Hum must have one source of truth for language facts.

That source of truth is not syntax highlighting regexes, editor plugins, or agent
prompts. It is the compiler front end plus semantic graph.

All tools should consume the same facts:

- `hum check`
- `hum graph`
- `hum syntax`
- `humfmt`
- `chirp`
- `hum lsp`
- `hum debug`
- `nectar`
- docs generation
- profilers
- agents

If two tools disagree about what code means, the toolchain is wrong.

## Syntax Highlighting Strategy

Hum needs highlighting in layers.

### Tier 0: TextMate-Style Grammar

Purpose:

- immediate editor support
- GitHub-style highlighting
- fallback for simple editors

Rules:

- highlight keywords, blocks, strings, comments, literals, and punctuation
- do not pretend regex highlighting understands semantics
- keep it generated from the grammar when possible

### Tier 1: Tree-Sitter Grammar

Purpose:

- fast structural highlighting
- incremental parsing while typing
- editor queries for blocks, symbols, folds, and selections
- useful output even with syntax errors

Rules:

- `tree-sitter-hum` should exist before Hum has many users
- syntax must avoid grammar tricks that make incremental parsing fragile
- queries should highlight intent blocks differently from execution blocks
- tests should include incomplete and malformed code, because editors live in broken code

### Tier 2: LSP Semantic Tokens

Purpose:

- highlight what syntax alone cannot know
- distinguish stores, capabilities, unsafe boundaries, effects, tests, proofs, and costs
- show moved/borrowed/changed names once ownership exists

Rules:

- semantic tokens come from compiler facts
- semantic tokens must degrade gracefully when the file is half-written
- theme categories should be stable enough for editors and docs

### Tier 3: Intent-Aware Visualization

Purpose:

- make Hum's unique blocks visible without visual noise

Examples:

- `uses:` names look like read capabilities
- `changes:` names look like write capabilities
- `protects:` and `trusts:` are security-colored
- `cost:` and `benchmarks:` are performance-colored
- generated tests/proofs show coverage state inline

This is where Hum can feel unlike every older language.

## LSP Strategy

Hum needs `hum lsp` as a first-party server.

Early LSP features:

- diagnostics from `hum check`
- document symbols for modules, apps, tasks, types, stores, and tests
- folding ranges for intent blocks
- hover explanations for section keywords and declared names
- semantic tokens
- formatting via `humfmt`
- code actions for missing `uses:`, `changes:`, `why:`, `cost:`, and `protects:`

Middle LSP features:

- go to definition
- find references
- rename with semantic graph validation
- call hierarchy
- inlay hints for inferred failure, cost, allocation, ownership, and effects
- code lens for tests, proofs, benchmarks, and generated docs
- workspace symbols through Nectar package graphs

Late LSP features:

- ownership visualization
- effect-flow visualization
- package trust visualization
- unsafe review packets
- generated repair plans for agents
- semantic search over `why:`, `uses:`, `changes:`, `protects:`, and `cost:`

LSP should be protocol-compatible, but Hum-specific powers should flow through
stable extension methods documented in the semantic graph schema.

## Editor Integration Strategy

Hum should not make every editor plugin reimplement the language. The compiler, `hum graph`, `humfmt`, and `hum lsp` should be the authority; editor integrations should be thin adapters.

Priority editor surfaces:

- VS Code and Cursor for reach and fast extension distribution
- Visual Studio for Windows, C++, C#, enterprise, regulated, and Microsoft-platform teams
- JetBrains IDEs for IntelliJ, PyCharm, Rider, WebStorm, and RustRover users
- Eclipse for Java-heavy enterprise teams
- Jupyter Notebook and JupyterLab for data, finance research, AI/ML, and literate demos once Hum can execute safely
- Neovim, Vim, Helix, Zed, and Sublime Text through LSP and grammar packages

See [EDITOR_AND_INTEGRATION_STRATEGY.md](EDITOR_AND_INTEGRATION_STRATEGY.md).

## Debugger Strategy

Hum needs a Debug Adapter Protocol implementation, but a normal line debugger is
not enough.

A Hum debugger should answer:

- what task am I in?
- which `needs:` or `ensures:` promise failed?
- which `uses:` were read?
- which `changes:` happened?
- what failure path am I on?
- what allocation happened here?
- what trust boundary did I cross?
- what generated test or proof covers this path?

Required layers:

1. source breakpoints at tasks, tests, and `does:` lines
2. expression inspection for locals, params, stores, and results
3. contract breakpoints for `needs:`, `ensures:`, `protects:`, and `cost:` failures
4. effect tracing for IO, allocation, mutation, randomness, time, and foreign calls
5. replay hooks for deterministic tests
6. schedule visualization for concurrent/data-oriented tasks
7. source maps from Hum to Hum IR, bytecode, Cranelift, LLVM, or native debug info

The debugger should make intent visible, not just variables.

## Profiler And Trace Strategy

A 2050 systems language should let programmers profile promises.

Hum profiling should map runtime facts back to source claims:

- this task claimed `O(1)` but scanned a collection
- this `store` is memory dense but causing high probe counts
- this `does:` block allocates despite no visible allocation claim
- this `protects:` claim is using a variable-time operation
- this pipeline sorts more data than the source suggests
- this parallel schedule is blocked by one unnecessary `changes:` declaration

Profilers should read the same semantic graph as the debugger and LSP.

## Debug Info And Source Maps

Hum must preserve source identity across lowering.

Every lowering stage should carry:

- file path
- source span
- semantic graph node id
- task/test/store/type id
- intent section id
- generated code origin
- optimization provenance when code moves or disappears

Debug info is not only for humans. Agents and profilers need it too.

## Tooling-Driven Syntax Constraints

Hum syntax should be designed for tools from the start.

Reject or delay features that:

- require editor regexes to guess semantics
- make incremental parsing fragile
- create ambiguous block ownership
- let macros invent syntax the parser cannot understand
- make semantic tokens depend on global type solving for basic highlighting
- hide source spans behind generated code
- make rename/refactor unsafe
- make debugger stepping surprising

This is another reason to delay macros, arbitrary custom operators, and implicit
control flow.

## First Tooling Deliverables

Milestone 0 should produce:

1. stable diagnostic shape
2. stable source spans
3. `hum graph` with source-derived node ids
4. [MILESTONE_0_GRAMMAR.md](MILESTONE_0_GRAMMAR.md) as the bootstrap grammar contract
5. `hum syntax` machine-readable syntax surface
6. generated TextMate grammar sketch at [../editors/textmate/hum.tmLanguage.json](../editors/textmate/hum.tmLanguage.json)
7. Tree-sitter grammar design note
8. LSP capability plan
9. debugger data model sketch
10. golden tests for incomplete/broken source
11. editor-fixture `.hum` files

Before Hum is called alpha, it should have:

1. `hum lsp`
2. `tree-sitter-hum`
3. VS Code/Cursor extension
4. Visual Studio adapter plan
5. Neovim/Helix/Zed support path
6. DAP debug adapter prototype
7. semantic tokens
8. rename and go-to-definition
9. code actions for common intent mistakes
10. test/proof/benchmark code lens
11. `nectar` workspace indexing

## 2050 Toolchain Tests

Every release should test:

- parser on valid source
- parser on incomplete source
- parser on malformed source
- semantic graph stability
- LSP diagnostics latency
- semantic token stability
- formatter idempotence
- rename correctness
- debugger source mapping
- profiler source mapping
- agent graph consumption

This is how Hum avoids becoming a language whose compiler works but whose daily
experience feels bolted on.

## Brutal Rule

A language for 2050 must be designed for the tools that read it.

Hum source is for humans, compilers, debuggers, editors, profilers, package
managers, and agents at the same time. If those readers need different truths,
Hum has failed.

## Sources

- Crafting Interpreters table of contents: https://craftinginterpreters.com/contents.html
- Crafting Interpreters, "A Map of the Territory": https://craftinginterpreters.com/a-map-of-the-territory.html
- Language Server Protocol 3.18 specification: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/
- Debug Adapter Protocol overview: https://microsoft.github.io/debug-adapter-protocol/
- Tree-sitter introduction: https://tree-sitter.github.io/tree-sitter/
- Tree-sitter advanced parsing: https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html

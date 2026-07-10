# Hum Milestone 0 Grammar

Date: 2026-07-06
Status: bootstrap parser contract, not final language grammar

## Purpose

This document pins the grammar shape accepted by the Rust bootstrap parser in
Milestone 0.

It is intentionally narrower than the future language grammar:

- narrower because Milestone 0 only parses, checks, and emits graph facts
- still modest because type annotations and body lines are often captured as text while
  the formal core is still being pinned

Use this document for parser tests, docs, syntax highlighting sketches, and
agent prompts. Do not treat every permissive parser behavior here as a stable
language promise.

For the human-facing language overview, see [LANGUAGE_REFERENCE.md](LANGUAGE_REFERENCE.md).
For the machine-readable Milestone 0 syntax surface, run `hum syntax`; its schema is documented in [SYNTAX_SURFACE_SCHEMA.md](SYNTAX_SURFACE_SCHEMA.md). The syntax surface includes a section catalog with hover text and a semantic-token legend for editor and LSP adapters. For the generated TextMate grammar, run `hum syntax --format textmate`.
For the checked reference fixture, see [../examples/reference_surface.hum](../examples/reference_surface.hum).

## Notation

This is a descriptive grammar, not a generated parser grammar.

- `line` means one physical source line.
- `trim(line)` means leading and trailing whitespace removed.
- `indent(line)` counts leading space characters only.
- `text` means captured source text after trimming.
- `body-line` means a captured line inside a section.

## File Shape

```text
file        ::= ignorable* module-decl? top-level*
module-decl ::= "module" space module-path newline
top-level   ::= ignorable | item | unexpected-top-level-line
ignorable   ::= blank-line | comment-line
```

Current parser facts:

- A module declaration must appear at indentation 0.
- The parser accepts the last seen `module` declaration as file metadata.
- Module paths are dot-separated value identifiers. Each segment must match `[a-z_][a-z0-9_]*`.
- Unexpected non-blank top-level lines produce a warning, not a fatal parse
  error.

## Comments And Blank Lines

At top level and inside nested item ranges, the parser ignores lines where
`trim(line)` is empty or starts with `#` or `//`.

Inside sections, non-empty comment lines are preserved in `line_items` and
marked `meaningful: false` in `hum graph`.

## Item Headers

```text
item        ::= item-header "{" item-body "}"
item-header ::= app-header | type-header | store-header | task-header | test-header
```

Current item headers must:

- be at indentation 0 for top-level items
- be at the containing item indentation plus two spaces for nested items
- end with `{` on the same line
- have a matching `}` later in the file

Brace matching is character-based across lines. Milestone 0 does not yet have a
full token model, string model, or trivia-preserving concrete syntax tree.

## Item Kinds

```text
app-header   ::= "app" space value-ident
type-header  ::= "type" space type-ident
store-header ::= "store" space value-ident (":" store-type)?
task-header  ::= "task" space value-ident params? ("->" result-type)?
test-header  ::= "test" space test-signature
```

Current capture rules:

- App names, store names, and task names are value identifiers.
- Type names are type identifiers.
- `store-type` is trimmed text after the first `:` in a store header.
- `result-type` is trimmed text after `->` in a task header. The parser captures `from` relationships as part of this result text; `hum ownership-check` gives the currently supported `ResultType from parameter` form its ownership meaning.
- A nonconforming identifier produces `H0009`.

For the current executable app slice, one top-level app has exactly one
`starts with:` section with exactly one meaningful line matching
`value-ident`. That name resolves only to a directly nested task. This is a
checked structural rule layered on the permissive section parser; it is not a
new item-header grammar production and does not define app state.

## Identifier Grammar

Decision 0012 pins the current identifier grammar:

```text
value-ident ::= [a-z_][a-z0-9_]*
type-ident  ::= [A-Z][A-Za-z0-9]*
module-path ::= value-ident ("." value-ident)*
```

Value identifiers cover module path segments, app names, store names, task
names, parameter names, and field names. Type identifiers cover `type` names.
Spaces are not part of an identifier; a spaced name is a parse error (`H0009`)
with help that suggests the `snake_case` spelling. Test names may remain
multi-word phrases until the test grammar is pinned.

## Callable Signatures

```text
callable-signature ::= callable-name params? trailing?
params             ::= "(" param-list? ")"
param-list         ::= param ("," param)*
param              ::= param-permission? param-name ":" param-type
param-permission  ::= "borrow" | "change" | "consume"
```

Current capture rules:

- For tasks, `callable-name` is a value identifier before the first `(`.
- If no `(` appears, the whole task signature is the callable name and params are empty.
- Test names may still be multi-word phrases; test params use the same param grammar.
- `param-name` is a value identifier before `:` in a comma-separated parameter.
- An optional leading `borrow`, `change`, or `consume` before the parameter name is captured as the parameter permission; unmarked parameters default to `borrow`.
- `param-type` is trimmed text after `:` in that parameter.
- Missing parameter types produce an error.
- Missing `)` produces an error.
- Extra trailing text after a task parameter list produces a warning.

## Test Modifiers

Known test modifiers:

```text
unit
property
fuzz
regression
integration
model
```

Current parser behavior:

- If a test header has params, all trailing words after `)` are captured as
  modifiers.
- If a test header has no params, only recognized modifier words are peeled
  from the end of the test name.

## Item Body

```text
item-body ::= (section | nested-item | other-line)*
```

Milestone 0 scans item bodies for sections and, for `app`, nested items. Lines
that are not section headers, fields, or nested item headers may still be
captured as section lines if they occur under a section.

## Sections

```text
section        ::= section-header section-body
section-header ::= section-name ":"
section-name   ::= ascii-letter-or-space+
section-body   ::= body-line*
```

A section header is recognized when:

- it is indented exactly two spaces deeper than the containing item
- trimmed text ends with `:`
- the name before `:` is non-empty
- the name contains only ASCII letters and spaces

A section ends when another section header or item header appears at the same
section indentation.

## Section Lines

Current section line capture:

- every line after a section header belongs to that section until the next
  section or item header at the same indentation
- captured text is trimmed
- spans point to the physical source line and first visible source column
- empty captured lines count as section lines internally but are omitted from
  `hum graph` `line_items`
- comment-only lines are preserved in graph output and marked not meaningful

`needs:`, `ensures:`, `watch for:`, and `tests:` meaningful lines produce task
test obligations.

## Fields

```text
field ::= field-name ":" field-type
```

Current field parsing applies to `type` bodies at item indentation plus two
spaces. A field line is ignored if it is blank, a comment, a section header, or
an item header.

Field capture rules:

- `field-name` is a value identifier before the first `:`
- `field-type` is trimmed text after the first `:`
- no full type grammar is enforced yet

Milestone 1 executable checking also recognizes the direct place shapes `place ::= value-ident | value-ident "." value-ident | value-ident "[" uint-literal "]"` for local reads, field reads, and direct element reads. Direct `set` accepts an authorized mutable local name or direct field place, but not an element place. It recognizes the built-in call shape `list_append(change value-ident, expression)` as the first minimal list-growth operation. Session R adds the narrow local field-view binding shape `let value-ident = borrow value-ident "." value-ident`; writing that exact field invalidates the view, while writing a distinct direct field does not. Session S adds the narrow local element-view binding shape `let value-ident = borrow value-ident "[" uint-literal "]"`; `list_append` invalidates outstanding element views for the grown list. Session V adds exactly `writable-field-alias ::= "let" space value-ident space? "=" space? "change" space value-ident "." value-ident` in a straight-line task body. The exact alias name becomes an indirect `set value-ident = expression` write-through target for that source field; its recognized lifetime ends after its last syntactic use, overlapping source access rejects with H0808, and unsupported or escaping use rejects with H0809. This is not a full expression grammar and does not create general indexing, nested places, retained element views, general aliases, internal references, or broad flow-sensitive projection syntax.

## Current Known Sections

The parser accepts any section name matching the section header rule above. The
Milestone 0 checker recognizes this canonical task order for diagnostics:

```text
why
targets
uses
changes
needs
ensures
protects
trusts
fails when
watch for
cost
allocates
avoids
tradeoffs
optimizes
tests
does
```

The current graph emitter gives `targets:` a narrow structured interpretation
when lines use `triple:`, `requires:`, or `denies:`. Other `targets:` lines are
preserved as normal section text but are not portability facts in V0.
See [DIAGNOSTICS.md](DIAGNOSTICS.md), [FORMATTER.md](FORMATTER.md), and
[LANGUAGE_REFERENCE.md](LANGUAGE_REFERENCE.md).

## Deliberately Loose In Milestone 0

The bootstrap parser deliberately does not yet pin:

- final expression grammar
- final type grammar
- string literal grammar
- comments outside section preservation
- operator precedence
- import and visibility grammar
- lossless concrete syntax tree trivia

Those must be designed before serious formatter, LSP, Tree-sitter, or
self-hosting work depends on them.

## Parser Diagnostics Boundary

Milestone 0 parser diagnostics should stay helpful and conservative:

- malformed item headers produce stable errors
- missing callable `)` produces a stable error
- parameters without `:` types produce stable errors
- invalid identifiers produce stable errors with a snake_case or PascalCase suggestion
- unexpected top-level lines produce warnings
- unknown item kinds produce warnings when encountered through item parsing

See [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Future Grammar Work

Before Hum claims a stable language grammar, it needs:

- a lossless concrete syntax tree or equivalent token/trivia model
- golden malformed-source tests
- syntax-highlight keyword list generated from `hum syntax --format textmate` and this grammar surface
- TextMate and Tree-sitter grammar design notes
- a formal surface-to-core lowering map
- exact type and expression grammar

Until then, this document is a Milestone 0 parser contract and a warning label:
do not expand syntax faster than semantics, diagnostics, graph facts, and tools
can follow.

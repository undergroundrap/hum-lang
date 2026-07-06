# Hum Milestone 0 Grammar

Date: 2026-07-06
Status: bootstrap parser contract, not final language grammar

## Purpose

This document pins the grammar shape accepted by the Rust bootstrap parser in
Milestone 0.

It is intentionally narrower and looser than the future language grammar:

- narrower because Milestone 0 only parses, checks, and emits graph facts
- looser because names, types, and body lines are often captured as text while
  the formal core is still being pinned

Use this document for parser tests, docs, syntax highlighting sketches, and
agent prompts. Do not treat every permissive parser behavior here as a stable
language promise.

For the human-facing language overview, see [LANGUAGE_REFERENCE.md](LANGUAGE_REFERENCE.md).

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
module-decl ::= "module" space module-text newline
top-level   ::= ignorable | item | unexpected-top-level-line
ignorable   ::= blank-line | comment-line
```

Current parser facts:

- A module declaration must appear at indentation 0.
- The parser accepts the last seen `module` declaration as file metadata.
- Module text is captured as trimmed text after `module `.
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
app-header   ::= "app" space app-name
type-header  ::= "type" space type-name rest?
store-header ::= "store" space store-name (":" store-type)?
task-header  ::= "task" space callable-signature ("->" result-type)?
test-header  ::= "test" space test-signature
```

Current capture rules:

- `app-name` is all trimmed text after `app `.
- `type-name` is the first whitespace-separated word after `type `.
- `store-name` is trimmed text before the first `:` after `store `.
- `store-type` is trimmed text after that `:`.
- `result-type` is trimmed text after `->` in a task header.

These are parser facts, not final naming rules.

## Callable Signatures

```text
callable-signature ::= callable-name params? trailing?
params             ::= "(" param-list? ")"
param-list         ::= param ("," param)*
param              ::= param-name ":" param-type
```

Current capture rules:

- If no `(` appears, the whole signature is the callable name and params are
  empty.
- `callable-name` is trimmed text before the first `(`.
- `param-name` is trimmed text before `:` in a comma-separated parameter.
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
- spans point to the physical source line
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

- `field-name` is trimmed text before the first `:`
- `field-type` is trimmed text after the first `:`
- no full type grammar is enforced yet

## Current Known Sections

The parser accepts any section name matching the section header rule above. The
Milestone 0 checker recognizes this canonical task order for diagnostics:

```text
why
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

See [DIAGNOSTICS.md](DIAGNOSTICS.md), [FORMATTER.md](FORMATTER.md), and
[LANGUAGE_REFERENCE.md](LANGUAGE_REFERENCE.md).

## Deliberately Loose In Milestone 0

The bootstrap parser deliberately does not yet pin:

- final identifier grammar
- final module path grammar
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
- unexpected top-level lines produce warnings
- unknown item kinds produce warnings when encountered through item parsing

See [DIAGNOSTICS.md](DIAGNOSTICS.md).

## Future Grammar Work

Before Hum claims a stable language grammar, it needs:

- a lossless concrete syntax tree or equivalent token/trivia model
- golden malformed-source tests
- syntax-highlight keyword list generated from this grammar surface
- TextMate and Tree-sitter grammar design notes
- a formal surface-to-core lowering map
- exact identifier, path, type, and expression grammar

Until then, this document is a Milestone 0 parser contract and a warning label:
do not expand syntax faster than semantics, diagnostics, graph facts, and tools
can follow.

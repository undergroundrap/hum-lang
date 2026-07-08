# 0012: Adopt snake_case Identifiers, Retire Spaced Names

Date: 2026-07-08
Status: accepted

## Context

Milestone 0 allows spaces inside item names (`task add task`, `store work
items`) as a source-identity convenience. This was always flagged as not a
stable promise, but it has quietly become load-bearing: examples, fixtures,
coverage-key matching, and the TextMate grammar all depend on it.

Spaced names block the next required piece of work, the expression grammar.
Juxtaposition becomes ambiguous (`save item in work items` cannot be tokenized
deterministically once expressions exist), `covers:` matching already needs
canonical keys that absorb filler words, and rename refactors, grep, symbol
names, and FFI all degrade. Decision 0009 forbids English mimicry in stable
syntax; spaced identifiers are English mimicry living in the grammar.

## Decision

Hum identifiers use `snake_case`.

Accepted rules:

1. Item names, parameter names, field names, store names, module segments, and
   variant names match `[a-z_][a-z0-9_]*`. Type names match
   `[A-Z][A-Za-z0-9]*`.
2. Spaces are not part of any identifier. A spaced name is a parse error with a
   dedicated diagnostic that suggests the snake_case spelling.
3. Human phrasing lives in `why:` and other prose-bearing sections, not in
   names.
4. Test names may remain multi-word phrases only until the test grammar is
   pinned; the pinned grammar must resolve them to a single deterministic token
   rule, and snake_case is the default candidate.
5. All examples, fixtures, docs snippets, and generated grammars migrate to
   snake_case. No new spaced-name examples may be added anywhere.
6. Coverage-key matching simplifies accordingly: canonical keys stop absorbing
   filler words once names are single tokens.

## Consequences

The expression grammar is unblocked: names are single tokens and juxtaposition
is unambiguous.

The Milestone 0 parser, checker, coverage matcher, resolver, TextMate grammar,
`reference_surface.hum`, and every `.hum` fixture require a mechanical
migration pass. This is accepted one-time cost; delaying it makes it strictly
more expensive.

`hum explain` output for the new diagnostic should teach the rule in one line:
names are snake_case, sentences belong in `why:`.

## Alternatives Rejected

- Keep spaced names and disambiguate with context: makes the tokenizer
  grammar-dependent and every downstream tool heuristic.
- Quote spaced names (`task "add task"`): punctuation noise, still breaks
  grep/rename/FFI, and invites prose back into names.
- Defer the decision until the expression grammar forces it: the forcing event
  is now; deferral just grows the migration surface.

## BDFL Note

The readable-name experiment did its job: it proved the warmth belongs in
`why:`, not in the symbol table. Hum stays humane by explaining itself, not by
making the parser guess where a sentence ends.

# 0022: Escape Sequences In Text Literals

Date: 2026-09-22
Status: proposed. This record authorizes no implementation. It fixes the
semantic choice so that the Work Order implementing it cannot adopt the wrong
shape.

## Context

The newline probe for decision 0021 turned up a second gap, and it is bigger
than it looks. Hum text literals have no escape syntax. A probe returning the
literal `"\n"` produces two bytes (backslash, `n`), not a line feed — the
literal grammar has no escapes at all. The consequences run in both
directions:

- Input side: `text_split` cannot split a file into lines, because no literal
  can name a newline.
- Output side: `stdout_write` cannot print a line break. wordfreq's "one word
  per line" summary is impossible, and so is any program that prints more than
  one line.

This is not a wordfreq quirk. Every real program will hit it. It is a
language gap, and it has to be closed before wordfreq's Part 2 — and before
any multi-line output — can exist.

## Options considered

**A. Escape sequences in text literals** (recommended): a minimal set of
escapes decoded at literal-parse time — `\n` (line feed), `\t` (tab),
`\\` (backslash), `\"` (double quote). Unknown escapes are a checker error.

**B. A `newline()` builtin**: returns a one-character `Text` containing a
line feed. This avoids touching the lexer entirely.

Option B is rejected. It fixes the input side for `text_split` only by
awkward composition, and it leaves the output side just as awkward: every
program that wants a line break concatenates a builtin call instead of
writing a literal. The standard answer to "how does this language write a
newline" is escape sequences; a builtin is a permanent workaround tax on every
program. The lexer change is real, but it is the honest one.

## Decision

Adopt option A: text literals support the escapes `\n`, `\t`, `\\`, and
`\"`, and nothing else.

### Semantics (normative)

- `\n` decodes to U+000A (line feed). `\t` decodes to U+0009 (tab).
  `\\` decodes to a single backslash. `\"` decodes to a double quote.
- Any other backslash sequence (for example `\r`, `\0`, `\u`) is a
  **checker error** at parse time, not a silent literal. The language does
  not guess what an unknown escape means.
- A trailing backslash at the end of the literal (before the closing quote)
  is a checker error (unterminated escape).
- Escapes are decoded when the literal is built into the canonical AST.
  Today `canonical_expression_build` in `src/parser.rs` strips the quotes and
  takes the inner text verbatim; the decode happens exactly there, so every
  consumer of `CanonicalExpressionKind::TextLiteral` sees the decoded value.

### Evidence impact (normative)

This changes the canonical form of `TextLiteral` nodes: any literal
containing a backslash decodes to a different value than before. That is a
canonical-seal change, and it triggers the heavier Exhaustive evidence route
in CI. The implementing Work Order must run it.

### What this deliberately does not settle

- Unicode escapes (`\u{...}`): not adopted. No program in the current scope
  needs them; they need their own decision record.
- Raw strings or any alternative literal syntax: not adopted.
- Whether `\r` should ever be admitted: no. Unknown means error, full stop.

## Consequences

- The implementing Work Order provides fixtures covering: each of the four
  escapes decoding correctly, unknown escapes failing at check time with a
  typed error, a trailing backslash failing at check time, and a literal with
  no escapes decoding exactly as before (no behavior change for existing
  programs).
- `docs/LANGUAGE_REFERENCE.md` gains the escape table;
  `docs/DIAGNOSTICS.md` gains the unknown-escape error code.
- This decision does not authorize the implementation; the Work Order does.

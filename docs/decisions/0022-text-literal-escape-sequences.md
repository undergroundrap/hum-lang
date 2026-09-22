# 0022: Escape Sequences In Text Literals

Date: 2026-09-22
Status: accepted 2026-09-22. BDFL ruling on the review at 6cda589 (0022 sound;
three fixes folded into the text in the acceptance commit, no further review
round; verified at PR time).

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
  not guess what an unknown escape means. The diagnostic's span marks the bad
  escape itself (the backslash and its following character), not the whole
  literal.
- A trailing backslash at the end of the literal (before the closing quote)
  is a checker error (unterminated escape).
- Escapes are decoded when the literal is built into the canonical AST.
  Today `canonical_expression_build` in `src/parser.rs` strips the quotes and
  takes the inner text verbatim; the decode happens exactly there — one decode
  point — so every consumer of `CanonicalExpressionKind::TextLiteral` sees the
  decoded value.
- Quote-scanner agreement (normative): the parser has several quote-aware
  scanners (the literal scanners, `split_top_level_ranges_quoted`, the
  unterminated-literal detector). Some already treat `\"` as "don't end the
  literal here" while taking the value verbatim. After this decision every
  quote-aware scanner must agree on where a literal ends: an escaped quote
  never terminates the literal, in any scanner. A fixture must cover
  `f("a\"b, c", d)` — the escaped quote inside the argument list must not
  split the arguments.

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
- Emitter escaping (normative): decoded text can now contain real control
  characters, and it flows into the JSON and canonical outputs (the
  `push_str`-style emitters in `backend_contract`, `capabilities`,
  `core_contract`, and the other contract emitters). Every emitter must
  escape control characters — a real newline in a value must come out as
  `\n` in JSON, never as a raw line break. A fixture through `hum evidence`
  or `hum graph` using a `\n` literal proves the round trip.
- `docs/LANGUAGE_REFERENCE.md` gains the escape table;
  `docs/DIAGNOSTICS.md` gains the unknown-escape error code.
- This decision does not authorize the implementation; the Work Order does.

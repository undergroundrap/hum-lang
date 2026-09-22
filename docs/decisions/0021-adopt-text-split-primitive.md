# 0021: Adopt text_split As The Text Tokenization Primitive

Date: 2026-09-22
Status: proposed. This record authorizes no implementation. It fixes the
semantic choice so that the Work Order implementing it cannot adopt the wrong
shape.

## Context

Work Order 26 set out to write wordfreq, the first real Hum program, from the
existing language surface with no new builtins. The session stopped: word
tokenization — turning a file's `Text` into a `List Text` — is not expressible
in Milestone 0 Hum. The only text-slicing primitive, `slice_until(text, sep)`,
returns the head before the separator with no remainder operation. There is no
text length, indexing, containment test, concatenation, or iteration over text.
A natural splitter formulation type-checks but returns `["hum", "hum"]` for
`"hum lang hum"` — the first word twice, because "the rest of the text" cannot
be named.

The existing `examples/probes/word_count.hum` never splits text. It counts a
list literal written already split (`["hum", "lang", "hum", "agent"]`). Hum can
read a file but cannot break its text apart. This is a language capability gap,
not an interpreter bug, and the work-order process caught it the way it is
meant to: the session stopped instead of inventing surface.

A prior research assumption is falsified by this finding.
`notes/three_program_sequence.md` asserted "word tokenization has no stdlib
(the program must define its own splitting — fine, it keeps the program honest
about what the language provides)." That assumption is false: the program
*cannot* define its own splitting, because the primitives to define it with do
not exist. The note should be corrected, not deleted — the falsification is
part of the record.

## Options considered

**A. `slice_after(text, sep) -> Text`**: the tail after the first separator,
complementing `slice_until`. A program would loop over the remaining text until
empty.

**B. `text_split(text, sep) -> List Text`**: the whole tokenization in one
primitive, returning the list of pieces.

**C. Re-scope wordfreq** to take the word list as an explicit parameter,
avoiding the gap entirely.

Option C is rejected: it would make wordfreq a toy and would not prove the
"first real program" claim. The file-to-words composition is the heart of the
program.

Option A is rejected on termination-measure grounds. Under decision 0020, a
loop over the remaining text needs a decreasing termination measure, which
means a text-length primitive and probably an emptiness test as well. Option A
is not one new builtin; it is three, plus the termination-measure plumbing.
Each additional primitive is surface the language must then specify, test, and
carry.

## Decision

Adopt option B: `text_split(text: Text, sep: Text) -> List Text`.

The program gets back a list and uses `for each`, which is already bounded.
That is one new builtin, and it slots straight into the counting code in
`word_count.hum`.

### Edge-case semantics (normative)

- Leading separators produce a leading empty piece: `text_split(",a,b", ",")`
  is `["", "a", "b"]`.
- Trailing separators produce a trailing empty piece: `text_split("a,b,", ",")`
  is `["a", "b", ""]`.
- Repeated separators produce empty pieces between them: `text_split("a,,b", ",")`
  is `["a", "", "b"]`.
- Separator not present returns the whole text as the single piece:
  `text_split("abc", ",")` is `["abc"]`.
- Empty input returns a single empty piece: `text_split("", ",")` is `[""]`.
- Empty separator is a typed error (`SepEmpty`), raised at call time. A misuse
  fixture must cover it.

### Ownership and cost (normative)

- `text_split` returns **owned copies** of the pieces, not views into the
  source text. Views would run straight into the internal-references item in
  the ownership debt (backlog #4). Copies avoid that completely.
- The builtin **allocates**: one string per piece, plus the list. Its `cost:`
  and `allocates:` declarations must say so honestly. No zero-cost claim.

### What this deliberately does not settle

- String views or slices into text: not adopted, not specified. The
  internal-references ownership question stays open.
- A general string library (join, trim, case mapping, replacement): not
  adopted. `text_split` is the single tokenization primitive; anything further
  needs its own decision record.
- Whether `for each` over the resulting list needs additional bounds: no, it
  is already bounded by the list produced.

## Consequences

- The implementing Work Order must provide probes and positive, boundary, and
  misuse fixtures for `text_split` before wordfreq is written on top of it.
  The misuse fixtures include the empty-separator typed error and the
  leading/trailing/repeated separator edge cases above.
- `notes/three_program_sequence.md` must be corrected to record the falsified
  assumption, with a pointer to this decision.
- This decision does not authorize the implementation; the Work Order does.

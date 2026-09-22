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

A prior research assumption is falsified by this finding. The research package
(not the repo) contains `notes/three_program_sequence.md`, which asserted
"word tokenization has no stdlib (the program must define its own splitting —
fine, it keeps the program honest about what the language provides)." That
assumption is false: the program *cannot* define its own splitting, because
the primitives to define it with do not exist. The falsification is recorded
in the repo at `docs/research/hum-improvement-backlog-2026-09-22.md`, which is
where the research assumptions live in repo form.

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

### Matching rule (normative)

Exact substring matching, left to right, non-overlapping. The scan finds the
leftmost occurrence of the separator, emits the piece before it, then
continues scanning after the separator's end. For example,
`text_split("aaa", "aa")` is `["", "a"]`: the first `aa` matches at position
0, emitting the empty piece before it, and the remaining `a` contains no
further match.

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

### Empty separator (normative, under decision 0016)

A **literal** empty separator is a checker error: `text_split(text, "")` does
not type-check. Only a separator computed at runtime can fail at runtime,
raising the typed error `SepEmpty` through the normal `try` / `fail` path of
decision 0016. This keeps the common case — `text_split(line, " ")` — free of
`try` ceremony while keeping the genuinely dynamic case honest. A misuse
fixture must cover both the literal-rejection and the runtime failure.

### Ownership and cost (normative)

`slice_until` is already a view-deriving operation: `first_word.hum` returns a
view tied to its parameter under the V0 returned-view `from parameter` rule
(H0805). `text_split` nevertheless returns **owned copies** of the pieces, not
views. The reason is structural, not conservative: a `List Text` of views
would need element-alias and stored-view relationships — which view aliases
which piece, how long each view lives relative to the list, what happens when
the list is appended to — and decision 0014 explicitly locks the ownership
model against that shape. Copies avoid the entire question.

The builtin **allocates**: one string per piece, plus the list. Its `cost:`
and `allocates:` declarations must say so honestly. No zero-cost claim.

### Newline separators (empirical finding, 2026-09-22)

Hum text literals cannot express a newline. A probe returning the literal
`"\n"` produces two bytes (backslash, `n`), not a line feed — there is no
escape syntax in the literal grammar. This means `text_split` as specified
cannot split a file into lines until the language can name a newline. That
gap must be resolved — by whatever mechanism the BDFL chooses — before
wordfreq's Part 2 begins, because wordfreq on real files depends on it. This
decision does not settle the mechanism; it records the dependency.

### What this deliberately does not settle

- String views or slices into text beyond the existing `slice_until`: not
  adopted, not specified. The internal-references ownership question stays
  open.
- A general string library (join, trim, case mapping, replacement): not
  adopted. `text_split` is the single tokenization primitive; anything further
  needs its own decision record.
- The newline-literal mechanism: recorded above as an open dependency, not
  decided here.
- Whether `for each` over the resulting list needs additional bounds: no, it
  is already bounded by the list produced.

## Consequences

- The implementing Work Order must provide probes and positive, boundary, and
  misuse fixtures for `text_split` before wordfreq is written on top of it.
  The misuse fixtures include the literal-empty-separator checker rejection,
  the runtime `SepEmpty` failure, and the leading/trailing/repeated separator
  edge cases above.
- The falsified `three_program_sequence` assumption is recorded in
  `docs/research/hum-improvement-backlog-2026-09-22.md`, not by editing a file
  outside the repo.
- `docs/LANGUAGE_REFERENCE.md` gains an entry for `text_split`;
  `docs/DIAGNOSTICS.md` gains an entry for `SepEmpty`.
- This decision does not authorize the implementation; the Work Order does.

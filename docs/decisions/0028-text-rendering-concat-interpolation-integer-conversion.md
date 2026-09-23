# 0028: Text rendering — concatenation, interpolation, and integer-to-Text conversion

Date: 2026-09-23
Status: accepted 2026-09-23 (BDFL ruling on review at 5b57dbd), Option E.

## Context

wordfreq's friction ledger (PR #24, entry #5) found Hum's first big design
question: there is no text concatenation, no formatting/interpolation, and no
UInt-to-Text conversion, so no program can print `hum 3`. The honest output
with today's surface is one word per line (a histogram); the count is exposed
as a pure `wordfreq_count` task instead. The ledger asks 0027's question: is
concatenation/interpolation the GENERAL fix, or is the histogram an
acceptable program shape? No surface was invented there.

Ocean ruled as BDFL on 2026-09-23 (review at 5b57dbd): adopt Option E. The
full options analysis is kept below; D remains as the considered
alternative.

## Decision

Adopted 2026-09-23 (BDFL ruling on review at 5b57dbd): **Option E**.

- Ship integer-to-Text conversion now: one builtin per integer type
  (`uint_to_text(n: UInt) -> Text`, `int_to_text(n: Int) -> Text`),
  implementing the normative rendering below.
- Defer concatenation until a program demonstrates the need.

Rationale: 0027's motivating-evidence rule. wordfreq's friction ledger
(entry #5) demonstrates number rendering — `hum 3` lines are emittable
today via sequential `stdout_write` calls. Nothing has yet demonstrated
text-as-value concatenation; the config parser's need is anticipated, not
filed. E follows 0027's letter; D's early stake, though cheap, is still a
bet on unfiled evidence.

D's analysis is kept in this record as the considered alternative, not
deleted: if the trigger below fires and the demonstrated need matches D's
shape, the reasoning is already on file.

Trigger reopening A: the config parser — or any program — files a friction
entry showing it must build text as a value (returned, stored, or carried
in a failure payload). When that entry lands, decide A's shape then, with
that evidence in hand: binary vs list-of-parts (the 0024
linear-vs-quadratic analysis above stands), builtin vs operator (F's
analysis stands).

Open question 2 is settled: one builtin per integer type. A single shared
name would need overload resolution the checker does not have — Hum
deliberately has no overloading story (operator overloading is explicitly
delayed in LANGUAGE_REFERENCE) — and distinct names are explicit,
greppable, and match the builtin style (`text_split`, not `split`).

## Constraints from prior decisions

- **0021** bans a general string library: join, trim, case mapping, and
  replacement stay out. Its precedents still bind: owned copies (not views),
  conditional fallibility (literal misuse is a checker error, computed misuse
  is a typed failure), and honest `allocates:` / `cost:`.
- **0024** plus the perf-debt rule: any decision trading performance for
  checker simplicity says so explicitly and gets a ledger entry. PD-001
  (text_split's owned copies) is the template for allocation honesty.
- **0016**: infallible operations need no `try`; any new failure mode needs
  a nominal root and honest `fails when:` declarations.
- **0027**: default to no on additions. Each option below gets the
  general-vs-specific test.

## Option A: a concat builtin

### Surface

One builtin, e.g. `text_concat(a: Text, b: Text) -> Text` — the two-argument
shape mirrors `text_split`. Multi-part rendering nests: `text_concat("hum ",
uint_to_text(n))`. A noted variant is a list form, `text_concat(parts: List
Text) -> Text`, which would compose directly with `text_split` output.

Arity is also a performance decision (0024): repeated binary concat in an
accumulation loop is quadratic — each iteration copies the accumulated
prefix. The list form is linear: one allocation at the end. If binary ships
as the primary form, the decision must name the loop-quadratic shape and
steer accumulation toward the list form; the candidate perf-debt entry:

> **[Proposed] PD-00X: binary text_concat in accumulation loops is
> quadratic.** Each chained concat copies the prefix built so far. The
> list-of-parts form is the linear path. Resurfaces when a builder or
> optimizer exists, or when loop accumulation gets a dedicated form.

### Allocation and cost honesty

One owned string of length `len(a) + len(b)`, O(n) time. Declared in the
builtin's `allocates:` and `cost:`. No views, so no PD-001-shaped debt — but
the cost is stated, not implied.

### 0016 interaction

None: concatenating two valid Texts cannot fail. Infallible, so no `try`,
no failure declaration, no new diagnostics. This is A's structural advantage
over text_split — it closes a gap without growing the failure surface.

### Agents: easy vs error-prone

Easy. One explicit function, one argument order to learn. Nested chains for
longer lines are verbose but never mysterious; there is almost nothing to
get wrong.

### Who needs it

wordfreq's `word: count` lines, and the config parser's diagnostics —
0026's order has the config parser testing 0016 across the trust crossing,
and its failure messages will need to name keys and line numbers
(`unknown key "timeout" at line 12`). Any program that renders structured
text needs composition.

### General-vs-specific

General. Text composition is needed by every program that renders output;
the config parser needs it independently of wordfreq, so this is not one
program's fix. It is the same shape of decision as 0021: one primitive, not
a library.

## Option B: interpolation / a format form

### Surface

Either new literal syntax (`"hum {count}"`) or a builtin with a placeholder
mini-language (`format("hum {}", count)`). Both add a second language inside
the language: the template DSL, with its own arity and type rules.

### Allocation and cost honesty

One owned string; cost scales with the argument count. Honestly declarable —
the problem is not cost, it is surface.

### 0016 interaction

The sharp edge. Placeholder/argument mismatches must go somewhere: literal
templates can be checker errors (the text_split H0636 precedent — literal
misuse is caught at check time), computed templates become typed failures
with new nominal roots. Either way it is new failure or diagnostic
machinery. Worse: if interpolation accepts any type, it needs a display
story per type — that is the general string library through the back door,
against 0021.

### Agents: easy vs error-prone

Easy to write — every agent has seen interpolation in a dozen languages —
but error-prone at the mismatch boundary. Whether a bad template is a
helpful diagnostic or a silent wrong rendering determines whether agents
love or fear it. The quality bar is entirely in diagnostics that do not
exist yet.

### Who needs it

The same programs as A, with nicer syntax. Nothing needs interpolation that
does not need concatenation; it is a nicer way to spell A's capability.

### General-vs-specific

General in intent, but the surface is larger than the demonstrated need, and
specifier creep (padding, alignment, precision) is the predictable failure
mode. 0027's default-to-no bites hardest here: B asks the language to carry
a mini-language for a nicer spelling of A.

## Option C: integer-to-Text conversion

### Surface

One builtin per integer type, e.g. `uint_to_text(n: UInt) -> Text` and
`int_to_text(n: Int) -> Text` — or a single overloaded name. The narrowest
surface of the four options.

### Allocation and cost honesty

One small owned string, O(digits) time. Trivially declarable.

### 0016 interaction

None: every `UInt` and `Int` has a decimal rendering. Infallible, no `try`.

### Agents: easy vs error-prone

Trivially easy. There is nothing to misuse.

### Who needs it

wordfreq's counts, the config parser's line numbers — anywhere a number
meets rendered text.

### Correction: C alone solves wordfreq's output

The first draft of this analysis was wrong. wordfreq does not need
concatenation to emit `hum 3` lines — stdout output composes by sequential
writes:

```hum
try stdout_write(word)
try stdout_write(" ")
try stdout_write(uint_to_text(n))
try stdout_write("\n")
```

Concatenation is needed only to build text as a VALUE — a `Text` that is
returned, stored, or carried in a failure payload, e.g. the config parser's
error messages. That need is anticipated, not yet demonstrated: no program
has filed a friction entry for it. Under 0027's motivating-evidence rule
(additions cite motivating evidence), C arrives with a ledger entry behind
it; A arrives with a named future consumer but no entry yet.

### General-vs-specific

General and demonstrated: rendering numbers as text is wordfreq's actual
friction, and the config parser's line numbers need it too. C is the only
option whose motivating evidence already exists.

## Option D: minimal combination (A + C)

### Surface

Two to three builtins: concatenation plus integer rendering. The smallest
surface that lets a program render `hum 3` and lets the config parser render
`line 12: unknown key "timeout"`.

### Allocation and cost honesty

The sum of the parts, each honestly declared. No new debt shape.

### 0016 interaction

Both pieces are infallible, so D adds no failure modes, no diagnostics, and
no `try` burden anywhere. This is D's strongest structural argument: it
closes the demonstrated gap without growing the failure surface at all.

### Agents: easy vs error-prone

Two easy primitives composed explicitly. Verbose for long lines, never
mysterious. The failure mode is aesthetic, not correctness.

### Who needs it

wordfreq and the config parser — the two programs in 0027's current order.

### General-vs-specific

Each piece is independently general (composition + rendering are not
wordfreq's private needs), and the combination is exactly the demonstrated
need, nothing more. D passes 0027's test where B strains it.

## Option E: C now, A when the config parser demonstrates it (adopted)

### Surface

Ship only integer rendering now. Concatenation waits until the config
parser — or another program — files a friction entry showing that
text-as-value composition is actually needed.

### The honest weighing against D

For E (the 0027-purist position): default-to-no exists precisely to stop
"almost certainly needed" from shipping surface. Anticipated need has a
habit of being wrong about the shape — the real friction entry might want
the list form, or structured error values instead of rendered text — and
then D's early binary concat is the wrong primitive shipped early. If the
need never materializes, the language stays smaller forever.

For D: the config parser's error values will need text composition —
failure payloads cannot be built from sequential writes, which go to
stdout, not to callers. D's extra surface is one or two infallible
builtins with zero failure modes and zero new diagnostics. The cost of E
being wrong is a second decision cycle in the middle of the config parser;
the cost of D being wrong is a small, independently general builtin
shipped early.

This is the closest call in the record. E follows 0027's letter; D bets a
small, cheap-to-be-wrong stake on 0027's spirit — the evidence is named and
concrete, just not yet filed.

## Option F: an operator (`+` or `++`)

### Surface

New syntax and type-checker work. `a + b` over `Text` is operator
overloading — which LANGUAGE_REFERENCE explicitly delays "until the formal
core, graph, diagnostics, and tooling can explain them." Spending that
deferred decision on string concatenation would be a strange first use. A
distinct `++` avoids overloading but still adds syntax, parsing, and
type-checking machinery for one builtin's job, plus its own misuse
diagnostics.

### Allocation and cost honesty

Identical to A. The operator changes spelling, not cost.

### 0016 interaction

None beyond A's: infallible either way.

### Agents: easy vs error-prone

This is F's real argument: it is what agents will reach for. Every
mainstream language spells concatenation `+` or `++`, so the builtin's
honest cost is the confused first attempt — an agent writing `"hum " + "3"`
and hitting a checker error.

### Why a builtin beats it — or doesn't

The builtin wins on machinery: no new syntax, no overload story, no new
diagnostics, explicit and greppable. And most of F's value is capturable
without the operator: a targeted diagnostic on `+` applied to `Text`
("use text_concat") redirects the agent's first instinct at the cost of one
error message instead of new syntax. F's remaining advantage is pure
spelling — real, but not load-bearing.

### General-vs-specific

The least general-shaped option: new syntax serving a single operation.

## Normative: integer rendering

Whichever option renders integers (C, D, or E) implements this exactly:

- base 10;
- `-` prefix for negatives (`Int`); no sign for `UInt`;
- no leading zeros;
- no digit separators;
- ASCII digits `0`–`9` only;
- locale-independent and deterministic: the same input produces the same
  bytes on every target.

## Out of scope: the frequency summary itself

Stated plainly: neither C nor D completes wordfreq's frequency summary.
Rendering `hum 3` lines is the easy half; finding the unique words and
their counts without a map/dictionary type is quadratic nested loops. That
is a separate friction-ledger item — a future map/dictionary design
question — not 0028's scope. 0028 is about rendering text, not about the
data structure that holds the counts.

## What is NOT on the table

- join-with-separator, trim, case mapping, replacement, padding/alignment
  specifiers: 0021's ban stands and is not re-litigated here.
- A general Display/ToString trait: the string library by another name.
- Newline modes on `stdout_write`: ledger entry #10's separate question.
- Multi-separator splitting: ledger entry #6, adjacent but separate.

## Open questions

Resolved by the ruling:

- Q2 (one builtin per type vs a single name): settled — one per type; see
  Decision.
- Q3 (is the histogram acceptable?): answered no — integer rendering
  ships.
- Q4 (E vs D): decided — E adopted; D kept as the considered alternative.

Deferred to the trigger:

- Q1 (binary vs list-of-parts): decided with the config parser's evidence,
  when A reopens. The 0024 linear-vs-quadratic analysis above stands.
- Q5 (is D a stable resting point?): moot unless A reopens; revisited then.

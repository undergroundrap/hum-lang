# 0028: Text rendering — concatenation, interpolation, and integer-to-Text conversion

Date: 2026-09-23
Status: proposed 2026-09-23. BDFL rules and merges.

## Context

wordfreq's friction ledger (PR #24, entry #5) found Hum's first big design
question: there is no text concatenation, no formatting/interpolation, and no
UInt-to-Text conversion, so no program can print `hum 3`. The honest output
with today's surface is one word per line (a histogram); the count is exposed
as a pure `wordfreq_count` task instead. The ledger asks 0027's question: is
concatenation/interpolation the GENERAL fix, or is the histogram an
acceptable program shape? No surface was invented there.

This record lays out the options. It does not choose — the taste call is
Claude's.

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
Text) -> Text`, which would compose directly with `text_split` output; the
arity shape is part of the taste call.

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

### General-vs-specific

Fails alone: without concatenation, `uint_to_text` cannot produce `hum 3`
— `"hum " + "3"` still needs joining. C is a complement, not an
alternative. As a standalone decision it would be shaped like a
program-specific fix; paired with A, it is the general fix.

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

## What is NOT on the table

- join-with-separator, trim, case mapping, replacement, padding/alignment
  specifiers: 0021's ban stands and is not re-litigated here.
- A general Display/ToString trait: the string library by another name.
- Newline modes on `stdout_write`: ledger entry #10's separate question.
- Multi-separator splitting: ledger entry #6, adjacent but separate.

## Open questions for the taste call

1. If A: binary concat vs the list-of-parts variant.
2. If C: one builtin per integer type vs a single name.
3. The ledger's original question still stands: is the histogram (no new
   surface) actually acceptable? If yes, none of this ships.
4. Is D a stable resting point, or does its verbosity for long lines
   predict a future join-with-separator request — i.e., is D the general
   fix or the first step toward the banned library?

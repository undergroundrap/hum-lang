# Algorithmic Foundations Charter

Date: 2026-08-03
Status: **non-normative.** This charter states principles that future decisions
may cite. It selects no syntax, versions no schema, adds no command, names no
package, and authorizes no work order. Where it and an accepted decision
record disagree, the decision record wins.

## Why this exists

A 2026 external research portfolio of ten mathematics and theoretical
computer science results prompted the question of what Hum should learn from
frontier algorithmic work. The durable answer is not a list of mathematical
domains to implement. It is that a strong algorithmic language must preserve
distinctions that ordinary programming environments routinely blur.

These principles stand independently of that portfolio's eventual status.
They would remain correct if every individual result were revised.

## Principles

**1. Representation is algorithmic content.** An expression tree, a shared
DAG, an arithmetic circuit, a formula, and a straight-line program are not
interchangeable. Neither are an adjacency list, CSR, an edge list, and a
packed bitset. Neither are a fixed-width integer, a mathematical integer, an
arbitrary-precision integer, a rational, a modular integer, a floating
approximation, and a validated interval. Representation, layout, and
allocation stay visible in the semantic graph until the compiler has a
justified reason to erase them.

**2. Arithmetic domains stay distinct, and their meaning stays stable.** A
theorem over mathematical integers, rationals, or reals does not justify
fixed-width or floating-point code without separate range, rounding, and
approximation evidence. A profile may *forbid* an operation, *require*
checked arithmetic, or *require* evidence. A profile must never *reinterpret*
an operator: `a + b` cannot mean checked arithmetic under one profile and
wrapping arithmetic under another. Source meaning is not deployment-dependent.
Alternate behavior is spelled explicitly at the call site.

**3. A complexity claim names its model.** `O(n)` alone is underspecified. A
useful claim identifies its size measures, arithmetic model (unit-cost versus
bit complexity), case (worst, expected, amortized, high-probability),
allocation behavior, precision dependence, and evidence state. Callback costs
compose visibly: a traversal invoking a callback `n` times cannot advertise
total `O(n)` without stating how callback cost composes. An operation-count
bound is not an elapsed-time bound and is not a deadline.

**4. Proof, measurement, and observation are different evidence.** A
benchmark is not a proof. A successful external prover build is not proof of
novelty, not proof that the formal statement matches the intended informal
one, and not authority to enable a compiler optimization. Hum owns source
semantics, obligation identity, and evidence policy; external engines return
receipts.

**5. Witnesses and receipts are distinct artifacts.** Keep these separate:
a **witness** an algorithm returns; a **counterexample**; a **certificate**
an independent checker validates; a **proof artifact** from a formal system;
and a **verification receipt** recording that some checker accepted
something. An algorithm returning an independently checkable witness is often
more valuable than one asserting correctness.

**6. A lower bound constrains promises; it does not produce an algorithm.**
Hardness results, impossibility results, and asymptotic bounds bound what any
implementation can claim. They yield no solver. In particular, circuit and
formula lower bounds are a standing warning against claiming that common
subexpression elimination or sharing can always compress algebraic
computation.

**7. Stable status is earned by evidence, breadth, and settled semantics.**
Specialist mathematics belongs in packages, labs, or examples -- never in the
stable standard library by enthusiasm. Promotion follows the existing
program-driven ladder: one program keeps it local, two unrelated programs
justify an experimental package, several independent programs with settled
semantics justify `std`.

## Deliberately not decided here

Numeric type names and spellings; wrapping and saturating surface forms;
`BigInt` and `Rational` representation and normalization strategy; graph view
abstractions, especially any mutable view; package boundaries and names; any
algorithm-claim schema version; any new CLI command; any syntax appearing in
research probes. Research probe code is illustrative and selects no grammar.

A new evidence schema is not justified until concrete probes demonstrate
facts that the existing resource-report, math-obligation, evidence, graph,
and benchmark surfaces genuinely cannot represent. Two sources of truth for
the same fact is a defect, not a feature.

## Suggested first concrete work, when the critical path allows

None of this is on the path to Hum's first compiled artifact and none of it
is authorized here. When capacity exists, the smallest revealing corpus is
three probes using currently reachable primitives:

- packed Hamming distance -- bit layout, tail canonicalization, popcount
  lowering, allocation-free loops, word-RAM complexity;
- union-find -- mutation, ownership, amortized versus worst-case claims,
  hidden allocation, invariants;
- read-only graph view with degeneracy ordering -- multiple layouts,
  certificate generation and checking, cross-layout differential results.

Those three would expose what a complexity and evidence model actually needs,
in the order: charter, arithmetic-semantics decision, probe corpus, then a
schema derived from the probes rather than imagined ahead of them.

## Provenance

Distilled 2026-08-03 from an external research report archived at
`docs/research/2026-08-03-astra-ten-advances-algorithmic-foundations.md` and
an independent critique of it. The report's principles are preserved; its
schema versions, command proposals, package names, numeric scorecard totals,
and implementation schedule are treated as provisional and are not adopted.

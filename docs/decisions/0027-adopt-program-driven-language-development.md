# 0027: Adopt Program-Driven Language Development

Date: 2026-09-23
Status: proposed 2026-09-23. BDFL rules and merges.

## Context

Hum's north star (0024) is a claim about humans and agents reading, writing,
and verifying real programs. Prose cannot carry that claim; programs must.
This record adopts the loop Hum now follows: real programs drive the
language, friction is recorded per program, and additions clear a
general-fix test before they land.

## The loop

1. **Real programs drive the language.** wordfreq is the first real program
   (WO27). Next: the config-file parser and the state machine with ownership
   transfer chosen by 0026 to stress the closed-world accountability
   objective — not to demo syntax.
2. **Friction goes into per-program friction ledgers.** Every friction point
   is classified: program gap, tooling gap, docs gap, or language gap. The
   ledger is a deliverable alongside the program.
3. **Missing surface is a STOP; gaps become decision records before
   implementation.** Worked examples: 0021 recorded the missing newline-literal
   mechanism as an open dependency blocking wordfreq's Part 2 instead of
   inventing one; 0022 resolved it as a decision record first.
4. **The general-vs-specific test.** An addition must be the general fix, not
   a one-program special case. Program-specific helpers are rejected by
   default. Precedent: 0021's "no general string library" ban — `text_split`
   is the single tokenization primitive; anything further needs its own
   decision record.
5. **Finished programs join the bake-off corpus.** The scorecard is re-run
   against each finished program, so the 0014 ownership bet keeps being
   tested (backlog item 2). Corpus provenance is recorded per program.
6. **"Best for humans and agents" is measured four ways:** the friction
   ledgers; the agent repair benchmark (agents fixing broken programs —
   success rate and attempts); the human review study (backlog item 3); and
   CI minutes per PR.

## Consequences

- No language addition lands without the ledger entry that motivated it and
  the decision record that specified it.
- The general-vs-specific test is a review gate: a proposal that serves only
  its motivating program is rejected by default.
- The bake-off scorecard and the four measures are re-run as programs finish.
  A program that stops the loop's measures from moving is reported, not
  hidden.

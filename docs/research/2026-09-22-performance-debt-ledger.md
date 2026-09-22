# Performance Debt Ledger

Date seeded: 2026-09-22
Status: living document. Append-only: entries are never deleted, only
dispositioned (paid / kept deliberately / moved). Each entry names the
decision that made the tradeoff, what was traded, and the condition under
which it resurfaces.

## Rule

From decision 0024 (proposed): any decision that trades performance for
checker simplicity says so explicitly in its decision record and gets an
entry here. When a real backend is designed, every open entry is reviewed
and dispositioned. Debt recorded is debt that can be repaid; debt unnamed
is debt quietly accepted as final.

## Open entries

### PD-001: text_split returns owned copies

- Decision: 0021, section "Ownership and cost (normative)".
- Tradeoff: a `List Text` of views would need element-alias and stored-view
  ownership relationships -- which view aliases which piece, how long each
  view lives relative to the list, what happens on append -- and decision
  0014 explicitly locks the ownership model against that shape. Copies
  avoid the entire question.
- Cost: one string allocation per piece, plus the list. Declared honestly
  in the builtin's `cost:` and `allocates:`; no zero-cost claim (0021).
- Resurfaces when: views into text become expressible, i.e. the ownership
  debt stages covering internal references and element aliases are proven
  (backlog item 4).

### PD-002: list growth API unsettled

- Decision: 0014, "Not Settled".
- Tradeoff: allocation and growth semantics for lists are unspecified, so
  whatever the prototype does is provisional and cannot be relied on for
  performance reasoning.
- Cost: unnamed.
- Resurfaces when: the list growth API is specified.

### PD-003: contract checks deferred to runtime

- Decision: 0015 (classified runtime contract policy).
- Tradeoff: checks the compiler cannot discharge statically run at runtime.
  Each deferral is a performance decision; made by accident unless the
  classifier names it.
- Cost: per-check runtime cost, currently unnamed per check.
- Resurfaces when: the checker discharges more checks statically, or a
  real backend forces the accounting.

### PD-004: termination-measure checks

- Decision: 0020 (adopt termination measures and loop bounds).
- Tradeoff: termination measures and loop bounds are adopted as part of
  the language; their enforcement cost in the prototype is unnamed.
- Cost: unnamed.
- Resurfaces when: the enforcement strategy is specified for a real
  backend.

## Dispositioned entries

(none yet)

## Notes

- Seed source: the 2026-09-22 north-star review (Claude review relayed by
  Ocean; researcher draft). The seed items are Ocean's list plus Claude's
  framing: owned copies, list growth, runtime-deferred contract checks,
  termination-measure checks.
- This ledger tracks *performance* debt only. Checker-completeness debt
  lives in the ownership burn-down (decision 0014, backlog item 4); safety
  debt lives in the safety-profile architecture (backlog item 7).

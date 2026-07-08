# Numerics Policy And Text Model Research

Date: 2026-07-08
Status: distilled snapshot, triaged by external reviewer

## Provenance And Confidence

Source: agent-run deep research commissioned against the FORMAL_CORE open
question on floating point and the future Text/Bytes stdlib boundary.
Citation markers in the raw report were tool-mangled; substance was
cross-checked against reviewer knowledge. The report's own "thin
evidence"/"unknown" flags are preserved.

## 1. Integer Overflow

Findings accepted:

- Rust's debug-trap/release-wrap split (RFC 560) made semantics depend on
  build profile; friction reports persist a decade later. No empirical
  corpus proves the split right or wrong — the evidence for "checked is
  too expensive" is folklore, and the report demands measurement.
- Zig (`+%`, `+|`), Swift (checked default, `&+` opt-out), and Ada
  (constrained ranges, Constraint_Error) converge on the same lesson:
  the exceptional arithmetic mode must be explicit, not ambient.
- Real code mixes mathematical and bit-level arithmetic often enough
  (Understanding Integer Overflow in C/C++, TOSEM 2015) that ambient
  wrap semantics are indefensible.

Direction adopted:

```text
ordinary + - * on fixed-width integers trap in ALL build modes:
  profile-invariant semantics, same audit story everywhere
explicit families for deliberate bit work: wrapping_*, saturating_*,
  checked_*, overflowing_*
compile-time range analysis may elide checks only when semantics are
  provably identical
benchmark gate: before any "fast numerics" exception, measure checked
  arithmetic on representative kernels (x86-64 and ARM64) and publish the
  delta; the quantitative evidence does not exist elsewhere
type-level ranges remain attractive for contracts-heavy code (Ada lesson)
  as a future extension
```

Note: `hum run` has trapped on overflow since Session B. That behavior is
now evidence-backed policy, not a placeholder.

## 2. Floating Point And Decimal

Findings accepted:

- Reproducibility breakers are well-documented: FMA contraction, fast-math
  reassociation, NaN payload nondeterminism, relaxed SIMD, platform libm
  differences, x87 excess-precision history. Rust documents transcendental
  results as platform- and version-dependent; Wasm needs a deterministic
  profile plus NaN canonicalization to be fully deterministic; Java
  relaxed strictness in 1.2 and returned to always-strict in 17.
- Lockstep/replay practice (GGPO, Photon Quantum, the RTS lineage) either
  pins platforms tightly or removes floats from core simulation in favor
  of fixed-point. Quantum replaces every float with fixed-point FP.
- Safety-critical practice constrains and documents FP rather than banning
  it: defined standard compliance, no unreliable equality, auditable usage.
- Decimal is mandatory in finance practice (BigDecimal, .NET decimal),
  thin-to-unknown as a mandate in medical dosing. Library-first.

Direction adopted:

```text
two FP regimes:
  native mode: IEEE binary floats, no fast-math by default, contraction
    and relaxation only by explicit source-visible opt-in
  deterministic mode: fails closed - fixed rounding, canonical NaNs, FMA
    pinned on or off (never target-dependent), no platform libm for
    replay-critical code, SIMD deterministic or disallowed
lockstep simulation guidance: fixed-point / scaled integers in core
  simulation; floats stay out of replay-critical state
Float stays outside the trusted core (existing FORMAL_CORE stance) until
  the deterministic regime is specified
decimal: library first; builtin only if a finance/medical wedge demands it
safety profiles: FP use is annotated and call-graph reportable
```

## 3. Text And String Model

Findings accepted:

- The three mainstream models are all coherent but serve different
  centers: Go (bytes, minimal validation), Rust (guaranteed-valid UTF-8,
  integer indexing forbidden), Swift (grapheme-correct surface). Swift's
  retrospective is the richest: it kept the Unicode-correct surface but
  switched storage to UTF-8, added small-string optimization, and
  deprecated misused offset APIs. Go's json v2 experiment now rejects
  invalid UTF-8 by default — even byte-centric ecosystems validate at API
  boundaries.
- Indexing honesty is non-negotiable: only byte indexing is O(1) on UTF-8.
  No design may pretend s[i] is "the i-th character."
- Windows is decisive for a Windows-first language: UTF-16 APIs, unpaired
  surrogates in real filenames (the Go filename bug), WTF-8 as the
  lossless bridge, Rust's OsStr as the working precedent including its
  documented ergonomic pain.
- Zero-copy parsing wants bytes-first with validation as an explicit
  obligation (simdjson lesson) — which maps directly onto Hum contracts:
  "well-formed bytes," "valid UTF-8," and "normalized domain text" are
  distinct checkable claims.
- Small-string optimization is a backend concern, not surface semantics.
  Embedded/no-alloc profiles need a fixed-capacity text/buffer family.

Direction adopted:

```text
three tiers at the stdlib boundary:
  Bytes:  transport, parsing, zero-copy; no text guarantees
  Text:   guaranteed-valid UTF-8; integer indexing forbidden; explicit
          bytes()/scalars()/graphemes() views and cursor APIs
  OsText: lossless platform-native bridge (WTF-8-shaped on Windows);
          every filesystem and process API takes OsText, not Text
Text round-trips to UTF-8 exactly; OsText round-trips platform-native
  exactly
fixed-capacity text/buffer family before any embedded claim
```

## 4. Source And Identifier Policy

Findings accepted:

- Trojan Source (bidi controls, confusables) is a real reviewed-source
  attack class; UTS #39/UAX #31 are the governing standards; Rust's
  confusables lint regime is the ready-made blueprint if Unicode
  identifiers are ever wanted.
- Hum is already ahead by accident of decision 0012: value identifiers
  are `[a-z_][a-z0-9_]*` and type names `[A-Z][A-Za-z0-9]*` — ASCII-only
  by construction. The entire confusable/normalization problem space is
  currently unrepresentable in Hum source.

Direction adopted:

```text
record the ASCII-only identifier grammar as deliberate policy, not
  accident: any future Unicode-identifier proposal must bring UTS #39
  confusable detection, NFC normalization, and a pinned Unicode version
  per language edition with it
extend check_text_hygiene to reject bidirectional control characters in
  all source and docs (closes the remaining Trojan Source vector in
  string literals and comments)
source files remain UTF-8 without BOM (existing policy, confirmed)
```

## Open Verification Debt

- No trustworthy modern benchmark corpus exists for always-checked integer
  arithmetic or grapheme segmentation costs; both are required Hum
  benchmarking programs, not literature lookups.
- Swift's current identifier-normalization enforcement status is unknown;
  irrelevant to Hum while identifiers stay ASCII.

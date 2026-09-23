# 0026: Name Closed-World Accountability As Hum's Non-Bolt-On Property

Date: 2026-09-22
Status: proposed (revision 2 — pre-issuance review fixes folded in).
Recommendation only; the BDFL rules. Execution review: Claude, 2026-09-22
— with disclosure that the thesis wording is Claude's, so that review
covers execution accuracy only, not the truth of the thesis (see Ruling).
A second reviewer independent of the thesis authorship should weigh the
thesis itself before any PR.

## Evidence Labels

Claims in this record are labeled:

- **[Pinned]** — established by Hum repository sources cited inline.
- **[External]** — established by targeted research conducted 2026-09-22
  against primary sources (tool documentation, papers, project repositories).
- **[Inference]** — the author's engineering judgment. Argued, not proven.

## Context

Backlog item 6 names the landscape's sharpest counterargument against Hum as
a language project: everything except the intent ladder could be bolted onto
rustc tomorrow as JSON output plus a linter. If intent sections turn out to
be expressible as Rust attributes or macros plus a linter, Hum's language bet
weakens to "a really good toolchain" **[Pinned]**
(docs/research/hum-improvement-backlog-2026-09-22.md §6). The item's
evidence bar: the record names the property, the bolt-on attempt that fails,
and what dies if the claim is wrong. Ocean approves, revises, or rejects —
it is his call whether Hum is a language project or a toolchain project.

This record proposes the property, tests it against the strongest available
bolt-on attempt, maps which Hum decisions carry it and which current gaps
break it, states the falsifier, and draws the consequence for choosing
programs 2 and 3.

## The Property

**Closed-world accountability:** for every obligation in a Hum program, the
compiler can say whether it was proved, bounded, or explicitly trusted — and
nothing can escape that account. **[Inference]** (thesis under test; the
phrasing is Claude's, the testing is this record's).

Two halves, both load-bearing:

1. **The ledger exists as a compiler-produced artifact.** Not five dialects
   a human assembles, but one account the compiler emits: every obligation
   labeled, every trusted item enumerated.
2. **The world is closed.** There is no language construct — macro, build
   hook, foreign call, unsafe block — that silently leaves the account.
   Trust can be *explicit* (a first-class label), never *ambient*.

If either half is bolt-on-able, the property is not Hum's.

## The Bolt-On Attempt (Steelman)

The strongest version of "Rust already does this", assembled from
best-in-class parts **[External]**:

> Take safe Rust. Forbid `unsafe` in every first-party crate via workspace
> lints — a hard compiler gate, un-overridable, running on post-expansion
> code, also catching `no_mangle`, `export_name`, and `link_section`. Vet
> the dependency closure with cargo-vet plus `cargo geiger` in CI. Verify
> the critical core with Verus — whole-crate SMT verification with ghost
> code and linear ghost permissions, the one tool that *can* reason about
> unsafe code — and use Creusot contracts or Kani harnesses where each fits
> best (deductive unbounded proofs; exhaustive bounded checking with
> unwinding assertions; stubs printed per harness). Confine I/O code to
> cap-std voluntarily. Result: memory safety from the compiler, functional
> correctness where specified, unsafe code itself verified, supply chain by
> vetting. Every obligation is compiler-checked, proved, bounded, or
> explicitly trusted. That *is* the ledger — assembled from best-in-class
> parts.

Each part is real and each practitioner cited would endorse their part. The
steelman is strong at the single-crate, single-tool level. It fails exactly
at the thesis point. (Flux and Aeneas were considered as steelman parts and
are addressed under points 1, 4, and 6 below.)

## Where The Steelman Breaks

**1. The ledger does not exist as an artifact.** "Explicitly trusted" is
scattered across mutually invisible places: `#[trusted]` attributes
(Creusot, Flux), hand-written `extern_spec!` axioms, `kani::assume()`
calls, `#[kani::stub]`s, Verus's `external_body` and
`assume_specification` annotations, and the audited-but-unverified
dependency set. No tool emits the unified list; a human assembles it by
reading half a dozen dialects — real Verus projects each built their own
manual "honesty" enumerations because Verus emits none. Hum's thesis
demands the *compiler* say it. The ecosystem requires a person to.
**[External]**

**2. The unsafe ban is per-crate, not per-program.** The `unsafe_code` lint
is allow-by-default and opt-in per crate **[External]** (rustc lint docs).
Dependencies are untouched by your crate's `forbid`; practitioners run the
full dependency scan "only for visibility" because core dependencies
legitimately use unsafe **[External]** (aegaeon unsafe-code-policy,
2026-07). Build scripts are separate crates — your `forbid` does not apply
unless the build-script crate declares it too **[External]** — and they
execute arbitrary code at compile time, emit linker flags, and generate
`include!`d source. Proc-macro crates are separate crates with arbitrary
compile-time execution; their generated `unsafe` is caught post-expansion
**[Inference]** (lints run on expanded HIR; no counterexample found), but
the macro's own logic — what it generates from file, environment, or
network input — is unverified. PL/Rust, which uses `forbid(unsafe_code)` as
a real sandboxing boundary, states it outright: "It is the administrator's
responsibility to properly vet external dependencies" **[External]**
(plrust.io lint docs). A `cargo update` can silently grow the trusted-unsafe
set and nothing in the language stops the build. **[Inference]**

**3. FFI is the unverifiable floor, and every real program bottoms out
there.** Creusot cannot reason about unsafe Rust — the rust-lang
std-verification goal doc states it flatly **[External]** — so foreign
functions and std's unsafe interior are opaquely trusted via user-written
axioms. Kani treats a reachable foreign call as an unsupported construct:
verification *fails* unless the call is hand-stubbed, i.e. trusted
**[External]** (Kani issue #3679; stubbing docs). Verus is the honest best
case: it *can* verify unsafe code through linear ghost permissions, and
real verified systems (Anvil's Kubernetes controllers) are built on it
**[External]**. But its trust doors are explicit and load-bearing:
`external_body` assumes a specification without verifying the body
("wrong specifications can subvert Verus's guarantees"), and
`assume_specification` is carried under a CAUTION banner as unchecked
**[External]**. A `--no-cheating` flag rejects all of them — and real
projects cannot pass it (gale carries 133 `external_body` instances plus 2
`assume_specification` calls, documented by hand) **[External]**. The
trusted base is not small or stable: it is all of std's unsafe plus every
dependency's unsafe plus every stub, plus the forked rustc driver, Z3, and
vstd's assumed specs — with build scripts and proc macros executing outside
verification entirely.

**4. No composition rule.** A Creusot proof of `f` and a Kani bounded-proof
of `g` do not combine into a program-level theorem. The trust bases are
disjoint (Why3 plus SMT solvers vs. CBMC), the notions of "done" differ,
and no checker consumes both outputs to certify coverage of the program's
obligations. Aeneas makes the point sharpest: the "account" is split
across two systems (Rust source plus Lean proofs), the link between them
— translation faithfulness — is the single largest trusted assumption and
is itself unverified, and with no whole-program analysis at all a
program-level ledger is not even representable in its architecture.
**[External + Inference]**

**5. The capability leg is fictional.** There is no production capability
linter for Rust. cap-std is a voluntary library; nothing stops any crate in
the closure from calling `std::fs::File::open("/anything")`, and cap-std
itself ships `Dir::open_ambient_dir`, documented as not sandboxed
**[External]** (cap-std README). Ambient authority remains one call away,
anywhere in the closure.

**6. Each tool admits soundness gaps a ledger would have to label.** Kani
verifies sequential code only: no data races, no aliasing-model violations,
no inline assembly **[External]** (Kani soundness docs). Creusot cannot
reason about unsafe Rust **[External]** (rust-lang std-verification goals)
and its proofs are architecture-specific **[External]** (Creusot
limitations); the full MIR-to-functional pipeline has no mechanized
soundness proof **[External]** (Creusot ICFEM'22 paper). Flux tracks
pointer alignment and provenance but not values written through pointers
**[External]**, and cannot distinguish logic errors from undefined
behavior **[External]**; its refinements are restricted to a decidable
quantifier-free fragment, so properties like sortedness are hard to
specify **[External]**. Every tool trusts large unverified components:
rustc's MIR construction, solver and CBMC correctness, the Charon
translation.

**7. "Bounded" is not compiler-tracked.** Kani's bound is a flag plus
human-audited `assume`s — and `kani::assume()` that eliminates all paths
makes verification "succeed" trivially; the ecosystem's own guidance warns
reviewers to demand `kani::cover!` **[External]**. Hum's "bounded" must be a
label the compiler assigns and defends, not a number in a harness file.
**[Inference]**

The obstacles are structural — per-crate lints, tool-fragmented accounts,
an unverifiable FFI floor, no composition — not matters of more annotation
effort. **[Inference]**

## Hum's Own Trusted Base

The obvious counter, stated before a critic states it: the closed world is
at the *Hum source* level. Below it sits Hum's own toolchain — checker,
interpreter, JIT — written in Rust, with its own dependency closure
**[Pinned]** (0002: Rust bootstrap until staged self-hosting is proven;
0017: the main crate is a Rust crate held under `#![deny(unsafe_code)]`).
**[Inference]** That is the same position as Rust trusting rustc. The
thesis is about *the program's* obligations, not the absence of a trusted
computing base: the ledger accounts for Hum programs; it does not account
for the toolchain that computes it, and no Rust tool accounts for rustc
either. What the thesis forbids is *unacknowledged* trust *inside* the
program's world — which is why Hum's own `external-trust` label and the
priced JIT exception sit at the boundary, named, rather than inside it,
silent.

## Which Hum Decisions Carry The Property

**0015 — the ledger's vocabulary.** Every contract eligible for
mode-dependent treatment must eventually be classified `proved`,
`boundary`, `unproved`, or `external-trust`; classification is
conservative and ordered; unknown fails closed as `unproved`; boundary,
unproved, and external-trust contracts are never elided **[Pinned]**
(0015-adopt-classified-runtime-contract-policy.md). This is the
accountability classification itself: the four labels are exactly the
"proved, bounded, or explicitly trusted" account, with `boundary` and
`external-trust` as the two honest names for trust the compiler cannot
discharge.

**0016 — no hidden control flow.** Known fallible calls never propagate
implicitly; unwrapped `try` requires matching nominal error roots; the
runtime carrier retains root identity, origin, and every propagation site
**[Pinned]** (0016-adopt-explicit-causal-typed-failure.md). Obligations
cannot silently move: every failure path is a labeled, visible edge in the
account.

**0017 — the closed world.** Closed direct-call capability analysis: the app
declares the maximum source authority, tasks declare transitive closure,
callers cover callees, unknown capability-like IDs fail closed **[Pinned]**.
Source declaration is never consent; effective authority is the intersection
of source maximum, reachable closure, and exact operator grant, minus deny
**[Pinned]**. The main crate keeps `#![deny(unsafe_code)]` as its default,
with exactly one reviewed, locally allowed JIT invocation boundary — a
priced, scoped exception that grants no broader authority **[Pinned]**.
Sandbox-bypass authority (process launch, FFI, unsafe, unrestricted import)
is a separate severity tier, never an ordinary grant **[Pinned]**
(0017-adopt-structural-app-authority-boundary.md).

**0020 — "bounded" made precise.** Termination measures (qualitative
progress evidence) and quantitative loop bounds are separate concepts that
do not merge; an invariant is not progress evidence **[Pinned]**
(0020-adopt-termination-measures-and-loop-bounds.md). A bounded claim must
say which kind of bound it carries — the compiler-tracked answer to the
steelman's point 7.

**SPEC non-goals — the world stays closed by design.** "Do not make macros
the first escape hatch." "Do not hide effects behind innocent-looking
calls." **[Pinned]** (SPEC.md). The language refuses the two constructs
most likely to smuggle obligations out of the account.

**0005 — the composition point.** External verifiers are evidence
producers, not compiler authority **[Pinned]**
(0001–0011 index; 0005-keep-verifiers-as-evidence-producers.md). When Hum
eventually consumes Creusot- or Kani-shaped evidence, the *compiler* still
authors the ledger — the answer to the steelman's point 4 is architectural,
not aspirational.

**0014 §3 — a mechanism instance.** Checked source relationships for
returned and stored views **[Pinned]**
(0014-adopt-ownership-model.md §3): one concrete place where the ledger's
discipline already reaches into the type system.

## Gaps That Currently Break It

Honesty first: today Hum does **not** deliver this property. The record is
a design commitment, and reading it as a status claim would be false.
**[Inference]**

- **0015's classifier is unimplemented.** "This decision defines the
  classifications that a future checker must assign; current Hum assigns
  none." **[Pinned]** The vocabulary exists; the assignment does not. Until
  it ships, output must say no contract has been mechanically classified.
- **0014's ownership debt.** Hum "must not claim full ownership safety,
  internal-reference support, disjoint-field precision, or memory-safety
  completeness until those repairs are built and tested against the corpus."
  **[Pinned]** Memory obligations are the largest unaccounted region.
- **`allocates:`, `cost:`, and `calls:` are declared-only.** They are
  parsed intent sections, not checked obligations **[Pinned]** (SPEC.md).
  The ledger cannot account for what it does not check; every declared-only
  section is a hole shaped exactly like a promise.
- **Higher-order blame is unsettled; effect polymorphism is pending**
  **[Pinned]** (0015 consequences). Tasks-as-values cross the account's
  current edge.
- **The JIT exception proves the world has a priced door.** 0017's single
  reviewed exception is scoped today. Each future exception must stay
  priced and scoped, or the "closed" claim rots one exemption at a time.
  **[Inference]** The exception count and the growth of the
  `external-trust` label are the internal metrics to watch.

## Alternatives Considered

- **Checked-vs-declared labeling as the property.** Folded in: it *is* the
  ledger's vocabulary (0015). Not a rival; the mechanism the property is
  written in.
- **Predicate-ladder evaluation semantics.** A mechanism, narrower than the
  property; carried by the ledger rather than competing with it.
- **0014 §3 view relationships.** A mechanism instance (listed above), not
  the whole account.
- **"No unsafe, period" as the property.** Rejected: 0017 already prices
  one exception, and the property is about the *account*, not the absence
  of trust. `external-trust` is a first-class label precisely so that
  trusted things are *named*, not banished. A property defined as "no
  trust" collapses the first time a real program needs the OS.

## The Falsifier

What evidence would show Hum is a toolchain project, not a language
**[Inference]**:

1. **A machine-checkable whole-program obligation ledger appears for
   Rust** — one artifact, every obligation labeled proved/bounded/trusted,
   the trusted set enumerated and diffable across builds, composed across
   tools. Nothing in the ecosystem produces this today; it would require
   Creusot-plus-Kani integration plus dependency accounting.
2. **A compiler-driver-level no-unsafe-in-closure gate ships** (deps and
   build scripts included — not per-crate, not policy-level) **plus
   enforced capability discipline program-wide.** The closed world would
   then be expressible as policy over rustc, and Hum's structural answer
   would be redundant.
3. **Empirical:** a large real program built the steelman way publishes its
   assembled ledger, showing the residual trusted set is small, stable, and
   reviewable. No such artifact exists today.
4. **Internal:** Hum's own world proves unmaintainable — the priced
   exceptions multiply, `external-trust` swallows the ledger, and the
   account becomes fiction. Watch the JIT-exception count.

**What dies if the claim is wrong** **[Pinned]** (backlog item 6): stop
growing the checker. A toolchain project should be stealing AI-Co's
diagnostic contract and PyBun's SARIF/CycloneDX emission, not building a
verification core. Every ownership and checker investment made on the
language bet would be going to the wrong place.

## Consequence For Programs 2 And 3

Programs 2 and 3 should be chosen to stress this property, not to demo
syntax **[Inference]** (recommendation):

- **Program 2: a config-file parser.** Exercises the `external-trust`
  boundary in its most common real-world shape: untrusted input crosses a
  checked boundary into proved internal handling. It tests 0016's typed
  failure across the trust crossing and forces honesty from `allocates:` /
  `cost:` for buffers — the declared-only sections most likely to lie
  first.
- **Program 3: a small state machine with ownership transfer.** Exercises
  0014's closure discipline plus 0015's proved internal obligations plus
  0017's authority boundary across state transitions. It tests whether the
  closed world holds when control moves, without obligations leaking at
  the handoff.

Both should be instrumented against the friction ledger: where exactly does
the accountability demand bite the programmer? The try-rule question
(design question #1) is downstream of this evidence — decide it with
wordfreq's ledger, not taste.

## What Acceptance Commits Us To

Accepting this record is not free **[Inference]** (recommendation):

- The 0015 classifier moves up in priority. It is the half of the property
  Hum does not have at all — vocabulary without assignment.
- Declared-only sections (`allocates:`, `cost:`, `calls:`) must either
  become checked obligations or be explicitly labeled as unaccounted. A
  promise-shaped hole is worse than no promise.
- The priced-exception discipline becomes a tracked metric: the JIT
  exception count and the growth of the `external-trust` label, reviewed
  like a budget.
- Programs 2 and 3 are chosen to stress the property (above), and their
  friction ledgers feed back into the try-rule decision.

## Sources

Every **[External]** claim above is checkable at one of these primary
sources (researched 2026-09-22):

**Verus**

- Verus paper (SMT verification, ghost code, linear ghost permissions,
  unsafe reasoning): https://arxiv.org/abs/2303.05491
- Anvil verified Kubernetes controllers:
  https://github.com/anvil-verifier/anvil
- `external_body` ("wrong specifications can subvert Verus's guarantees"):
  https://github.com/verus-lang/verus/blob/HEAD/source/docs/guide/src/calling-unverified-from-verified.md
- `assume_specification` (unchecked, CAUTION banner):
  https://github.com/verus-lang/verus/blob/HEAD/source/docs/guide/src/reference-assume-specification.md
- `--no-cheating` vs real-project trust inventories (gale: 133
  `external_body` + 2 `assume_specification`):
  https://github.com/pulseengine/gale/blob/HEAD/docs/research/verus-quickstart.md
  and https://github.com/pulseengine/gale/blob/HEAD/docs/safety/verification-honesty.md

**Flux**

- PLDI 2023 paper (refinement types for Rust):
  https://dl.acm.org/doi/10.1145/3591283 and http://arxiv.org/pdf/2207.04034
- Tool summary (checks, `trusted`/`ignore`, unsafe limits,
  quantifier-free ceiling):
  https://github.com/model-checking/verify-rust-std/blob/HEAD/doc/src/tools/flux.md
- `#[flux_rs::trusted]` ("simply *trust* that the specification is
  correct"):
  https://github.com/flux-rs/flux/blob/HEAD/book/src/guide/specifications.md

**Aeneas**

- ICFP 2022 paper (Rust-to-functional translation, no whole-program
  analysis): https://arxiv.org/abs/2206.07185
- Architecture overview:
  https://github.com/aeneasverif/aeneas/blob/HEAD/documentation/aeneas-overview.md
- Translation trust base (`#print axioms`, unverified translator):
  https://github.com/pulseengine/ordeal/blob/HEAD/docs/formal-verification.md

**Creusot**

- "A deductive verifier for (safe) Rust code":
  https://github.com/creusot-rs/creusot/blob/HEAD/ARCHITECTURE.md
- Limitations (architecture-specific proofs):
  https://github.com/creusot-rs/creusot/blob/HEAD/guide/src/limitations.md
- ICFEM'22 paper (RustHornBelt mechanization gap):
  https://jhjourdan.mketjh.fr/pdf/denis2022creusot.pdf
- Community std-verification lessons (incl. the Flux pointer-write
  limitation at §2.2): https://arxiv.org/pdf/2510.01072

**Kani**

- Soundness ("What Kani Does NOT Check"; CBMC caveats):
  https://github.com/model-checking/kani/blob/HEAD/docs/src/soundness.md
- RFC 0004 (no unbounded control flow):
  https://github.com/model-checking/kani/blob/HEAD/rfc/src/rfcs/0004-loop-contract-synthesis.md
- Issue #3679 (foreign function unsupported; aws-lc-sys):
  https://github.com/model-checking/kani/issues/3679
- Stubbing (`#[kani::stub]`, per-harness stub printing):
  https://github.com/model-checking/kani/blob/HEAD/docs/src/reference/experimental/stubbing.md
- Vacuous-`assume` guidance (`kani::cover!`):
  https://github.com/strawgate/memagent/blob/HEAD/dev-docs/references/kani-verification.md

**Lints, policy, capabilities**

- rustc `unsafe_code` lint (catches `unsafe`, `no_mangle`,
  `export_name`, `link_section`):
  https://doc.rust-lang.org/stable/nightly-rustc/rustc_lint/builtin/static.UNSAFE_CODE.html
- rust-lang 2024-H2 std-verification goal ("cannot reason about unsafe
  Rust"): https://github.com/rust-lang/goals/blob/HEAD/src/2024h2/std-verification.md
- aegaeon unsafe-code policy (per-crate geiger gate; deps scan for
  visibility only; build scripts are separate crates):
  https://github.com/codetakt/aegaeon/blob/HEAD/docs/policies/unsafe-code-policy.md
- PL/Rust lint docs ("the administrator's responsibility to properly vet
  external dependencies"): https://plrust.io/config-lints.html
- cap-std README (`open_ambient_dir` not sandboxed; "not a sandbox"):
  https://github.com/bytecodealliance/cap-std/blob/HEAD/README.md
- Creusot project trust inventory ("What remains trusted"):
  https://github.com/crumplecup/elicitation/blob/HEAD/CREUSOT_TRACKING.md

## Ruling

Proposed. This record is a recommendation only: Ocean rules — accept,
revise, or reject. It is his call whether Hum is a language project or a
toolchain project.

**Review history.** Pre-issuance execution review: Claude, 2026-09-22 —
with disclosure that the thesis wording is Claude's, so that review covers
execution accuracy (pinned-claim spot checks, honesty of the gaps,
falsifier specificity), not the truth of the thesis itself. Its required
fixes are folded into this revision: Verus, Flux, and Aeneas in the
steelman; Hum's own trusted base named; source URLs added. A second
reviewer, independent of the thesis authorship, should weigh whether the
thesis is true before acceptance. No PR until then.

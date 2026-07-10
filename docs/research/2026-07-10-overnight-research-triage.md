# Overnight Deep-Research Batch: Triage Index

Date: 2026-07-10
Status: triaged by the architect-reviewer; raw reports in `deep/`

## Provenance And Confidence

Eight agent-run deep-research reports commissioned against the standing
decision queue, run overnight, triaged on return. Inline citation markers
were tool-mangled and stripped on import; source identities survive in
prose (system names, papers, dates). The raw reports are archived as
plain-text attachments in `deep/` because their embedded pseudocode is
markdown-hostile; this index is the maintained layer. Substance was
cross-checked against reviewer knowledge; each report's own
"thin evidence"/"unknown" flags are preserved and were, throughout,
placed where they belong. Batch verdict: seven strong, zero weak, two
change near-term plans.

## Verdicts And Routing

1. [Capability grants and operator consent](deep/2026-07-10-capability-grants-consent.txt)
   is STRONG and feeds decision 0017 and Sessions Y/Z now. It validates
   the 0017 algebra (declared intersected with granted, default-empty,
   deny-wins) against Deno, Fuchsia, WASI, Pony, and seL4 convergence.
   Adopted as reviewer gate criteria for Y/Z: typed exact grants
   (kind/scope/strength/lifetime); sandbox-bypass authorities as a
   separate severity tier; consent prompts as a scarce resource (no
   startup prompts, task-coupled, fatigue budget); audit logs joining
   policy snapshot + decision event + exercise event with request IDs,
   passing a forensic replay test; persistence explicit and reviewable,
   wildcards visibly dangerous.
2. [Effect polymorphism corpus](deep/2026-07-10-effect-polymorphism-corpus.txt)
   is STRONG and defines bake-off queue #1's advocate documents.
   Decisive: Effekt-style second-class computations fail half the corpus
   by design (stored callbacks and returned closures are inexpressible)
   and are eliminated as a core candidate. The bake-off is Koka rows vs
   Flix formulas vs Scala-style capture checking, with the reviewer note
   that capture checking may be underweighted for Hum specifically,
   since decision 0017 makes authority-as-values the actual effect
   story. Gates adopted verbatim: ten-program type-check, one-diagnostic
   rejections, near-principal inference, closure capture of owned
   resources. Two toy front-end prototypes beat argument.
3. [Flagship wedge](deep/2026-07-10-flagship-wedge.txt) is STRONG and
   names the product: the air-gapped update validator (one command, one
   verdict, one evidence dossier). Work Order 6's Sessions Z-AD build
   almost exactly its primitive set; Work Order 7 candidate. The
   toolchain report makes Hum's own release kit the tool's first
   customer. Seven launch-demo tests adopted for whenever it ships.
4. [Structured concurrency](deep/2026-07-10-structured-concurrency.txt)
   is STRONG and feeds bake-off queue #2 and the fault-domain work
   order. Synthesis: Trio cancellation semantics (cancelled means it did
   not happen unless documented; visible checkpoints), Swift transfer
   checking at spawn boundaries (fits moves/views/linears), Java reified
   task tree, Kotlin as footgun catalog. Hum ruling absorbed: children
   inherit cancellation lineage and tracing automatically, authority
   never. Cancel-safety classification (safe/partial/unsafe) becomes
   stdlib spec; two-phase cleanup; linear-cancel conformance suite.
5. [Generics and coherence](deep/2026-07-10-generics-coherence.txt) is
   STRONG and feeds bake-off queue #3. Coherence with sanctioned
   adapters; no specialization in any MVP; explicit erased forms (the
   Swift any lesson); no implicit instance search early. Open territory
   identified: no language ships law-bearing generic bounds, so
   contracts on type parameters with witness blame (a third blame party)
   is novel ground where Hum's machinery fits. Cost-model neutrality
   gate: semantics identical across interpreted, dictionary, and
   monomorphized lowering.
6. [Interpreter to native](deep/2026-07-10-interpreter-to-native.txt) is
   STRONG and proposes a backend-ladder amendment for the appropriate
   ADR moment: Wasm/WASI components move earlier (deny-by-default
   runtime aligns with 0017 exactly), Cranelift is for fast dev builds
   rather than Windows release polish (unwinding and debug info still
   weak), the interpreter stays the permanent semantic oracle behind
   differential gates, and backend promotion criteria should be
   published before any backend exists. SPARK anchors check-cost
   reality: roughly 10% typical, 95-98% provable away.
7. [Windows toolchain shipping](deep/2026-07-10-windows-toolchain-shipping.txt)
   is STRONG and pre-positions the public alpha. Dual trust signals
   (simple offline signature plus attestation, SLSA L2), both SBOM
   formats from one graph, four Windows channels, reproducible-release
   policy rather than marketing claim, air-gapped kit rather than bare
   binary, no auto-update. The 2026 fact worth knowing early: EV
   certificates no longer bypass SmartScreen, so the reputation
   cold-start must be designed around.
8. [Language book pedagogy](deep/2026-07-10-language-book-pedagogy.txt)
   is STRONG and upgrades backlog 21. Delayed hard-idea reveal; Brown
   evidence for visualized ownership chapters and conceptual questions;
   an executable book with machine-readable snippet classes and a
   Windows CI lane; an edition-tied stable book plus experimental track;
   dropout telemetry as a product metric.

9. [Agent eval design](deep/2026-07-10-agent-eval-design.txt) is STRONG
   and the most methodologically hardened of the batch; it governs risk
   R016's gate on any public agent-native claim. Adopted: claims are
   staged (tool-learnable with docs, then feedback-learnable with
   compiler output only, then maintainable at scale) and never blurred;
   the honest phrasing is "no known Hum exposure under a documented
   non-exposure protocol," never "absent from all training data"; the
   harness enforces no internet, no VCS history, immutable plus hidden
   tests, a hidden grader, cheat-labeled trajectory review, and post-hoc
   adversarial equivalence checks after green; diagnostic-utilization is
   measured causally (the next patch must touch the cited span);
   statistics need roughly 384 sealed tasks for plus-or-minus five
   points, Wilson intervals, preregistered scaffolds stress-tested under
   semantic prompt mutation; the publication package is six artifacts
   including trajectories and a human-novice baseline; and the primary
   evaluation track is Windows-native, matching the language.

## Cross-Cutting Observation

Four independent research passes have now converged on the spine Hum
already has: deny-by-default authority, explicit transfer at boundaries,
evidence artifacts over promises, interpreter as ground truth. The
design sits at an attractor other careful thinkers fall into; that
convergence is itself adoption evidence.

## Deferred Actions

- WORKORDER backlog edits (wedge tool as Work Order 7 candidate;
  capability gates noted in Session Y/Z review criteria) wait for the
  Session X commit, because WORKORDER.md is in that session's diff.
- The backend-ladder amendment (Wasm components earlier) is an ADR for
  the first-backend decision moment, not now.

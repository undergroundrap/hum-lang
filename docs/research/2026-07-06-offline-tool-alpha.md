<!--
Research artifact imported on 2026-07-06.
Normalization: explicit UTF-8 decode, Deep Research UI citation markers stripped, typographic punctuation converted to ASCII, saved as UTF-8 without BOM.
Source names are preserved, but citation-only evidence cells may be blank; future runs should request direct source URLs in the Markdown body.
-->
## Executive thesis

**[Fact - current Hum state from your prompt, 2026-07-06]** Hum is not yet credible as a public "systems language" alpha because it does not execute code, has no production runtime, no package/deployment story, no stdlib, and no demonstrated safety/performance boundary.

**[Inference]** The smallest credible public alpha is therefore **not** "Hum can replace Rust/Go/Python." It is:

> **Hum 0.1 Alpha: an offline evidence-producing language profile for security and change-review tools.**

The alpha should let engineers write a small class of **deterministic, file-only, no-network Hum tools** that parse release/change bundles, check declared intent/effects against policy, and emit audit artifacts: semantic graph, effect report, SBOM, provenance, tests, profile report, diagnostics, and benchmarks.

**[Fact - 2026-07-06]** This wedge matches what high-trust software teams are already being pushed toward: NIST SSDF is a secure software development framework published in **February 2022**; OMB M-22-18, dated **2022-09-14**, requires U.S. federal agencies to obtain secure-development attestations for third-party software and says agencies may require artifacts such as SBOMs; OMB M-23-16, dated **2023-06-09**, reaffirmed that direction and discusses provenance for internal and third-party code; SLSA v1.2 defines supply-chain levels and provenance; and NTIA's SBOM minimum-elements report defines SBOM data, automation, and practice/process expectations. ([NIST Cybersecurity Resources][1])

**[Inference]** Hum's first serious wedge is: **"I can inspect, run, and audit this offline tool and know exactly what it can read, write, prove, deny, and emit."** That is more credible for cybersecurity, DevOps/SRE, and defense/offline engineers than claiming a new language can already beat Rust at systems programming.

---

# 1. Smallest credible public alpha: exact feature set, dependency order

Name it something like:

> **Hum 0.1-alpha: `offline-tool@0.1` profile**

One-line promise:

> **Write a deterministic local Hum tool that reads declared input files, performs bounded checks, writes only to a declared evidence directory, and emits machine-readable security evidence.**

Non-promise:

> **Not for production services, kernels, embedded, networking, async, plugins, FFI, package ecosystems, or general application development.**

## Required profile

**[Inference]** The alpha should ship exactly one executable profile:

```text
profile offline-tool@0.1

Allowed:
- parse Hum source
- check Hum source
- run bounded Hum tasks
- read declared files/directories
- write only to declared output directory
- parse strict JSON
- emit canonical JSON evidence
- compute SHA-256 digests
- compare manifests, SBOMs, provenance, and policy facts
- run Hum tests

Denied:
- network
- process execution
- dynamic loading
- FFI
- unsafe
- threads
- wall-clock-dependent logic
- randomness
- environment-variable reads by default
- writes outside --out
- hidden mutation
- undeclared file reads
```

**[Fact - 2026-07-06]** NSA's memory-safety guidance, updated as version 1.1 in **April 2023**, recommends memory-safe languages where feasible, but also notes that mature infrastructure is hard to shift and that unsafe functions/libraries can remain risk points even in memory-safe languages.

**[Inference]** For Hum, that means the alpha should avoid FFI, unsafe extensions, and dependency sprawl entirely. The trust story must be simpler than Rust's, not merely similar.

## Feature set ranked by dependency order

| Order | Alpha feature                                       | Concrete acceptance criteria                                                                                                                                                                                                                                                                                                                                                                                                        | Required evidence artifacts                                                             |
| ----: | --------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
|     0 | **Alpha charter and threat model**                  | `docs/alpha-charter.md` states: alpha scope, non-goals, threat model, supported OSes, "not production," and what evidence Hum emits. Every public claim has a test or artifact reference.                                                                                                                                                                                                                                           | `alpha-charter.md`, `threat-model.md`, `claims-matrix.json`                             |
|     1 | **Hermetic Rust bootstrap build**                   | Builds from source on Windows x64 and one Linux x64 target using documented commands. Compiler crate uses `#![forbid(unsafe_code)]`. No third-party crates. No network required after toolchain install. `hum --version` prints commit, build profile, target, date, and feature flags.                                                                                                                                             | `build-manifest.json`, `checksums.sha256`, `unsafe-review.md`, `compiler-sbom.cdx.json` |
|     2 | **Pinned alpha language subset**                    | A versioned grammar accepts only `task`, `type`, `store`, `test`, and checked blocks needed for the demo: `why`, `uses`, `changes`, `needs`, `ensures`, `protects`, `watch for`, `cost`, `avoids`, `tests`, `proves`, `does`. Unsupported syntax fails with named diagnostics.                                                                                                                                                      | `grammar.md`, `language-subset.v0.1.md`, parser golden tests                            |
|     3 | **Formal-core mapping for the subset**              | Every accepted surface construct lowers to a small documented core. No accepted syntax may be "graph-only" without defined meaning. Core output is deterministic and source-spanned.                                                                                                                                                                                                                                                | `core.json`, `formal-core-subset.md`, `surface-to-core-map.json`                        |
|     4 | **Canonical semantic graph v0.1**                   | `hum graph` emits stable, versioned JSON with deterministic node IDs, source spans, declared intent, inferred effects, tests, dependencies, and evidence links. Same input produces byte-identical graph under `--deterministic`.                                                                                                                                                                                                   | `hum-graph.v0.1.json`, `graph-schema.v0.1.json`, graph golden tests                     |
|     5 | **Static type checker for the subset**              | Static types cover records, enums, strings, bools, integers, lists, maps, optional/result forms, and domain-specific evidence records. No implicit mutation, null, narrowing, or unchecked fallthrough. All diagnostics have codes and source spans.                                                                                                                                                                                | `type-report.json`, diagnostic snapshots                                                |
|     6 | **Effect and capability checker**                   | `offline-tool@0.1` rejects undeclared reads, writes, mutation, network, process execution, clock, randomness, FFI, dynamic loading, and output path escapes. Declared vs inferred effects must be displayed side by side.                                                                                                                                                                                                           | `hum-effects.v0.1.json`, denied-effect tests, `profile-report.json`                     |
|     7 | **Deterministic interpreter for bounded Hum tasks** | `hum run <task> --profile offline-tool@0.1 --in <dir> --out <dir>` executes only bounded file/JSON/hash/check/report tasks. No recursion in alpha. Loops only over bounded collections. Same input/output under three repeated runs.                                                                                                                                                                                                | `run-trace.json`, `observed-effects.json`, `profile-report.json`                        |
|     8 | **Minimal security stdlib**                         | Ships only what the demo needs: strict JSON parser/writer, canonical JSON emitter, SHA-256, path normalization, digest manifest parser, SBOM reader subset, SLSA/in-toto provenance reader subset, sorted maps/lists, diagnostics helpers. JSON parser rejects duplicate object names for security predictability. RFC 8259 says object names SHOULD be unique and duplicate names create unpredictable behavior. ([RFC Editor][2]) | `stdlib-graph.json`, known-answer hash tests, JSON negative corpus                      |
|     9 | **Evidence emitters**                               | `hum evidence` emits graph, effects, profile, SBOM, provenance, tests, diagnostics, benchmarks, audit summary, and checksums into one evidence directory. SBOM must contain NTIA minimum fields where known. Provenance should use in-toto/SLSA-compatible structure.                                                                                                                                                               | `evidence/` directory with fixed filenames listed below                                 |
|    10 | **Test runner and generated checks**                | `hum test` runs handwritten `test` blocks and generated negative checks from `ensures`, `protects`, `avoids`, and `watch for`. Failing tests show source spans and intent clauses, not just assertions.                                                                                                                                                                                                                             | `hum-tests.v0.1.json`, optional `junit.xml`, coverage-by-graph-node report              |
|    11 | **Human-grade diagnostics**                         | Every failure has: code, severity, source span, violated intent clause, inferred fact, suggested fix, and evidence link. No parser/typechecker/interpreter panics on invalid input corpus.                                                                                                                                                                                                                                          | `diagnostics.json`, `diagnostics-catalog.md`, panic-free fuzz/negative corpus results   |
|    12 | **Demo corpus and benchmark harness**               | Public corpus contains known-good and seeded-bad release bundles. Benchmark command records hardware, OS, date, compiler commit, corpus size, wall time, peak memory if available, and output hashes. No performance superiority claim unless raw data supports it.                                                                                                                                                                 | `hum-bench.v0.1.json`, `benchmark.md`, corpus manifests                                 |
|    13 | **Release packet**                                  | A GitHub release or source tarball contains source, binary if offered, lock/build instructions, hashes, SBOM, provenance, tests, benchmark results, docs, and the full demo evidence packet.                                                                                                                                                                                                                                        | `release-evidence/`, `release-checksums.sha256`, `release-provenance.intoto.json`       |

---

# 2. Recommended killer demo

## Primary demo: **HumGate - Air-Gapped Release and Change Gate**

**[Inference]** This is the best first demo because it uses Hum's current strengths: intent, effects, diagnostics, semantic graph, offline operation, and evidence. It avoids competing head-on with Rust/Go/Python as a production service runtime.

### Demo story

A defense/SRE/security team receives a release bundle intended for an offline or restricted environment.

The bundle contains:

```text
release-bundle/
  artifacts/
    service.exe
    helper.dll
  manifests/
    checksums.sha256
  sbom/
    sbom.cdx.json
  provenance/
    provenance.intoto.json
  deploy/
    k8s-deployment.json
    terraform-plan.json
  policy/
    release_gate.hum
```

The Hum policy says, in readable checked intent:

```hum
task approve_release_bundle(bundle: ReleaseBundle) -> Approval
why
  prevent importing unreviewed software into an offline environment
uses
  bundle.sbom
  bundle.provenance
  bundle.checksums
  bundle.deploy_plan
needs
  all artifacts have matching digests
  all runtime components are declared in the SBOM
  provenance builder identity is allowed
  deployment does not add public egress
changes
  writes evidence report only
protects
  offline enclave boundary
  artifact integrity
  operator review time
avoids
  undeclared network paths
  privileged containers
  host filesystem mounts
  dependency drift
watch for
  digest mismatch
  SBOM missing component
  provenance subject mismatch
  deployment requests 0.0.0.0/0 egress
  privileged container flag
ensures
  approval is issued only if all checks pass
tests
  seeded_good_bundle_passes
  seeded_bad_bundle_fails_with_explanations
does
  ...
```

### What the demo proves

**[Inference]** The "wait, this is different" moment should be:

```text
Hum does not merely run a script.
Hum shows exactly:
- what the tool is allowed to read
- what it is allowed to write
- what it promises to protect
- what evidence supports approval
- what evidence caused rejection
- which source intent clause each decision came from
- which generated tests guard that behavior
- which artifacts can be handed to security/procurement/review teams
```

### Demo acceptance criteria

| Area              | Acceptance criteria                                                                                                                                                                                                                                                                            |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Offline operation | Runs on a fresh Windows x64 machine with no network access after install/source checkout. Also runs on one Linux x64 target.                                                                                                                                                                   |
| No hidden powers  | Effect report proves no network, process execution, FFI, dynamic loading, clock, randomness, or writes outside `--out`.                                                                                                                                                                        |
| Determinism       | With `--deterministic`, three runs over the same bundle produce byte-identical graph, effects, diagnostics, approval result, and evidence JSON.                                                                                                                                                |
| Positive corpus   | At least 5 known-good bundles pass. Each pass emits an approval trace explaining why.                                                                                                                                                                                                          |
| Negative corpus   | At least 15 seeded-bad bundles fail, including digest mismatch, missing SBOM component, provenance subject mismatch, unapproved builder ID, public egress, privileged container, host mount, output path escape, undeclared file read, duplicate JSON key, and missing required intent clause. |
| Diagnostics       | Every rejection has diagnostic code, source span, violated intent clause, observed fact, and suggested remediation.                                                                                                                                                                            |
| Evidence packet   | Every run emits complete `evidence/` directory with graph, effects, profile, SBOM summary, provenance check, tests, diagnostics, benchmark, and audit summary.                                                                                                                                 |
| Benchmark honesty | Publishes raw timing/memory results on a dated machine. The demo only needs to be fast enough for review use; it must not claim Rust/C++-class performance.                                                                                                                                    |
| Build trust       | Compiler source forbids unsafe Rust, uses no third-party crates, and ships its own compiler SBOM/checksums.                                                                                                                                                                                    |
| Review usefulness | A skeptical engineer can answer in under 5 minutes: "What can this tool do to my machine?" and "Why did it approve or reject this release?"                                                                                                                                                    |

### Required HumGate evidence directory

```text
evidence/
  hum-graph.v0.1.json
  hum-effects.v0.1.json
  hum-profile.v0.1.json
  hum-diagnostics.v0.1.json
  hum-tests.v0.1.json
  hum-bench.v0.1.json
  hum-policy-trace.v0.1.json
  hum-sbom-summary.v0.1.json
  hum-provenance-check.v0.1.json
  hum-audit.md
  compiler-sbom.cdx.json
  release-sbom.cdx.json
  release-provenance.intoto.json
  checksums.sha256
```

**[Fact - 2026-07-06]** CycloneDX describes itself as a modular framework for software/system transparency and includes components, dependency graphs, vulnerabilities/VEX, formulation, provenance/pedigree, and declarations; its site lists CycloneDX 1.7 with release date **2025-10-21**. SPDX lists the SPDX specification as ISO/IEC 5962:2021 and current SPDX 3.0. ([CycloneDX][3])

**[Inference]** For alpha, pick **one required SBOM format** to avoid scope creep. I would choose **CycloneDX JSON** for the demo because its object model is broad enough for software, services, dependencies, vulnerabilities, formulation, and declarations. Emit SPDX later.

---

# 3. Two backup demos

## Backup demo A: **Agent-safe runbook dry-run gate**

**[Inference]** Good for cybersecurity and SRE, but more dangerous than HumGate if it actually executes commands. Keep it dry-run-only in alpha.

Scenario:

```text
An AI agent proposes:
- rotate logs
- restart service
- upload diagnostics
- inspect secrets
- open outbound URL
```

Hum accepts only the declared local read/write actions and rejects secrets/network/process actions unless explicitly authorized by profile.

Acceptance criteria:

| Area            | Criteria                                                                                                      |
| --------------- | ------------------------------------------------------------------------------------------------------------- |
| Execution mode  | Dry-run only for alpha. No process execution.                                                                 |
| Input           | Agent plan as strict JSON.                                                                                    |
| Policy          | Hum file declares allowed paths, forbidden secrets, allowed outputs, and operator approval requirements.      |
| Rejection cases | Network access, process execution, secret exfiltration, output path escape, and undeclared mutation all fail. |
| Evidence        | Emits graph, effect report, policy trace, denied actions, tests, and audit summary.                           |

Why it is a backup:

**[Inference]** It is trend-relevant, but if Hum touches real command execution too early, skeptical engineers will see a new attack surface rather than a trust tool.

## Backup demo B: **Offline SBOM/provenance verifier**

Scenario:

```text
Given an artifact directory, SBOM, provenance file, and checksum manifest,
Hum verifies component presence, digest consistency, builder identity,
source repo identity, and declared dependencies.
```

Acceptance criteria:

| Area                | Criteria                                                                                                          |
| ------------------- | ----------------------------------------------------------------------------------------------------------------- |
| Offline operation   | Fully local.                                                                                                      |
| Standards alignment | Reads pinned CycloneDX JSON subset and SLSA/in-toto provenance subset.                                            |
| Negative cases      | Missing component, digest mismatch, unknown builder, duplicate JSON key, missing timestamp, malformed provenance. |
| Evidence            | Emits SBOM summary, provenance check, graph, effects, diagnostics, and audit markdown.                            |

Why it is a backup:

**[Inference]** This is easier to build and useful, but it risks looking like "a Go/Python script with nicer syntax." HumGate is stronger because it connects intent, deployment change, evidence, and capability checking.

---

# 4. Evidence artifacts Hum must emit

| Artifact                         | Required minimum contents                                                                                                         | Why skeptical engineers care                                                                                                                                           |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `hum-graph.v0.1.json`            | Tasks, types, stores, tests, intent clauses, source spans, dependencies, inferred effects, evidence links, diagnostics links.     | Lets humans/tools inspect meaning without rereading source.                                                                                                            |
| `hum-effects.v0.1.json`          | Declared effects, inferred effects, observed runtime effects, denied effects, profile constraints, source spans.                  | Core differentiator versus Python/Go scripts.                                                                                                                          |
| `hum-profile.v0.1.json`          | Profile name/version, allowed/denied capabilities, OS, compiler commit, resource limits, deterministic mode.                      | Shows the operational sandbox contract.                                                                                                                                |
| `hum-policy-trace.v0.1.json`     | For each pass/fail decision: input fact, policy clause, source span, evidence file, result.                                       | Turns intent blocks into reviewable evidence.                                                                                                                          |
| `hum-diagnostics.v0.1.json`      | Codes, severity, spans, violated clause, inferred fact, remediation, related graph nodes.                                         | Makes failures useful in security review.                                                                                                                              |
| `hum-tests.v0.1.json`            | Handwritten tests, generated tests, negative corpus results, graph-node coverage.                                                 | Shows promises are checked, not just comments.                                                                                                                         |
| `hum-bench.v0.1.json`            | Command, corpus, machine, OS, date, compiler commit, elapsed time, peak memory if available, output hashes.                       | Prevents vague performance claims.                                                                                                                                     |
| `compiler-sbom.cdx.json`         | Hum compiler component, version, source commit, dependencies, supplier, timestamp, hashes.                                        | Demonstrates Hum applies evidence to itself.                                                                                                                           |
| `release-sbom.cdx.json`          | Demo bundle SBOM or normalized summary with supplier/component/version/hash/dependency information.                               | Aligns with SBOM expectations. NTIA lists supplier, component name, version, identifiers, dependency relationship, author, and timestamp as minimum data fields.       |
| `release-provenance.intoto.json` | Builder identity, source material, subjects/digests, build invocation, timestamps where available.                                | Aligns with SLSA/in-toto provenance expectations. SLSA describes provenance as verifiable information about where, when, and how an artifact was produced. ([SLSA][4]) |
| `unsafe-review.md`               | Statement that compiler and runtime profile use no `unsafe`, no FFI, no third-party crates, no dynamic loading; CI checks listed. | Directly addresses hidden-power concerns.                                                                                                                              |
| `hum-audit.md`                   | Human-readable summary: approved/rejected, why, protected assets, failed checks, commands to reproduce.                           | The artifact a reviewer can actually read.                                                                                                                             |
| `checksums.sha256`               | Hashes of inputs, outputs, compiler binary if shipped, and evidence files.                                                        | Supports reproducibility and tamper detection.                                                                                                                         |

**[Inference]** Do not make SARIF, OpenTelemetry, SPDX, Rego, CUE, Kubernetes-native admission control, or Terraform-native parsing mandatory in the first alpha. JSON evidence with stable schemas is enough for credibility.

---

# 5. What to explicitly defer

| Defer                                       | Reason                                                                                               |
| ------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Native code generation                      | Competing with Rust/C++ before semantics/effects are proven is a trap.                               |
| Network services                            | Go/Rust already win; Hum has no runtime maturity.                                                    |
| Process execution                           | Too risky for first alpha; it turns Hum into a sandbox/security product before the model is proven.  |
| FFI and `unsafe`                            | Would destroy the "no hidden power" trust story before the review model is mature.                   |
| Package registry                            | A registry creates supply-chain liability. Alpha should be source-only and vendored/offline.         |
| Broad stdlib                                | Every API needs semantics/effects/diagnostics/tests. Keep stdlib tiny.                               |
| Kubernetes YAML/HCL parsers                 | Use exported canonical JSON for alpha. Native parsers are scope creep.                               |
| Cloud-hosted services or telemetry          | Violates offline-first/defense trust story.                                                          |
| AI agent execution                          | Dry-run policy checking only; real execution later.                                                  |
| Formal proof integration with Lean/F*/Dafny | Valuable later, but alpha needs checked semantics and evidence before theorem-prover ambition.       |
| Performance claims versus Rust/C++          | Benchmarks should show honesty and determinism, not superiority.                                     |
| Compliance claims                           | Hum can emit evidence; it cannot certify FedRAMP, CUI, HIPAA, PCI, ATO, or RMF compliance by itself. |

**[Fact - 2026-07-06]** NIST SP 800-37 Rev. 2 describes the Risk Management Framework as a process involving categorization, control selection, implementation, assessment, authorization, and continuous monitoring; NIST SP 800-53 provides a catalog of security and privacy controls; NIST SP 800-171 Rev. 3 covers protecting Controlled Unclassified Information in nonfederal systems. A programming language can help generate evidence, but it does not replace those organizational processes. ([NIST Cybersecurity Resources][5])

---

# 6. What would make "just use Rust/Go/Python" still correct?

## Use Rust when

**[Fact]** Rust already has a mature story for memory safety in safe code, while allowing explicitly marked `unsafe` for low-level operations; Rust documentation describes unsafe blocks as enabling specific low-level capabilities and recommends wrapping unsafe code in safe abstractions. ([Rust Documentation][6])

**[Fact]** Cargo supports vendoring dependencies and locked/offline/frozen operation, which matters for air-gapped and reproducible builds. ([Rust Documentation][7])

**[Inference]** Therefore, "just use Rust" is correct if:

* You need production native performance now.
* You need mature crates, crypto, parsers, network stacks, embedded support, or OS APIs.
* You need FFI.
* You need a production CLI/service, not an evidence experiment.
* Your team already has Rust review expertise.
* Hum cannot produce materially better effect/evidence artifacts than a Rust tool plus SBOM/provenance pipeline.

## Use Go when

**[Inference]** "Just use Go" is correct if:

* The tool is a normal SRE/cloud/security CLI.
* You need Kubernetes, cloud SDKs, HTTP, concurrency, cross-compilation, and fast team onboarding.
* The main value is operational automation, not language-level intent/effect evidence.
* You can get sufficient auditability from Go code review, SBOM generation, SLSA provenance, and policy tools.

## Use Python when

**[Inference]** "Just use Python" is correct if:

* It is a short-lived internal script.
* Library availability matters more than static guarantees.
* Human review cost is low.
* Runtime determinism and effect control are not central.
* The output is exploratory, not a high-trust approval gate.

## Use existing policy/supply-chain tools when

**[Inference]** "Do not use Hum" is correct if:

* OPA/Rego, CUE, SLSA tooling, in-toto, SBOM scanners, or CI policy engines already solve the problem.
* Hum's semantic graph does not reduce review time.
* Hum's evidence is not accepted by the target team's actual process.
* The Hum compiler itself becomes a larger trust burden than the script/policy it replaces.

---

# 7. The narrow alpha success bar

**[Inference]** A skeptical cybersecurity/SRE/defense engineer should be able to run:

```bash
hum check policy/release_gate.hum
hum graph policy/release_gate.hum --out evidence/hum-graph.v0.1.json
hum test policy/release_gate.hum --corpus corpus/
hum run approve_release_bundle \
  --profile offline-tool@0.1 \
  --in release-bundle/ \
  --out evidence/ \
  --deterministic
hum evidence evidence/
```

And then answer:

```text
1. What did this tool promise to do?
2. What files did it read?
3. What files could it write?
4. Did it try to use network/process/unsafe/clock/random?
5. Which policy clause approved or rejected the release?
6. Which tests prove the seeded bad cases fail?
7. Which artifact hashes and provenance were checked?
8. Can I reproduce the evidence packet offline?
```

If the answer to any of those is "not yet," the alpha is not credible.

---

# 8. Public alpha cut line

## Must be in alpha

**[Inference]**

* Parser/checker for the pinned subset.
* Formal-core subset document.
* Canonical semantic graph schema.
* Type checker.
* Effect/profile checker.
* Deterministic bounded interpreter.
* Strict JSON and SHA-256 support.
* SBOM/provenance/checksum readers for demo subset.
* Evidence emitters.
* Test runner.
* Golden positive/negative corpus.
* Windows-first build proof.
* No unsafe, no FFI, no third-party crates.
* Honest benchmark harness.
* Docs that define every accepted construct.

## Must not be in alpha

**[Inference]**

* "Hello world" general-purpose language marketing.
* Package registry.
* Native backend.
* Network/async.
* Production claims.
* Broad standard library.
* AI codegen claims.
* "Compliance-ready" claims.
* Runtime plugins.
* Unsafe escape hatches.
* Hidden compiler magic without graph evidence.

---

# 9. Durable source URLs

All source facts below were checked for this answer on **2026-07-06**.

1. **NIST SSDF SP 800-218**, published **February 2022**
   URL: [https://csrc.nist.gov/pubs/sp/800/218/final](https://csrc.nist.gov/pubs/sp/800/218/final)
   DOI: [https://doi.org/10.6028/NIST.SP.800-218](https://doi.org/10.6028/NIST.SP.800-218)

2. **OMB M-22-18**, dated **2022-09-14**
   URL: [https://www.whitehouse.gov/wp-content/uploads/2022/09/M-22-18.pdf](https://www.whitehouse.gov/wp-content/uploads/2022/09/M-22-18.pdf)

3. **OMB M-23-16**, dated **2023-06-09**
   URL: [https://www.whitehouse.gov/wp-content/uploads/2023/06/M-23-16-Update-to-M-22-18-Enhancing-Software-Security.pdf](https://www.whitehouse.gov/wp-content/uploads/2023/06/M-23-16-Update-to-M-22-18-Enhancing-Software-Security.pdf)

4. **NTIA SBOM Minimum Elements Report**
   URL: [https://www.ntia.gov/files/ntia/publications/sbom_minimum_elements_report.pdf](https://www.ntia.gov/files/ntia/publications/sbom_minimum_elements_report.pdf)

5. **SLSA v1.2 specification**, approved/current page as checked **2026-07-06**
   URL: [https://slsa.dev/spec/v1.2/](https://slsa.dev/spec/v1.2/)

6. **SLSA provenance v1.2**
   URL: [https://slsa.dev/spec/v1.2/provenance](https://slsa.dev/spec/v1.2/provenance)

7. **in-toto project**
   URL: [https://in-toto.io/](https://in-toto.io/)

8. **CycloneDX specification overview**, lists CycloneDX 1.7 release date **2025-10-21**
   URL: [https://cyclonedx.org/specification/overview/](https://cyclonedx.org/specification/overview/)

9. **CycloneDX security use cases**
   URL: [https://cyclonedx.org/use-cases/security/](https://cyclonedx.org/use-cases/security/)

10. **SPDX specifications**, SPDX listed as ISO/IEC 5962:2021 and current SPDX 3.0
    URL: [https://spdx.dev/use/specifications/](https://spdx.dev/use/specifications/)

11. **NIST SP 800-171 Rev. 3**, published **May 2024**
    URL: [https://csrc.nist.gov/pubs/sp/800/171/r3/final](https://csrc.nist.gov/pubs/sp/800/171/r3/final)
    DOI: [https://doi.org/10.6028/NIST.SP.800-171r3](https://doi.org/10.6028/NIST.SP.800-171r3)

12. **NIST SP 800-53 Rev. 5 update 1**, original Rev. 5 date **September 2020**, planning note updated **2025-08-27**
    URL: [https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final](https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final)

13. **NIST SP 800-37 Rev. 2**, published **December 2018**
    URL: [https://csrc.nist.gov/pubs/sp/800/37/r2/final](https://csrc.nist.gov/pubs/sp/800/37/r2/final)

14. **NSA Cybersecurity Information Sheet: Software Memory Safety**, version 1.1, **April 2023**
    URL: [https://media.defense.gov/2022/Nov/10/2003112742/-1/-1/0/CSI_SOFTWARE_MEMORY_SAFETY.PDF](https://media.defense.gov/2022/Nov/10/2003112742/-1/-1/0/CSI_SOFTWARE_MEMORY_SAFETY.PDF)

15. **Rust Book: Unsafe Rust**
    URL: [https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html)

16. **Cargo vendor/offline/frozen documentation**
    URL: [https://doc.rust-lang.org/cargo/commands/cargo-vendor.html](https://doc.rust-lang.org/cargo/commands/cargo-vendor.html)

17. **RFC 8259: JSON**, published **December 2017**
    URL: [https://www.rfc-editor.org/rfc/rfc8259](https://www.rfc-editor.org/rfc/rfc8259)

18. **RFC 8785: JSON Canonicalization Scheme**
    URL: [https://www.rfc-editor.org/rfc/rfc8785](https://www.rfc-editor.org/rfc/rfc8785)

19. **NIST FIPS 180-4 Secure Hash Standard**
    URL: [https://csrc.nist.gov/pubs/fips/180-4/upd1/final](https://csrc.nist.gov/pubs/fips/180-4/upd1/final)

20. **Reproducible Builds definition**
    URL: [https://reproducible-builds.org/docs/definition/](https://reproducible-builds.org/docs/definition/)

21. **Reproducible Builds project overview**
    URL: [https://reproducible-builds.org/](https://reproducible-builds.org/)

22. **OpenTelemetry overview**, page last modified **2026-04-06**
    URL: [https://opentelemetry.io/docs/what-is-opentelemetry/](https://opentelemetry.io/docs/what-is-opentelemetry/)

---

## Bottom line

**[Inference]** The first alpha should make one claim only:

> **Hum can produce a small offline security tool whose intent, effects, inputs, outputs, tests, and evidence are inspectable and reproducible.**

If Hum cannot ship that, public alpha is premature.

If Hum can ship that, the killer demo is **HumGate**, not a web server, not a package manager, not a Rust competitor, and not a general-purpose language pitch.

[1]: https://csrc.nist.gov/pubs/sp/800/218/final "SP 800-218, Secure Software Development Framework (SSDF) Version 1.1: Recommendations for Mitigating the Risk of Software Vulnerabilities | CSRC"
[2]: https://www.rfc-editor.org/rfc/rfc8259 "RFC 8259: The JavaScript Object Notation (JSON) Data Interchange Format | RFC Editor"
[3]: https://cyclonedx.org/specification/overview/ "Specification Overview | CycloneDX"
[4]: https://slsa.dev/spec/v1.2/provenance "SLSA - Provenance"
[5]: https://csrc.nist.gov/pubs/sp/800/37/r2/final "SP 800-37 Rev. 2, Risk Management Framework for Information Systems and Organizations: A System Life Cycle Approach for Security and Privacy | CSRC"
[6]: https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html "Unsafe Rust - The Rust Programming Language"
[7]: https://doc.rust-lang.org/cargo/commands/cargo-vendor.html "cargo vendor - The Cargo Book"

# Hum Language Project Risk Register 2026

Date: 2026-07-06

## Purpose

This is Hum's ground-truth risk register for making a new programming language
on a real developer machine.

The goal is to keep Hum ambitious without letting the project hurt the machine,
overpromise safety, or recreate known language/toolchain failures.

Hum treats maker safety and user safety as one design obligation. See
[SAFETY_OF_MAKER_AND_USER.md](SAFETY_OF_MAKER_AND_USER.md).

## Brutal Conclusion

The riskiest part of making Hum is not choosing the wrong keyword.

The riskiest parts are:

- running untrusted code
- downloading dependencies too early
- trusting package build scripts
- lowering into backend undefined behavior incorrectly
- exposing casual FFI
- accepting compiler bugs as rare corner cases
- letting scope grow faster than proof
- self-hosting before the compiler is trustworthy
- claiming security before the evidence exists

Hum should treat those as project-level hazards, not later cleanup tasks.

## Risk Severity Key

```text
critical  can compromise the machine, toolchain, users, or safety claims
high      can make Hum unsound, unsafe, or impossible to trust
medium    can slow adoption, portability, or maintainability
low       annoying but recoverable
```

## Risk Register

### R001: Tool Executes Untrusted Code

Severity: critical.

Bad outcome:

`.hum` source, package metadata, generated code, plugins, build scripts, or test
fixtures cause the toolchain to execute attacker-controlled code.

Ground truth:

- Cargo build scripts are compiled and run before package builds.
- Build scripts can build C code, find system libraries, generate code, pass
  linker flags, set environment values, and produce metadata.
- That power is useful, but it is code execution.

Hum rule:

Milestone 0 must never execute Hum source, generated code, package scripts,
plugins, dependency hooks, or foreign code. It may parse, check, and emit graph
data only.

Future gate:

Nectar build scripts require a capability sandbox, declared inputs/outputs,
offline default, generated artifact hashes, and explicit BDFR approval before
implementation.

### R002: Package Manager Becomes A Remote Code Loader

Severity: critical.

Bad outcome:

Nectar downloads and runs something before Hum has package trust, provenance,
signing, lockfiles, and sandboxing.

Ground truth:

- OWASP 2025 lists software supply chain failures and software/data integrity
  failures as top application security risks.
- SLSA exists because build provenance and artifact traceability need standard
  machine-checkable evidence.
- OpenSSF Scorecard explicitly checks for risky practices around dependencies,
  build systems, signed releases, branch protection, dangerous workflows,
  fuzzing, and static analysis.

Hum rule:

Nectar starts offline. Its first job is local manifest parsing, profile metadata,
evidence packets, lockfile design, and reproducibility records.

Future gate:

Remote registries require package signing, provenance verification, dependency
review, build-script capability policy, vulnerability feeds, and rollback-safe
updates.

### R003: Compiler Or Backend Miscompilation

Severity: critical.

Bad outcome:

The compiler accepts correct Hum but emits code with different behavior.

Ground truth:

- Compiler bugs can propagate into generated binaries.
- Fuzzing and differential testing find compiler bugs that ordinary examples
  miss.
- Deep learning compiler research shows that optimizer bugs can corrupt
  downstream results even when source models look correct.

Hum rule:

Native codegen is not Milestone 0. Hum must first have stable parsing,
diagnostics, graph output, golden tests, and a tiny executable core.

Future gate:

Before native optimization, require IR verifier tests, differential tests,
translation tests, fuzzed parser/IR inputs, and graph-to-runtime traceability.

### R004: Backend Undefined Behavior

Severity: critical.

Bad outcome:

Hum promises safety, but lowering to LLVM or another backend creates undefined
behavior, poison values, invalid assumptions, or optimizer-permitted behavior
that violates Hum semantics.

Ground truth:

- LLVM IR has immediate and deferred undefined behavior.
- Poison values can propagate through expression graphs.
- LLVM documentation warns tests should not accidentally trigger unnecessary
  undefined behavior.

Hum rule:

LLVM is a backend target, not Hum's semantics. Hum IR must define behavior first.
Lowering must prove or test that backend assumptions preserve Hum meaning.

Future gate:

No backend optimization is stable without a semantics note, UB audit, backend
test, and a way to blame source promises when behavior changes.

### R005: FFI Makes Safety Optional

Severity: critical.

Bad outcome:

Hum gets adoption by calling C, C++, Rust, Python, or system libraries, but
foreign boundaries silently bypass ownership, layout, exception, panic, thread,
and lifetime rules.

Ground truth:

Existing ecosystems are necessary for adoption. They are also outside Hum's
safety model unless wrapped.

Hum rule:

Interop is allowed only through explicit wrapper contracts. Foreign code must be
visible in source and graph output.

Future gate:

Every FFI boundary needs ABI, layout, ownership, threading, failure, panic,
trust, tests, and profile policy.

### R006: Self-Hosting Too Early

Severity: high.

Bad outcome:

Hum rewrites its compiler in Hum before the bootstrap compiler is trustworthy,
making bugs harder to diagnose and safety claims weaker.

Ground truth:

- Trusting-trust attacks show that source review alone cannot prove a compiler
  binary honestly corresponds to source.
- Diverse double-compiling is a known countermeasure for compiler binary/source
  correspondence under specific assumptions.
- Reproducible builds help establish that artifacts can be recreated from the
  same source and environment.

Hum rule:

Self-hosting stays late. Hum may implement non-critical compiler tools in Hum
first, but not parser/typechecker/borrow checker/optimizer/codegen until the
Rust bootstrap has strong tests and graph stability.

Future gate:

Self-hosting requires stage0/stage1/stage2 plans, reproducible builds, golden
graphs, differential tests, and eventually diverse-build or DDC-style thinking.

### R007: Fuzzer And Test Theater

Severity: high.

Bad outcome:

Hum says "we fuzz" or "we test" but targets are too broad, nondeterministic,
slow, stale, or not tied to real risk.

Ground truth:

- libFuzzer expects fuzz targets to tolerate empty, huge, malformed input; avoid
  `exit`; be deterministic; be fast; avoid excessive memory use; and be narrow.
- Continuous fuzzing is useful, but harnesses can degrade or miss coverage if
  not maintained.

Hum rule:

Parser fuzzing must use narrow local targets and deterministic fixtures. Generated
test obligations must link back to source promises.

Future gate:

Fuzz targets need coverage tracking, corpus policy, crash minimization, stale
harness checks, and CI only after local proof.

### R008: False Security Claims

Severity: high.

Bad outcome:

Hum claims "more secure than Rust" or "zero vulnerabilities" before evidence
exists.

Ground truth:

No serious language can promise absolute zero vulnerabilities. Security claims
must be scoped by threat model, profile, implementation maturity, tests, proofs,
and known limitations.

Hum rule:

Say:

```text
no known vulnerabilities under this scoped threat model
```

Do not say:

```text
zero vulnerabilities
```

Future gate:

Every security claim needs a threat model, evidence artifact, profile, and known
limitations.

### R009: Scope Creep

Severity: high.

Bad outcome:

Hum tries to become Rust, Python, Bash, Terraform, Kubernetes, PyTorch, Julia,
C++, and a proof assistant at the same time.

Ground truth:

Language projects fail when the surface grows faster than implementation,
diagnostics, docs, tooling, tests, and pedagogy.

Hum rule:

Everything goes through the BDFR scope order:

1. local safety
2. parser
3. diagnostics
4. semantic graph
5. intent checks
6. generated local tests
7. tiny executable core

Future gate:

No feature enters stable without the governance admission packet.

### R010: Syntax Before Semantics

Severity: high.

Bad outcome:

Hum accumulates beautiful syntax whose meaning is vague, hard to check, or hard
to lower.

Ground truth:

Readable syntax is not a language. A language needs semantics, diagnostics,
tooling, tests, and compatibility.

Hum rule:

Every new surface form must lower into `FORMAL_CORE.md` or remain a sketch.

Future gate:

No executable syntax without core semantics, graph representation, tests, and
diagnostics.

### R011: Portability Surprise

Severity: high.

Bad outcome:

Hum works on one machine but silently behaves differently on Windows, Linux,
macOS, Wasm, embedded, or future targets.

Ground truth:

Portability issues hide in paths, endian, layout, pointer width, clocks,
randomness, atomics, filesystem behavior, process model, dynamic linking, and
floating-point behavior.

Hum rule:

Target assumptions must be explicit. Tier 0 is this Windows dev host. Tier 1+
comes after local proof.

Future gate:

Portability requires target triples, profile availability, path semantics,
layout checks, and reproducible toolchain metadata.

### R012: Compatibility Regret

Severity: medium.

Bad outcome:

Hum releases features early, then cannot remove them without breaking users.

Ground truth:

Language compatibility is a debt instrument. Stable mistakes last for decades.

Hum rule:

Use stability levels: sketch, experimental, candidate, stable, legacy.

Future gate:

Stable features require migration story, edition story, diagnostics, formatter,
semantic graph, docs, and rejected alternatives.

### R013: Tooling Trust Gap

Severity: medium.

Bad outcome:

Editors, LSP, formatters, debuggers, generators, or agents modify code in ways
the compiler cannot explain or verify.

Ground truth:

Tooling becomes part of the language experience. If tools act on prose instead
of graph facts, agents and IDEs will guess.

Hum rule:

`hum graph` is the source of machine-readable truth. Agents should consume graph
facts and diagnostics, not scrape terminal vibes.

Future gate:

Tool actions need stable diagnostic codes, source spans, fix intent, and graph
node identity.

### R014: Benchmark Theater

Severity: medium.

Bad outcome:

Hum claims performance wins from unreproducible benchmarks, warmed caches,
cherry-picked datasets, or machine-specific behavior.

Ground truth:

Performance results need reproducible inputs, target metadata, profiles,
hardware facts, variance, and failure cases.

Hum rule:

Benchmarks are evidence packets, not marketing claims.

Future gate:

Nectar benchmark packets need machine/profile metadata, dataset hashes,
compiler flags, iteration policy, variance, and baseline comparisons.

### R015: Unicode, Text, And Path Confusion

Severity: medium.

Bad outcome:

Hum treats text, bytes, paths, identifiers, shell arguments, URLs, and user
visible characters as the same thing.

Ground truth:

Text and path bugs become portability, security, and usability bugs.

Hum rule:

`Text`, `Bytes`, `Path`, `Url`, `Command`, `Identifier`, and `Secret` must stay
separate types.

Future gate:

Text/path APIs need Unicode versioning, platform path semantics, encoding
policy, and injection-safe builders.

### R016: Agent Cold-Start On Novel Syntax

Severity: medium.

Bad outcome:

Hum markets itself as agent-native, but agents write Hum worse than they
write Rust or Python because no model has Hum in its training data. Early
agent-written Hum is clumsy, adoption demos underwhelm, and the
agent-native claim reads as irony.

Ground truth:

LLM agents perform best on syntax they have seen at scale. A new language
starts at zero corpus. Familiar surface shapes help, but the durable fix
is tool-side: the compiler must teach the language through machine-
readable syntax surfaces, examples on demand, blame diagnostics, and
machine-applicable fixes, so an agent can converge from feedback instead
of priors.

Hum rule:

Agent ergonomics are measured, not assumed. The agent round-trip metric
(spec plus compiler output only, count the loops to green) is the
standing benchmark, and its results are friction records that indict
`diagnostics`.

Future gate:

Before any public agent-native claim, publish round-trip results for at
least two frontier models on unseen tasks, and ship the machine-applicable
fix surface (WORKORDER backlog) so diagnostics repair as well as explain.

## Project Safety Checklist

Before any new feature:

```text
does it execute code?
does it access network?
does it write outside fixtures?
does it read secrets?
does it make authority visible to the person building it?
does it make authority visible to the person running it?
does it trust a package?
does it call foreign code?
does it lower to backend UB?
does it depend on platform behavior?
does it make a security claim?
does it add stable syntax?
```

If yes, the feature needs a gate document before implementation.

## BDFR Ground Truth

Hum is allowed to be ambitious.

Hum is not allowed to be reckless.

The project wins by being:

- local-first
- offline-first
- maker-safe
- user-safe
- evidence-first
- interop-honest
- scope-controlled
- graph-native
- fuzzable
- reproducible
- portable
- humble about security claims

This risk register should be updated whenever Hum discovers a new class of
mistake.

## Sources

- Cargo build scripts: https://doc.rust-lang.org/cargo/reference/build-scripts.html
- LLVM IR Undefined Behavior Manual: https://llvm.org/docs/UndefinedBehavior.html
- LLVM libFuzzer: https://llvm.org/docs/LibFuzzer.html
- SLSA v1.2 specification: https://slsa.dev/spec/v1.2/
- OpenSSF Scorecard: https://scorecard.dev/
- OWASP Top 10:2025: https://owasp.org/Top10/2025/
- Reproducible Builds documentation: https://reproducible-builds.org/docs/
- Countering Trusting Trust through Diverse Double-Compiling: https://arxiv.org/abs/1004.5548
- A Systematic Impact Study for Fuzzer-Found Compiler Bugs: https://arxiv.org/abs/1902.09334
- NNSmith, compiler fuzzing for deep-learning compilers: https://arxiv.org/abs/2207.13066
- An Empirical Study of Fuzz Harness Degradation: https://arxiv.org/abs/2505.06177

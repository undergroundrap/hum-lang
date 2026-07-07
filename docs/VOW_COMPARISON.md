# Vow Comparison Notes

Date: 2026-07-06

## Sources Read

- Vow website: https://vow-lang.com/
- Vow repository: https://github.com/vow-lang/vow
- Vow design document: https://github.com/vow-lang/vow/blob/main/docs/vow_design.md
- Vow contract reference: https://github.com/vow-lang/vow/blob/main/docs/spec/contracts.md
- Vow contract methodology: https://github.com/vow-lang/vow/blob/main/docs/spec/contracts-methodology.md
- Vow arena memory design: https://github.com/vow-lang/vow/blob/main/docs/design/arena_memory.md
- Vow CI workflow: https://github.com/vow-lang/vow/blob/main/.github/workflows/ci.yml
- Vow agent skill: https://github.com/vow-lang/vow/blob/main/skills/vow/SKILL.md

## Vow In One Sentence

Vow is an agent-first verified programming language: functions carry contracts,
the compiler verifies them with ESBMC, and the toolchain emits structured output
for agent repair loops.

That is close enough to Hum's territory to be useful, but different enough that
Hum should not copy it blindly.

## Ideas Hum Should Take Seriously

### Contracts Are Compiler Inputs

Vow treats preconditions, postconditions, and loop invariants as verification
inputs, not documentation.

Hum should do the same, but with richer labels:

- `needs:` should become preconditions.
- `ensures:` should become postconditions.
- `keeps:` should become invariants.
- `protects:` should become security obligations.
- `watch for:` should become generated tests, fuzz targets, and review prompts.

### Blame Semantics

Vow's most practical idea is explicit blame:

- If a precondition fails, the caller is wrong.
- If a postcondition fails, the callee is wrong.
- If an invariant fails, the loop or mutation body is wrong.

Hum should generalize this:

```text
needs:      caller blame
ensures:    callee blame
keeps:      body or mutation blame
protects:   security-boundary blame
trusts:     trust-boundary blame
changes:    capability-boundary blame
allocates:  allocation-policy blame
```

This gives agents and humans a shared debugging map.

### Structured Diagnostics

Vow emits JSON diagnostics, counterexamples, and build results. That is exactly
the right direction.

Hum should always have two diagnostic faces:

- human display for reading
- stable machine JSON for tools and agents

The JSON should include task name, block name, source span, expected fact,
observed fact, blame target, suggested repair area, and generated tests.

### Counterexample-Guided Repair

Vow uses a CEGIS-style loop: compile, verify, read counterexample, repair,
repeat.

Hum should adopt the loop, but widen it:

```text
parse -> type check -> effect check -> contract check -> fuzz -> benchmark -> explain
```

Each stage should produce facts an agent can use without scraping terminal text.

### Explicit Effects

Vow treats effects as part of function types. Hum already wants this.

Hum should make effects readable at the task level:

```text
uses:
  clock.now
  random.secure

changes:
  sessions

allocates:
  one session record
```

The compiler can lower those declarations to a smaller internal effect set.

### Linear Resources

Vow has linear types for values that must be consumed exactly once.

Hum should use this for:

- file handles
- sockets
- locks
- raw buffers
- transactions
- capabilities
- unsafe or foreign resources

The surface wording can stay readable, while the checker enforces exactly-once
use.

### Canonical Form

Vow prefers one canonical style because agents handle canonical source better
than flexible style wars.

Hum should also ship a formatter from day one. Human readability does not mean
unlimited style choices.

### Compiler-Shipped Agent Docs

Vow ships an agent skill generated from the compiler/toolchain.

Hum should copy the principle: `humc agent docs` should emit the exact grammar,
diagnostic schema, examples, and repair workflow for the installed compiler
version.

### Arena And Region Memory

Vow's arena-per-scope direction is interesting because it avoids exposing
region syntax everywhere while still giving predictable allocation behavior.

Hum should explore a hybrid:

- Rust-like ownership for aliasing and mutation safety.
- Region or arena inference for allocation placement.
- `allocates:` blocks for human-visible memory policy.
- Optional explicit allocators for hot systems code.

## Where Hum Should Be Better

### Human And Agent First-Class

Vow deliberately designs for the agent more than the human.

Hum should take the opposite bet: the best agent language is also deeply
readable by a human. Humans should be able to scan a task and understand:

- what it uses
- what it changes
- what it promises
- what it protects
- why it exists
- where it can fail
- what edge cases matter

Agents then get the same meaning as structured compiler data.

### Richer Intent Than `requires` And `ensures`

Classic contracts are powerful but narrow.

Hum should keep them, then add the missing systems context:

```text
why:
uses:
changes:
creates:
deletes:
needs:
assumes:
ensures:
keeps:
protects:
trusts:
watch for:
allocates:
calls:
fails when:
optimizes:
tests:
proves:
does:
```

Not every task needs every block. Critical systems code should be able to carry
this depth without becoming a separate design document.

### Generics Without Template Chaos

Vow's design currently avoids user-defined generics and traits to keep
verification tractable.

Hum probably cannot avoid generics forever if it wants Rust/C++ systems power.
The better path is constrained generics:

- generic parameters must have explicit capabilities
- generic contracts must be visible
- monomorphization should be predictable
- dynamic dispatch should be explicit
- no C++-style template metaprogramming as the default escape hatch

This is an open design area, but Hum should not give up reusable systems code
too early.

### Data Structure Intent

Hum should make data structure choice a compiler-visible topic.

Example:

```text
store sessions: map SessionId -> Session {
  expects:
    up to 20 million active sessions

  optimizes:
    lookup speed
    memory density
    collision resistance

  protects:
    hash flooding must not cause unbounded slowdown
}
```

The compiler can choose, verify, or reject an implementation based on declared
intent and benchmarks.

### Security Context As Syntax

Vow has verification contracts. Hum should add security and trust language
directly:

```text
protects:
  token cannot be guessed
  expired session cannot authenticate

trusts:
  random.secure is cryptographically strong
  operating system isolates process memory
```

That gives reviewers, agents, tests, and static tools a place to attach
security reasoning.

### Verification Tiers

Hum should avoid making one verifier the whole story.

Possible tiers:

- type and ownership checks
- effect and capability checks
- contract checking
- bounded model checking
- SMT/property proving where practical
- runtime debug checks
- fuzz and property tests
- sanitizer builds
- benchmark assertions

The language should let a project choose proof depth by risk.

## 2026-07-07 Repo Audit Updates

### CI And Supply Chain

Vow is ahead of Hum in CI sophistication: it caches Rust builds, pins its ESBMC verifier artifact by version and SHA-256, runs verifier-evaluation suites, and checks self-hosted bootstrap paths. Hum should copy the discipline, not the whole cost profile yet.

Immediate Hum adoption:

- use official GitHub actions only for hosted CI plumbing
- cache Cargo registry data, Cargo git checkouts, and `target` per runner OS
- keep hosted jobs short and cancellable while the repo is private
- do not add external verifier downloads until Hum has checksum, source, and evidence policy for them

### Cache Is Not Authority

Vow has an important verifier-cache safety rule: cached successful proofs from disk are not trusted. Its compile cache keys include dependency content and an ABI seed, and its verifier cache keys include verifier configuration. Hum should treat this as a language doctrine.

Hum cache keys should eventually include source content, dependency graph, compiler version, semantic graph schema version, profile, target triple, backend, ABI seed, verifier or solver version, proof bounds, benchmark profile, and relevant environment facts. A cached failure or counterexample can speed repair. A cached success should never be the only reason a release claims correctness.

### Strong Contracts Beat Many Contracts

Vow's contract methodology is a real lesson. It distinguishes strong contracts from hollow contracts: tautologies, vacuous proofs, weak postconditions, and artificial verifier bounds. This matters directly to Hum because `needs:`, `ensures:`, `keeps:`, `protects:`, `trusts:`, `allocates:`, and `optimizes:` can all become hollow if the compiler only checks that text exists.

Hum should make contract quality a first-class diagnostic area:

- `needs:` domains must be satisfiable and not verifier-shaped
- `ensures:` clauses should reject meaningful wrong implementations
- `keeps:` invariants should be inductive, not decorative
- `protects:` and `trusts:` claims should identify actual threat or trust boundaries
- `optimizes:` claims should be tied to benchmark evidence and allowed variance

### Self-Hosting Proof

Vow is already much farther along on self-hosting. Its CI runs bootstrap stages and fixed-point checks. Hum should learn from that but not rush into it. For Hum, self-hosting is not a marketing badge; it is proof that the language can express its own compiler more clearly, safely, and checkably than the Rust bootstrap.

Hum should require differential tests first, then staged compiler rewrites, then fixed-point checks across stage outputs. A cached build can shorten the loop, but no cache may mask a mismatch between stage compilers.

### Compiler-Generated Agent Docs

Vow's agent skill is a good direction: the toolchain should tell agents exactly how to write, compile, verify, and repair code for the installed version. Hum should eventually generate `hum agent docs` from the compiler's own grammar, diagnostics, graph schema, capabilities, examples, and repair recipes.

The difference is audience. Vow asks what language agents should write. Hum should ask what source form lets humans, compilers, IDEs, security reviewers, profilers, package tools, and agents share the same intent without a translation layer.

### Where Hum Should Diverge

Vow excludes user-defined generics, traits, closures, and several abstraction tools to protect bounded verification. That is coherent for Vow, but Hum should not inherit it as a blanket rule. Hum wants broad systems adoption, so the better target is constrained power:

- proof-friendly defaults
- explicit capability and effect surfaces
- profile-gated abstraction features
- strong diagnostics for hidden dispatch, hidden allocation, and hard-to-prove code
- generated monomorphic specializations where they are clearly better

The principle should be: Hum may allow more power than Vow, but every added power must produce more evidence, clearer blame, and a better paved road.

## Direct Design Updates For Hum

1. Add blame semantics to the spec.
2. Treat `watch for:` as a test and fuzz generation input.
3. Treat `protects:` as a security obligation, not prose.
4. Treat `trusts:` as an explicit verification boundary.
5. Keep the no-headers direction.
6. Require machine-readable diagnostics from the first compiler milestone.
7. Plan an agent-doc command generated from the compiler version.
8. Keep generics as an open design problem instead of banning them early.
9. Explore region or arena inference under readable ownership syntax.
10. Make `store` declarations a signature feature of Hum.

## Current Repo Follow-Through

As of 2026-07-07, Hum has started turning this comparison into behavior:

- `H0110` warns on hollow contract-like lines before they become fake evidence.
- `hum.syntax_surface.v0` exposes task obligation `blame` mappings for tools.
- `hum.semantic_graph.v0` emits `blame` on generated task obligations.
- `hum.syntax_surface.v0` exposes evidence obligation mappings for `protects:` and `trusts:`.
- `hum.semantic_graph.v0` emits task-level `evidence_obligations` from `protects:` and `trusts:` with `verification_status: unverified`.
- Hosted CI uses current official GitHub actions and cache discipline.

Still open from this list: evidence obligations need links to proof, test,
review, sanitizer, and profile artifacts; `hum agent docs` needs a generated
command; and arena/region inference remains design work before implementation.

## Hum Positioning

Vow asks: what language should agents write?

Hum asks: what language lets humans, agents, and compilers share the same
systems intent?

That is the wedge.

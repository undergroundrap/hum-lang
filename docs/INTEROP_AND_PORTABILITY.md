# Hum Interop And Portability Strategy

Date: 2026-07-06

## Purpose

Hum should be practical enough to adopt the existing world and strict enough not
to inherit its worst risks silently.

This document defines how Hum should approach C, C++, Rust, Python, Wasm, system
libraries, and target portability.

## Brutal Thesis

Hum will not be adopted if it cannot use existing libraries.

Hum will not be safe if using existing libraries is casual.

The rule is:

```text
interop is allowed
ambient trust is not
```

Foreign code must be visible in source, semantic graph output, docs, tests,
profiles, package metadata, and review packets.

## Adopt Before Rewrite

Hum should not start by rewriting every library.

Correct order:

1. Use trusted existing libraries behind explicit boundaries.
2. Wrap them with Hum contracts.
3. Test, fuzz, benchmark, and profile the boundary.
4. Replace pieces only when Hum can prove better safety, speed, portability, or
   readability.

This gives Hum adoption before total ecosystem ownership.

## Milestone Policy

### Milestone 0

No FFI.

No generated code execution.

No dependency build scripts.

No package downloads.

No network registry access.

Milestone 0 is parse, check, graph, diagnostics, fixtures, and local tests only.

### Milestone 1-2

Interop remains design-only unless needed for a tiny local interpreter/runtime
proof.

Any runtime experiment must stay local and reversible.

### Milestone 4

First real interop work belongs with unsafe and FFI because foreign code is a
safety boundary.

## Interop Boundary Model

A foreign boundary must declare:

```text
foreign c library zlib {
  why:
    use mature compression implementation before Hum stdlib has one

  trusts:
    zlib version is pinned
    wrapper validates buffer lengths

  protects:
    compressed input cannot write past output buffer
    decompression cannot allocate without declared budget

  layout:
    c abi
    bytes are contiguous

  ownership:
    caller owns input
    caller owns output buffer
    foreign code borrows both during call

  threading:
    reentrant
    no shared global mutable state used

  fails when:
    input is malformed
    output buffer is too small

  tests:
    malformed input fuzz target
    round trip property test
}
```

This is more than documentation. It is future graph data.

## C Interop

C ABI is the first foreign boundary because it is the lowest common denominator.

Rules:

- use explicit ABI names
- no implicit ownership transfer
- no raw pointer crossing without a layout and lifetime contract
- no callback crossing without threading and reentrancy rules
- no nullable pointer without `Option`-style representation
- no C string without encoding and lifetime rules
- no variadic calls until much later, if ever
- no generated bindings trusted without review

Hum should prefer small handwritten C wrappers at first.

## C++ Interop

Do not pretend the C++ ABI is simple.

Rules:

- prefer C ABI facade wrappers around C++ libraries
- do not expose C++ templates, exceptions, RTTI, or ownership models directly
- convert C++ exceptions to typed Hum failure at the wrapper boundary
- pin compiler, standard library, and build flags when ABI matters
- make allocator ownership explicit

C++ interop is important for adoption, but it should never look ordinary.

## Rust Interop

Rust is the best bootstrap language for Hum, but Rust ABI is not a stable general
interop contract.

Rules:

- call Rust libraries through explicit wrapper crates
- expose stable C ABI or Wasm/process boundaries when crossing into Hum
- keep Rust panic behavior out of Hum unless translated at the boundary
- preserve ownership and borrowing contracts in wrapper APIs
- treat unsafe Rust inside dependencies as part of the trust packet

Hum should love Rust without pretending Rust crates are plug-and-play native Hum
modules.

## Python Interop

Python interop matters for AI/ML, data science, scripting adoption, and existing
automation.

Rules:

- start with data exchange and extension boundaries, not embedding arbitrary
  Python execution into trusted Hum tasks
- treat Python objects as foreign, reference-counted, runtime-owned values
- make GIL/threading behavior explicit
- mark interpreter state as a capability
- keep untrusted Python code out of safety-critical profiles
- prefer stable data formats and zero-copy protocols where possible

Python is a bridge, not the foundation of Hum safety.

## Wasm Interop

Wasm is the best early sandbox story.

Uses:

- plugin sandboxing
- deterministic local execution experiments
- portable demos
- untrusted extension boundaries
- browser or edge targets later

Rules:

- explicit imports and exports
- capability-limited host calls
- deterministic profile options
- fuel or resource budget hooks
- no hidden filesystem or network authority

Wasm should become Hum's first serious untrusted-code boundary before native
plugins.

## Process Boundary Interop

Sometimes the safest interop is a process boundary.

Rules:

- command construction must be typed
- stdin/stdout/stderr must be bounded
- timeout and cancellation required
- environment variables explicit
- working directory explicit
- secrets redacted
- exit status mapped to typed failure

This is useful for tools, compilers, formatters, and legacy command-line
programs, but it must not become shell-string programming.

## Package Build Scripts

Package build scripts are code execution.

Nectar must not support arbitrary dependency build scripts early.

When build scripts eventually exist, they need:

- declared capabilities
- no network by default
- sandbox profile
- reproducible input/output declaration
- generated artifact hashes
- graph output
- review diagnostics

Postinstall-style arbitrary execution should be rejected.

## Portability Tiers

### Tier 0

Current development host:

```text
windows x86_64
```

Must pass local tests before anything else matters.

### Tier 1

Primary hosted targets:

```text
windows x86_64
windows arm64
linux x86_64
linux arm64
macos arm64
macos x86_64
```

Expect full toolchain support eventually.

### Tier 2

Portable sandbox and server targets:

```text
wasm32-wasi
wasm32-unknown
linux musl
```

Expect restricted profile support.

### Tier 3

Specialized systems targets:

```text
no std
no heap
firmware
kernel
hard realtime
certified toolchain
```

Expect smaller language and stdlib subsets.

## Portability Rules

Hum source should not hide target differences.

The compiler and stdlib must make these explicit:

- endian behavior
- integer width
- pointer width
- alignment
- layout
- path semantics
- filesystem capabilities
- clock availability
- randomness source
- atomic support
- SIMD and accelerator features
- floating-point precision and determinism
- OS process model
- networking availability

Portable code should be able to ask:

```text
target supports atomic u64
target path kind is windows
target endian is little
target has filesystem read
```

## Semantic Graph Requirements

The graph should expose:

- foreign libraries
- ABI names
- layout assumptions
- ownership transfer
- trust assumptions
- unsafe blocks used by wrappers
- target tiers
- required target features
- unavailable APIs for a profile
- generated bindings
- wrapper tests
- fuzz targets
- portability diagnostics

Agents should never infer FFI risk from raw source text alone.

## Rejected Shortcuts

Hum should reject:

- automatic internet package downloads in early Nectar
- arbitrary package install scripts
- raw shell command strings as ordinary APIs
- direct C++ ABI exposure as a first-class stable feature
- pretending Rust ABI is stable for general dynamic interop
- Python execution inside safety-critical profiles
- FFI without ownership and layout contracts
- hidden native binaries in packages
- generated bindings without review metadata

## Near-Term Work

Before writing real FFI code:

1. Add `foreign` only as parsed design syntax or fixtures.
2. Reserve semantic graph fields for interop facts.
3. Create local examples for safe wrapper contracts without executing foreign
   code.
4. Add diagnostics for missing `trusts:`, `protects:`, ownership, and layout.
5. Keep all tests local.

## BDFR Decision

Hum should use existing libraries to earn adoption.

Hum should wrap them so users know exactly where the old world enters the new
one.

That is how Hum can be practical without becoming unsafe by association.

# Hum Memory Safety Model

Date: 2026-07-06

## Goal

Hum should be memory safe by default and systems-capable by design.

The honest target is:

```text
Safe Hum should prevent the same broad classes of memory bugs Rust prevents,
while making mutation, effects, trust, and unsafe reasoning easier to see.
```

Hum should not claim to be more memory safe than Rust until the checker earns
that claim. The first real advantage should be auditability: Hum makes more of
the engineer's safety reasoning part of the source.

## Bugs Safe Hum Must Prevent

Safe Hum must prevent:

- use after free
- double free
- dangling references
- iterator invalidation that creates invalid references
- data races
- reading uninitialized memory
- null dereference in ordinary values
- accidental shared mutation
- hidden mutation of global state
- resource leaks that violate declared ownership

Unsafe Hum may expose lower-level power, but unsafe must be small, named, and
reviewable.

## Rust Ideas Hum Should Keep

Hum should keep these Rust design lessons:

### Ownership

Every value has an owner unless it is borrowed or shared through a checked form.

Beginner explanation:

```text
One part of the program is responsible for cleaning this thing up.
```

### Moves

Passing ownership moves responsibility.

```hum
let session = create session(user)
let saved = save session(session)
```

After a move, the old name cannot be used unless the value was explicitly copied
or borrowed.

### Borrowing

Borrowing lets code look at a value without owning it.

```hum
borrow session
```

Beginner explanation:

```text
You can look at it, but you do not own it.
```

### Exclusive Change

Changing a value requires exclusive access.

```hum
change session
```

Beginner explanation:

```text
If you are changing it, nobody else gets to change or read it in a conflicting way.
```

### Immutable By Default

Local names are immutable by default.

```hum
let count: UInt = 0
```

Mutable locals are explicit:

```hum
change attempts: UInt = 0
set attempts = attempts + 1
```

### Typed Failure

Recoverable failure should use typed results, not invisible exceptions.

```hum
task load config(path: Path) -> Result Config, ConfigError {
  fails when:
    file does not exist
    file is invalid
}
```

### Exhaustive Match

When a value has known cases, every case should be handled.

```hum
match result {
  ok value:
    use value
  error reason:
    show reason
}
```

## Hum Ideas Beyond Rust

Hum adds visible contracts around the places where systems bugs hide.

### Visible Capabilities

A task must say what outside resources it reads:

```hum
uses:
  clock.now
  random.secure
  sessions
```

### Visible Mutation

A task must say what outside resources it changes:

```hum
changes:
  sessions
```

Then this is allowed:

```hum
does:
  save session in sessions
```

Without `changes: sessions`, the compiler should reject it.

### Visible Trust

Security-sensitive assumptions must be named:

```hum
trusts:
  random.secure is cryptographically strong
  clock.now does not move backward by more than 5 minutes
```

### Visible Protection

Security promises must be named:

```hum
protects:
  session token cannot be guessed
  expired token cannot work
```

### Visible Unsafe

Unsafe code must carry its review packet in the source:

```hum
unsafe task read device register(address: Address) -> UInt32 {
  why:
    read hardware state that safe Hum cannot model directly

  needs:
    address is aligned for UInt32
    address belongs to mapped device memory

  protects:
    safe code cannot read arbitrary process memory

  trusts:
    operating system mapped this device range correctly

  watch for:
    volatile read must not be optimized away
    address must not come from user input

  does:
    read volatile UInt32 from address
}
```

Unsafe should feel like a sealed lab bench, not a casual escape hatch.

See [UNSAFE_POLICY.md](UNSAFE_POLICY.md) for the full unsafe review-packet and
profile policy.

## Stores Instead Of Global Variables

Hum should avoid casual global variables.

For process-wide or module-wide state, use `store`:

```hum
private store sessions: Map SessionId -> Session {
  why:
    remember active sessions

  protects:
    session ids cannot be guessed
}
```

A `store` is not just a variable. It is named program memory with purpose,
visibility, invariants, and access rules.

## Visibility Defaults

Hum should use few visibility levels:

- local names live only inside their block
- module items are private by default
- `export` means public API
- `package` means available inside the package
- `private` may be written when clarity matters

Avoid C++-style visibility puzzles. Hum should make boundaries obvious.

## Proposed Ownership Words

Hum should use words that read clearly:

- `owned`: this code owns the value
- `borrow`: read without owning
- `change`: exclusive mutable access
- `shared`: shared thread-safe access, only through checked types

Example:

```hum
task rename session(change session: Session, new name: Text) {
  needs:
    new name is not empty

  changes:
    session

  does:
    set session.name = new name
}
```

## What Safe Hum Should Forbid

Safe Hum should forbid:

- hidden mutable globals
- data races
- null ordinary values
- use after move
- mutation through shared borrow
- unbounded unsafe pointer arithmetic
- calling foreign code without a declared `foreign` boundary
- IO or allocation hidden inside a task that claims not to do it

## What Hum Should Delay

Do not design these too early:

- full lifetime syntax
- advanced generic associated types
- higher-kinded types
- macro system
- async runtime model
- custom allocators everywhere
- arbitrary implicit conversions

Hum should first make the common safe path beautiful and strict.

## Brutal Standard

Hum's safety story is not good enough if experts understand it but beginners
cannot explain it.

A beginner should be able to say:

```text
This task can use these things.
It can change these things.
It needs these facts first.
It promises these facts after.
It owns this value.
It only borrows that value.
It enters unsafe code here, and this is why.
```

That is the memory-safety story Hum should chase.
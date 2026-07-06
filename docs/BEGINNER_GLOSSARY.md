# Hum Beginner Glossary

Date: 2026-07-06

This glossary explains Hum words for people who do not already think like
programmers.

Hum should not make beginners memorize mystery words before they can build
something. Each term should answer a normal human question.

## Tiny Mental Model

A Hum program is made of promises.

A `task` says what it is trying to do, what it uses, what it changes, what must
be true before it runs, what it promises after it runs, and the exact steps it
takes.

## Program

A program is the whole thing you are building.

Example: a task list, a game, a server, a compiler.

## Module

A module is a named section of a program.

Beginner phrase:

```text
This file belongs to this part of the program.
```

Hum:

```hum
module examples.task_list
```

## Type

A `type` describes what a thing is made of.

Beginner phrase:

```text
A task has a title and a done flag.
```

Hum:

```hum
type Task {
  title: Text
  done: Bool
}
```

## Field

A field is one named piece inside a type.

In this type, `title` and `done` are fields:

```hum
type Task {
  title: Text
  done: Bool
}
```

## Value

A value is an actual piece of data.

Examples:

- `"buy milk"`
- `false`
- `42`
- a `Task`

## Let

`let` gives a name to a value that will not change.

Beginner phrase:

```text
Remember this value under this name, and do not change it.
```

Hum:

```hum
let title: Text = "buy milk"
```

## Change

`change` gives a name to a value that may change.

Beginner phrase:

```text
This value is allowed to change in this task.
```

Hum:

```hum
change attempts: UInt = 0
```

## Set

`set` changes a value that was already declared with `change`, or a value the
task is allowed to mutate.

Hum:

```hum
set attempts = attempts + 1
```

## Task

A `task` is an action the program can perform.

Beginner phrase:

```text
This is something the program knows how to do.
```

Hum:

```hum
task add task(title: Text) -> Result Task, TaskError {
  why:
    let the user remember something to do

  does:
    create task
}
```

## Parameter

A parameter is information a task needs from the caller.

In this task, `title` is a parameter:

```hum
task add task(title: Text) {
  does:
    use title
}
```

## Result

A result is what a task gives back.

Hum:

```hum
task add task(title: Text) -> Result Task, TaskError
```

Beginner phrase:

```text
This task either gives back a Task or fails with a TaskError.
```

## Store

A `store` is program memory with a purpose.

Use it instead of casual global variables.

Beginner phrase:

```text
This is where the program remembers something important.
```

Hum:

```hum
store tasks: List Task {
  why:
    remember the user's tasks
}
```

## Why

`why:` says why code exists.

Hum:

```hum
why:
  let the user remember something to do
```

If the `why:` is not clear, the code probably is not ready.

## Uses

`uses:` lists outside things a task reads or depends on.

Hum:

```hum
uses:
  clock.now
  random.secure
  sessions
```

Beginner phrase:

```text
This task looks at or depends on these things.
```

## Changes

`changes:` lists outside things a task is allowed to modify.

Hum:

```hum
changes:
  sessions
```

Beginner phrase:

```text
This task is allowed to change sessions.
```

If a task changes something but does not list it here, the compiler should stop
it.

## Needs

`needs:` says what must be true before a task runs.

Hum:

```hum
needs:
  title is not empty
```

Beginner phrase:

```text
Do not run this task unless this is true first.
```

## Ensures

`ensures:` says what must be true after a task succeeds.

Hum:

```hum
ensures:
  new task is saved
  new task is not done
```

Beginner phrase:

```text
If this task works, these things must be true afterward.
```

## Fails When

`fails when:` lists expected ways the task can fail.

Hum:

```hum
fails when:
  title is empty
```

Beginner phrase:

```text
This is not a crash. This is a known failure case.
```

## Watch For

`watch for:` lists edge cases that deserve attention.

Hum:

```hum
watch for:
  title may be only spaces
```

Beginner phrase:

```text
This is the kind of future bug we are worried about.
```

## Protects

`protects:` says what security or safety promise this task defends.

Hum:

```hum
protects:
  session token cannot be guessed
```

Beginner phrase:

```text
This task helps keep this thing safe.
```

## Trusts

`trusts:` says what assumption the task depends on.

Hum:

```hum
trusts:
  random.secure is cryptographically strong
```

Beginner phrase:

```text
This must be true, but this task does not prove it by itself.
```

## Cost

`cost:` says how much work or memory the task should use.

Hum:

```hum
cost:
  time: O(tasks)
  space: O(1)
  allocates: nothing
  check: warn
```

Beginner phrase:

```text
This tells readers and tools how expensive the task is expected to be.
```

## Allocates

`allocates:` says what new memory the task creates.

Hum:

```hum
allocates:
  one task
```

Beginner phrase:

```text
This task creates this much new memory.
```

## Avoids

`avoids:` says what the code is intentionally not doing.

Hum:

```hum
avoids:
  network call inside loop
```

Beginner phrase:

```text
Do not change this code in a way that does this bad thing.
```

## Tradeoffs

`tradeoffs:` explains a decision that has costs and benefits.

Hum:

```hum
tradeoffs:
  linear display is expected because every visible task must be shown
```

Beginner phrase:

```text
We chose this path for this reason, even though another path exists.
```

## Does

`does:` contains the exact steps of the task.

Hum:

```hum
does:
  if title is empty {
    fail TaskError.empty_title
  }

  save task in tasks
  return task
```

Beginner phrase:

```text
This is what actually happens.
```

## If

`if` means choose based on a condition.

Hum:

```hum
if task.done {
  show task as completed
} else {
  show task as active
}
```

## For Each

`for each` repeats something for every item in a collection.

Hum:

```hum
for each task in tasks {
  show task
}
```

## While

`while` repeats while something is true.

Hum should prefer visible bounds when possible:

```hum
while attempts < 16 {
  set attempts = attempts + 1
}
```

## Match

`match` handles each possible case of a value.

Hum:

```hum
match result {
  ok task:
    show task
  error reason:
    show reason
}
```

Beginner phrase:

```text
For each possible shape this value can have, say what to do.
```

## Test

A `test` checks that code behaves as promised.

Hum:

```hum
test add task rejects empty title {
  does:
    expect add task("") fails with TaskError.empty_title
}
```

Beginner phrase:

```text
This proves one promise still works.
```

## Regression Test

A regression test protects against a bug that happened before.

Hum:

```hum
test empty title with spaces is rejected regression {
  regression:
    found when title "   " was accepted as nonempty
}
```

Beginner phrase:

```text
This bug happened once. Do not let it come back.
```

## Private

`private` means only this part of the program can use it.

Hum:

```hum
private store sessions: Map SessionId -> Session
```

## Export

`export` means other parts of the program are allowed to use it.

Hum:

```hum
export task create session(user: User) -> SessionToken
```

## Package

`package` means usable inside the package, but not public to everyone.

Hum:

```hum
package task rotate token(session: Session)
```

## Unsafe

`unsafe` means the compiler cannot fully prove the operation is safe by itself.

Unsafe code must explain its reason and promises.

Beginner phrase:

```text
This is powerful code. It needs extra proof and review.
```

## Rule Of Thumb

If a beginner asks, "where does this happen?", Hum should point to one of these
blocks:

- `why:` for purpose
- `uses:` for what it reads
- `changes:` for what it mutates
- `needs:` for before
- `ensures:` for after
- `fails when:` for known failure
- `watch for:` for edge cases
- `protects:` for security
- `trusts:` for assumptions
- `cost:` for performance
- `does:` for steps
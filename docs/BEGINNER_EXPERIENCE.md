# Hum Beginner Experience

Date: 2026-07-06

Companion doc: [BEGINNER_GLOSSARY.md](BEGINNER_GLOSSARY.md).

## Thesis

Hum should let a person who knows English but not programming describe useful
software, then guide them into precise code.

The goal is not to let vague English execute. The goal is to make the first
programming concepts feel like ordinary reasoning:

- what exists
- what should happen
- what the task uses
- what the task changes
- what must be true first
- what must be true after
- what can go wrong
- what edge cases matter

## Brutal Truth

Beginners do not struggle because programming lacks English words.

They struggle because most languages introduce too many hidden concepts at once:

- state
- mutation
- types
- scope
- control flow
- null
- errors
- files
- dependencies
- naming
- side effects
- testing

Hum should reveal those concepts in the order a human naturally asks questions.

## The Guided Authoring Flow

Hum tools should be able to ask:

1. What are you trying to make?
2. What things exist in the program?
3. What should the program remember?
4. What actions can happen?
5. What does each action use?
6. What does each action change?
7. What must be true before each action?
8. What must be true after each action?
9. What can go wrong?
10. What edge cases worry you?

Those answers become Hum blocks.

## First Concepts

### Thing

Beginner word: thing.

Hum word: `type`.

```text
type Task {
  title: Text
  done: Bool
}
```

### Memory

Beginner word: remember.

Hum word: `store`.

```text
store tasks: list Task {
  why:
    remember the user's tasks
}
```

### Action

Beginner word: action.

Hum word: `task`.

```text
task add task(title: Text) -> Result Task, TaskError {
  why:
    let the user remember something to do

  changes:
    tasks

  needs:
    title is not empty

  ensures:
    new task is saved
    new task is not done

  does:
    create task with title
    save task in tasks
    return task
}
```

### Choice

Beginner word: if.

Hum word: `if`.

```text
if task is done {
  show completed style
} else {
  show active style
}
```

### Repetition

Beginner word: for each.

Hum word: `for each`.

```text
for each task in tasks {
  show task
}
```

### Failure

Beginner word: what can go wrong.

Hum word: `fails when` and `fail`.

```text
fails when:
  title is empty

does:
  if title is empty {
    fail TaskError.empty_title
  }
```

### Test

Beginner word: check.

Hum word: `test`.

```text
test add task rejects empty title {
  does:
    expect add task("") fails with TaskError.empty_title
}
```
### Cost

Beginner words: how much work.

Hum word: `cost`.

```text
cost:
  time: O(tasks)
  space: O(1)
  allocates: nothing
```

### Avoid

Beginner words: do not do it that way.

Hum word: `avoids`.

```text
avoids:
  saving empty titles
  network call inside loop
```
## Beginner Standard Library Surface

The first human-facing standard library should include:

- show text
- ask for text
- ask yes/no
- read a file
- save a file
- list items
- filter items
- sort items
- find first item
- count items
- current time
- random safe token
- simple HTTP request
- simple JSON read/write

These are not necessarily primitive compiler features. They are the first
teaching surface.

## What The Compiler Should Teach

When a beginner writes:

```text
does:
  save task in tasks
```

but forgot:

```text
changes:
  tasks
```

the compiler should not say only:

```text
error: undeclared mutation
```

It should say:

```text
error: this task changes `tasks`, but `tasks` is not listed under `changes:`

why:
  Hum makes mutation visible so readers know what this task can modify.

fix:
  add `tasks` under `changes:` or stop saving to `tasks`.
```

Beginner diagnostics should teach the mental model.

## Agent-Assisted Beginner Mode

Hum should support an agent workflow:

```text
hum new
hum ask
hum draft
hum explain
hum check
hum tests
```

Flow:

1. Human describes the program in ordinary language.
2. Agent drafts Hum source.
3. Compiler rejects vague or unsafe parts.
4. Agent repairs using machine diagnostics.
5. Human reads the final source as promises.

The compiler remains the authority.

## What We Should Not Do

- Do not execute arbitrary English.
- Do not hide types forever.
- Do not hide errors.
- Do not hide mutation.
- Do not call generated code correct because an agent wrote it.
- Do not teach beginners habits that fail in systems code.

## First Beginner Demo

Build a tiny task list before a web server.

It teaches:

- `type`
- `store`
- `task`
- `why`
- `changes`
- `needs`
- `ensures`
- `fails when`
- `for each`
- `if`

Then build the session-store demo to show Hum can scale from beginner clarity to
security-sensitive systems thinking.
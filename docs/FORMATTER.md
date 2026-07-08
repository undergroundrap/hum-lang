# Hum Formatter

Date: 2026-07-06

Tool name: `humfmt`

## Thesis

Humfmt should make Hum code look like Hum code everywhere.

A language built around readability cannot leave formatting as a community
argument. The formatter should be first-party, stable, boring, and strict.

## Commands

```powershell
humfmt file.hum
humfmt examples
humfmt --check examples
nectar fmt
nectar fmt --check
```

## Principles

1. One canonical style by default.
2. Minimal configuration early.
3. Preserve meaning exactly.
4. Preserve comments and intent blocks.
5. Make task headers, capability blocks, and `does:` bodies easy to scan.
6. Keep beginner examples polished.
7. Make diffs small and predictable.

Humfmt should not become a second language.

## Canonical Block Shape

Humfmt should keep this shape:

```hum
task add_item(title: Text) -> Result Task, TaskError {
  why:
    let the user remember something to do

  changes:
    tasks

  needs:
    title is not empty

  ensures:
    new item is saved
    new item is not done

  does:
    if title is empty {
      fail TaskError.empty_title
    }

    save item in tasks
    return item
}
```

## Section Ordering

Humfmt should prefer this task section order:

1. `why:`
2. `targets:`
3. `uses:`
4. `changes:`
5. `creates:`
6. `deletes:`
7. `needs:`
8. `assumes:`
9. `ensures:`
10. `keeps:`
11. `protects:`
12. `trusts:`
13. `fails when:`
14. `watch for:`
15. `cost:`
16. `allocates:`
17. `avoids:`
18. `tradeoffs:`
19. `optimizes:`
20. `tests:`
21. `benchmarks:`
22. `proves:`
23. `does:`

Early humfmt may warn about order before auto-reordering. Reordering prose-heavy
blocks can surprise users, so automatic movement should wait until the parser can
preserve all comments and trivia safely.

## Indentation

Use two spaces.

Reason:

- intent blocks are nested often
- examples stay compact
- code remains readable in docs

No tabs in formatted Hum source.

## Line Wrapping

Humfmt should wrap long machine syntax before it wraps human prose.

Human prose inside blocks should be preserved unless it clearly exceeds a project
line limit and can be wrapped without changing meaning.

## Formatter And Chirp Boundary

Humfmt fixes layout.

Chirp critiques meaning and style.

Examples:

- humfmt aligns and spaces sections
- chirp warns that `why:` is vague
- humfmt formats `cost:`
- chirp warns that `cost:` lacks `check:`

Do not make the formatter a linter.

## Formatter And Semantic Graph

Humfmt should eventually use the lossless syntax tree, not ad hoc text rewriting.

Required before serious auto-formatting:

- tokens with spans
- comments/trivia preserved
- block sections preserved
- stable parse errors
- golden formatting tests

## Brutal Rule

Humfmt should be so standard that people forget it exists.

If teams argue about Hum formatting, the formatter failed.
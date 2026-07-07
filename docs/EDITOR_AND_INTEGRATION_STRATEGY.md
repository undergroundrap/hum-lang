# Editor And Integration Strategy

Date: 2026-07-06

## Purpose

Hum should be easy for beginners to open and serious enough for enterprise teams to integrate. The language should not depend on one editor, one vendor, one OS, or one plugin ecosystem.

The strategy is:

```text
first-party language facts -> first-party CLI/LSP/formatter/graph -> thin editor adapters
```

## Adoption Surface

Current public developer-survey data makes the editor surface broad: VS Code and Visual Studio are top-tier, IntelliJ IDEA and Cursor are major, PyCharm and Jupyter matter for Python/data users, Eclipse remains relevant in enterprise Java, and Vim/Neovim remain important for expert workflows.

Hum should therefore avoid editor-specific truth. Editor support should consume stable Hum tools.

## First-Party Tooling Is The Product

Hum should own these tools:

- `hum check`: diagnostics and local validation, including `hum.check.v0` JSON
- `hum graph`: semantic graph JSON for tools, agents, and docs
- `hum syntax`: syntax surface JSON for highlighting, grammar generation, and adapters
- `humfmt`: canonical formatting
- `hum lsp`: language intelligence through the Language Server Protocol
- `hum debug`: future Debug Adapter Protocol bridge
- `nectar`: package, profile, evidence, and workspace metadata

Plugins should be adapters, not alternate implementations of Hum semantics.

The current adapter contract is tracked in
[LSP_CAPABILITY_MATRIX.md](LSP_CAPABILITY_MATRIX.md). That matrix is the
ground-truth bridge between first-party CLI/schema output and editor features.

## Editor Priority Tiers

### Tier 0: Editor-Neutral Foundation

- `.editorconfig`
- `.gitattributes`
- repo-relative commands
- UTF-8 without BOM
- line-ending normalization
- stable CLI output
- stable semantic graph JSON
- stable syntax surface JSON from `hum syntax`
- section hover metadata from the syntax surface
- semantic-token legend from the syntax surface
- generated TextMate grammar at [../editors/textmate/hum.tmLanguage.json](../editors/textmate/hum.tmLanguage.json)

This tier makes Hum usable in plain terminals, basic editors, and enterprise source scanners.

### Tier 1: LSP Clients

Support any editor that can run an LSP client:

- VS Code and Cursor
- Visual Studio where LSP integration is available or practical
- Eclipse through LSP4E-style integration
- IntelliJ/PyCharm through an LSP bridge or native plugin shell
- Neovim, Vim, Helix, Zed, Sublime Text, and similar editors

The LSP server must be the same `hum lsp` binary everywhere.

### Tier 2: Native Shell Adapters

Thin adapters may be useful for high-adoption or enterprise-heavy editors:

- VS Code/Cursor extension for easy install, TextMate grammar, commands, and LSP wiring
- Visual Studio VSIX for Windows enterprise teams, solution workflows, and MSBuild-adjacent discovery
- JetBrains plugin shell for IntelliJ/PyCharm/RustRover users
- Eclipse plugin or LSP4E package for Java-heavy enterprise environments
- Jupyter kernel or notebook integration for data workflows when Hum has an execution story

These adapters should not reimplement parsing, type rules, formatting, or diagnostics.

### Tier 3: Rich Hum Experiences

After core tools exist, editor integrations can add:

- intent-block visualization
- semantic graph explorer
- profile/evidence panel
- unsafe review packet viewer
- generated test/proof/benchmark code lenses
- package trust visualization
- notebook cells backed by reproducible Hum kernels

## Visual Studio Strategy

Visual Studio matters for Windows, C++, C#, enterprise, regulated software, and Microsoft-platform teams.

Near-term Hum should not create a heavy Visual Studio extension before `hum lsp` exists. The credible path is:

1. Ensure CLI commands work in Visual Studio terminals and external tools.
2. Publish a language configuration and syntax grammar only when grammar stabilizes.
3. Use `hum lsp` as the source of diagnostics, navigation, completion, and semantic tokens.
4. Create a VSIX only when the LSP and semantic graph are stable enough to justify enterprise install trust.
5. Keep all Visual Studio state out of the main repo unless there is a dedicated extension package.

## Jupyter And Data Strategy

Jupyter matters for data science, finance research, AI/ML, notebooks, demos, and literate exploration.

Hum should not rush notebooks before execution exists. The credible path is:

1. Keep `.hum` examples plain and checkable today.
2. Add notebook export or documentation rendering only after Hum has executable semantics.
3. Add a Jupyter kernel only when cells can run safely and reproducibly.
4. Make notebook execution declare file, network, secret, and package authority.
5. Keep `.ipynb_checkpoints/` and exploratory notebooks out of the core repo unless they are curated examples.

## Plugin Security Rules

Editor plugins must not become a supply-chain trap.

Rules:

- install should be optional
- plugins should shell out only to trusted local Hum binaries or embedded signed binaries
- no hidden network calls
- no telemetry without explicit opt-in
- no arbitrary project script execution
- no duplicate parser semantics
- no writing outside the workspace without explicit user action
- version compatibility must be visible

## What To Build First

1. stable CLI diagnostics
2. stable spans and semantic graph node IDs
3. `hum graph` schema versioning
4. `hum syntax` as the first editor-neutral syntax surface
5. TextMate grammar generated from `hum syntax --format textmate` and refreshed with `tools/update_textmate_grammar.ps1`
6. LSP capability matrix tied to first-party CLI/schema facts
7. `hum lsp` with diagnostics, symbols, folding, hover, and formatting hooks
8. VS Code/Cursor adapter because it reaches the most users fastest
9. Neovim/Helix/Zed docs because LSP users can self-serve
10. Visual Studio adapter when Windows enterprise workflows justify it
11. JetBrains/Eclipse adapter docs and package shells
12. Jupyter kernel only after executable Hum exists

## Sources

- Stack Overflow Developer Survey 2025, Dev IDEs: https://survey.stackoverflow.co/2025/technology#1-integrated-development-environment
- Language Server Protocol overview: https://microsoft.github.io/language-server-protocol/
- VS Code language extensions overview: https://code.visualstudio.com/api/language-extensions/overview
- Visual Studio language service and editor extension points: https://learn.microsoft.com/en-us/visualstudio/extensibility/language-service-and-editor-extension-points
- Eclipse LSP4E: https://eclipse.dev/lsp4e/
- JupyterLab LSP: https://jupyterlab-lsp.readthedocs.io/en/latest/
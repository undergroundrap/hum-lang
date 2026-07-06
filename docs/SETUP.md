# Hum Setup

Date: 2026-07-06

## Purpose

Hum should be easy to open from a terminal, VS Code, Cursor, PyCharm, IntelliJ, Neovim, Helix, Zed, or another editor without needing machine-specific paths.

The setup rule is:

```text
portable defaults in the repo, personal editor state outside the repo
```

## Beginner Path

1. Install Git.
2. Install Rust with `rustup`.
3. Open a new terminal after installation so `cargo` is on `PATH`.
4. From the repo root, run:

```powershell
cargo test
cargo run -- check examples
```

If `cargo` is not found, fix the shell or editor environment so Cargo is on `PATH`. Do not copy a machine-specific Cargo path into docs, examples, scripts, or committed editor settings.

## Full Local Verification

Run this before opening a pull request, publishing a snapshot, or making a release-style commit:

```powershell
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
cargo run -- check examples
.\tools\check_text_hygiene.ps1
.\tools\check_public_readiness.ps1
```

## Editor Setup

Hum does not require a blessed editor.

Use an editor that can:

- open the repo root as a folder
- respect `.editorconfig`
- run Cargo commands from the integrated terminal or task runner
- use `rust-analyzer` for the Rust bootstrap compiler
- leave local workspace state uncommitted

Recommended editor behavior:

- VS Code and Cursor: open the repo folder, install `rust-analyzer`, and keep local `.vscode/` or workspace files untracked.
- PyCharm and IntelliJ: open the repo folder with Rust support enabled and keep `.idea/` and `*.iml` files untracked.
- Neovim, Helix, Zed, and similar editors: use `rust-analyzer` for Rust files and keep editor caches outside the repo.
- Plain terminal users: Cargo commands are enough for Milestone 0.

Hum syntax highlighting is not stable yet. Until `humfmt`, `chirp`, and `hum lsp` exist, `.hum` files are source sketches checked by the Rust bootstrap CLI.

## Path Rules

Public docs, examples, tests, and scripts should use repo-relative paths:

```text
examples/task_list.hum
docs/ARCHITECTURE.md
tools/check_text_hygiene.ps1
```

Avoid committing:

- absolute Windows, macOS, Linux, WSL, or network-share paths
- local home-directory paths
- editor install paths
- local Python, Rust, Java, or toolchain paths
- per-user IDE task, launch, workspace, or interpreter files
- shell-specific fixes that only work on one machine

If a command needs a local absolute path as a temporary workaround, keep it in your shell history, local editor settings, or private notes. The committed repo should show the portable command.

## Troubleshooting

If Cargo works in a terminal but not in an editor, restart the editor after installing Rust. If it still fails, configure the editor's environment locally and leave that config uncommitted.

If Git warns about line endings, keep `.gitattributes` and `.editorconfig` in place. The repo normalizes text files to LF while allowing Windows batch files to use CRLF.

If a hygiene or public-readiness check fails, fix the named file and line before committing. These checks are meant to catch setup and pathing problems before new contributors inherit them.
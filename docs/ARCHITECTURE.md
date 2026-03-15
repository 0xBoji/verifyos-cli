# Architecture

This repo is evolving into a multi-surface product: a Rust backend (scanner + engine), a CLI, and editor/front-end clients.

## Target Layout (Init)

```
apps/
  cli/            # Rust CLI entrypoint (voc)
  backend/        # Rust HTTP API (uploads + scan)
  frontend/       # Web UI or TUI shell (future)
packages/
  core/           # Rust engine + rules + report model
editors/
  vscode/         # VS Code extension (LSP + Action Center)
docs/
  ARCHITECTURE.md # This document
  STRUCTURE.md    # Tree + ownership notes
```

## Mapping From Current Repo

We keep the existing Rust crate at the repo root for stability while we bootstrap the new layout.

- `src/` today is the functional core and CLI.
- `editors/vscode/` already maps cleanly to the `editors/` area.
- `apps/cli` and `packages/core` are created as placeholders until we are ready to split into a workspace.

## Why This Split

- **Backend (Rust)** remains the single source of truth for rules, evidence, and repair hints.
- **Frontend** (web/TUI) can focus on UX without duplicating logic.
- **Editor clients** stay thin and launch `voc lsp` so diagnostics are always consistent with CLI.

## Migration Plan (When Ready)

1. Create a Cargo workspace at the root.
2. Move current `src/` into `packages/core`.
3. Add `apps/cli` crate that depends on `packages/core`.
4. Keep `editors/vscode` unchanged (it shells out to `voc`).

This keeps CI stable and avoids breaking existing releases while we grow the product.

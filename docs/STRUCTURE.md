# Repository Structure

```
.
├── apps/
│   ├── cli/            # CLI app shell (future workspace split)
│   ├── backend/        # Rust HTTP API for uploads + scan
│   └── frontend/       # Web UI or TUI shell (future)
├── packages/
│   └── core/           # Rust engine + rules + reports (future)
├── editors/
│   └── vscode/         # VS Code extension
├── docs/
│   ├── ARCHITECTURE.md # Design overview
│   └── STRUCTURE.md    # This file
├── src/                # Current Rust core + CLI (active today)
├── tests/              # Regression tests
└── examples/           # Fixtures
```

## Ownership Notes

- `src/` is still the live backend/CLI implementation.
- `apps/` and `packages/` are initialized to guide the eventual workspace split.
- `editors/` is production-ready and ships the VS Code extension.

# verifyOS VS Code Extension

This extension is intentionally thin. It starts the existing `voc lsp` binary so the Rust CLI remains the source of truth for diagnostics.

## Requirements

- `voc` installed and available on `PATH`, or a custom path set via `verifyOS.path`
- A workspace containing `Info.plist`, `.plist`, or `.xcprivacy` files

## Development

```bash
cd editors/vscode
npm install
npm run compile
```

Press `F5` in VS Code to launch an Extension Development Host.

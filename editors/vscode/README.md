# verifyOS for VS Code

`verifyOS` brings App Store submission diagnostics into VS Code by launching `voc lsp` from the `verifyOS-cli` Rust project.

It keeps the rule engine in Rust, keeps scans local, and surfaces findings directly in the **Problems** pane while you work on:

- `Info.plist`
- `PrivacyInfo.xcprivacy`

## Why this extension exists

App Store review failures are often caused by metadata drift, missing privacy declarations, overly broad ATS settings, or incomplete usage descriptions. `verifyOS` gives you faster feedback before you package and submit an `.ipa`.

The VS Code extension stays intentionally thin:

- the Rust CLI remains the source of truth
- no bundle data is sent to remote servers
- diagnostics come from `voc lsp`, not duplicated TypeScript logic
- packaged releases can ship a bundled `voc` binary for zero-config startup

## What you get

- Diagnostics in VS Code's **Problems** pane
- A clean output channel for the verifyOS language server
- Fast restart command when you change configuration
- The same rules and profiles as the `voc` CLI
- Zero-config startup on supported packaged builds via a bundled `voc` binary

## Requirements

- A workspace containing `Info.plist`, `.plist`, or `.xcprivacy` files
- Either:
  - a packaged extension build that already bundles `voc`, or
  - `voc` installed and available on `PATH`, or a custom path set via `verifyOS.path`

If you prefer using your own CLI install:

```bash
cargo install verifyos-cli
```

## Quick start

1. Install the `verifyOS` extension
2. If your build does not bundle `voc`, make sure it is available on your shell `PATH`
3. Open a workspace that contains `Info.plist` or `PrivacyInfo.xcprivacy`
4. Open one of those files and check the **Problems** pane

## Commands

- `verifyOS: Restart Language Server`
- `verifyOS: Show Output`

## Settings

### `verifyOS.path`

Fallback path to the `voc` binary. Use an absolute path if `voc` is not on `PATH`.

### `verifyOS.useBundledBinary`

Prefer the bundled `voc` binary that ships inside the extension when one is available for the current platform.

### `verifyOS.profile`

Rule profile passed to `voc lsp`.

- `basic`
- `full`

### `verifyOS.trace.server`

Trace level for the language server.

- `off`
- `messages`
- `verbose`

## Development

```bash
cd editors/vscode
npm ci
npm run compile
```

Press `F5` in VS Code to launch an Extension Development Host.

## Packaging and publishing

```bash
cd editors/vscode
npm ci
npm run package
```

The repository also includes `.github/workflows/vscode-extension.yml`, which packages a `.vsix` on tags and can publish to the VS Code Marketplace and Open VSX when `VSCE_PAT` and `OVSX_PAT` are configured.

That workflow also builds release binaries for:

- macOS arm64
- macOS x64
- Linux x64
- Windows x64

and places them under `bin/` inside the packaged extension so Marketplace installs can work out of the box.

# verifyOS for VS Code

`verifyOS` brings App Store submission diagnostics into VS Code by launching the existing `voc lsp` binary from the `verifyOS-cli` Rust project.

It keeps the rule engine in Rust, keeps scans local, and surfaces findings directly in the **Problems** pane while you work on:

- `Info.plist`
- `PrivacyInfo.xcprivacy`

## Why this extension exists

App Store review failures are often caused by metadata drift, missing privacy declarations, overly broad ATS settings, or incomplete usage descriptions. `verifyOS` gives you faster feedback before you package and submit an `.ipa`.

The VS Code extension is intentionally thin:

- the Rust CLI remains the source of truth
- no bundle data is sent to remote servers
- diagnostics come from `voc lsp`, not duplicated TypeScript logic

## What you get

- Diagnostics in VS Code's **Problems** pane
- A clean output channel for the verifyOS language server
- Fast restart command when you change configuration
- The same rules and profiles as the `voc` CLI

## Requirements

- `voc` installed and available on `PATH`, or a custom path set via `verifyOS.path`
- A workspace containing `Info.plist`, `.plist`, or `.xcprivacy` files

Install the CLI first if you have not already:

```bash
cargo install verifyos-cli
```

## Quick start

1. Install the `verifyOS` extension
2. Make sure `voc` is available on your shell `PATH`
3. Open a workspace that contains `Info.plist` or `PrivacyInfo.xcprivacy`
4. Open one of those files and check the **Problems** pane

## Commands

- `verifyOS: Restart Language Server`
- `verifyOS: Show Output`

## Settings

### `verifyOS.path`

Path to the `voc` binary. Use an absolute path if `voc` is not on `PATH`.

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

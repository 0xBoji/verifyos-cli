# verifyOS-cli

[![Crates.io](https://img.shields.io/crates/v/verifyos-cli.svg)](https://crates.io/crates/verifyos-cli)
[![Docs.rs](https://img.shields.io/docsrs/verifyos-cli)](https://docs.rs/verifyos-cli)
[![CI](https://github.com/0xBoji/verifyos-cli/actions/workflows/rust.yml/badge.svg)](https://github.com/0xBoji/verifyos-cli/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`verifyOS-cli` is a pure Rust, cross-platform CLI tool designed to scan Apple app bundles (like `.ipa`, `.app`, `Info.plist`, and Mach-O binaries) for App Store rejection risks before submission. Operating locally or within an automated CI pipeline, it catches potential App Store Connect rejections left of the build process.

The App Store Connect validation step is historically a "black box" that costs developers hours of waiting. By shifting validation to your local machine—or a fast, cheap Linux runner—verifyOS-cli empowers solo developers and robust teams alike. Unlike Apple's toolchain (`codesign`, `otool`), this tool is built entirely in Rust.

## What it does

- Acts as a local static analysis orchestrator for iOS/macOS apps.
- **Privacy Manifests**: Checks for missing `PrivacyInfo.xcprivacy` and API-required privacy labels.
- **Permissions (Info.plist)**: Validates the inclusion of mandatory descriptions (e.g., `NSLocationWhenInUseUsageDescription`) against linked frameworks.
- **Code Signatures**: Extracts binary entitlements and spots mismatches with the embedded provisioning profile.
- **Export Compliance**: Inspects binaries for the `ITSAppUsesNonExemptEncryption` to avoid manual Web UI confirmations.
- **Architecture & Metadata**: Ensures proper Mach-O universal sizes and UI asset configurations.

## Installation

### From crates.io

```bash
cargo install verifyos-cli
```

## Quick start

Run the CLI tool against your `.ipa` or `.app` path:

```bash
verifyos-cli path/to/YourApp.ipa
```

## Architecture

This project is structured as a Cargo Workspace to keep compilation times low and modularity high:

- **`verifyos-cli`**: Binary crate: CLI frontend, argument parsing, terminal UI
- **`verifyos-core`**: Core orchestrator and execution engine
- **`verifyos-parsers`**: Format parsers (`zip` extraction, `plist` mapping, `goblin`/`apple-codesign` Mach-O inspection)
- **`verifyos-rules`**: Trait-based rule engine representing static checks

## Conventional Commits

To ensure the automated semantic versioning and changelog parsing through the `release-plz` bot behaves properly, developers MUST use **Git Conventional Commits** format:

*   **`feat:`** A new feature (correlates to a MINOR `v0.X.0` bump).
*   **`fix:`** A bug fix (correlates to a PATCH `v0.0.X` bump).
*   **`docs:`** Documentation only changes.
*   **`style:`** Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc).
*   **`chore:`** Changes to the build process or auxiliary tools and libraries such as documentation generation.

## CI and releases

- CI: lint + tests on push and pull request.
- Automated release PRs: `release-plz` workflow.
- Publishing: crates.io + GitHub release artifacts.

## License

MIT

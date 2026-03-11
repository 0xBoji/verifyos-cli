# verifyOS-cli

[![Crates.io](https://img.shields.io/crates/v/verifyos-cli.svg)](https://crates.io/crates/verifyos-cli)
[![Docs.rs](https://img.shields.io/docsrs/verifyos-cli)](https://docs.rs/verifyos-cli)
[![CI](https://github.com/0xBoji/verifyos-cli/actions/workflows/rust.yml/badge.svg)](https://github.com/0xBoji/verifyos-cli/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`verifyOS-cli` is a pure Rust, cross-platform CLI tool designed to scan Apple app bundles (like `.ipa`, `.app`, `Info.plist`, and Mach-O binaries) for App Store rejection risks before submission. Operating locally or within an automated CI pipeline, it catches potential App Store Connect rejections left of the build process.

The App Store Connect validation step is historically a "black box" that costs developers hours of waiting. By shifting validation to your local machine—or a fast, cheap Linux runner—verifyOS-cli empowers solo developers and robust teams alike. Unlike Apple's toolchain (`codesign`, `otool`), this tool is built entirely in Rust.

## What it does

- Acts as a local static analysis orchestrator for iOS/macOS apps.
- **Ruleset metadata**: Every finding includes `rule_id`, `severity`, and `category` (Privacy, Entitlements, Metadata, etc.).
- **Privacy Manifests**: Checks for missing `PrivacyInfo.xcprivacy`.
- **Permissions (Info.plist)**: Uses a heuristic Mach-O scan to infer which `NS*UsageDescription` keys are required, then validates presence and non-empty values.
- **Entitlements**: Detects debug-only entitlements (like `get-task-allow=true`) and flags mismatches between app entitlements and `embedded.mobileprovision` (APNs, keychain groups, iCloud containers).
- **CI-friendly reports**: Outputs `table`, `json`, or `sarif` with evidence and remediation recommendations.

## Installation

### From crates.io

```bash
cargo install verifyos-cli
```

## Quick start

Run the CLI tool against your `.ipa` or `.app` path:

```bash
verifyos-cli --app path/to/YourApp.ipa
```

### Output Formats

Table (default):

```bash
verifyos-cli --app path/to/YourApp.ipa --format table
```

JSON:

```bash
verifyos-cli --app path/to/YourApp.ipa --format json > report.json
```

SARIF (for GitHub code scanning, etc.):

```bash
verifyos-cli --app path/to/YourApp.ipa --format sarif > report.sarif
```

### Baseline Mode

Suppress existing findings by providing a baseline JSON report. Only *new* failing findings will be shown:

```bash
verifyos-cli --app path/to/YourApp.ipa --format json > baseline.json
verifyos-cli --app path/to/YourApp.ipa --baseline baseline.json
```

Baseline matching currently uses `rule_id + evidence` for failing findings.

### Example Passing Output
```text
 Analysis complete!                                                       
╭──────────────────────────────────┬─────────────┬──────────┬────────┬────────────────╮
│ Rule                             ┆ Category    ┆ Severity ┆ Status ┆ Message        │
╞══════════════════════════════════╪═════════════╪══════════╪════════╪════════════════╡
│ Missing Privacy Manifest         ┆ Privacy     ┆ ERROR    ┆ PASS   ┆ PASS           │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Missing Camera Usage Description ┆ Permissions ┆ ERROR    ┆ PASS   ┆ PASS           │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Debug Entitlements Present       ┆ Entitlements┆ ERROR    ┆ PASS   ┆ PASS           │
╰──────────────────────────────────┴─────────────┴──────────┴────────┴────────────────╯
```

### Example Failing Output (Exits with code 1)
```text
 Analysis complete!                                                       
╭──────────────────────────────────┬─────────────┬──────────┬────────┬──────────────────────────────────────────────╮
│ Rule                             ┆ Category    ┆ Severity ┆ Status ┆ Message                                      │
╞══════════════════════════════════╪═════════════╪══════════╪════════╪══════════════════════════════════════════════╡
│ Missing Privacy Manifest         ┆ Privacy     ┆ ERROR    ┆ FAIL   ┆ Missing PrivacyInfo.xcprivacy                │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Missing required usage description keys ┆ Privacy ┆ WARNING ┆ FAIL ┆ Missing required usage description keys      │
╰──────────────────────────────────┴─────────────┴──────────┴────────┴──────────────────────────────────────────────╯
```

## Architecture

This project is structured with modularity in mind:

- **`core/`**: Orchestrator and logic execution engine.
- **`parsers/`**: Format handlers (`zip` extraction, `plist` mapping, `goblin`/`apple-codesign` Mach-O inspection).
- **`rules/`**: Trait-based rule engine representing the validation checks.

## Conventional Commits

To ensure the automated semantic versioning and changelog parsing through the `release-plz` bot behaves properly, developers MUST use **Git Conventional Commits** format:

*   **`feat:`** A new feature (correlates to a MINOR `v0.X.0` bump).
*   **`fix:`** A bug fix (correlates to a PATCH `v0.0.X` bump).
*   **`docs:`** Documentation only changes.
*   **`chore:`** Changes to the build process or auxiliary tools.

## CI and releases

- CI: lint + tests on push and pull request.
- Automated release PRs: `release-plz` workflow.
- Publishing: crates.io + GitHub release artifacts.

## License

MIT

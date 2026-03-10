# verifyOS-cli

verifyOS-cli is a pure Rust, cross-platform CLI tool designed to scan Apple app bundles (like `.ipa`, `.app`, `Info.plist`, and Mach-O binaries) for App Store rejection risks before submission. Operating locally or within an automated CI pipeline, it catches potential App Store Connect rejections left of the build process.

## Why verifyOS-cli?

The App Store Connect validation step is historically a "black box" that costs developers hours of waiting. By shifting validation to your local machine—or a fast, cheap Linux runner—verifyOS-cli empowers solo developers and robust teams alike. Unlike Apple's toolchain (`codesign`, `otool`), this tool is built entirely in Rust, enabling analysis of property lists, Mach-O binaries, and distribution archives seamlessly across all major operating systems.

## What it Inspects (Rejection Risks)

*   **Privacy Manfiests:** Checks for missing `PrivacyInfo.xcprivacy` and API-required privacy labels.
*   **Permissions (Info.plist):** Validates the inclusion of mandatory descriptions (e.g., `NSLocationWhenInUseUsageDescription`) against linked frameworks.
*   **Code Signatures:** Extracts binary entitlements and spots mismatches with the embedded provisioning profile.
*   **Export Compliance:** Inspects binary for the `ITSAppUsesNonExemptEncryption` to avoid manual Web UI confirmations.
*   **Architecture & Metadata:** Ensures proper Mach-O universal sizes and UI asset configurations.

## Architecture

This project is structured as a Cargo Workspace to keep compilation times low and modularity high:

```text
verifyOS-workspace/
├── Cargo.toml
├── crates/
│   ├── verifyos-cli/      # Binary crate: CLI frontend, argument parsing, terminal UI
│   ├── verifyos-core/     # Library: Core orchestrator and execution engine
│   ├── verifyos-parsers/  # Library: Format parsers (zip extraction, plist mapping, Mach-O inspection)
│   └── verifyos-rules/    # Library: The Trait-based rule engine representing static checks
```

## Tech Stack highlights

*   **`clap`:** Type-safe, declarative CLI argument parsing.
*   **`miette`:** Diagnostic parser error handling for crystal-clear CLI feedback.
*   **`apple-codesign` / `goblin`:** Pure Rust Apple Code Signature and Mach-O handlers.
*   **`plist` / `zip`:** Archive extraction and Apple property list mapping.

## CI/CD and Release Workflow

We use a fully automated release cycle driven by the `release-plz` bot and GitHub Actions:
- On every push and Pull Request, the CI suite (`cargo fmt`, `clippy`, `test`) is guaranteed via `rust.yml`.
- Standard GitHub Tags trigger the `release.yml` pipeline, instantly compiling release binaries for macOS and Linux (`x86_64`, `aarch64`) and attaching them to the new GitHub Application Release. 
- The integrated `release-plz` bot automatically scans for new version intervals, updates `Cargo.toml`, generates changelogs, publishes the tag backwards to the Git repo, and even uploads to `crates.io`.

## Conventional Commits

To ensure the automated semantic versioning and changelog parsing through the `release-plz` bot behaves properly, developers MUST use **Git Conventional Commits** format:

*   **`feat:`** A new feature (correlates to a MINOR `v0.X.0` bump).
*   **`fix:`** A bug fix (correlates to a PATCH `v0.0.X` bump).
*   **`docs:`** Documentation only changes.
*   **`style:`** Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc).
*   **`refactor:`** A code change that neither fixes a bug nor adds a feature.
*   **`perf:`** A code change that improves performance.
*   **`test:`** Adding missing tests or correcting existing tests.
*   **`chore:`** Changes to the build process or auxiliary tools and libraries such as documentation generation.

Example:
`feat: Add check for missing NSCameraUsageDescription`
`fix: correct zip stream reading panic on large .ipa files`

## License

MIT

<p align="center">
  <img src="icons/verifyOS.png" alt="verifyOS icon" />
</p>


<p align="center">
  <a href="https://crates.io/crates/verifyos-cli">
    <img src="https://img.shields.io/crates/v/verifyos-cli.svg" alt="Crates.io" />
  </a>
  <a href="https://docs.rs/verifyos-cli">
    <img src="https://img.shields.io/docsrs/verifyos-cli" alt="Docs.rs" />
  </a>
  <a href="https://github.com/0xBoji/verifyos-cli/actions/workflows/rust.yml">
    <img src="https://github.com/0xBoji/verifyos-cli/actions/workflows/rust.yml/badge.svg" alt="CI" />
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT" />
  </a>
</p>

`verifyOS-cli` is an AI agent-friendly, pure Rust CLI for scanning Apple app bundles (like `.ipa`, `.app`, `Info.plist`, and Mach-O binaries) for App Store rejection risks before submission. It is built for developers, vibecoders, and automation workflows that want fast, local feedback before App Store Connect becomes the bottleneck.

The App Store Connect validation step is historically a "black box" that costs developers hours of waiting. By shifting validation to your local machine, CI runner, or AI agent loop, verifyOS-cli helps teams catch rejection risks early and produce structured output an agent can use to patch issues faster. Unlike Apple's toolchain (`codesign`, `otool`), this tool is built entirely in Rust.

## What it does

- Acts as a local static analysis orchestrator for iOS/macOS apps.
- **Ruleset metadata**: Every finding includes `rule_id`, `severity`, and `category` (Privacy, Entitlements, Metadata, etc.).
- **Privacy Manifests**: Checks for missing `PrivacyInfo.xcprivacy`.
- **Permissions (Info.plist)**: Uses a heuristic Mach-O scan to infer which `NS*UsageDescription` keys are required, then validates presence and non-empty values.
- **LSApplicationQueriesSchemes**: Audits custom URL scheme allowlists for duplicates, invalid entries, and potential private schemes.
- **UIRequiredDeviceCapabilities**: Flags capabilities that don't match observed binary usage.
- **ATS**: Flags overly broad ATS exceptions (global allows, includes subdomains, insecure TLS settings).
- **Bundle leakage**: Fails if sensitive files (e.g., `.p12`, `.pem`, `.mobileprovision`, `.env`) are found inside the app bundle.
- **Versioning**: Ensures `CFBundleShortVersionString` and `CFBundleVersion` are valid and present.
- **Extension entitlements**: Validates extension entitlements are a subset of the host app and required keys exist for common extension types.
- **Privacy SDK cross-check**: Warns if common SDK signatures are detected but PrivacyInfo.xcprivacy lacks declarations.
- **Entitlements**: Detects debug-only entitlements (like `get-task-allow=true`) and flags mismatches between app entitlements and `embedded.mobileprovision` (APNs, keychain groups, iCloud containers).
- **Signing**: Ensures embedded frameworks/extensions are signed with the same Team ID as the app binary.
- **CI-friendly reports**: Outputs `table`, `json`, or `sarif` with evidence and remediation recommendations.

## Installation

### From crates.io

```bash
cargo install verifyos-cli
```

This installs the `voc` binary for the CLI.

## Quick start

Run the CLI tool against your `.ipa` or `.app` path:

```bash
voc --app path/to/YourApp.ipa
```

Bootstrap an `AGENTS.md` file for AI agent workflows:

```bash
voc init
```

If `AGENTS.md` already exists, `voc init` preserves your custom content and replaces only the managed `verifyos-cli` block.

### Profiles

Run a smaller core ruleset or the full scan:

```bash
voc --app path/to/YourApp.ipa --profile basic
```

```bash
voc --app path/to/YourApp.ipa --profile full
```

`full` is the default if `--profile` is omitted.

### Exit thresholds

Control when the CLI exits with code `1`:

```bash
voc --app path/to/YourApp.ipa --fail-on off
```

```bash
voc --app path/to/YourApp.ipa --fail-on warning
```

`error` is the default if `--fail-on` is omitted.

### Rule selectors

Run only specific rules or exclude noisy ones by rule ID:

```bash
voc --app path/to/YourApp.ipa --include RULE_PRIVATE_API,RULE_ATS_AUDIT
```

```bash
voc --app path/to/YourApp.ipa --exclude RULE_PRIVATE_API
```

Selectors apply after the chosen `--profile`, so `basic` plus `--include` can narrow the set even further.

### Rule inventory

List available rules and their default profile membership:

```bash
voc --list-rules
```

Machine-readable inventory for agents and CI:

```bash
voc --list-rules --format json
```

Inspect one rule in detail:

```bash
voc --show-rule RULE_PRIVATE_API
voc --show-rule RULE_PRIVATE_API --format json
```

### Config file

If `verifyos.toml` exists in the current working directory, `voc` will load it automatically. You can also point to a specific config file:

```bash
voc --app path/to/YourApp.ipa --config verifyos.toml
```

Example config:

```toml
format = "table"
profile = "full"
fail_on = "error"
timings = "off"
include = []
exclude = []
```

CLI flags override config file values.

### AGENTS.md bootstrap

Generate or refresh an `AGENTS.md` playbook in the current directory:

```bash
voc init
```

Write to a custom path:

```bash
voc init --path docs/AGENTS.md
```

Scan an app first and inject the current failing rules into `AGENTS.md`:

```bash
voc init --from-scan path/to/YourApp.ipa
```

Use a lighter profile when you only want a quick playbook refresh:

```bash
voc init --from-scan path/to/YourApp.ipa --profile basic
```

Keep only new or regressed risks relative to an older report:

```bash
voc init --from-scan path/to/YourApp.ipa --baseline old-report.json
```

The generated block includes:
- a recommended `voc` workflow for quick and release scans
- AI agent fix-loop rules
- a live rule inventory with `rule_id`, category, severity, and default profiles
- an optional `Current Project Risks` section with priority order and suggested fix scopes from the latest scan

When `--baseline` is provided with `--from-scan`, the `Current Project Risks` section only keeps findings that are new or regressed compared with the older JSON report. That keeps the playbook focused on what changed in the current branch.

`voc init` uses a managed block, so you can safely keep your own notes above or below it.

### Output Formats

Table (default):

```bash
voc --app path/to/YourApp.ipa --format table
```

JSON:

```bash
voc --app path/to/YourApp.ipa --format json > report.json
```

SARIF (for GitHub code scanning, etc.):

```bash
voc --app path/to/YourApp.ipa --format sarif > report.sarif
```

Markdown report (agent-friendly):

```bash
voc --app path/to/YourApp.ipa --md-out report.md
```

Agent fix pack:

```bash
voc --app path/to/YourApp.ipa --agent-pack fixes.json
```

The agent pack writes a machine-readable JSON file with failing findings only, including `rule_id`, `message`, `evidence`, `recommendation`, `priority`, `suggested_fix_scope`, `target_files`, `patch_hint`, and `why_it_fails_review`.

Markdown agent pack:

```bash
voc --app path/to/YourApp.ipa --agent-pack fixes.md --agent-pack-format markdown
```

Bundle agent pack:

```bash
voc --app path/to/YourApp.ipa --agent-pack .verifyos-agent --agent-pack-format bundle
```

`--agent-pack-format` supports `json`, `markdown`, and `bundle`. `bundle` writes both `agent-pack.json` and `agent-pack.md`, with the Markdown output grouped by `suggested_fix_scope` so AI agents and humans can work from the same fix queue.
The extra patch-hint fields are designed so an AI agent can jump directly to likely edit targets such as `Info.plist`, `PrivacyInfo.xcprivacy`, entitlements, or bundled SDK/resources without guessing first.

Timing summary:

```bash
voc --app path/to/YourApp.ipa --timings
```

Full timing details:

```bash
voc --app path/to/YourApp.ipa --timings full
```

`--timings` by itself defaults to `summary`, which prints total scan time, slowest rules, and cache activity without adding a per-rule time column. Use `--timings full` when you want the table and markdown outputs to include per-rule execution times too.
JSON and Markdown reports still carry timing data for automation and profiling.
The timing summary also highlights the slowest rules so you can spot hot paths quickly.
It also includes cache hit/miss activity for artifact scans so we can tell whether a slow run is coming from repeated IO or genuinely expensive rules.
JSON and SARIF outputs now expose machine-readable perf metadata too, including `slow_rules`, `total_duration_ms`, and cache telemetry.

### Baseline Mode

Suppress existing findings by providing a baseline JSON report. Only *new* failing findings will be shown:

```bash
voc --app path/to/YourApp.ipa --format json > baseline.json
voc --app path/to/YourApp.ipa --baseline baseline.json
```

Baseline matching currently uses `rule_id + evidence` for failing findings.

### Example Passing Output
```text
Analysis complete!
╭────────────────────────────────────────┬─────────────┬──────────┬────────┬─────────╮
│ Rule                                   ┆ Category    ┆ Severity ┆ Status ┆ Message │
╞════════════════════════════════════════╪═════════════╪══════════╪════════╪═════════╡
│ Missing Privacy Manifest               ┆ Privacy     ┆ ERROR    ┆ PASS   ┆ PASS    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ Missing Camera Usage Description       ┆ Permissions ┆ ERROR    ┆ PASS   ┆ PASS    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ LSApplicationQueriesSchemes Audit      ┆ Metadata    ┆ WARNING  ┆ PASS   ┆ PASS    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ UIRequiredDeviceCapabilities Audit     ┆ Metadata    ┆ WARNING  ┆ PASS   ┆ PASS    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ ATS Exceptions Too Broad               ┆ ATS         ┆ WARNING  ┆ PASS   ┆ PASS    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ Sensitive Files in Bundle              ┆ Bundling    ┆ ERROR    ┆ PASS   ┆ PASS    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ Info.plist Versioning Consistency      ┆ Metadata    ┆ WARNING  ┆ PASS   ┆ PASS    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ Extension Entitlements Compatibility   ┆ Entitlements┆ WARNING  ┆ PASS   ┆ PASS    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ Privacy Manifest vs SDK Usage          ┆ Privacy     ┆ WARNING  ┆ PASS   ┆ PASS    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┤
│ Debug Entitlements Present             ┆ Entitlements┆ ERROR    ┆ PASS   ┆ PASS    │
╰────────────────────────────────────────┴─────────────┴──────────┴────────┴─────────╯
```

### Example Failing Output (Exits with code 1)
```text
Analysis complete!
╭────────────────────────────────────────┬─────────────┬──────────┬────────┬────────────────────────────────────────────────────────────╮
│ Rule                                   ┆ Category    ┆ Severity ┆ Status ┆ Message                                                    │
╞════════════════════════════════════════╪═════════════╪══════════╪════════╪════════════════════════════════════════════════════════════╡
│ Missing Privacy Manifest               ┆ Privacy     ┆ ERROR    ┆ FAIL   ┆ Missing PrivacyInfo.xcprivacy                              │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Missing required usage description keys┆ Privacy     ┆ WARNING  ┆ FAIL   ┆ Missing required usage description keys                    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ LSApplicationQueriesSchemes Audit      ┆ Metadata    ┆ WARNING  ┆ PASS   ┆ PASS                                                       │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ UIRequiredDeviceCapabilities Audit     ┆ Metadata    ┆ WARNING  ┆ PASS   ┆ PASS                                                       │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ ATS Exceptions Too Broad               ┆ ATS         ┆ WARNING  ┆ PASS   ┆ PASS                                                       │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Sensitive Files in Bundle              ┆ Bundling    ┆ ERROR    ┆ PASS   ┆ PASS                                                       │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Info.plist Versioning Consistency      ┆ Metadata    ┆ WARNING  ┆ PASS   ┆ PASS                                                       │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Extension Entitlements Compatibility   ┆ Entitlements┆ WARNING  ┆ PASS   ┆ PASS                                                       │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Privacy Manifest vs SDK Usage          ┆ Privacy     ┆ WARNING  ┆ PASS   ┆ PASS                                                       │
╰────────────────────────────────────────┴─────────────┴──────────┴────────┴────────────────────────────────────────────────────────────╯
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

- CI: fmt + clippy + tests on push and pull request.
- Automated release PRs: `release-plz` workflow.
- Publishing: crates.io + GitHub release artifacts.

## License

MIT

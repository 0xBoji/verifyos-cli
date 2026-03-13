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

### From Release (via curl)

Download the pre-built binary for your platform:

```bash
# macOS (Apple Silicon)
curl -L https://github.com/0xBoji/verifyos-cli/releases/latest/download/verifyos-cli-macos-arm64 -o voc && chmod +x voc

# macOS (Intel)
curl -L https://github.com/0xBoji/verifyos-cli/releases/latest/download/verifyos-cli-macos-amd64 -o voc && chmod +x voc

# Linux (amd64)
curl -L https://github.com/0xBoji/verifyos-cli/releases/latest/download/verifyos-cli-linux-amd64 -o voc && chmod +x voc
```

Move it to your PATH to use it globally:
```bash
mv voc /usr/local/bin/
```


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

[init]
output_dir = ".verifyos"
write_commands = true
shell_script = true
fix_prompt = true
profile = "basic"

[doctor]
output_dir = ".verifyos"
fix = true
repair = ["pr-comment"]
freshness_against = "report.json"
plan_out = ".verifyos/repair-plan.md"
profile = "basic"
open_pr_brief = true
open_pr_comment = true
```

Top-level keys apply to normal `voc --app ...` scans. `[init]` and `[doctor]` let you keep agent-workflow defaults in one place so you do not have to repeat `--output-dir`, `--profile`, or PR handoff flags every run.

CLI flags still override config file values.

### AGENTS.md bootstrap

Generate or refresh an `AGENTS.md` playbook in the current directory:

```bash
voc init
```

Write to a custom path:

```bash
voc init --path docs/AGENTS.md
```

Use one root directory for generated init assets:

```bash
voc init --output-dir .verifyos --from-scan path/to/YourApp.ipa
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

Refresh `AGENTS.md` and generate an agent bundle in one step:

```bash
voc init --from-scan path/to/YourApp.ipa --agent-pack-dir .verifyos-agent
```

Also inject copy-paste follow-up commands for the next agent loop:

```bash
voc init --from-scan path/to/YourApp.ipa --agent-pack-dir .verifyos-agent --write-commands
```

Generate a runnable follow-up script too:

```bash
voc init --from-scan path/to/YourApp.ipa --agent-pack-dir .verifyos-agent --write-commands --shell-script
```

Generate a dedicated AI handoff prompt too:

```bash
voc init --output-dir .verifyos --from-scan path/to/YourApp.ipa --fix-prompt
```

The generated block includes:
- a recommended `voc` workflow for quick and release scans
- AI agent fix-loop rules
- a live rule inventory with `rule_id`, category, severity, and default profiles
- an optional `Current Project Risks` section with priority order and suggested fix scopes from the latest scan
- an optional pointer to `agent-pack.json` and `agent-pack.md` when `--agent-pack-dir` is used
- an optional `Next Commands` section with exact re-scan and report refresh commands when `--write-commands` is used
- an optional `.verifyos-agent/next-steps.sh` when `--shell-script` is used
- an optional `fix-prompt.md` when `--fix-prompt` is used
- an optional `repair-plan.md` pointer so fix prompts and PR handoff docs share the same repair source

When `--baseline` is provided with `--from-scan`, the `Current Project Risks` section only keeps findings that are new or regressed compared with the older JSON report. That keeps the playbook focused on what changed in the current branch.

`voc init` uses a managed block, so you can safely keep your own notes above or below it.

### Doctor

Run a quick self-check on the current setup:

```bash
voc doctor
```

Check an output root created by `voc init --output-dir`:

```bash
voc doctor --output-dir .verifyos
```

Show a repair plan first without rewriting anything:

```bash
voc doctor --output-dir .verifyos --repair pr-comment --plan --format json
```

Write the same preview as a Markdown handoff file:

```bash
voc doctor --output-dir .verifyos --from-scan path/to/YourApp.ipa --repair pr-comment --plan --plan-out .verifyos/repair-plan.md
```

When `--plan` is paired with `--from-scan`, the JSON output also includes `plan_context` so agents can see:
- whether the preview is based on `fresh-scan` or `existing-assets`
- the exact scan artifact path
- the baseline path in play, if any
- the freshness source used for stale-asset checks
- the effective repair targets

Repair a broken or missing local agent setup in place:

```bash
voc doctor --output-dir .verifyos --fix
```

Repair only selected outputs:

```bash
voc doctor --output-dir .verifyos --fix --repair pr-comment
voc doctor --output-dir .verifyos --fix --repair agent-bundle
```

Repair and refresh the setup from a fresh scan:

```bash
voc doctor --output-dir .verifyos --fix --from-scan path/to/YourApp.ipa --profile basic
```

Generate a PR-ready brief alongside the refreshed agent assets:

```bash
voc doctor --output-dir .verifyos --fix --from-scan path/to/YourApp.ipa --profile basic --open-pr-brief
```

Generate a shorter PR comment draft for sticky comments or manual updates:

```bash
voc doctor --output-dir .verifyos --fix --from-scan path/to/YourApp.ipa --profile basic --open-pr-comment
```

Use a specific report file as the freshness source:

```bash
voc doctor --output-dir .verifyos --freshness-against report.json
```

`voc doctor` validates:
- config parsing
- `AGENTS.md` presence
- referenced agent assets like `agent-pack.json`, `agent-pack.md`, and `next-steps.sh`
- whether generated agent assets look stale compared with the newest `report.json` or `report.sarif` in the output root, or a file passed through `--freshness-against`
- sample `voc` commands inside `AGENTS.md`
- `next-steps.sh` command health, including whether follow-up flags like `--open-pr-brief` and `--open-pr-comment` still match the managed block

When `--fix` is enabled, `voc doctor` will:
- create or refresh `AGENTS.md`
- recreate `.verifyos-agent/agent-pack.json`
- recreate `.verifyos-agent/agent-pack.md`
- recreate `.verifyos-agent/next-steps.sh`
- recreate `fix-prompt.md`
- repair the managed `verifyos-cli` block so its pointers line up with the chosen output root again

`--repair` lets you scope that work to just the targets you want: `agents`, `agent-bundle`, `fix-prompt`, `pr-brief`, or `pr-comment`.

`--plan` adds a repair preview so you can see which files would be rebuilt before you run a write operation.

When `--fix --from-scan` is enabled, `voc doctor` does the same repair work but uses a fresh app scan to repopulate:
- `Current Project Risks`
- `agent-pack.json`
- `agent-pack.md`
- `fix-prompt.md`
- `next-steps.sh`
- follow-up commands that point back to the scanned artifact

When `--open-pr-brief` is added, `voc doctor` also writes `pr-brief.md` with:
- a concise risk summary
- current findings in patch order
- target files and patch hints
- validation commands for the next review loop
- a pointer back to `repair-plan.md`

When `--open-pr-comment` is added, `voc doctor` also writes `pr-comment.md` with:
- a shorter review summary for GitHub PR comments
- top risks only
- quick validation commands

You can also combine `--baseline old-report.json` with `--fix --from-scan` to keep only new or regressed risks in the repaired setup.

### GitHub Actions wrapper

This repo ships a reusable workflow at `.github/workflows/voc-analysis.yml` for CI and PR review flows.

Manual run from the Actions tab:

```text
Workflow: voc Analysis
Inputs:
- app_path
- baseline_path (optional)
- profile
- fail_on
- output_dir
- comment_on_pr
- pr_number (optional)
```

Reusable workflow example:

```yaml
name: App review

on:
  pull_request:
    branches: ["main"]

jobs:
  voc:
    uses: 0xBoji/verifyos-cli/.github/workflows/voc-analysis.yml@main
    with:
      app_path: path/to/YourApp.ipa
      baseline_path: baseline.json
      profile: full
      fail_on: error
      output_dir: .verifyos-ci
      doctor_repair: pr-comment
      comment_on_pr: true
      comment_mode: sticky
      pr_number: ${{ github.event.pull_request.number }}
```

The workflow generates and uploads:
- `report.sarif`
- `AGENTS.md`
- `fix-prompt.md`
- `repair-plan.md`
- `pr-brief.md`
- `pr-comment.md`
- `doctor.json`
- `.verifyos-agent/agent-pack.json`
- `.verifyos-agent/agent-pack.md`
- `.verifyos-agent/next-steps.sh`

When `comment_on_pr` is enabled and a PR number is available, the workflow also updates a sticky PR comment from `pr-comment.md` when present, with a safe fallback to an inline summary if that file is missing.

`doctor_repair` lets the workflow scope `voc doctor --fix` to specific outputs such as `pr-comment` or `agent-bundle`. `comment_mode` controls whether `voc pr-comment` emits a sticky marker (`sticky`) or a plain body (`plain`).

If the workflow inputs are left empty and `verifyos.toml` exists, `voc-analysis.yml` will also read:

```toml
[ci]
doctor_repair = ["pr-comment"]
comment_mode = "sticky"
```

Those values act as repository defaults for the reusable workflow.

You can build the same sticky body locally or in custom CI steps with:

```bash
voc pr-comment --output-dir .verifyos-ci --from-plan --scan-exit 1 --doctor-exit 0 --sticky-marker
voc pr-comment --from-plan --plan-path /tmp/repair-plan.md --sticky-marker
```

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

`verifyOS-cli` is organized as a layered scanner plus an AI-agent handoff system:

- **`src/main.rs`**: CLI entrypoint, subcommands (`scan`, `init`, `doctor`), output routing, and exit policy.
- **`src/core/`**: Scan orchestration, rule execution timing, and artifact-context lifecycle.
- **`src/parsers/`**: Low-level readers for `.ipa`, `.app`, `Info.plist`, provisioning profiles, Mach-O usage/signing/SDK scans, and related bundle metadata.
- **`src/rules/`**: Trait-based App Store review rules grouped by concern such as privacy, entitlements, signing, ATS, metadata, and bundling.
- **`src/report/`**: Normalized report model plus renderers for table, JSON, SARIF, Markdown, agent-pack JSON, and agent-pack Markdown.
- **`src/profiles.rs`**: Rule inventory, default profile membership, and CLI-facing rule metadata.
- **`src/agents.rs`**: `AGENTS.md` managed block generation, current-risk summaries, next-step commands, and fix-prompt rendering.
- **`src/doctor.rs`**: Project self-checks for config, `AGENTS.md`, referenced assets, and repair-oriented setup validation.
- **`tests/`**: CLI, report, config, and rule-level regression coverage for both normal scans and agent workflows.

The design goal is to keep scanning concerns, report rendering, and AI-agent onboarding separate enough that we can keep adding rules without tangling the developer workflow around them.

## One-Page System Data Flow

1. **Input**
   `voc` receives an `.ipa` or `.app`, optional config, profile, include/exclude filters, baseline, and output targets.
2. **Artifact preparation**
   `core::engine` resolves the app bundle, while `parsers/` load plist files, provisioning data, Mach-O metadata, and bundle resources.
3. **Cached scan context**
   `ArtifactContext` caches expensive lookups such as usage scans, signing summaries, bundle file indexes, entitlements, and plist reads so multiple rules can reuse them.
4. **Rule execution**
   `rules/` run against the shared artifact context and emit normalized results with:
   - `rule_id`
   - `category`
   - `severity`
   - `message`
   - `evidence`
   - `recommendation`
5. **Report normalization**
   `report/` converts engine output into a stable report model, applies timing metadata, cache telemetry, and optional baseline suppression.
6. **Primary outputs**
   The CLI renders one of:
   - table
   - JSON
   - SARIF
   - Markdown
   - agent-pack JSON/Markdown/bundle
7. **Agent workflow outputs**
   `voc init` and `voc doctor --fix` can materialize:
   - `AGENTS.md`
   - `.verifyos-agent/agent-pack.json`
   - `.verifyos-agent/agent-pack.md`
   - `.verifyos-agent/next-steps.sh`
   - `fix-prompt.md`
   - `pr-brief.md` when `--open-pr-brief` is enabled
   - `pr-comment.md` when `--open-pr-comment` is enabled
8. **CI / PR integration**
   The reusable workflow `.github/workflows/voc-analysis.yml` runs scans in GitHub Actions, uploads SARIF and agent assets, and can post a sticky PR summary comment.
9. **Repair / refresh loop**
   Developers or AI agents patch the suggested target files, rerun `voc`, compare against the previous baseline or agent pack, and repeat until findings clear.

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

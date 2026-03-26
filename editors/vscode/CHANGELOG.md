# Changelog
 
## 0.1.19

- Design Refresh: Applied "Apple Liquid Glass" branding with System Blue accents
- Updated bundled `voc` binary to the latest version (v0.12.0)

## 0.1.18

- UX Refinement: Revert 'Start' button icons to `play` (▶) for better clarity while keeping `check` for Activity Bar

## 0.1.17

- Activity Bar Identity: Switched sidebar icon to the standard checkmark (`$(check)`) symbol

## 0.1.16

- Icon Refinement: Use `check` (tick) icons for scan and handoff execution buttons

## 0.1.15

- Transform Action Center into a hierarchical Accordion-style UI
- Scan and Handoff actions are now collapsible, revealing a detailed "Start" button when expanded
- Removed QuickPick popups in favor of more natural TreeView interaction

## 0.1.14

- Enhanced UX with progress indicators ("Scanning...") for all CLI actions
- Added success and error toast notifications
- Added "Clear Output" command to the Action Center
- Added QuickPick confirmation pop-up when starting scans to prevent accidental execution

## 0.1.13

- Refine `AGENTS.md` structure: wrap generated content in horizontal rules (`---`)
- Improve spacing when injecting content into existing `AGENTS.md`

## 0.1.12

- Stabilize exit code handling for scans with findings (no more 'Command failed' popups)
- Add lifecycle diagnostics to Language Server to debug silent exits
- Maintain ZIP/directory-aware scanning fix

## 0.1.11

- Handle exit code 1 (findings found) gracefully without showing error notifications
- Improve CLI command logging in the output panel

## 0.1.10

- Fix 'Zip Error: Is a directory' when scanning `.app` bundles via the sidebar
- Bundled updated `voc` binary with raw directory scanning support

## 0.1.8

- Revert publisher ID to 0xboji as requested
- Maintain Mac ARM64 binary bundling and UI reliability fixes

## 0.1.7

- Fix publisher ID to match marketplace account
- Move UI registration to early activation phase for better reliability

## 0.1.6

- Registered the Action Center tree provider explicitly for more reliable sidebar activation
- Added a dedicated activity-bar icon with a cleaner verification-style mark

## 0.1.4

- Fixed Action Center activation so the sidebar always registers its tree data provider when opened
- Kept the branded verifyOS sidebar experience intact with the status/action icons

## 0.1.3

- Added a branded Action Center sidebar in the activity bar
- Added quick actions for bundle scans, handoff generation, Problems, output, and language server restart
- Improved background activation so verifyOS feels live earlier in the editor session
- Refreshed Marketplace presentation and bundled-binary startup behavior

## 0.1.2

- Bundled platform-specific `voc` binaries into packaged Marketplace builds
- Switched the Marketplace icon to the 128px verifyOS logo
- Added clearer extension documentation for bundled startup

## 0.1.1

- Improved Marketplace presentation with branding, better overview copy, and packaging polish

## 0.1.0

- Initial VS Code extension release for `voc lsp`

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.12.0](https://github.com/0xBoji/verifyOS/compare/v0.11.0...v0.12.0) - 2026-03-17

### Added

- *(frontend)* implement diagnostic AST visualization with pan/zoom and UI refinements
- *(frontend)* implement pan and zoom for AST diagnostic tree
- *(frontend)* add interactive details panel to AST visualization
- *(frontend)* transition AST visualization to Modal to fix layout issues
- *(frontend)* implement diagnostic AST visualization with Apple-inspired design
- implement multi-target scanning and fix aggregate scan discrepancies

### Fixed

- *(frontend)* improve AST mode connectors node spacing and label readability
- *(ci)* whitelist package-lock.json to fix release-plz
- *(ci)* fix formatting and ReportData initialization in tests

### Other

- Remove unused `FiActivity` icon and `selectedNode` state, and add ESLint disable comments.
- *(frontend)* update comprehensive README and workflow documentation
- *(frontend)* theme-aware custom scrollbars
- *(frontend)* add custom iOS-inspired scrollbar styles
- update .dockerignore with new result patterns

## [0.11.0](https://github.com/0xBoji/verifyOS/compare/v0.10.1...v0.11.0) - 2026-03-17

### Added

- *(engine)* implement multi-target scanning and result aggregation
- *(frontend)* implement smart folder discovery and noise filtering
- *(frontend)* add client-side folder zipping support
- support project-only scans and improve frontend folder upload feedback
- *(frontend)* allow uploading .zip and Xcode project files in scan tool

### Fixed

- *(core)* allow project-only scans even if project is unparseable

## [0.10.1](https://github.com/0xBoji/verifyOS/compare/v0.10.0...v0.10.1) - 2026-03-17

### Fixed

- *(lint)* resolve cargo fmt and clippy violations across workspace

## [0.10.0](https://github.com/0xBoji/verifyOS/compare/v0.9.0...v0.10.0) - 2026-03-17

### Added

- *(core)* implement recursive .app bundle discovery in zip extractor
- *(frontend)* add interactive severity filtering and bulk tree controls
- *(frontend)* refine Findings Explorer UI and update example report
- *(frontend)* add hierarchical Findings Explorer tree view
- *(rules)* implement Xcode 26 build mandate and enhance privacy sdk checks

### Fixed

- *(rules)* prevent test failures on empty manifests and downgrade Xcode mandate
- *(frontend)* ensure Findings Explorer and summaries only include failures
- *(backend)* fix ownership errors and restore optimized MUSL build

### Other

- add agent workflow for CI/CD pipelines
- add agent workflows for frontend, backend, and extension
- update README badges and resources with official links

## [0.9.0](https://github.com/0xBoji/verifyOS/compare/v0.8.2...v0.9.0) - 2026-03-16

### Fixed

- *(parser)* validate ipa path

## [0.8.2](https://github.com/0xBoji/verifyOS/compare/v0.8.1...v0.8.2) - 2026-03-16

### Added

- *(frontend)* add google analytics
- *(frontend)* wire google oauth callback
- *(frontend)* remove email login block
- *(frontend)* refine login layout
- *(frontend)* add email auth panel
- *(frontend)* sync header and scan widths
- *(frontend)* match quick scan width to header
- *(frontend)* round favicon asset
- *(frontend)* set header width to 75%
- *(frontend)* widen header to full width
- *(frontend)* widen header panel
- *(frontend)* use library icons in footer
- *(frontend)* adjust logo styling and footer icons
- *(frontend)* add logo and favicon
- *(frontend)* pin footer to bottom
- *(frontend)* polish footer links
- *(frontend)* move project links into footer
- *(frontend)* move agent bundle action to report
- *(frontend)* add project links
- *(backend)* accept zipped app bundles
- *(frontend)* show scanning status
- *(frontend)* colorize severity pills
- *(frontend)* add custom copy icon
- *(frontend)* colorize summary and copy state
- *(frontend)* remove hero stats
- *(frontend)* simplify hero actions
- *(frontend)* add project zip hero action
- *(frontend)* add advanced scan options
- *(frontend)* link docs and show example report
- *(cli)* support xcworkspace project context
- *(frontend)* expand quick scan width
- *(frontend)* move quick scan below hero
- *(frontend)* add report summary view
- *(frontend)* stabilize scan layout

### Fixed

- *(frontend)* align header width
- *(frontend)* let quick scan match header width
- *(frontend)* align header and scan widths
- *(frontend)* restore header width
- *(frontend)* round header logo image
- *(frontend)* reduce header height
- *(frontend)* use single header logo
- *(frontend)* use vscode icon from vsc pack
- *(frontend)* restore header logo shape

### Other

- *(frontend)* remove login note
- *(frontend)* drop login wiring
- *(frontend)* add ga setup
- *(git)* ignore terraform secrets
- *(frontend)* document backend url
- *(frontend)* install react-icons
- *(frontend)* expand supported artifact copy
- add project context examples
- ignore local lockfiles

## [0.8.1](https://github.com/0xBoji/verifyOS/compare/v0.8.0...v0.8.1) - 2026-03-15

### Added

- *(frontend)* enable bundle upload
- *(frontend)* wire upload to backend
- *(frontend)* switch footer to backend-first copy
- *(frontend)* add ios-friendly landing UI
- *(frontend)* scaffold nextjs app

### Fixed

- *(frontend)* show backend errors precisely

### Other

- add backend and frontend start steps
- update repository links
- *(architecture)* add backend frontend layout
- *(architecture)* init monorepo layout

## [0.8.0](https://github.com/0xBoji/verifyOS/compare/v0.7.3...v0.8.0) - 2026-03-15

### Added

- *(vscode)* refine action center ui and activity bar identity
- add cli support for xcode project auto-detection
- integrate xcode project awareness into core engine and context
- implement xcode project parsing infrastructure

### Fixed

- *(test)* synchronize editor contract tests with v0.1.18

### Other

- ignore RUSTSEC-2024-0436 (paste unmaintained) in deny.toml
- update agents.md with refined managed block identity

## [0.7.3](https://github.com/0xBoji/verifyOS/compare/v0.7.2...v0.7.3) - 2026-03-13

### Fixed

- *(vscode)* release 0.1.5

## [0.7.2](https://github.com/0xBoji/verifyOS/compare/v0.7.1...v0.7.2) - 2026-03-13

### Added

- *(vscode)* add action center sidebar

### Fixed

- *(vscode)* activate action center on view
- *(ci)* add safety guards for PR number in release-plz workflow

### Other

- *(vscode)* release 0.1.4
- *(vscode)* prepare 0.1.3 release
- *(vscode)* clarify background diagnostics

## [0.7.1](https://github.com/0xBoji/verifyOS/compare/v0.7.0...v0.7.1) - 2026-03-13

### Added

- *(lsp)* implement language server, consolidate profiles, and refactor reports

## [0.7.0](https://github.com/0xBoji/verifyOS/compare/v0.6.1...v0.7.0) - 2026-03-13

### Added

- *(vscode)* bundle platform binaries
- *(vscode)* polish marketplace presentation
- *(lsp)* publish bundle diagnostics
- *(vscode)* add thin editor extension
- *(lsp)* add voc language server command

### Fixed

- *(release)* export release pr summary env
- *(release)* stop renaming release-plz branches
- *(vscode)* clean extension packaging
- *(release)* tolerate existing smart branch names
- *(cli)* use scan profile in doctor command
- *(release)* target the current release pr branch

### Other

- *(vscode)* document bundled extension startup
- *(release)* enrich release pr metadata
- *(vscode)* document cli and extension
- *(ci)* add vscode extension packaging flow

## [0.6.1](https://github.com/0xBoji/verifyOS/compare/v0.6.0...v0.6.1) - 2026-03-13

### Added

- *(cli)* add analyze-size command
- *(handoff)* add bundle manifest output
- *(cli)* add handoff command

### Fixed

- *(size)* normalize report paths across platforms

### Other

- *(release)* improve release branch naming
- document analyze-size
- *(report)* snapshot handoff markdown
- *(ci)* support workflow repair plan overrides

## [0.6.0](https://github.com/0xBoji/verifyOS/compare/v0.5.0...v0.6.0) - 2026-03-13

### Added

- *(report)* cross-link handoff artifacts
- *(cli)* support explicit pr comment plan paths
- *(report)* link repair plans across handoff docs
- *(ci)* allow pr comments from repair plan
- *(agents)* surface repair plans in managed block
- *(config)* support doctor plan output defaults
- *(doctor)* write markdown repair plans
- *(doctor)* enrich repair plan context
- *(config)* extend doctor defaults

### Other

- *(ci)* snapshot plan-based pr comments
- *(doctor)* snapshot repair plan markdown
- *(ci)* publish repair plan artifact
- *(cli)* extract parse support helpers
- *(ci)* load workflow defaults from config
- *(cli)* extract agent io helpers
- *(ci)* add workflow contract coverage
- *(cli)* split command handlers
- *(cli)* centralize agent asset layout

## [0.5.0](https://github.com/0xBoji/verifyOS/compare/v0.4.0...v0.5.0) - 2026-03-13

### Added

- *(doctor)* add repair plans and freshness source
- *(doctor)* support selective repairs
- *(ci)* add pr comment renderer command
- *(doctor)* warn on stale agent assets
- *(config)* add init and doctor defaults
- *(doctor)* validate next steps script
- *(cli)* add doctor pr comment output
- enhance doctor/init commands and add curl installation to README
- *(cli)* add doctor pr brief output
- *(cli)* refresh doctor fixes from scans

### Other

- *(ci)* expose doctor repair and comment mode
- *(cli)* escape config paths for windows
- *(ci)* use pr comment artifact in workflow
- *(cli)* make doctor context check path-agnostic
- *(ci)* align release workflows with voc
- add commit discipline rules

## [0.4.0](https://github.com/0xBoji/verifyOS/compare/v0.3.1...v0.4.0) - 2026-03-12

### Added

- *(ci)* add voc analysis workflow
- *(cli)* add doctor and init output assets
- *(cli)* add init shell script
- *(cli)* add init command hints
- *(cli)* add init agent pack bundle
- *(cli)* support init baseline filtering
- *(cli)* add init from scan
- *(report)* add agent patch hints
- *(cli)* add agents bootstrap
- *(report)* add agent pack bundle formats
- *(report)* add agent pack output
- *(cli)* add rule detail lookup

### Fixed

- *(cli)* keep init agent outputs consistent

## [0.3.1](https://github.com/0xBoji/verifyOS/compare/v0.3.0...v0.3.1) - 2026-03-12

### Added

- *(cli)* add rule inventory

### Other

- *(crate)* improve ai-agent positioning
- Merge pull request #13 from 0xBoji/chore/release-plz2026-03-12T06-28-25Z
- *(crate)* add ai keywords

## [0.3.0](https://github.com/0xBoji/verifyOS/compare/v0.2.3...v0.3.0) - 2026-03-12

### Added

- *(cli)* add terminal banner
- *(report)* expose perf metadata
- *(cli)* add timing detail levels
- *(report)* add cache telemetry
- *(report)* highlight slowest rules
- *(report)* add timing summaries
- *(cli)* add config file support
- *(cli)* add rule selectors
- *(cli)* add fail-on threshold

### Other

- *(core)* cache bundle resources
- *(core)* cache bundle metadata
- *(core)* cache artifact scans

## [0.2.3](https://github.com/0xBoji/verifyOS/compare/v0.2.2...v0.2.3) - 2026-03-12

### Fixed

- *(ci)* avoid action download failures

### Other

- *(ci)* grant checks permission
- *(ci)* expand workflows

## [0.2.2](https://github.com/0xBoji/verifyOS/compare/v0.2.1...v0.2.2) - 2026-03-12

### Added

- *(privacy)* cross-check sdk usage
- *(extensions)* validate entitlements
- *(info-plist)* validate versioning
- *(bundle)* detect sensitive files
- *(ats)* flag overly broad exceptions

### Fixed

- *(ci)* resolve clippy warnings

### Other

- *(ci)* mention clippy
- center logo and title in readme
- resize readme icon
- add verifyOS icon

## [0.2.1](https://github.com/0xBoji/verifyOS/compare/v0.2.0...v0.2.1) - 2026-03-11

### Added

- *(info-plist)* audit device capabilities

### Other

- *(cli)* ship voc as the only binary
- *(cli)* switch examples to voc
- use voc in baseline example
- *(release)* fix release pr title

## [0.2.0](https://github.com/0xBoji/verifyOS/compare/v0.1.7...v0.2.0) - 2026-03-11

### Added

- *(info-plist)* audit LSApplicationQueriesSchemes
- *(signing)* check embedded team id consistency

## [0.1.7](https://github.com/0xBoji/verifyOS/compare/v0.1.6...v0.1.7) - 2026-03-11

### Added

- *(rules)* add ats privacy api export metadata

### Fixed

- *(ci)* use audit-check ignore input
- *(output)* align table examples

### Other

- *(ci)* fix audit and deny config
- *(ci)* align audit config
- *(repo)* drop skills gitlink
- *(ci)* commit lockfile and adjust deny
- add agent configs
- *(ci)* add security audit and cache

## [0.1.6](https://github.com/0xBoji/verifyOS/compare/v0.1.5...v0.1.6) - 2026-03-11

### Other

- fix output examples

## [0.1.1](https://github.com/0xBoji/verifyOS/compare/v0.1.0...v0.1.1) - 2026-03-11

### Other

- update architecture section in README to reflect single-crate move

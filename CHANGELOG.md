# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.2](https://github.com/0xBoji/verifyos-cli/compare/v0.6.1...v0.6.2) - 2026-03-13

### Fixed

- *(release)* target the current release pr branch

## [0.6.1](https://github.com/0xBoji/verifyos-cli/compare/v0.6.0...v0.6.1) - 2026-03-13

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

## [0.6.0](https://github.com/0xBoji/verifyos-cli/compare/v0.5.0...v0.6.0) - 2026-03-13

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

## [0.5.0](https://github.com/0xBoji/verifyos-cli/compare/v0.4.0...v0.5.0) - 2026-03-13

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

## [0.4.0](https://github.com/0xBoji/verifyos-cli/compare/v0.3.1...v0.4.0) - 2026-03-12

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

## [0.3.1](https://github.com/0xBoji/verifyos-cli/compare/v0.3.0...v0.3.1) - 2026-03-12

### Added

- *(cli)* add rule inventory

### Other

- *(crate)* improve ai-agent positioning
- Merge pull request #13 from 0xBoji/chore/release-plz2026-03-12T06-28-25Z
- *(crate)* add ai keywords

## [0.3.0](https://github.com/0xBoji/verifyos-cli/compare/v0.2.3...v0.3.0) - 2026-03-12

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

## [0.2.3](https://github.com/0xBoji/verifyos-cli/compare/v0.2.2...v0.2.3) - 2026-03-12

### Fixed

- *(ci)* avoid action download failures

### Other

- *(ci)* grant checks permission
- *(ci)* expand workflows

## [0.2.2](https://github.com/0xBoji/verifyos-cli/compare/v0.2.1...v0.2.2) - 2026-03-12

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

## [0.2.1](https://github.com/0xBoji/verifyos-cli/compare/v0.2.0...v0.2.1) - 2026-03-11

### Added

- *(info-plist)* audit device capabilities

### Other

- *(cli)* ship voc as the only binary
- *(cli)* switch examples to voc
- use voc in baseline example
- *(release)* fix release pr title

## [0.2.0](https://github.com/0xBoji/verifyos-cli/compare/v0.1.7...v0.2.0) - 2026-03-11

### Added

- *(info-plist)* audit LSApplicationQueriesSchemes
- *(signing)* check embedded team id consistency

## [0.1.7](https://github.com/0xBoji/verifyos-cli/compare/v0.1.6...v0.1.7) - 2026-03-11

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

## [0.1.6](https://github.com/0xBoji/verifyos-cli/compare/v0.1.5...v0.1.6) - 2026-03-11

### Other

- fix output examples

## [0.1.1](https://github.com/0xBoji/verifyos-cli/compare/v0.1.0...v0.1.1) - 2026-03-11

### Other

- update architecture section in README to reflect single-crate move

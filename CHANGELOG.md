<!--
This changelog was created using the `clu` binary
(https://github.com/MalteHerrmann/changelog-utils).
-->
# Changelog

## Unreleased

### Improvements

- (ci) [#105](https://github.com/MalteHerrmann/changelog-utils/pull/105) Use nextest as test runner.
- (all) [#103](https://github.com/MalteHerrmann/changelog-utils/pull/103) Refactor into submodules.
- (cli) [#97](https://github.com/MalteHerrmann/changelog-utils/pull/97) Refactor Git operations into dedicated module.
- (cli) [#94](https://github.com/MalteHerrmann/changelog-utils/pull/94) Fail PR creation when having problems parsing the LLM response.
- (cli) [#93](https://github.com/MalteHerrmann/changelog-utils/pull/93) Add check for empty diffs when creating PRs.
- (ci) [#92](https://github.com/MalteHerrmann/changelog-utils/pull/92) Update changelog lint action to v0.3.0 and adjust config.

### Features

- (cli) [#96](https://github.com/MalteHerrmann/changelog-utils/pull/96) Add functionality to check diff for entry on current PR.

## [v1.5.0](https://github.com/MalteHerrmann/changelog-utils/releases/tag/v1.5.0) - 2025-05-19

### Features

- (cli) [#87](https://github.com/MalteHerrmann/changelog-utils/pull/87) Add version flag to CLI application.
- (cli) [#91](https://github.com/MalteHerrmann/changelog-utils/pull/91) Add CLI command to get specific version release notes.

### Improvements

- (cli) [#83](https://github.com/MalteHerrmann/changelog-utils/pull/83) Improve JSON parsing from LLM responses with regex extraction.
- (all) [#90](https://github.com/MalteHerrmann/changelog-utils/pull/90) Refactor fixed contents export for better maintainability.

### Bug Fixes

- (cli) [#86](https://github.com/MalteHerrmann/changelog-utils/pull/86) Fix PR number insert when creating new PR.

## [v1.4.0](https://github.com/MalteHerrmann/changelog-utils/releases/tag/v1.4.0) - 2025-05-15

### Bug Fixes

- (docker) [#75](https://github.com/MalteHerrmann/changelog-utils/pull/75) Adjust required Rust version in Dockerfile.

### Improvements

- (config) [#81](https://github.com/MalteHerrmann/changelog-utils/pull/81) Refactors to have config adjustment methods.
- (config) [#80](https://github.com/MalteHerrmann/changelog-utils/pull/80) Convert to have deterministic order of change types in config.
- (cli) [#76](https://github.com/MalteHerrmann/changelog-utils/pull/76) Improve error handling when creating a PR and only the local changelog commit fails.

### Features

- (cli) [#78](https://github.com/MalteHerrmann/changelog-utils/pull/78) Add AI-assisted changelog generation to create PRs.

## [v1.3.0](https://github.com/MalteHerrmann/changelog-utils/releases/tag/v1.3.0) - 2025-03-30

### Improvements

- (cli) [#65](https://github.com/MalteHerrmann/changelog-utils/pull/65) Apply auto fixes to new entries.
- (ci) [#60](https://github.com/MalteHerrmann/changelog-utils/pull/60) Bump changelog linter to v0.2.1.

### Bug Fixes

- (cli) [#71](https://github.com/MalteHerrmann/changelog-utils/pull/71) Fix creating PRs functionality plus refactors and TODOs.
- (lint) [#61](https://github.com/MalteHerrmann/changelog-utils/pull/61) Fix version comparison.

### Features

- (cli) [#74](https://github.com/MalteHerrmann/changelog-utils/pull/74) Enable adding entries for previous PRs.
- (config) [#70](https://github.com/MalteHerrmann/changelog-utils/pull/70) Add changelog path to the configuration.
- (cli) [#68](https://github.com/MalteHerrmann/changelog-utils/pull/68) Commit and push changelog entry after adding.
- (cli) [#67](https://github.com/MalteHerrmann/changelog-utils/pull/67) Add option to push branch to remote.
- (cli) [#63](https://github.com/MalteHerrmann/changelog-utils/pull/63) Enable switching between release types when not specifying a version.

## [v1.2.0](https://github.com/MalteHerrmann/changelog-utils/releases/tag/v1.2.0) - 2024-08-03

### Features

- (cli) [#56](https://github.com/MalteHerrmann/changelog-utils/pull/56) Add CLI command to create a PR that conforms to `clu` configuration.
- (crud) [#54](https://github.com/MalteHerrmann/changelog-utils/pull/54) Add flag to auto-accept retrieved PR information.
- (lint) [#46](https://github.com/MalteHerrmann/changelog-utils/pull/46) Add support for linter escapes.

### Improvements

- (config) [#55](https://github.com/MalteHerrmann/changelog-utils/pull/55) Use change type abbreviations instead of patterns in config.
- (all) [#53](https://github.com/MalteHerrmann/changelog-utils/pull/53) Minor codebase improvements.
- (crud) [#48](https://github.com/MalteHerrmann/changelog-utils/pull/48) Use authenticated requests when checking open PRs.
- (config) [#51](https://github.com/MalteHerrmann/changelog-utils/pull/51) Get available configuration from existing changelog during initialization.

### Bug Fixes

- (crud) [#58](https://github.com/MalteHerrmann/changelog-utils/pull/58) Use abbreviations from config to derive change type from PR info.

## [v1.1.2](https://github.com/MalteHerrmann/changelog-utils/releases/tag/v1.1.2) - 2024-06-30

### Bug Fixes

- (lint) [#45](https://github.com/MalteHerrmann/changelog-utils/pull/45) Use correct line number.

## [v1.1.1](https://github.com/MalteHerrmann/changelog-utils/releases/tag/v1.1.1) - 2024-06-30

### Bug Fixes

- (all) [#44](https://github.com/MalteHerrmann/changelog-utils/pull/44) Update cargo lock file and check for this going forward.

## [v1.1.0](https://github.com/MalteHerrmann/changelog-utils/releases/tag/v1.1.0) - 2024-06-30

### Improvements

- (lint) [#42](https://github.com/MalteHerrmann/changelog-utils/pull/42) Add line numbers to lint output and add reviewdog to Dockerfile.
- (config) [#41](https://github.com/MalteHerrmann/changelog-utils/pull/41) Add Git origin to initialized configuration.
- (ci) [#36](https://github.com/MalteHerrmann/changelog-utils/pull/36) Add changelog linter as CI action.
- (crud) [#31](https://github.com/MalteHerrmann/changelog-utils/pull/31) Prefill input to add an entry with pull request information.
- (ci) [#30](https://github.com/MalteHerrmann/changelog-utils/pull/30) Add changelog diff check to CI actions.
- (all) [#28](https://github.com/MalteHerrmann/changelog-utils/pull/28) Update Cargo manifest with more information and updated version.
- (config) [#27](https://github.com/MalteHerrmann/changelog-utils/pull/27) Adjust configuration to have sorted entries.
- (all) [#26](https://github.com/MalteHerrmann/changelog-utils/pull/26) Add clippy and address linter warnings.

### Bug Fixes

- (crud) [#38](https://github.com/MalteHerrmann/changelog-utils/pull/38) Keep legacy contents during export.

## [v1.0.0](https://github.com/MalteHerrmann/changelog-utils/releases/tag/v1.0.0) - 2024-06-15

### Features

- (docker) [#18](https://github.com/MalteHerrmann/changelog-utils/pull/18) Add Docker configuration.
- (crud) [#16](https://github.com/MalteHerrmann/changelog-utils/pull/16) Add `release` command.
- (crud) [#12](https://github.com/MalteHerrmann/changelog-utils/pull/12) Implement adding new entries.
- (config) [#7](https://github.com/MalteHerrmann/changelog-utils/pull/7) Add `init` and `config` subcommands.
- (lint) [#5](https://github.com/MalteHerrmann/changelog-utils/pull/5) Implement fix flag for linter CLI.
- (lint) [#4](https://github.com/MalteHerrmann/changelog-utils/pull/4) Rewrite linter implementation in Rust.
- (lint) [#1](https://github.com/MalteHerrmann/changelog-utils/pull/1) Add initial implementation for linter in Python.

### Improvements

- (crud) [#17](https://github.com/MalteHerrmann/changelog-utils/pull/17) Get PR number from GitHub to prefill input.
- (crud) [#14](https://github.com/MalteHerrmann/changelog-utils/pull/14) Keep comments at head of file.
- (lint) [#6](https://github.com/MalteHerrmann/changelog-utils/pull/6) Remove Python implementation.

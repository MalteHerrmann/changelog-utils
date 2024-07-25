<!--
This changelog was created using the `clu` binary
(https://github.com/MalteHerrmann/changelog-utils).
-->
# Changelog

## Unreleased

### Features

- (crud) [#54](https://github.com/MalteHerrmann/changelog-utils/pull/54) Add flag to auto-accept retrieved PR information.
- (lint) [#46](https://github.com/MalteHerrmann/changelog-utils/pull/46) Add support for linter escapes.

### Improvements

- (all) [#53](https://github.com/MalteHerrmann/changelog-utils/pull/53) Minor codebase improvements.
- (crud) [#48](https://github.com/MalteHerrmann/changelog-utils/pull/48) Use authenticated requests when checking open PRs.
- (config) [#51](https://github.com/MalteHerrmann/changelog-utils/pull/51) Get available configuration from existing changelog during initialization.

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

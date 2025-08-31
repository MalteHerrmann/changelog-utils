<!--
Some comments at head of file...
-->
# Changelog

## Unreleased

### State Machine Breaking

- (p256-precompile) [#1922](https://github.com/evmos/evmos/pull/1922) Add `secp256r1` curve precompile.
- (distribution-precompile) [#1949](https://github.com/evmos/evmos/pull/1949) Add `ClaimRewards` custom transaction.
- (swagger) [#2218](https://github.com/evmos/evmos/pull/2218) Use correct version of proto dependencies to generate swagger.
- (go) [#1687](https://github.com/evmos/evmos/pull/1687) Bump Evmos version to v14.

### API Breaking

- (inflation) [#2015](https://github.com/evmos/evmos/pull/2015) Rename `inflation` module to `inflation/v1`.
- (ante) [#2078](https://github.com/evmos/evmos/pull/2078) Deprecate legacy EIP-712 ante handler.
- (evm) [#1851](https://github.com/evmos/evmos/pull/1851) Enable [EIP 3855](https://eips.ethereum.org/EIPS/eip-3855) (`PUSH0` opcode) during upgrade.

## [v15.0.0](https://github.com/evmos/evmos/releases/tag/v15.0.0) - 2023-10-31

### API Breaking

- (vesting) [#1862](https://github.com/evmos/evmos/pull/1862) Add Authorization Grants to the Vesting extension.
- (app) [#555](https://github.com/evmos/evmos/pull/555) `v4.0.0` upgrade logic.


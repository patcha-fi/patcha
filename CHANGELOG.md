# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[semantic versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `hook-runtime`: the lifecycle callback surface (`HookCallback`), the
  `HookContext` an adapter hands in, and the folded `HookResult` decision.
- `hook-runtime`: the `Hook` trait and a deterministic `Registry` (install
  order, first veto wins, last fee override wins).
- Six builtin hooks: dynamic-fee, time-lock, whitelist-gate, range-order,
  anti-mev, kyc-gate — each parameter-driven and unit-tested.
- `hook-adapters`: Orca Whirlpools and Raydium CLMM event mappers into a
  venue-neutral `HookContext`.
- `patcha-hook-executor`: Anchor program with registry, per-pool install, params
  validation, and a trigger boundary that reverts on a `before*` veto.
- `@patcha/hook-library`: cross-language metadata (slugs, params, cable colors).
- `@patcha/sdk`: PDA derivation, borsh params encoders, and a simulate client.
- Documentation: architecture, hook specification, and security notes.

### Notes

- The Anchor program builds against Anchor 0.31 and is mainnet-ready; the
  on-chain deploy is pending.

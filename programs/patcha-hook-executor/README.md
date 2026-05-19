# patcha-hook-executor

Anchor program that brings Uniswap-v4-style hooks to Solana CLMMs. Registers
hooks, installs them per pool with a borsh params blob, and triggers them at
lifecycle points where the matching builtin logic decides allow / deny /
fee-override.

The builtin hook logic is a 1:1 port of `crates/hook-runtime`, so a backend
simulation and an on-chain trigger return the same decision.

```bash
anchor build
anchor test
```

This crate is a separate workspace (idiomatic `anchor init` layout) and is
excluded from the root workspace so the core crates unit-test on a plain stable
toolchain without the Solana SBF toolchain installed. See
[docs/security.md](../../docs/security.md) for the account and arithmetic model.

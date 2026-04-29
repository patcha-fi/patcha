# patcha-hook-runtime

Toolchain-free Rust engine for Patcha hooks. Defines the Uniswap-v4-style
lifecycle callbacks, the `HookContext` an adapter hands in, the folded
`HookResult` decision, the `Hook` trait, and a deterministic `Registry`.

The crate has no Solana dependency, so it unit-tests in milliseconds and is
reused by the backend simulator. The on-chain Anchor program ports this surface
1:1 so a simulation and an on-chain trigger agree.

```bash
cargo test -p patcha-hook-runtime
```

See [docs/hooks-spec.md](../../docs/hooks-spec.md) for the callback model.

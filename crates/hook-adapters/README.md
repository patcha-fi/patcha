# patcha-hook-adapters

Maps Orca Whirlpools and Raydium CLMM lifecycle events into the venue-neutral
`HookContext` the builtin hooks evaluate. Each mapping is pure and toolchain-free
so it is reused verbatim by the backend simulator and the on-chain trigger path.

```bash
cargo test -p patcha-hook-adapters
```

The on-chain counterpart lives in
`programs/patcha-hook-executor/src/adapters`.

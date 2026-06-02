# Contributing

Thanks for your interest in Patcha. This is an early-stage framework and the
surface is still moving, so open an issue to discuss a change before a large PR.

## Development

The Rust core has no Solana dependency and tests on a plain stable toolchain:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The Anchor program is a separate workspace under `programs/` and needs the
Solana + Anchor toolchain:

```bash
cd programs/patcha-hook-executor
anchor build
```

The TypeScript packages typecheck with the project's pinned compiler:

```bash
cd packages/hook-library && npm install && npx tsc --noEmit
```

## Ground rules

A new builtin hook must keep parity across layers: add it to
`@patcha/hook-library` (metadata), `crates/hook-runtime` (engine + tests), and
`programs/patcha-hook-executor` (on-chain port), and keep the slug identical in
all three. The runtime arithmetic and the on-chain arithmetic must match so a
simulation equals an on-chain trigger.

## Commit style

Keep commits small and focused, with a short description of what changed and
why. Run the test suite before pushing.

![Patcha](assets/banner.png)

# Patcha

**Patch your liquidity.** Compose hooks like patching modules, then run them on
Solana CLMMs.

[![Website](https://img.shields.io/badge/site-patcha.fi-D4AF37?style=flat-square)](https://patcha.fi)
[![Docs](https://img.shields.io/badge/docs-architecture-3D2817?style=flat-square)](docs/architecture.md)
[![X](https://img.shields.io/badge/x-@patcha__fi-1A1A1A?style=flat-square)](https://x.com/patcha_fi)
[![CI](https://img.shields.io/github/actions/workflow/status/patcha-fi/patcha/ci.yml?branch=main&style=flat-square&label=build)](https://github.com/patcha-fi/patcha/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-7CB07A?style=flat-square)](LICENSE)
[![Stars](https://img.shields.io/github/stars/patcha-fi/patcha?style=flat-square&color=FFD93D)](https://github.com/patcha-fi/patcha/stargazers)
[![Rust](https://img.shields.io/badge/rust-stable-E63946?style=flat-square)](crates)
[![TypeScript](https://img.shields.io/badge/typescript-5.7-5BC0EB?style=flat-square)](packages)
[![Solana](https://img.shields.io/badge/solana-CLMM-9D6BFF?style=flat-square)](programs)
[![Anchor](https://img.shields.io/badge/anchor-0.31-D4AF37?style=flat-square)](programs/patcha-hook-executor)

`6/6 hooks · 41/41 tests · Orca + Raydium · Anchor 0.31 · mainnet-ready`

---

## The hook problem

Uniswap v4 made the AMM programmable: a pool can call out to a *hook* contract
at fixed points in its lifecycle — before and after initialize, add/remove
liquidity, swap, and donate — so liquidity providers can attach custom logic
(dynamic fees, gating, limit orders, MEV defenses) without forking the AMM.

Solana's concentrated-liquidity AMMs — Orca Whirlpools and Raydium CLMM — do not
expose that extension point. They do not call arbitrary programs on the swap
path, so the v4 pattern does not transfer directly.

Patcha brings the same idea to Solana from the **integration boundary**. A
router program or an off-chain keeper observes a pool lifecycle event, maps it
into a venue-neutral context, and calls the Patcha executor. Hooks decide
allow / deny / fee-override; a veto in a `before*` callback reverts the
transaction.

## How a hook evaluates

```rust
use patcha_hook_runtime::{builtin::DynamicFee, Hook, HookCallback, HookContext, Dex};

let fee_hook = DynamicFee::default();           // base 30 bps, max 100 bps
let mut ctx = HookContext::new(HookCallback::BeforeSwap, Dex::OrcaWhirlpool, [0u8; 32]);
ctx.amount_in = 500_000_000;                    // 0.5 SOL-equivalent

let decision = fee_hook.evaluate(&ctx);
assert!(decision.allow);
assert_eq!(decision.fee_override_bps, Some(65)); // interpolated toward the cap
```

The exact same arithmetic is ported into the Anchor program, so a backtest in
the backend and a trigger on-chain return the same fee.

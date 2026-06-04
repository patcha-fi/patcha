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

## Six built-in modules

Each module is a patch cable color in the designer; the slug is the stable
identifier shared by the runtime, the program, the SDK, and the CLI.

| Module | Slug | Category | Callbacks | What it does |
| --- | --- | --- | --- | --- |
| Dynamic Fee | `dynamic-fee` | fees | beforeSwap, afterSwap | Ramps the fee from a base to a cap using swap size as a volatility proxy |
| TimeLock | `time-lock` | timing | beforeAddLiquidity, beforeRemoveLiquidity | Blocks liquidity actions until an unlock timestamp |
| WhitelistGate | `whitelist-gate` | gating | beforeSwap, beforeAddLiquidity | Restricts actions to an allowlist (Merkle root committed on-chain) |
| RangeOrder | `range-order` | range | afterSwap | Fills a one-sided order as the tick crosses a target |
| AntiMEV | `anti-mev` | mev | beforeSwap, afterSwap | Vetoes swaps that move price past a per-block cap; credits prevented MEV |
| KYCGate | `kyc-gate` | kyc | beforeSwap, beforeAddLiquidity | Requires a verified-credential attestation before acting |

Folding rules when several hooks share a callback: hooks run in install order,
the first veto short-circuits, the last fee override wins, and credited MEV
accumulates.

## Architecture

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': {'primaryColor': '#1A1A1A', 'primaryTextColor': '#F0EAD6', 'primaryBorderColor': '#D4AF37', 'lineColor': '#D4AF37', 'secondaryColor': '#3D2817', 'tertiaryColor': '#1A1A1A', 'fontFamily': 'JetBrains Mono, monospace'}}}%%
flowchart LR
    designer["web designer\npatchbay UI"] --> sdk["@patcha/sdk"]
    cli["patcha CLI"] --> sdk
    sdk --> lib["@patcha/hook-library"]
    sdk --> backend["backend\n/hook/simulate"]
    backend --> rt["hook-runtime\n(Rust engine)"]
    lib --> rt
    rt -. "1:1 port" .-> prog["patcha-hook-executor\nAnchor program"]
    sdk --> prog
    prog --> orca["Orca Whirlpools"]
    prog --> ray["Raydium CLMM"]
```

The decision logic lives once as toolchain-free Rust (`crates/hook-runtime`) and
is ported 1:1 into the Anchor program. The metadata (slugs, params, cable
colors) lives once in `@patcha/hook-library` and is read by every other layer,
so the web designer, SDK, CLI, runtime, and program never drift.

## Repository layout

```
patcha/
├── crates/
│   ├── hook-runtime/     toolchain-free engine — callbacks, registry, 6 builtins
│   └── hook-adapters/    Orca / Raydium lifecycle event -> HookContext mappers
├── programs/
│   └── patcha-hook-executor/   Anchor program — register / install / trigger
├── packages/
│   ├── hook-library/     cross-language metadata (slugs, params, cables)
│   └── ts-sdk/           PDA derivation + params encoders + simulate client
└── docs/                 architecture · hooks-spec · security
```

## Quickstart

Build and test the Rust core (no Solana toolchain required for the engine):

```bash
git clone https://github.com/patcha-fi/patcha
cd patcha
cargo test --workspace
```

Typecheck the TypeScript packages:

```bash
cd packages/hook-library
npm install
npx tsc --noEmit
```

Encode a hook's params and derive its PDAs with the SDK:

```ts
import { PatchaClient, encode } from '@patcha/sdk';

const client = new PatchaClient();
const blob = encode.dynamicFee({
  baseFeeBps: 30,
  maxFeeBps: 100,
  pivotAmount: 1_000_000_000n,
});

const installation = client.installationPda(pool, 'dynamic-fee');
const result = await client.simulateHook('dynamic-fee', { baseFeeBps: 30 }, pool, 'orca', 30);
```

## Hook callbacks

Patcha keeps the full Uniswap v4 `IHooks` surface — ten callbacks in five
before/after pairs — mapped onto the Solana CLMM lifecycle. The discriminants
are a stable wire ABI (reused in event encoding), so they are never reordered.
The complete mapping is in [docs/hooks-spec.md](docs/hooks-spec.md).

## Status

- Engine and adapters: complete, unit-tested (`cargo test --workspace`).
- Anchor program: builds against Anchor 0.31; mainnet-ready, deploy pending.
- The program has not had a third-party audit — see
  [docs/security.md](docs/security.md) before custodying funds.

## References

- Adams, H. et al. *Uniswap v4 Core* whitepaper, Uniswap Labs, 2024 —
  <https://app.uniswap.org/whitepaper-v4.pdf>
- Uniswap v4 hooks documentation —
  <https://docs.uniswap.org/contracts/v4/concepts/hooks>
- Orca Whirlpools documentation —
  <https://dev.orca.so>
- Raydium CLMM documentation —
  <https://docs.raydium.io>
- Anchor framework (PDA + account model) —
  <https://www.anchor-lang.com>

## License

MIT — see [LICENSE](LICENSE).

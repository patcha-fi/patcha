# Hooks specification

Patcha maps Uniswap v4's ten hook callbacks onto the Solana CLMM lifecycle
(Orca Whirlpools, Raydium CLMM, and Meteora DLMM). A hook is a small module
installed against a pool; the on-chain executor invokes it at the matching
point in the pool's lifecycle.

## DEX venues

The executor stamps each lifecycle event with a venue tag, so the same six
builtin hooks compose with any of the three venues.

| Venue | DEX tag | Mainnet program |
|---|---|---|
| Orca Whirlpools | `0` | `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc` |
| Raydium CLMM | `1` | `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK` |
| Meteora DLMM | `2` | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` |

Meteora DLMM uses discrete bins; the LbPair `activeId` is the analogue of a
CLMM tick and is what `DynamicFee`, `RangeOrder`, and `AntiMEV` consume.

## Uniswap v4 callbacks → Solana CLMM trigger

| Uniswap v4 callback     | Patcha CLMM trigger          |
| ----------------------- | ---------------------------- |
| `beforeInitialize`      | before pool/position init    |
| `afterInitialize`       | after pool/position init     |
| `beforeAddLiquidity`    | before `increaseLiquidity`   |
| `afterAddLiquidity`     | after `increaseLiquidity`    |
| `beforeRemoveLiquidity` | before `decreaseLiquidity`   |
| `afterRemoveLiquidity`  | after `decreaseLiquidity`    |
| `beforeSwap`            | before swap CPI              |
| `afterSwap`             | after swap CPI               |
| `beforeDonate`          | before fee donation          |
| `afterDonate`           | after fee donation           |

## Builtin hooks

| Hook            | Category | Reacts on                                                  |
| --------------- | -------- | ---------------------------------------------------------- |
| Dynamic Fee     | fees     | `beforeSwap`, `afterSwap`                                  |
| TimeLock        | timing   | `beforeAddLiquidity`, `beforeRemoveLiquidity`              |
| WhitelistGate   | gating   | `beforeSwap`, `beforeAddLiquidity`                         |
| RangeOrder      | range    | `afterSwap`                                                |
| AntiMEV         | mev      | `beforeSwap`, `afterSwap`                                  |
| KYCGate         | kyc      | `beforeSwap`, `beforeAddLiquidity`                         |
| PriceImpactCap  | gating   | `beforeSwap`                                               |
| JIT-Defense     | mev      | `beforeSwap`, `beforeAddLiquidity`, `beforeRemoveLiquidity`|

`PriceImpactCap` rejects swaps whose estimated price impact exceeds a per-swap
cap (LP-owned slippage ceiling). `JIT-Defense` rejects same-block
add-swap-remove patterns from one wallet (just-in-time LP attack defense). Both
ride the same `install_hook_burning` path the other six do; see
[token-economics.md](token-economics.md) for the holder-tier burn table.

The eight builtin hooks and their parameter schemas are shared across the web
designer, SDK, CLI, and VS Code extension from a single hook-library package, so
all surfaces agree on slugs, parameters, and on-chain encoding.

Reference: Uniswap v4 hooks whitepaper (Uniswap Labs, 2024).

# Architecture

Patcha is a monorepo: a Next.js web app, a FastAPI backend (hook simulation +
DEX adapters), a Rust hook runtime, an Anchor hook-executor program, a
TypeScript SDK, a CLI, and a VS Code extension.

Requests flow from the web app to the backend through same-origin `/api/*`
route handlers (no cross-origin calls). The Anchor program enforces installed
hooks via PDA-derived accounts on **Orca Whirlpools, Raydium CLMM, and
Meteora DLMM** pools.

## Components

| Package | Purpose |
|---|---|
| `apps/web` | Next.js Hook Designer, marketplace, devtools, docs |
| `service/` | FastAPI backend (hook simulate, DAS proxy, marketplace listing) |
| `packages/anchor-program` | Anchor 0.31 `patcha_hook_executor` program (mainnet `EPcW7e8…rNRa`) |
| `packages/hook-runtime` | Pure-Rust hook eval, shared off-chain (backtest) and on-chain |
| `packages/hook-library` | Six standard hooks: schemas + metadata |
| `packages/sdk-ts` | TypeScript SDK — `PatchaClient` for register/install/trigger |
| `packages/cli` | `patcha-cli` — `init` / `create` / `list` / `simulate` / `install` / `deploy` |
| `packages/vscode-extension` | VS Code Designer webview |
| `packages/whirlpools-adapter` | Orca Whirlpools wrapper (`@orca-so/whirlpools-sdk`) |
| `packages/raydium-adapter` | Raydium CLMM wrapper (`@raydium-io/raydium-sdk-v2`) |
| `packages/meteora-adapter` | Meteora DLMM wrapper (`@meteora-ag/dlmm`) |

## DEX adapter boundary

Each CLMM venue exposes its own pool schema and quote function. Patcha
normalizes them via a Rust adapter (`src/adapters/{orca,raydium,meteora}.rs`)
that maps a venue-specific lifecycle event into a neutral `TriggerCtx`. The
eight standard hooks consume that context unchanged across venues.

| Venue | Adapter | DEX tag | Mainnet program |
|---|---|---|---|
| Orca Whirlpools | `orca.rs` | `0` | `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc` |
| Raydium CLMM | `raydium.rs` | `1` | `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK` |
| Meteora DLMM | `meteora.rs` | `2` | `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` |

Meteora DLMM uses discrete bins instead of ticks; the LbPair `activeId` is
the analogue of a CLMM tick and is what the adapter maps to `TriggerCtx.tick`
for `DynamicFee`, `RangeOrder`, and `AntiMEV` to consume identically.

## On-chain executor

Program ID: `EPcW7e8RxBNPpQK2XKoKG9maWH6QvmU3ejxifoU5rNRa` (Solana mainnet).

Instructions (Anchor 0.31):

- `initialize_registry` — one-time setup of the hook registry PDA
- `register_hook(slug, kind, code_hash)` — register a hook in the marketplace
- `install_hook(pool, slug, dex, params_blob)` — install on a pool; idempotent
  (re-install with the same `(pool, slug)` updates params without bumping the
  install counter)
- `update_params(params_blob)` — installer-only param refresh
- `trigger_hook(callback, ...)` — emit `HookTriggered` event
- `uninstall_hook` — deactivate without closing PDAs

All eight standard hooks are registered on mainnet
(`dynamic-fee`, `time-lock`, `whitelist-gate`, `range-order`, `anti-mev`,
`kyc-gate`, `price-impact-cap`, `jit-defense`). The executor accepts all
three DEX tags (`0=orca`, `1=raydium`, `2=meteora`).

## Requests flow

```
browser → /market | /designer | /devtools (Next.js)
       └─ /api/hook/list      → builtin hook library (server-side)
       └─ /api/hook/simulate  → service/ FastAPI → byte-identical Rust fee fn
       └─ /api/das/asset/…    → Helius DAS proxy

CLI → service/ /hook/simulate   (offchain backtest)
   → mainnet RPC → patcha_hook_executor  (onchain install / register / trigger)

SDK → mainnet RPC → patcha_hook_executor
```

The off-chain backtest and the on-chain executor compute the fee from a
**byte-identical** function: the number printed by `patcha simulate` is the
exact number a live `HookTriggered` event would emit.

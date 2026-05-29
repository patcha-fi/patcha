# Architecture

Patcha is a hook framework for Solana concentrated-liquidity market makers
(CLMMs). It mirrors the Uniswap v4 hook model — custom logic patched onto the
lifecycle of a pool — and adapts it to Orca Whirlpools and Raydium CLMM, which
do not call arbitrary programs on their swap path.

## Layers

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': {'primaryColor': '#1A1A1A', 'primaryTextColor': '#F0EAD6', 'primaryBorderColor': '#D4AF37', 'lineColor': '#D4AF37', 'secondaryColor': '#3D2817', 'tertiaryColor': '#1A1A1A', 'fontFamily': 'JetBrains Mono, monospace'}}}%%
flowchart TB
    subgraph clients["clients"]
        web["web designer\n(modular patchbay)"]
        cli["patcha CLI"]
        sdk["@patcha/sdk"]
    end

    subgraph core["shared core"]
        lib["@patcha/hook-library\nslugs · params · cables"]
        rt["hook-runtime (Rust)\nallow / deny / fee"]
    end

    subgraph chain["Solana"]
        prog["patcha-hook-executor\nAnchor program"]
        orca["Orca Whirlpools"]
        ray["Raydium CLMM"]
    end

    backend["backend simulate\n/hook/simulate"]

    web --> sdk
    cli --> sdk
    sdk --> lib
    sdk --> backend
    backend --> rt
    sdk --> prog
    lib --> rt
    rt -. "1:1 port" .-> prog
    prog --> orca
    prog --> ray
```

## Why a trigger boundary

Uniswap v4 encodes hook permissions in the hook contract address and the pool
manager calls the hook directly. Solana CLMMs do not expose that extension
point, so Patcha is driven at the **integration boundary**: a router program or
an off-chain keeper observes a pool lifecycle event (swap, add/remove
liquidity), maps it into a venue-neutral `HookContext`, and calls
`trigger_hook`. A veto in a `before*` callback reverts the transaction.

## Decision parity

The same hook arithmetic exists twice:

- `crates/hook-runtime` — toolchain-free std Rust, used by the backend
  simulator and by unit tests. No Solana dependency, so it tests in
  milliseconds.
- `programs/patcha-hook-executor/src/hooks` — a 1:1 port that decodes params
  from a borsh blob in the `Params` PDA.

Because the slugs, parameter layouts, and arithmetic are identical, a backtest
in the backend and a trigger on-chain produce the same allow/deny/fee decision.

## PDA layout

| PDA | Seeds | Holds |
| --- | --- | --- |
| registry | `["hook_registry"]` | admin, hook count, install count |
| hook meta | `["hook", slug]` | kind, code hash, author, audited flag |
| installation | `["installation", pool, slug]` | pool, installer, dex, active flag |
| params | `["params", installation]` | borsh-encoded hook params |

## Repository layout

```
patcha/
├── crates/
│   ├── hook-runtime/     toolchain-free engine (callbacks, registry, builtins)
│   └── hook-adapters/    Orca / Raydium event -> HookContext mappers
├── programs/
│   └── patcha-hook-executor/   Anchor program (registry + install + trigger)
├── packages/
│   ├── hook-library/     cross-language metadata (slugs, params, cables)
│   └── ts-sdk/           PDA derivation + params encoders + simulate client
└── docs/
```

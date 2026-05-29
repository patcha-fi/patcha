# Hook specification

## Callback model

Patcha keeps the Uniswap v4 `IHooks` surface — ten lifecycle callbacks in five
before/after pairs — and maps each onto the Solana CLMM lifecycle. The
discriminants are a stable wire ABI reused in event encoding, so they are never
reordered.

| Tag | Callback | Phase | CLMM lifecycle point |
| --- | --- | --- | --- |
| 0 | `beforeInitialize` | before | pool creation |
| 1 | `afterInitialize` | after | pool creation |
| 2 | `beforeAddLiquidity` | before | LP deposit |
| 3 | `afterAddLiquidity` | after | LP deposit |
| 4 | `beforeRemoveLiquidity` | before | LP withdraw |
| 5 | `afterRemoveLiquidity` | after | LP withdraw |
| 6 | `beforeSwap` | before | swap |
| 7 | `afterSwap` | after | swap |
| 8 | `beforeDonate` | before | donate / fee top-up |
| 9 | `afterDonate` | after | donate / fee top-up |

A `before*` callback may veto the action (the CLMM reverts) or override the swap
fee. An `after*` callback is observational — it records state but cannot revert.

## Decision type

Each hook returns a decision the engine folds into one result:

- `allow` — `false` aborts the lifecycle action.
- `fee_override_bps` — optional new swap fee in basis points (last write wins).
- `mev_bps_saved` — basis points of MEV the hook reports prevented (telemetry).
- `reason` — surfaced in logs and the simulate endpoint.

When several hooks subscribe to the same callback they run in install order: the
first veto short-circuits, the last fee override wins, and `mev_bps_saved`
accumulates.

## Builtin parameters

| Hook | Slug | Callbacks | Key params |
| --- | --- | --- | --- |
| Dynamic Fee | `dynamic-fee` | beforeSwap, afterSwap | `baseFeeBps`, `maxFeeBps`, `pivotAmount` |
| TimeLock | `time-lock` | beforeAddLiquidity, beforeRemoveLiquidity | `unlockTs` |
| WhitelistGate | `whitelist-gate` | beforeSwap, beforeAddLiquidity | `merkleRoot`, `allowed` |
| RangeOrder | `range-order` | afterSwap | `tickTarget`, `direction` |
| AntiMEV | `anti-mev` | beforeSwap, afterSwap | `maxPriceMoveBps`, `referenceDepth` |
| KYCGate | `kyc-gate` | beforeSwap, beforeAddLiquidity | `attestationAuthority`, `attested` |

## Dynamic-fee interpolation

`DynamicFee` ramps the fee linearly from `baseFeeBps` to `maxFeeBps`, using the
swap size relative to `pivotAmount` as a cheap volatility proxy:

```
fee = base + (max - base) * min(amount_in, pivot) / pivot
```

The same expression is implemented in `crates/hook-runtime` and in the Anchor
port, so a simulated fee equals the on-chain fee for any input.

## Community hooks

Slugs outside the six builtins are registered with `kind = community` and start
unaudited. The executor treats unknown slugs as a no-op (always allow); their
logic is supplied by the integrator's own program or keeper.

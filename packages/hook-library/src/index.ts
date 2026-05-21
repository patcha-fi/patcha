/**
 * @patcha/hook-library
 *
 * Metadata for the 6 standard (builtin) Patcha hooks. The on-chain logic lives
 * in the Anchor program (`programs/patcha-hook-executor`); this package is the
 * single source of truth for slugs, categories, cable colors, and parameter
 * schemas consumed by the web designer, SDK, CLI, and marketplace.
 */

export type HookCategory =
  | 'fees'
  | 'timing'
  | 'gating'
  | 'range'
  | 'mev'
  | 'kyc';

/** A single configurable hook parameter (input schema). */
export interface HookParam {
  key: string;
  label: string;
  type: 'number' | 'boolean' | 'address' | 'duration' | 'percent';
  default?: number | boolean | string;
  description: string;
}

export interface HookDefinition {
  slug: string;
  /** Short modular-synth panel label (e.g. "dyn-fee"). */
  moduleLabel: string;
  displayName: string;
  category: HookCategory;
  /** Cable color token (modular-synth patch cable). */
  cableColor: string;
  description: string;
  /** Uniswap v4 callbacks this hook reacts to. */
  callbacks: string[];
  params: HookParam[];
}

export const HOOK_LIBRARY: readonly HookDefinition[] = [
  {
    slug: 'dynamic-fee',
    moduleLabel: 'dyn-fee',
    displayName: 'Dynamic Fee',
    category: 'fees',
    cableColor: 'cable-red',
    description:
      'Adjust the pool fee in real time based on volatility or volume bands.',
    callbacks: ['beforeSwap', 'afterSwap'],
    params: [
      {
        key: 'baseFeeBps',
        label: 'base fee (bps)',
        type: 'number',
        default: 30,
        description: 'Baseline fee applied at low volatility.',
      },
      {
        key: 'maxFeeBps',
        label: 'max fee (bps)',
        type: 'number',
        default: 100,
        description: 'Upper bound when volatility spikes.',
      },
    ],
  },
  {
    slug: 'time-lock',
    moduleLabel: 'tlock',
    displayName: 'TimeLock',
    category: 'timing',
    cableColor: 'cable-yellow',
    description:
      'Gate liquidity actions until a timestamp or after a cooldown window.',
    callbacks: ['beforeAddLiquidity', 'beforeRemoveLiquidity'],
    params: [
      {
        key: 'unlockTs',
        label: 'unlock at',
        type: 'duration',
        description: 'Unix timestamp before which the action is blocked.',
      },
    ],
  },
  {
    slug: 'whitelist-gate',
    moduleLabel: 'gate',
    displayName: 'WhitelistGate',
    category: 'gating',
    cableColor: 'cable-green',
    description: 'Restrict swaps or LP actions to an allowlist of addresses.',
    callbacks: ['beforeSwap', 'beforeAddLiquidity'],
    params: [
      {
        key: 'merkleRoot',
        label: 'allowlist root',
        type: 'address',
        description: 'Merkle root of the permitted address set.',
      },
    ],
  },
  {
    slug: 'range-order',
    moduleLabel: 'range-ord',
    displayName: 'RangeOrder',
    category: 'range',
    cableColor: 'cable-cyan',
    description:
      'Execute one-sided range orders that convert as price crosses a tick.',
    callbacks: ['afterSwap'],
    params: [
      {
        key: 'tickTarget',
        label: 'target tick',
        type: 'number',
        description: 'Tick at which the range order completes.',
      },
    ],
  },
  {
    slug: 'anti-mev',
    moduleLabel: 'anti-mev',
    displayName: 'AntiMEV',
    category: 'mev',
    cableColor: 'cable-purple',
    description:
      'Dampen sandwich/MEV extraction via per-block caps and dynamic slippage.',
    callbacks: ['beforeSwap', 'afterSwap'],
    params: [
      {
        key: 'maxPriceMoveBps',
        label: 'max move (bps)',
        type: 'number',
        default: 50,
        description: 'Reject swaps that move price beyond this per block.',
      },
    ],
  },
  {
    slug: 'kyc-gate',
    moduleLabel: 'kyc',
    displayName: 'KYCGate',
    category: 'kyc',
    cableColor: 'cable-grey',
    description:
      'Require a verified-credential attestation before permitting actions.',
    callbacks: ['beforeSwap', 'beforeAddLiquidity'],
    params: [
      {
        key: 'attestationAuthority',
        label: 'attestor',
        type: 'address',
        description: 'Authority whose attestation is required.',
      },
    ],
  },
] as const;

/** Count of builtin hooks — surfaced in the header stat line. */
export const HOOK_COUNT = HOOK_LIBRARY.length;

/** Builtin slugs, in marketplace order. */
export const BUILTIN_SLUGS = HOOK_LIBRARY.map((h) => h.slug);

export function getHook(slug: string): HookDefinition | undefined {
  return HOOK_LIBRARY.find((h) => h.slug === slug);
}

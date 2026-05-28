/**
 * @patcha/sdk — TypeScript developer SDK for the Patcha hook framework.
 *
 * Derives the program PDAs, encodes each builtin hook's on-chain params blob
 * (byte-compatible with the borsh structs in
 * `programs/patcha-hook-executor/src/hooks/*`), and talks to the Patcha backend
 * (`/hook/simulate`, `/hook/list`). PDA seeds, callback tags, and dex tags are
 * shared with the on-chain program and the Rust runtime so all three layers
 * agree.
 */

import { PublicKey } from '@solana/web3.js';
import {
  HOOK_LIBRARY,
  HOOK_COUNT,
  BUILTIN_SLUGS,
  getHook,
  type HookDefinition,
} from '@patcha/hook-library';

export { HOOK_LIBRARY, HOOK_COUNT, BUILTIN_SLUGS, getHook };
export type { HookDefinition };

export const SDK_VERSION = '0.1.0' as const;

/** Default program id (placeholder until a mainnet deploy injects the real id). */
export const DEFAULT_PROGRAM_ID = new PublicKey(
  'EPcW7e8RxBNPpQK2XKoKG9maWH6QvmU3ejxifoU5rNRa'
);

/** Hook kind tags (HookMeta.kind on-chain). */
export const HOOK_KIND_BUILTIN = 0;
export const HOOK_KIND_COMMUNITY = 1;

/** DEX venue tags (orca=0, raydium=1) — matches the runtime + program. */
export const DEX = { orca: 0, raydium: 1 } as const;
export type DexName = keyof typeof DEX;

/** Lifecycle callback tags (mirror HookCallback discriminants in the runtime). */
export const CALLBACK = {
  beforeInitialize: 0,
  afterInitialize: 1,
  beforeAddLiquidity: 2,
  afterAddLiquidity: 3,
  beforeRemoveLiquidity: 4,
  afterRemoveLiquidity: 5,
  beforeSwap: 6,
  afterSwap: 7,
  beforeDonate: 8,
  afterDonate: 9,
} as const;
export type CallbackName = keyof typeof CALLBACK;

/** Range-order fill direction tags (mirror DIR_ABOVE / DIR_BELOW on-chain). */
export const DIRECTION = { above: 0, below: 1 } as const;
export type DirectionName = keyof typeof DIRECTION;

const enc = (s: string) => new TextEncoder().encode(s);

// --- Little-endian borsh writers -------------------------------------------

function u32le(value: number): Uint8Array {
  const b = new Uint8Array(4);
  new DataView(b.buffer).setUint32(0, value, true);
  return b;
}

function i32le(value: number): Uint8Array {
  const b = new Uint8Array(4);
  new DataView(b.buffer).setInt32(0, value, true);
  return b;
}

function u64le(value: bigint): Uint8Array {
  const b = new Uint8Array(8);
  new DataView(b.buffer).setBigUint64(0, value, true);
  return b;
}

function i64le(value: bigint): Uint8Array {
  const b = new Uint8Array(8);
  new DataView(b.buffer).setBigInt64(0, value, true);
  return b;
}

function concat(parts: Uint8Array[]): Uint8Array {
  const total = parts.reduce((n, p) => n + p.length, 0);
  const out = new Uint8Array(total);
  let off = 0;
  for (const p of parts) {
    out.set(p, off);
    off += p.length;
  }
  return out;
}

function vecOfKeys(keys: Uint8Array[]): Uint8Array {
  return concat([u32le(keys.length), ...keys]);
}

/** Params encoders — each returns a borsh blob matching the on-chain struct. */
export const encode = {
  dynamicFee(p: { baseFeeBps: number; maxFeeBps: number; pivotAmount: bigint }) {
    return concat([
      u32le(p.baseFeeBps),
      u32le(p.maxFeeBps),
      u64le(p.pivotAmount),
    ]);
  },
  timeLock(p: { unlockTs: bigint }) {
    return i64le(p.unlockTs);
  },
  antiMev(p: { maxPriceMoveBps: number; referenceDepth: bigint }) {
    return concat([u32le(p.maxPriceMoveBps), u64le(p.referenceDepth)]);
  },
  rangeOrder(p: { tickTarget: number; direction: DirectionName }) {
    return concat([i32le(p.tickTarget), Uint8Array.of(DIRECTION[p.direction])]);
  },
  whitelistGate(p: { merkleRoot: Uint8Array; allowed: Uint8Array[] }) {
    return concat([p.merkleRoot, vecOfKeys(p.allowed)]);
  },
  kycGate(p: { attestationAuthority: Uint8Array; attested: Uint8Array[] }) {
    return concat([p.attestationAuthority, vecOfKeys(p.attested)]);
  },
};

export interface SimulateResult {
  feeAprBps: number;
  vsBaselineBps: number;
  metrics: Record<string, number>;
}

export interface PatchaClientOptions {
  programId?: PublicKey;
  apiUrl?: string;
}

const DEFAULT_API_URL = 'https://patcha-service-production.up.railway.app';

/**
 * Client for the Patcha hook executor program PDAs + backend.
 *
 * ```ts
 * const client = new PatchaClient();
 * const blob = encode.dynamicFee({ baseFeeBps: 30, maxFeeBps: 100, pivotAmount: 1_000_000_000n });
 * const installation = client.installationPda(pool, 'dynamic-fee');
 * const result = await client.simulateHook('dynamic-fee', { baseFeeBps: 30 }, pool.toBase58(), 'orca', 30);
 * ```
 */
export class PatchaClient {
  readonly programId: PublicKey;
  readonly apiUrl: string;

  constructor(opts: PatchaClientOptions = {}) {
    this.programId = opts.programId ?? DEFAULT_PROGRAM_ID;
    this.apiUrl = (opts.apiUrl ?? DEFAULT_API_URL).replace(/\/$/, '');
  }

  // --- PDA derivation -------------------------------------------------------

  registryPda(): PublicKey {
    return PublicKey.findProgramAddressSync(
      [enc('hook_registry')],
      this.programId
    )[0];
  }

  hookMetaPda(slug: string): PublicKey {
    return PublicKey.findProgramAddressSync(
      [enc('hook'), enc(slug)],
      this.programId
    )[0];
  }

  installationPda(pool: PublicKey, slug: string): PublicKey {
    return PublicKey.findProgramAddressSync(
      [enc('installation'), pool.toBuffer(), enc(slug)],
      this.programId
    )[0];
  }

  paramsPda(installation: PublicKey): PublicKey {
    return PublicKey.findProgramAddressSync(
      [enc('params'), installation.toBuffer()],
      this.programId
    )[0];
  }

  // --- Backend (simulate / list) -------------------------------------------

  /** Run a backtest against the Patcha backend `/hook/simulate`. */
  async simulateHook(
    slug: string,
    params: Record<string, number | string | boolean>,
    pool: string,
    dex: DexName,
    periodDays: number
  ): Promise<SimulateResult> {
    const res = await fetch(`${this.apiUrl}/hook/simulate`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ hook: { slug, params }, pool, dex, periodDays }),
    });
    if (!res.ok) {
      throw new Error(`simulateHook: ${res.status} ${await res.text()}`);
    }
    return (await res.json()) as SimulateResult;
  }

  /** Fetch the marketplace hook list from the backend `/hook/list`. */
  async getHookList(): Promise<unknown> {
    const res = await fetch(`${this.apiUrl}/hook/list`);
    if (!res.ok) {
      throw new Error(`getHookList: ${res.status} ${await res.text()}`);
    }
    return res.json();
  }
}

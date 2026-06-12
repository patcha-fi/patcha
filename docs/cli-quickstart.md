# patcha-cli quickstart

`patcha-cli` is the command-line entry point to the Patcha hook executor on
Solana. It scaffolds hook projects, backtests builtins against live pools,
installs hooks on chain (with the holder-tier PATCHA burn), and surfaces
the burn-tier table.

## Prerequisites

The CLI is a thin client on top of Solana and your own keypair, so you need
the Solana toolchain even though `patcha-cli` itself is one npm install.

1. **Node 20 or higher.** `node --version` should print `v20.x` or above.
   Get it from [nodejs.org](https://nodejs.org) or with `nvm install 20`.
2. **Solana CLI installed and on your `PATH`.** Install with
   `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"` and
   verify with `solana --version`. Without it, the CLI can still scaffold
   and simulate, but cannot sign or send transactions.
3. **A Solana keypair file.** `solana-keygen new` creates one at
   `~/.config/solana/id.json` by default. The CLI reads from that path
   unless you pass `--keypair <path>`.
4. **Some SOL in that wallet for tx fees.** A few hundredths of a SOL is
   enough for many installs (~0.000005 SOL per signature). Top up via
   `solana airdrop 1` on devnet, or a transfer on mainnet.
5. **(Burn path only)** **PATCHA in that same wallet, ≥ the burn amount
   for your tier.** See [token-economics.md](token-economics.md) for the
   exact tier table. `patcha install --no-burn` skips this requirement
   and uses the legacy non-burning instruction.

The burn never auto-fires from holding alone — it only runs when *you* call
`patcha install ...` and sign the transaction yourself, and it burns only
from that same signer wallet. PATCHA sitting in a wallet is never touched
otherwise.

## Install

```bash
npm i -g patcha-cli@latest
patcha --version
```

Or, if you'd rather pin to the tarball mirrored from the site:

```bash
npm i -g https://patcha.fi/downloads/patcha-cli-latest.tgz
```

## First commands

```bash
# 1. browse the hook marketplace
patcha list

# 2. quote your wallet's burn tier (read-only, no signature)
patcha tiers --wallet <your-wallet-pubkey>

# 3. backtest a builtin hook against a real pool (no wallet needed)
patcha simulate dynamic-fee --pool <pool-addr> --dex orca

# 4. install a hook on a real pool (mainnet tx; signs from your keypair)
patcha install dynamic-fee --pool <pool-addr> --dex orca

# 5. scaffold your own hook project
patcha init my-hook && cd my-hook && patcha simulate hook.toml
```

## Global flags

Every command accepts:

- `--cluster mainnet|devnet|testnet|localnet` (default `mainnet`)
- `--rpc <url>` (overrides `--cluster` and `PATCHA_RPC`)
- `--wallet <path>` (default `~/.config/solana/id.json`)

`patcha install` additionally accepts:

- `--pool <addr>` (required)
- `--dex orca|raydium|meteora` (default `orca`)
- `--keypair <path>` (overrides `--wallet` for this command only)
- `--program <id>` (override the executor program id; rarely needed)
- `--no-burn` (use the legacy `install_hook` ix with no PATCHA burn)

`patcha tiers` accepts:

- `--wallet <addr|path>` — pubkey to quote (or a keypair file)
- `--rpc <url>` / `--cluster <cluster>` to read from a non-default RPC

## Common errors

- `insufficient PATCHA for tier N: need X, wallet holds Y, short Z` — your
  signer wallet doesn't have enough PATCHA to cover the tier's burn. Buy
  the shortfall, or pass `--no-burn` to use the legacy free path.
- `keypair not found at <path>` — `solana-keygen new` to create one, or
  pass `--keypair <path>` / set `ANCHOR_WALLET`.
- `Insufficient funds for rent / transaction simulation failed` — add SOL
  to the signer wallet.
- `failed to get recent blockhash` on patcha-cli before 0.3.x — the
  bundled fetch shim was incompatible with Undici; upgrade to
  `patcha-cli@0.3.1` or later.

## Where to go next

- The full burn-tier spec lives in [token-economics.md](token-economics.md).
- Hook callback semantics are in [hooks-spec.md](hooks-spec.md).
- Anchor program PDAs and the security model are in
  [architecture.md](architecture.md) and [security.md](security.md).

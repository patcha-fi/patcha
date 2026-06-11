# PATCHA token economics

The `install_hook_burning` instruction on the mainnet executor burns a
tier-derived amount of PATCHA from the installer's wallet on every call. The
burn rate depends on the installer's share of total `PATCHA_MINT` supply —
holding is the discount.

## Burn-rate table

| Tier | Holder share of supply | Burn per `install_hook_burning` |
| ---- | ---------------------- | ------------------------------- |
| T1   | ≥ 2.0%                 | 100 PATCHA                      |
| T2   | ≥ 1.0%                 | 300 PATCHA                      |
| T3   | ≥ 0.5%                 | 1,000 PATCHA                    |
| T4   | ≥ 0.1%                 | 5,000 PATCHA                    |
| T5   | < 0.1%                 | 50,000 PATCHA                   |

The table is a pure function over public on-chain state — the same
`calculate_burn(supply, holder_balance)` runs in the executor program, in the
SDK, and in the CLI's `patcha tiers` command, so on-chain and off-chain
results agree by construction.

## Why this shape

- **T1 (≥ 2.0%)** — whale rate. ~$520 of PATCHA committed today buys
  effectively-free installs (100 PATCHA ≈ $0.003 per call). The discount
  rewards real conviction, not speculative passers-through.
- **T2 / T3 / T4** — graded holder rates. As your share of supply drops, the
  per-install burn rises. Each tier was sized so a tier-N holder can run
  ~200–5,000 installs before their balance falls into the next bracket.
- **T5 (< 0.1%)** — non-holder rate. The wallet has to acquire ≥ 50,000
  PATCHA to install one hook with the burn path. After the install the
  balance is roughly zero again, which forces a buy → install → buy loop.
  That loop is the supply-side pressure of the design.

## Hard invariants

- **`PATCHA_MINT` is a constant** in the executor (`AKSYuSqinmiYt5pSQxsfb4m97seTP37s32TSs9Lpump`).
  The burn instruction cannot be retargeted at a different mint.
- **The mint's mint authority is `null`.** No party — including the dev — can
  issue new PATCHA. Burn is therefore a permanent supply reduction, not a
  cosmetic transfer.
- **The mint's freeze authority is `null`.** No party can freeze a holder's
  token account; the discount applies to whoever holds.
- **No admin switch.** `INSTALL_BURN_AMOUNT` is not a const that an admin can
  flip; the burn amount is derived per call by `calculate_burn` and the only
  knob — the tier table itself — is a const. Changing it requires a public
  program upgrade transaction that any observer can see on Solscan.
- **Insufficient balance reverts** with `PatchaError::InsufficientPatcha`
  (`#6011`). The CLI surfaces the exact shortfall (`need / have / short`)
  and points the user at the mint address or the `--no-burn` escape hatch.
- **Legacy `install_hook` is preserved.** Older integrations can keep calling
  the non-burning instruction via `patcha install --no-burn` so the upgrade
  is non-breaking.

## Verifying your tier

```bash
npm i -g patcha-cli@latest
patcha tiers --wallet <your-wallet-pubkey>
```

The command reads the live `PATCHA_MINT` supply and the wallet's Token-2022
ATA balance, then prints the wallet's tier, the burn amount per install,
and how many installs remain before the next tier change.

## Verifying the burn on chain

Every `install_hook_burning` call emits a `PatchaBurned` event carrying the
burn amount, the tier, the holder balance before, and the supply before.
You can confirm a specific transaction on Solscan and reconcile the supply
delta on chain:

```bash
spl-token display AKSYuSqinmiYt5pSQxsfb4m97seTP37s32TSs9Lpump
solana account AKSYuSqinmiYt5pSQxsfb4m97seTP37s32TSs9Lpump --output json
```

`mintAuthority: null` and `freezeAuthority: null` are visible in both
outputs, so the "no new mints / no freezes" claim is independently
verifiable.

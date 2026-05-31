# Security notes

Patcha is pre-audit software. These notes describe the controls already in the
program and the assumptions an integrator should hold.

## Account model

- Every PDA is derived from fixed seeds and a stored bump; the bump is recorded
  at `init` and re-checked on later access so a caller cannot substitute an
  off-curve address.
- `Params` carries a back-reference to its `Installation` and is validated with
  Anchor's `has_one` so a params account cannot be paired with the wrong
  installation.
- `update_params` and `uninstall_hook` require the original `installer`
  (`has_one = installer`); a third party cannot retune or remove a hook.

## Arithmetic

- All counters (`hook_count`, `install_count`, `trigger_count`) use
  `checked_add` and return `Overflow` rather than wrapping.
- The release profile sets `overflow-checks = true` so arithmetic is checked in
  the built artifact, not only in debug.
- Fee and price-impact math runs in `u128` before narrowing, so the `u64`
  inputs cannot overflow the intermediate product.

## Params validation

- A params blob is schema-checked at install and update time
  (`validate_params`); a malformed non-empty blob is rejected before it is ever
  stored, so a bad install can never be silently coerced.
- An empty blob decodes to the hook's documented defaults.
- The blob is bounded by `MAX_PARAMS_LEN` (256 bytes).

## Trigger boundary

- `trigger_hook` checks the callback tag is in range and the installation is
  active before dispatching.
- A veto in a `before*` callback returns `HookVetoed`, which reverts the calling
  transaction; `after*` callbacks cannot revert.
- The caller is the integration boundary (router/keeper). Gating hooks in this
  reference path treat the caller as the acting sender; a production integration
  should pass the true end-user as the gated identity.

## Known limitations

- WhitelistGate and KYCGate ship with a materialized address set for parity with
  the simulator. A production deployment should verify a Merkle proof against
  the committed root (WhitelistGate) or an attestation account issued by the
  authority (KYCGate).
- The program has not been through a third-party audit. Do not use it to custody
  funds without one.

## Reporting

Report suspected vulnerabilities privately to the maintainers via the contact on
the project website rather than opening a public issue.

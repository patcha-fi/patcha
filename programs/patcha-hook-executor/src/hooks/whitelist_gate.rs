//! WhitelistGate — restrict swaps / LP actions to an allowlist.
//! Category: gating. Callbacks: beforeSwap, beforeAddLiquidity. Cable: green.
//!
//! Port of `crates/hook-runtime/src/builtin/whitelist_gate.rs`. The runtime
//! models membership with an explicit address set (empty == open gate); the
//! on-chain params carry both the committed Merkle root (informational) and the
//! materialized allowlist so the decision is identical to a simulation. A
//! production deployment can additionally verify a Merkle proof against `root`.

use super::{reason, HookOutcome, TriggerCtx};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct WhitelistGateParams {
    /// Committed allowlist Merkle root (32 bytes); informational on-chain.
    pub merkle_root: [u8; 32],
    /// Materialized allowlist (bounded by the params blob size).
    pub allowed: Vec<[u8; 32]>,
}

pub fn is_allowed(p: &WhitelistGateParams, who: &[u8; 32]) -> bool {
    // Empty allowlist == open gate (default-permissive), matching the runtime.
    p.allowed.is_empty() || p.allowed.iter().any(|a| a == who)
}

pub fn evaluate(ctx: &TriggerCtx, blob: &[u8]) -> Result<HookOutcome> {
    let p = super::decode_params::<WhitelistGateParams>(blob)?;
    if is_allowed(&p, &ctx.sender.to_bytes()) {
        Ok(HookOutcome::allow())
    } else {
        Ok(HookOutcome::deny(reason::WHITELIST_BLOCKED))
    }
}

//! DynamicFee — retune the pool fee in real time from a swap-size volatility
//! proxy. Category: fees. Callbacks: beforeSwap, afterSwap. Cable: red.
//!
//! Port of `crates/hook-runtime/src/builtin/dynamic_fee.rs`. The `fee_for`
//! interpolation is byte-identical so a backend simulation and an on-chain
//! trigger agree on the basis-point fee.

use super::{reason, HookOutcome, TriggerCtx, CB_BEFORE_SWAP};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct DynamicFeeParams {
    pub base_fee_bps: u32,
    pub max_fee_bps: u32,
    /// Swap size (base lamports) at which the fee reaches the cap.
    pub pivot_amount: u64,
}

impl Default for DynamicFeeParams {
    fn default() -> Self {
        // Matches hook-library defaults: base 30 bps, max 100 bps.
        DynamicFeeParams {
            base_fee_bps: 30,
            max_fee_bps: 100,
            pivot_amount: 1_000_000_000, // 1 SOL-equivalent in lamports
        }
    }
}

/// Fee for a given input size, clamped to `[base, max]`.
pub fn fee_for(p: &DynamicFeeParams, amount_in: u64) -> u32 {
    if p.pivot_amount == 0 || p.max_fee_bps <= p.base_fee_bps {
        return p.base_fee_bps;
    }
    let span = p.max_fee_bps - p.base_fee_bps;
    let ratio_num = amount_in.min(p.pivot_amount) as u128;
    let extra = (span as u128 * ratio_num) / p.pivot_amount as u128;
    p.base_fee_bps + extra as u32
}

pub fn evaluate(ctx: &TriggerCtx, blob: &[u8]) -> Result<HookOutcome> {
    let p = super::decode_params::<DynamicFeeParams>(blob)?;
    if ctx.callback == CB_BEFORE_SWAP {
        Ok(HookOutcome::allow()
            .with_fee(fee_for(&p, ctx.amount_in))
            .with_reason(reason::DYNAMIC_FEE_APPLIED))
    } else {
        // afterSwap is accounting-only here.
        Ok(HookOutcome::allow())
    }
}

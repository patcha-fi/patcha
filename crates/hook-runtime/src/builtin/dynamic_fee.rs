//! DynamicFee — retune the pool fee in real time from a volatility proxy.
//!
//! Category: fees. Callbacks: beforeSwap, afterSwap. Cable: red.
//!
//! On `beforeSwap` the hook interpolates between `base_fee_bps` and
//! `max_fee_bps` using the size of the incoming swap as a cheap volatility
//! proxy (larger prints relative to `pivot_amount` push the fee toward the cap).
//! This mirrors Uniswap v4's canonical dynamic-fee hook adapted to a CLMM where
//! the adapter supplies `amount_in`.

use crate::builtin::HookCategory;
use crate::registry::Hook;
use crate::{HookCallback, HookContext, HookResult};

#[derive(Clone, Copy, Debug)]
pub struct DynamicFee {
    pub base_fee_bps: u32,
    pub max_fee_bps: u32,
    /// Swap size (base lamports) at which the fee reaches the cap.
    pub pivot_amount: u64,
}

impl Default for DynamicFee {
    fn default() -> Self {
        // Matches hook-library defaults: base 30 bps, max 100 bps.
        DynamicFee {
            base_fee_bps: 30,
            max_fee_bps: 100,
            pivot_amount: 1_000_000_000, // 1 SOL-equivalent in lamports
        }
    }
}

impl DynamicFee {
    pub const SLUG: &'static str = "dynamic-fee";
    pub const CATEGORY: HookCategory = HookCategory::Fees;

    /// Fee for a given input size, clamped to `[base, max]`.
    pub fn fee_for(&self, amount_in: u64) -> u32 {
        if self.pivot_amount == 0 || self.max_fee_bps <= self.base_fee_bps {
            return self.base_fee_bps;
        }
        let span = self.max_fee_bps - self.base_fee_bps;
        // Linear ramp, saturating at the pivot.
        let ratio_num = amount_in.min(self.pivot_amount) as u128;
        let extra = (span as u128 * ratio_num) / self.pivot_amount as u128;
        self.base_fee_bps + extra as u32
    }
}

impl Hook for DynamicFee {
    fn slug(&self) -> &'static str {
        Self::SLUG
    }

    fn callbacks(&self) -> &'static [HookCallback] {
        &[HookCallback::BeforeSwap, HookCallback::AfterSwap]
    }

    fn evaluate(&self, ctx: &HookContext) -> HookResult {
        match ctx.callback {
            HookCallback::BeforeSwap => HookResult::allow().with_fee(self.fee_for(ctx.amount_in)),
            // afterSwap is accounting-only here.
            _ => HookResult::allow(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Dex;

    fn swap_ctx(amount: u64) -> HookContext {
        let mut c = HookContext::new(HookCallback::BeforeSwap, Dex::OrcaWhirlpool, [7u8; 32]);
        c.amount_in = amount;
        c
    }

    #[test]
    fn small_swap_is_base_fee() {
        let h = DynamicFee::default();
        assert_eq!(h.fee_for(0), 30);
    }

    #[test]
    fn large_swap_is_capped() {
        let h = DynamicFee::default();
        assert_eq!(h.fee_for(u64::MAX), 100);
        assert_eq!(h.fee_for(h.pivot_amount), 100);
    }

    #[test]
    fn midpoint_interpolates() {
        let h = DynamicFee::default();
        // Halfway to the pivot ~ midpoint of [30, 100].
        assert_eq!(h.fee_for(h.pivot_amount / 2), 65);
    }

    #[test]
    fn evaluate_emits_fee_override() {
        let h = DynamicFee::default();
        let r = h.evaluate(&swap_ctx(h.pivot_amount));
        assert!(r.allow);
        assert_eq!(r.fee_override_bps, Some(100));
    }
}

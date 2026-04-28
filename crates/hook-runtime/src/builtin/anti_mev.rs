//! AntiMEV — dampen sandwich/MEV extraction with a per-swap price-move cap.
//!
//! Category: mev. Callbacks: beforeSwap, afterSwap. Cable: purple.
//!
//! On `beforeSwap` the hook estimates the price impact of the incoming swap
//! (basis points of `amount_in` against a reference depth) and vetoes swaps that
//! would move price beyond `max_price_move_bps` in a single block — the move a
//! sandwich attacker needs. Allowed swaps are credited with the MEV the cap
//! prevents, surfaced by the simulate endpoint as `mev_bps_saved`.

use crate::builtin::HookCategory;
use crate::registry::Hook;
use crate::{HookCallback, HookContext, HookResult};

#[derive(Clone, Copy, Debug)]
pub struct AntiMev {
    /// Reject swaps whose estimated per-block price move exceeds this.
    pub max_price_move_bps: u32,
    /// Reference liquidity depth (base lamports) used to estimate impact.
    pub reference_depth: u64,
}

impl Default for AntiMev {
    fn default() -> Self {
        AntiMev {
            max_price_move_bps: 50,
            reference_depth: 1_000_000_000, // 1 SOL-equivalent
        }
    }
}

impl AntiMev {
    pub const SLUG: &'static str = "anti-mev";
    pub const CATEGORY: HookCategory = HookCategory::Mev;

    /// Estimated price impact of a swap in basis points (linear depth model).
    pub fn estimated_move_bps(&self, amount_in: u64) -> u32 {
        if self.reference_depth == 0 {
            return 0;
        }
        let bps = (amount_in as u128 * 10_000) / self.reference_depth as u128;
        bps.min(u32::MAX as u128) as u32
    }
}

impl Hook for AntiMev {
    fn slug(&self) -> &'static str {
        Self::SLUG
    }

    fn callbacks(&self) -> &'static [HookCallback] {
        &[HookCallback::BeforeSwap, HookCallback::AfterSwap]
    }

    fn evaluate(&self, ctx: &HookContext) -> HookResult {
        if ctx.callback != HookCallback::BeforeSwap {
            return HookResult::allow();
        }
        let move_bps = self.estimated_move_bps(ctx.amount_in);
        if move_bps > self.max_price_move_bps {
            HookResult::deny("anti-mev: swap exceeds per-block price-move cap")
        } else {
            // Credit the headroom we enforced as MEV the attacker couldn't take.
            HookResult::allow().with_mev_saved(self.max_price_move_bps - move_bps)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Dex;

    fn swap(amount: u64) -> HookContext {
        let mut c = HookContext::new(HookCallback::BeforeSwap, Dex::RaydiumClmm, [5u8; 32]);
        c.amount_in = amount;
        c
    }

    #[test]
    fn small_swap_allowed_and_credited() {
        let h = AntiMev::default();
        // 0.1% of depth -> 10 bps move, under the 50 bps cap.
        let r = h.evaluate(&swap(h.reference_depth / 1000));
        assert!(r.allow);
        assert_eq!(r.mev_bps_saved, 40);
    }

    #[test]
    fn large_swap_vetoed() {
        let h = AntiMev::default();
        // 1% of depth -> 100 bps move, over the 50 bps cap.
        let r = h.evaluate(&swap(h.reference_depth / 100));
        assert!(!r.allow);
    }

    #[test]
    fn after_swap_is_noop() {
        let h = AntiMev::default();
        let mut c = swap(u64::MAX);
        c.callback = HookCallback::AfterSwap;
        assert!(h.evaluate(&c).allow);
    }
}

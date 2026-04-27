//! WhitelistGate — restrict swaps / LP actions to an allowlist.
//!
//! Category: gating. Callbacks: beforeSwap, beforeAddLiquidity. Cable: green.
//!
//! On-chain the allowlist is committed as a Merkle root (`merkle_root` param)
//! and callers supply a proof. The toolchain-free runtime models membership
//! directly with an explicit address set so the engine can be simulated; the
//! Anchor port swaps this for Merkle-proof verification against the root.

use crate::builtin::HookCategory;
use crate::registry::Hook;
use crate::{HookCallback, HookContext, HookResult};

#[derive(Clone, Debug, Default)]
pub struct WhitelistGate {
    /// Committed allowlist Merkle root (32 bytes); informational in the runtime.
    pub merkle_root: [u8; 32],
    /// Materialised allowlist used for simulation.
    pub allowed: Vec<[u8; 32]>,
}

impl WhitelistGate {
    pub const SLUG: &'static str = "whitelist-gate";
    pub const CATEGORY: HookCategory = HookCategory::Gating;

    pub fn with_allowed(allowed: Vec<[u8; 32]>) -> Self {
        WhitelistGate {
            merkle_root: [0u8; 32],
            allowed,
        }
    }

    pub fn is_allowed(&self, who: &[u8; 32]) -> bool {
        // Empty allowlist == open gate (default-permissive scaffold state).
        self.allowed.is_empty() || self.allowed.iter().any(|a| a == who)
    }
}

impl Hook for WhitelistGate {
    fn slug(&self) -> &'static str {
        Self::SLUG
    }

    fn callbacks(&self) -> &'static [HookCallback] {
        &[HookCallback::BeforeSwap, HookCallback::BeforeAddLiquidity]
    }

    fn evaluate(&self, ctx: &HookContext) -> HookResult {
        if self.is_allowed(&ctx.sender) {
            HookResult::allow()
        } else {
            HookResult::deny("whitelist-gate: sender not on allowlist")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Dex;

    fn swap_from(sender: [u8; 32]) -> HookContext {
        let mut c = HookContext::new(HookCallback::BeforeSwap, Dex::OrcaWhirlpool, [9u8; 32]);
        c.sender = sender;
        c
    }

    #[test]
    fn open_when_allowlist_empty() {
        let h = WhitelistGate::default();
        assert!(h.evaluate(&swap_from([1u8; 32])).allow);
    }

    #[test]
    fn blocks_non_member() {
        let h = WhitelistGate::with_allowed(vec![[1u8; 32]]);
        assert!(h.evaluate(&swap_from([1u8; 32])).allow);
        assert!(!h.evaluate(&swap_from([2u8; 32])).allow);
    }
}

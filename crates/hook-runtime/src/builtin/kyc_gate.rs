//! KYCGate — require a verified-credential attestation before acting.
//!
//! Category: kyc. Callbacks: beforeSwap, beforeAddLiquidity. Cable: grey.
//!
//! A swap or LP action is permitted only if the sender holds a valid attestation
//! from `attestation_authority`. On-chain this checks an attestation account /
//! SAS credential; the toolchain-free runtime models it with an explicit set of
//! attested senders so the engine can be simulated.

use crate::builtin::HookCategory;
use crate::registry::Hook;
use crate::{HookCallback, HookContext, HookResult};

#[derive(Clone, Debug, Default)]
pub struct KycGate {
    /// Authority whose attestation is required (32-byte pubkey).
    pub attestation_authority: [u8; 32],
    /// Senders holding a valid attestation (simulation model).
    pub attested: Vec<[u8; 32]>,
}

impl KycGate {
    pub const SLUG: &'static str = "kyc-gate";
    pub const CATEGORY: HookCategory = HookCategory::Kyc;

    pub fn with_attested(authority: [u8; 32], attested: Vec<[u8; 32]>) -> Self {
        KycGate {
            attestation_authority: authority,
            attested,
        }
    }

    pub fn is_attested(&self, who: &[u8; 32]) -> bool {
        self.attested.iter().any(|a| a == who)
    }
}

impl Hook for KycGate {
    fn slug(&self) -> &'static str {
        Self::SLUG
    }

    fn callbacks(&self) -> &'static [HookCallback] {
        &[HookCallback::BeforeSwap, HookCallback::BeforeAddLiquidity]
    }

    fn evaluate(&self, ctx: &HookContext) -> HookResult {
        if self.is_attested(&ctx.sender) {
            HookResult::allow()
        } else {
            HookResult::deny("kyc-gate: sender lacks a valid attestation")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Dex;

    fn swap_from(sender: [u8; 32]) -> HookContext {
        let mut c = HookContext::new(HookCallback::BeforeSwap, Dex::OrcaWhirlpool, [6u8; 32]);
        c.sender = sender;
        c
    }

    #[test]
    fn attested_passes_unattested_blocked() {
        let h = KycGate::with_attested([0u8; 32], vec![[1u8; 32]]);
        assert!(h.evaluate(&swap_from([1u8; 32])).allow);
        assert!(!h.evaluate(&swap_from([2u8; 32])).allow);
    }

    #[test]
    fn default_blocks_everyone() {
        // No attestations registered -> closed by default (kyc is deny-by-default).
        let h = KycGate::default();
        assert!(!h.evaluate(&swap_from([1u8; 32])).allow);
    }
}

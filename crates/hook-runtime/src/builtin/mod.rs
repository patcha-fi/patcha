//! The six standard Patcha hooks.
//!
//! Slugs, categories and callback subscriptions are kept byte-identical to
//! `packages/hook-library/src/index.ts` (the cross-language source of truth) so
//! the web designer, SDK, CLI and this runtime never drift. Each module holds a
//! small, real, parameter-driven implementation — not a placeholder — so the
//! engine can be exercised end-to-end before the Anchor port.

pub mod anti_mev;
pub mod dynamic_fee;
pub mod kyc_gate;
pub mod range_order;
pub mod time_lock;
pub mod whitelist_gate;

pub use anti_mev::AntiMev;
pub use dynamic_fee::DynamicFee;
pub use kyc_gate::KycGate;
pub use range_order::RangeOrder;
pub use time_lock::TimeLock;
pub use whitelist_gate::WhitelistGate;

use crate::registry::Hook;

/// Hook category, mirrors `HookCategory` in hook-library.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HookCategory {
    Fees,
    Timing,
    Gating,
    Range,
    Mev,
    Kyc,
}

impl HookCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            HookCategory::Fees => "fees",
            HookCategory::Timing => "timing",
            HookCategory::Gating => "gating",
            HookCategory::Range => "range",
            HookCategory::Mev => "mev",
            HookCategory::Kyc => "kyc",
        }
    }
}

/// The six builtin slugs, in marketplace order. Single source for the count
/// surfaced in the header stat line ("6/6 hooks").
pub const BUILTIN_SLUGS: [&str; 6] = [
    "dynamic-fee",
    "time-lock",
    "whitelist-gate",
    "range-order",
    "anti-mev",
    "kyc-gate",
];

/// Instantiate all six builtin hooks with their default parameters.
pub fn all() -> Vec<Box<dyn Hook>> {
    vec![
        Box::new(DynamicFee::default()),
        Box::new(TimeLock::default()),
        Box::new(WhitelistGate::default()),
        Box::new(RangeOrder::default()),
        Box::new(AntiMev::default()),
        Box::new(KycGate::default()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HookCallback;

    #[test]
    fn all_returns_six_in_order() {
        let hooks = all();
        assert_eq!(hooks.len(), 6);
        let slugs: Vec<&str> = hooks.iter().map(|h| h.slug()).collect();
        assert_eq!(slugs, BUILTIN_SLUGS);
    }

    #[test]
    fn every_builtin_subscribes_to_at_least_one_callback() {
        for h in all() {
            assert!(!h.callbacks().is_empty(), "{} has no callbacks", h.slug());
        }
    }

    #[test]
    fn builtin_callbacks_match_hook_library() {
        // Mirror of packages/hook-library/src/index.ts `callbacks` arrays.
        let hooks = all();
        let find = |slug: &str| hooks.iter().find(|h| h.slug() == slug).unwrap();

        assert_eq!(
            find("dynamic-fee").callbacks(),
            &[HookCallback::BeforeSwap, HookCallback::AfterSwap]
        );
        assert_eq!(
            find("time-lock").callbacks(),
            &[
                HookCallback::BeforeAddLiquidity,
                HookCallback::BeforeRemoveLiquidity
            ]
        );
        assert_eq!(
            find("whitelist-gate").callbacks(),
            &[HookCallback::BeforeSwap, HookCallback::BeforeAddLiquidity]
        );
        assert_eq!(find("range-order").callbacks(), &[HookCallback::AfterSwap]);
        assert_eq!(
            find("anti-mev").callbacks(),
            &[HookCallback::BeforeSwap, HookCallback::AfterSwap]
        );
        assert_eq!(
            find("kyc-gate").callbacks(),
            &[HookCallback::BeforeSwap, HookCallback::BeforeAddLiquidity]
        );
    }

    #[test]
    fn category_labels() {
        assert_eq!(HookCategory::Fees.as_str(), "fees");
        assert_eq!(HookCategory::Mev.as_str(), "mev");
        assert_eq!(HookCategory::Kyc.as_str(), "kyc");
    }
}

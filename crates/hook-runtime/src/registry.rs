//! Hook trait + registry.
//!
//! A [`Hook`] is a unit of custom CLMM logic that subscribes to a subset of the
//! lifecycle [`HookCallback`]s. The [`Registry`] holds the installed hooks and,
//! given a fired callback + [`HookContext`], runs each subscribed hook and folds
//! their [`HookResult`]s into a single decision the CLMM acts on.

use crate::{HookCallback, HookContext, HookResult};

/// A unit of custom liquidity logic patched onto a CLMM lifecycle point.
///
/// Implementations are pure: `evaluate` reads the context and returns a
/// decision. Side effects (CPI, account writes) belong to the Anchor program
/// that wraps the runtime.
pub trait Hook {
    /// Stable slug — identical to `packages/hook-library/src/index.ts`.
    fn slug(&self) -> &'static str;

    /// Lifecycle callbacks this hook reacts to.
    fn callbacks(&self) -> &'static [HookCallback];

    /// Whether this hook should run for a given callback.
    fn subscribes_to(&self, cb: HookCallback) -> bool {
        self.callbacks().contains(&cb)
    }

    /// Evaluate the hook against a lifecycle context.
    fn evaluate(&self, ctx: &HookContext) -> HookResult;
}

/// An ordered set of installed hooks. Resolution is deterministic: hooks run in
/// install order, the first veto wins, and fee overrides apply last-write.
#[derive(Default)]
pub struct Registry {
    hooks: Vec<Box<dyn Hook>>,
}

impl Registry {
    /// Empty registry.
    pub fn new() -> Self {
        Registry { hooks: Vec::new() }
    }

    /// Registry preloaded with the six standard builtin hooks.
    pub fn with_builtins() -> Self {
        let mut r = Registry::new();
        for h in crate::builtin::all() {
            r.register(h);
        }
        r
    }

    /// Install a hook.
    pub fn register(&mut self, hook: Box<dyn Hook>) {
        self.hooks.push(hook);
    }

    /// Number of installed hooks.
    pub fn len(&self) -> usize {
        self.hooks.len()
    }

    /// `true` if no hooks are installed.
    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }

    /// Look up an installed hook by slug.
    pub fn get(&self, slug: &str) -> Option<&dyn Hook> {
        self.hooks
            .iter()
            .find(|h| h.slug() == slug)
            .map(|h| h.as_ref())
    }

    /// Run every hook subscribed to `ctx.callback` and fold the results.
    ///
    /// Folding rules:
    ///   - any veto (`allow == false`) short-circuits and is returned verbatim;
    ///   - the last fee override wins;
    ///   - `mev_bps_saved` accumulates across hooks.
    pub fn evaluate(&self, ctx: &HookContext) -> HookResult {
        let mut folded = HookResult::allow();
        for hook in self.hooks.iter().filter(|h| h.subscribes_to(ctx.callback)) {
            let r = hook.evaluate(ctx);
            if !r.allow {
                return r;
            }
            if r.fee_override_bps.is_some() {
                folded.fee_override_bps = r.fee_override_bps;
            }
            folded.mev_bps_saved = folded.mev_bps_saved.saturating_add(r.mev_bps_saved);
        }
        folded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dex, HookResult};

    struct AlwaysDeny;
    impl Hook for AlwaysDeny {
        fn slug(&self) -> &'static str {
            "always-deny"
        }
        fn callbacks(&self) -> &'static [HookCallback] {
            &[HookCallback::BeforeSwap]
        }
        fn evaluate(&self, _ctx: &HookContext) -> HookResult {
            HookResult::deny("test veto")
        }
    }

    fn ctx(cb: HookCallback) -> HookContext {
        HookContext::new(cb, Dex::OrcaWhirlpool, [1u8; 32])
    }

    #[test]
    fn empty_registry_allows() {
        let r = Registry::new();
        assert!(r.is_empty());
        assert!(r.evaluate(&ctx(HookCallback::BeforeSwap)).allow);
    }

    #[test]
    fn builtins_registry_has_six() {
        let r = Registry::with_builtins();
        assert_eq!(r.len(), 6);
    }

    #[test]
    fn lookup_by_slug() {
        let r = Registry::with_builtins();
        assert!(r.get("dynamic-fee").is_some());
        assert!(r.get("anti-mev").is_some());
        assert!(r.get("does-not-exist").is_none());
    }

    #[test]
    fn veto_short_circuits() {
        let mut r = Registry::new();
        r.register(Box::new(AlwaysDeny));
        let res = r.evaluate(&ctx(HookCallback::BeforeSwap));
        assert!(!res.allow);
        assert_eq!(res.reason, Some("test veto"));
    }

    #[test]
    fn callback_filter_skips_unsubscribed() {
        let mut r = Registry::new();
        r.register(Box::new(AlwaysDeny)); // only subscribes to BeforeSwap
                                          // AfterSwap should not trigger the deny.
        assert!(r.evaluate(&ctx(HookCallback::AfterSwap)).allow);
    }
}

//! Patcha hook runtime.
//!
//! Event interceptor + execution engine that sits between a CLMM swap/liquidity
//! lifecycle event and the installed hook logic. Mirrors the Uniswap v4 hook
//! callback surface (before/after initialize, add/remove liquidity, swap,
//! donate) adapted to Solana CLMM (Orca Whirlpools, Raydium CLMM) via Anchor
//! CPI.
//!
//! The crate is split into three layers:
//!   - this module: the lifecycle `HookCallback` enum, the `HookContext` that an
//!     adapter hands in at a lifecycle point, and the `HookResult` the engine
//!     hands back to the CLMM (allow / deny / fee override).
//!   - [`registry`]: the `Hook` trait every hook implements and the `Registry`
//!     that resolves a callback to the hooks that subscribe to it.
//!   - [`builtin`]: the six standard hooks (kept slug-identical to
//!     `packages/hook-library/src/index.ts`).
//!
//! The on-chain Anchor program consumes this surface; nothing here depends on
//! the Solana toolchain so the engine unit-tests in milliseconds.

pub mod builtin;
pub mod registry;

pub use registry::{Hook, Registry};

/// Crate version, surfaced for SDK/CLI handshake checks.
pub fn version() -> &'static str {
    "0.1.0"
}

/// Hook lifecycle callbacks, mapped one-to-one from Uniswap v4's 10-callback
/// `IHooks` interface onto the Solana CLMM lifecycle. Discriminants are a stable
/// wire ABI: they are reused in PDA seeds and event encoding, so never reorder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HookCallback {
    BeforeInitialize = 0,
    AfterInitialize = 1,
    BeforeAddLiquidity = 2,
    AfterAddLiquidity = 3,
    BeforeRemoveLiquidity = 4,
    AfterRemoveLiquidity = 5,
    BeforeSwap = 6,
    AfterSwap = 7,
    BeforeDonate = 8,
    AfterDonate = 9,
}

impl HookCallback {
    /// Every callback, in wire order. Length is asserted to be 10 in tests.
    pub const ALL: [HookCallback; 10] = [
        HookCallback::BeforeInitialize,
        HookCallback::AfterInitialize,
        HookCallback::BeforeAddLiquidity,
        HookCallback::AfterAddLiquidity,
        HookCallback::BeforeRemoveLiquidity,
        HookCallback::AfterRemoveLiquidity,
        HookCallback::BeforeSwap,
        HookCallback::AfterSwap,
        HookCallback::BeforeDonate,
        HookCallback::AfterDonate,
    ];

    /// Stable wire tag for a callback (used in PDA seeds and event encoding).
    pub fn tag(self) -> u8 {
        self as u8
    }

    /// Reconstruct a callback from its wire tag. `None` if out of range.
    pub fn from_tag(tag: u8) -> Option<HookCallback> {
        HookCallback::ALL.get(tag as usize).copied()
    }

    /// `true` for the `before*` half of each pair — the phase where a hook may
    /// still veto the action.
    pub fn is_before(self) -> bool {
        matches!(
            self,
            HookCallback::BeforeInitialize
                | HookCallback::BeforeAddLiquidity
                | HookCallback::BeforeRemoveLiquidity
                | HookCallback::BeforeSwap
                | HookCallback::BeforeDonate
        )
    }

    /// Canonical Uniswap-v4-style name, used in metadata and the SDK.
    pub fn name(self) -> &'static str {
        match self {
            HookCallback::BeforeInitialize => "beforeInitialize",
            HookCallback::AfterInitialize => "afterInitialize",
            HookCallback::BeforeAddLiquidity => "beforeAddLiquidity",
            HookCallback::AfterAddLiquidity => "afterAddLiquidity",
            HookCallback::BeforeRemoveLiquidity => "beforeRemoveLiquidity",
            HookCallback::AfterRemoveLiquidity => "afterRemoveLiquidity",
            HookCallback::BeforeSwap => "beforeSwap",
            HookCallback::AfterSwap => "afterSwap",
            HookCallback::BeforeDonate => "beforeDonate",
            HookCallback::AfterDonate => "afterDonate",
        }
    }
}

/// The CLMM venue a lifecycle event originates from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dex {
    OrcaWhirlpool,
    RaydiumClmm,
}

impl Dex {
    pub fn as_str(self) -> &'static str {
        match self {
            Dex::OrcaWhirlpool => "orca",
            Dex::RaydiumClmm => "raydium",
        }
    }
}

/// Everything the engine knows at the moment a lifecycle callback fires. The
/// adapter (Orca/Raydium CPI shim) populates this from the live instruction.
///
/// Addresses are kept as raw 32-byte arrays so the struct stays toolchain-free;
/// the Anchor binding maps `Pubkey` <-> `[u8; 32]` at the CPI boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HookContext {
    pub callback: HookCallback,
    pub dex: Dex,
    /// Pool / whirlpool account.
    pub pool: [u8; 32],
    /// Initiating signer (swapper or LP).
    pub sender: [u8; 32],
    /// Input amount in base-token lamports (0 for non-amount callbacks).
    pub amount_in: u64,
    /// Current pool sqrt-price (Q64.64), as reported by the CLMM.
    pub sqrt_price_x64: u128,
    /// Current pool tick.
    pub tick: i32,
    /// Unix timestamp of the slot.
    pub timestamp: i64,
    /// Slot / block height — used by per-block MEV caps.
    pub block_height: u64,
}

impl HookContext {
    /// Minimal constructor with neutral defaults; tests and adapters override
    /// only the fields a given callback cares about.
    pub fn new(callback: HookCallback, dex: Dex, pool: [u8; 32]) -> Self {
        HookContext {
            callback,
            dex,
            pool,
            sender: [0u8; 32],
            amount_in: 0,
            sqrt_price_x64: 0,
            tick: 0,
            timestamp: 0,
            block_height: 0,
        }
    }
}

/// What a hook returns to the engine. Defaults to "allow, no change"; a hook
/// flips `allow` to veto, or sets `fee_override_bps` to retune the swap fee.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookResult {
    /// `false` aborts the lifecycle action (the CLMM reverts).
    pub allow: bool,
    /// New fee in basis points to apply for this event, if the hook overrides.
    pub fee_override_bps: Option<u32>,
    /// MEV protection credited for this event, in basis points (telemetry).
    pub mev_bps_saved: u32,
    /// Human-readable reason, surfaced in logs and the simulate endpoint.
    pub reason: Option<&'static str>,
}

impl HookResult {
    /// Allow the action unchanged.
    pub fn allow() -> Self {
        HookResult {
            allow: true,
            fee_override_bps: None,
            mev_bps_saved: 0,
            reason: None,
        }
    }

    /// Veto the action with a reason.
    pub fn deny(reason: &'static str) -> Self {
        HookResult {
            allow: false,
            fee_override_bps: None,
            mev_bps_saved: 0,
            reason: Some(reason),
        }
    }

    /// Allow, but override the fee.
    pub fn with_fee(mut self, bps: u32) -> Self {
        self.fee_override_bps = Some(bps);
        self
    }

    /// Allow, crediting MEV protection.
    pub fn with_mev_saved(mut self, bps: u32) -> Self {
        self.mev_bps_saved = bps;
        self
    }
}

impl Default for HookResult {
    fn default() -> Self {
        HookResult::allow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_pinned() {
        assert_eq!(version(), "0.1.0");
    }

    #[test]
    fn callback_count_matches_uniswap_v4() {
        // Uniswap v4 defines exactly 10 hook callbacks.
        assert_eq!(HookCallback::ALL.len(), 10);
    }

    #[test]
    fn callback_tags_are_stable() {
        assert_eq!(HookCallback::BeforeInitialize.tag(), 0);
        assert_eq!(HookCallback::BeforeSwap.tag(), 6);
        assert_eq!(HookCallback::AfterDonate.tag(), 9);
    }

    #[test]
    fn callback_tag_roundtrips() {
        for cb in HookCallback::ALL {
            assert_eq!(HookCallback::from_tag(cb.tag()), Some(cb));
        }
        assert_eq!(HookCallback::from_tag(10), None);
        assert_eq!(HookCallback::from_tag(255), None);
    }

    #[test]
    fn before_after_split_is_five_each() {
        let before = HookCallback::ALL.iter().filter(|c| c.is_before()).count();
        let after = HookCallback::ALL.iter().filter(|c| !c.is_before()).count();
        assert_eq!(before, 5);
        assert_eq!(after, 5);
    }

    #[test]
    fn callback_names_are_camel_case_v4() {
        assert_eq!(HookCallback::BeforeSwap.name(), "beforeSwap");
        assert_eq!(HookCallback::AfterAddLiquidity.name(), "afterAddLiquidity");
    }

    #[test]
    fn hook_result_builders() {
        assert!(HookResult::allow().allow);
        assert!(!HookResult::deny("nope").allow);
        assert_eq!(HookResult::allow().with_fee(80).fee_override_bps, Some(80));
        assert_eq!(HookResult::default(), HookResult::allow());
        assert_eq!(HookResult::allow().with_mev_saved(12).mev_bps_saved, 12);
    }

    #[test]
    fn dex_labels() {
        assert_eq!(Dex::OrcaWhirlpool.as_str(), "orca");
        assert_eq!(Dex::RaydiumClmm.as_str(), "raydium");
    }
}

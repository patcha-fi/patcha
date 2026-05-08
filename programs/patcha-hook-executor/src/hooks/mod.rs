//! Builtin hook logic, ported 1:1 from `crates/hook-runtime/src/builtin`.
//!
//! The off-chain runtime (toolchain-free std Rust) and this on-chain port share
//! identical slugs, parameters, and arithmetic so a simulation in the backend
//! and a trigger on mainnet produce the same allow/deny/fee decision. The only
//! difference is the host: here we decode params from a borsh blob stored in the
//! `Params` PDA and read `sender`/`timestamp` from the runtime context, while the
//! runtime takes them from a `HookContext` struct.
//!
//! Lifecycle callback tags mirror `HookCallback` in the runtime (stable wire ABI
//! reused in event encoding — never reorder).

use crate::error::PatchaError;
use anchor_lang::prelude::*;

pub mod anti_mev;
pub mod dynamic_fee;
pub mod kyc_gate;
pub mod range_order;
pub mod time_lock;
pub mod whitelist_gate;

/// Lifecycle callback tags (match `HookCallback` discriminants in the runtime).
pub const CB_BEFORE_INITIALIZE: u8 = 0;
pub const CB_AFTER_INITIALIZE: u8 = 1;
pub const CB_BEFORE_ADD_LIQUIDITY: u8 = 2;
pub const CB_AFTER_ADD_LIQUIDITY: u8 = 3;
pub const CB_BEFORE_REMOVE_LIQUIDITY: u8 = 4;
pub const CB_AFTER_REMOVE_LIQUIDITY: u8 = 5;
pub const CB_BEFORE_SWAP: u8 = 6;
pub const CB_AFTER_SWAP: u8 = 7;
pub const CB_BEFORE_DONATE: u8 = 8;
pub const CB_AFTER_DONATE: u8 = 9;

/// Highest valid callback tag (Uniswap v4 defines exactly 10 callbacks).
pub const CB_MAX: u8 = 9;

/// `true` for the `before*` half of each pair — the phase where a hook may still
/// veto the action (and where a deny must revert the lifecycle instruction).
pub fn is_before(callback: u8) -> bool {
    matches!(
        callback,
        CB_BEFORE_INITIALIZE
            | CB_BEFORE_ADD_LIQUIDITY
            | CB_BEFORE_REMOVE_LIQUIDITY
            | CB_BEFORE_SWAP
            | CB_BEFORE_DONATE
    )
}

/// The six builtin slugs, marketplace order. Mirror of `BUILTIN_SLUGS` in the
/// runtime and `HOOK_LIBRARY` order in hook-library.
pub const BUILTIN_SLUGS: [&str; 6] = [
    "dynamic-fee",
    "time-lock",
    "whitelist-gate",
    "range-order",
    "anti-mev",
    "kyc-gate",
];

/// Reason codes surfaced in the `HookTriggered` event (compact, on-chain-cheap
/// alternative to the runtime's `&'static str` reasons).
pub mod reason {
    pub const NONE: u8 = 0;
    pub const DYNAMIC_FEE_APPLIED: u8 = 1;
    pub const TIME_LOCK_LOCKED: u8 = 2;
    pub const WHITELIST_BLOCKED: u8 = 3;
    pub const RANGE_ORDER_FILLED: u8 = 4;
    pub const ANTI_MEV_BLOCKED: u8 = 5;
    pub const ANTI_MEV_CREDITED: u8 = 6;
    pub const KYC_NOT_ATTESTED: u8 = 7;
}

/// Everything a hook needs at trigger time. The Anchor `trigger_hook`
/// instruction assembles this from instruction args + the `Clock` sysvar + the
/// triggering signer. Mirror of `HookContext` in the runtime.
#[derive(Clone, Copy, Debug)]
pub struct TriggerCtx {
    pub callback: u8,
    pub dex: u8,
    pub sender: Pubkey,
    pub amount_in: u64,
    pub tick: i32,
    pub timestamp: i64,
}

/// A hook's decision. Mirror of `HookResult` in the runtime, with the reason as
/// a compact code instead of a string.
#[derive(Clone, Copy, Debug)]
pub struct HookOutcome {
    /// `false` aborts the lifecycle action (the CLMM reverts).
    pub allow: bool,
    /// New fee in basis points, if the hook overrides it (0 = no override).
    pub fee_override_bps: u32,
    pub has_fee_override: bool,
    /// MEV protection credited for this event, in basis points (telemetry).
    pub mev_bps_saved: u32,
    /// Reason code (see [`reason`]).
    pub reason_code: u8,
}

impl HookOutcome {
    pub fn allow() -> Self {
        HookOutcome {
            allow: true,
            fee_override_bps: 0,
            has_fee_override: false,
            mev_bps_saved: 0,
            reason_code: reason::NONE,
        }
    }

    pub fn deny(reason_code: u8) -> Self {
        HookOutcome {
            allow: false,
            fee_override_bps: 0,
            has_fee_override: false,
            mev_bps_saved: 0,
            reason_code,
        }
    }

    pub fn with_fee(mut self, bps: u32) -> Self {
        self.fee_override_bps = bps;
        self.has_fee_override = true;
        self
    }

    pub fn with_mev_saved(mut self, bps: u32) -> Self {
        self.mev_bps_saved = bps;
        self
    }

    pub fn with_reason(mut self, reason_code: u8) -> Self {
        self.reason_code = reason_code;
        self
    }
}

/// Decode a borsh params struct from a blob, falling back to `Default` when the
/// blob is empty (default-parameter installs). A malformed non-empty blob is a
/// hard error so a bad install can never be silently coerced.
pub fn decode_params<T: AnchorDeserialize + Default>(blob: &[u8]) -> Result<T> {
    if blob.is_empty() {
        return Ok(T::default());
    }
    T::try_from_slice(blob).map_err(|_| error!(PatchaError::InvalidParams))
}

/// Dispatch a callback to the matching builtin hook. Community hooks (slugs not
/// in [`BUILTIN_SLUGS`]) are a no-op for this executor — their logic is defined
/// by the integrator's own program/keeper — so they always allow.
pub fn dispatch(slug: &str, ctx: &TriggerCtx, params_blob: &[u8]) -> Result<HookOutcome> {
    match slug {
        "dynamic-fee" => dynamic_fee::evaluate(ctx, params_blob),
        "time-lock" => time_lock::evaluate(ctx, params_blob),
        "whitelist-gate" => whitelist_gate::evaluate(ctx, params_blob),
        "range-order" => range_order::evaluate(ctx, params_blob),
        "anti-mev" => anti_mev::evaluate(ctx, params_blob),
        "kyc-gate" => kyc_gate::evaluate(ctx, params_blob),
        _ => Ok(HookOutcome::allow()),
    }
}

/// Validate a params blob against the hook's schema at install/update time, so
/// a malformed blob is rejected before it is ever stored.
pub fn validate_params(slug: &str, params_blob: &[u8]) -> Result<()> {
    if params_blob.is_empty() {
        return Ok(());
    }
    match slug {
        "dynamic-fee" => {
            decode_params::<dynamic_fee::DynamicFeeParams>(params_blob)?;
        }
        "time-lock" => {
            decode_params::<time_lock::TimeLockParams>(params_blob)?;
        }
        "whitelist-gate" => {
            decode_params::<whitelist_gate::WhitelistGateParams>(params_blob)?;
        }
        "range-order" => {
            decode_params::<range_order::RangeOrderParams>(params_blob)?;
        }
        "anti-mev" => {
            decode_params::<anti_mev::AntiMevParams>(params_blob)?;
        }
        "kyc-gate" => {
            decode_params::<kyc_gate::KycGateParams>(params_blob)?;
        }
        // Community hooks carry an opaque blob; nothing to validate here.
        _ => {}
    }
    Ok(())
}

//! Orca Whirlpools adapter.
//!
//! Maps a whirlpool lifecycle event into a venue-neutral [`HookContext`]. The
//! integrator supplies the whirlpool account, the initiating signer, and the
//! swap amount / current tick read from the whirlpool state; this module stamps
//! the venue tag (`Dex::OrcaWhirlpool`) and the slot timestamp so the builtin
//! hooks evaluate the same context regardless of venue.

use patcha_hook_runtime::{Dex, HookCallback, HookContext};

/// Orca Whirlpools program id (mainnet).
pub const WHIRLPOOL_PROGRAM_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

/// Build a [`HookContext`] for a whirlpool swap event.
pub fn map_swap(
    callback: HookCallback,
    pool: [u8; 32],
    sender: [u8; 32],
    amount_in: u64,
    tick: i32,
    timestamp: i64,
) -> HookContext {
    let mut ctx = HookContext::new(callback, Dex::OrcaWhirlpool, pool);
    ctx.sender = sender;
    ctx.amount_in = amount_in;
    ctx.tick = tick;
    ctx.timestamp = timestamp;
    ctx
}

/// Build a [`HookContext`] for a whirlpool add/remove-liquidity event. Liquidity
/// callbacks carry no swap amount, so `amount_in` is left at zero.
pub fn map_liquidity(
    callback: HookCallback,
    pool: [u8; 32],
    sender: [u8; 32],
    tick: i32,
    timestamp: i64,
) -> HookContext {
    let mut ctx = HookContext::new(callback, Dex::OrcaWhirlpool, pool);
    ctx.sender = sender;
    ctx.tick = tick;
    ctx.timestamp = timestamp;
    ctx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_context_carries_orca_tag_and_fields() {
        let ctx = map_swap(
            HookCallback::BeforeSwap,
            [1u8; 32],
            [2u8; 32],
            500,
            42,
            1_700_000_000,
        );
        assert_eq!(ctx.dex, Dex::OrcaWhirlpool);
        assert_eq!(ctx.callback, HookCallback::BeforeSwap);
        assert_eq!(ctx.amount_in, 500);
        assert_eq!(ctx.tick, 42);
        assert_eq!(ctx.sender, [2u8; 32]);
        assert_eq!(ctx.timestamp, 1_700_000_000);
    }

    #[test]
    fn liquidity_context_has_zero_amount() {
        let ctx = map_liquidity(
            HookCallback::BeforeAddLiquidity,
            [3u8; 32],
            [4u8; 32],
            -10,
            1_700_000_001,
        );
        assert_eq!(ctx.dex, Dex::OrcaWhirlpool);
        assert_eq!(ctx.amount_in, 0);
        assert_eq!(ctx.tick, -10);
    }
}

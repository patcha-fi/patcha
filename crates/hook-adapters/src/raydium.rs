//! Raydium CLMM adapter.
//!
//! Symmetric with the Orca adapter: maps a Raydium concentrated-liquidity
//! lifecycle event into a venue-neutral [`HookContext`]. The integrator supplies
//! the pool, signer, amount, and current tick; this module stamps the venue tag
//! (`Dex::RaydiumClmm`) and the slot timestamp.

use patcha_hook_runtime::{Dex, HookCallback, HookContext};

/// Raydium CLMM program id (mainnet).
pub const RAYDIUM_CLMM_PROGRAM_ID: &str = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";

/// Build a [`HookContext`] for a Raydium CLMM swap event.
pub fn map_swap(
    callback: HookCallback,
    pool: [u8; 32],
    sender: [u8; 32],
    amount_in: u64,
    tick: i32,
    timestamp: i64,
) -> HookContext {
    let mut ctx = HookContext::new(callback, Dex::RaydiumClmm, pool);
    ctx.sender = sender;
    ctx.amount_in = amount_in;
    ctx.tick = tick;
    ctx.timestamp = timestamp;
    ctx
}

/// Build a [`HookContext`] for a Raydium add/remove-liquidity event. Liquidity
/// callbacks carry no swap amount, so `amount_in` is left at zero.
pub fn map_liquidity(
    callback: HookCallback,
    pool: [u8; 32],
    sender: [u8; 32],
    tick: i32,
    timestamp: i64,
) -> HookContext {
    let mut ctx = HookContext::new(callback, Dex::RaydiumClmm, pool);
    ctx.sender = sender;
    ctx.tick = tick;
    ctx.timestamp = timestamp;
    ctx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_context_carries_raydium_tag_and_fields() {
        let ctx = map_swap(
            HookCallback::BeforeSwap,
            [5u8; 32],
            [6u8; 32],
            900,
            -3,
            1_700_000_002,
        );
        assert_eq!(ctx.dex, Dex::RaydiumClmm);
        assert_eq!(ctx.callback, HookCallback::BeforeSwap);
        assert_eq!(ctx.amount_in, 900);
        assert_eq!(ctx.tick, -3);
        assert_eq!(ctx.sender, [6u8; 32]);
    }

    #[test]
    fn liquidity_context_has_zero_amount() {
        let ctx = map_liquidity(
            HookCallback::BeforeRemoveLiquidity,
            [7u8; 32],
            [8u8; 32],
            17,
            1_700_000_003,
        );
        assert_eq!(ctx.dex, Dex::RaydiumClmm);
        assert_eq!(ctx.amount_in, 0);
        assert_eq!(ctx.tick, 17);
    }
}

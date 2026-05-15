//! DEX adapter boundary.
//!
//! Orca Whirlpools and Raydium CLMM do not call arbitrary external programs on
//! their swap path, so the hook executor is driven at the integration boundary:
//! a router program or an off-chain keeper that observes a pool's lifecycle
//! event maps it into a [`TriggerCtx`] and invokes `trigger_hook`. These adapter
//! modules own that mapping — turning a venue-specific swap/liquidity event into
//! the venue-neutral context the builtin hooks evaluate.
//!
//! The off-chain Orca/Raydium SDK wrappers live in `crates/hook-adapters`; they
//! build the same `trigger_hook` instruction the SDK exposes.

pub mod orca;
pub mod raydium;

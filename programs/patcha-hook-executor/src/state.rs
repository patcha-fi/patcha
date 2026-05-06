//! On-chain account layouts (PDAs) for the Patcha hook executor.
//!
//! Sizing follows the project Anchor rules: every account reserves the 8-byte
//! discriminator (added by Anchor at `init` via `space = 8 + T::LEN`), each
//! `LEN` is computed field-by-field, and a trailing `_padding` keeps the struct
//! 8-byte aligned. Numeric PDA seeds are always little-endian; the seeds here are
//! byte strings + `Pubkey`s + the hook slug, so no integer encoding is involved.

use anchor_lang::prelude::*;

/// Maximum hook slug length (e.g. "whitelist-gate"). Kept in sync with the
/// cross-language source of truth `packages/hook-library/src/index.ts`.
pub const MAX_SLUG_LEN: usize = 32;

/// Maximum size of a serialized hook params blob (borsh-encoded hook params).
pub const MAX_PARAMS_LEN: usize = 256;

/// Hook kind tags (stored on `HookMeta.kind`).
pub const HOOK_KIND_BUILTIN: u8 = 0;
pub const HOOK_KIND_COMMUNITY: u8 = 1;

/// DEX venue tags — same order/labels as `Dex` in the hook runtime
/// (`OrcaWhirlpool` -> "orca" = 0, `RaydiumClmm` -> "raydium" = 1).
pub const DEX_ORCA: u8 = 0;
pub const DEX_RAYDIUM: u8 = 1;

/// Global hook registry. One per program. PDA: `["hook_registry"]`.
#[account]
pub struct HookRegistry {
    /// Authority that initialized the registry.
    pub admin: Pubkey, // 32
    /// Number of hooks registered (builtin + community).
    pub hook_count: u64, // 8
    /// Number of live installations across all pools.
    pub install_count: u64, // 8
    /// PDA bump.
    pub bump: u8, // 1
    pub _padding: [u8; 7], // 7
}

impl HookRegistry {
    pub const LEN: usize = 32 + 8 + 8 + 1 + 7; // 56
}

/// Per-hook metadata. PDA: `["hook", slug.as_bytes()]`.
#[account]
pub struct HookMeta {
    /// Hook slug (marketplace identifier).
    pub slug: String, // 4 + 32
    /// `HOOK_KIND_BUILTIN` or `HOOK_KIND_COMMUNITY`.
    pub kind: u8, // 1
    /// Hash of the hook source / wasm (integrity anchor for community hooks).
    pub code_hash: [u8; 32], // 32
    /// Hook author (signer that registered it).
    pub author: Pubkey, // 32
    /// Whether the hook has cleared an audit.
    pub audited: bool, // 1
    /// Times this hook has been installed on a pool.
    pub install_count: u64, // 8
    /// PDA bump.
    pub bump: u8, // 1
    pub _padding: [u8; 5], // 5
}

impl HookMeta {
    pub const LEN: usize = (4 + MAX_SLUG_LEN) + 1 + 32 + 32 + 1 + 8 + 1 + 5; // 116
}

/// Per-pool hook installation. PDA: `["installation", pool, slug.as_bytes()]`.
#[account]
pub struct Installation {
    /// CLMM pool / whirlpool the hook is installed on.
    pub pool: Pubkey, // 32
    /// Installed hook slug.
    pub slug: String, // 4 + 32
    /// Wallet that installed (LP authority).
    pub installer: Pubkey, // 32
    /// `DEX_ORCA` or `DEX_RAYDIUM`.
    pub dex: u8, // 1
    /// Soft-delete flag (`uninstall_hook` flips this instead of closing).
    pub active: bool, // 1
    /// Unix timestamp of install.
    pub installed_at: i64, // 8
    /// Number of times the hook has been triggered on this pool.
    pub trigger_count: u64, // 8
    /// PDA bump.
    pub bump: u8, // 1
    pub _padding: [u8; 5], // 5
}

impl Installation {
    pub const LEN: usize = 32 + (4 + MAX_SLUG_LEN) + 32 + 1 + 1 + 8 + 8 + 1 + 5; // 124
}

/// Hook parameter blob for an installation. PDA: `["params", installation]`.
#[account]
pub struct Params {
    /// Back-reference to the owning installation (for `has_one` integrity).
    pub installation: Pubkey, // 32
    /// Borsh-encoded hook-specific params (decoded by the matching hook module).
    pub params_blob: Vec<u8>, // 4 + MAX_PARAMS_LEN
    /// PDA bump.
    pub bump: u8, // 1
    pub _padding: [u8; 3], // 3
}

impl Params {
    pub const LEN: usize = 32 + (4 + MAX_PARAMS_LEN) + 1 + 3; // 296
}

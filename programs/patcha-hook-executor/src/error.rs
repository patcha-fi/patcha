//! Program error codes.

use anchor_lang::prelude::*;

#[error_code]
pub enum PatchaError {
    #[msg("Caller is not authorized for this action")]
    Unauthorized,
    #[msg("Slug is empty or exceeds the maximum length")]
    InvalidSlug,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Hook is already installed on this pool")]
    AlreadyInstalled,
    #[msg("Signer is not the installer of this hook")]
    NotInstaller,
    #[msg("Params blob is invalid or exceeds the maximum size")]
    InvalidParams,
    #[msg("Hook registry has not been initialized")]
    RegistryNotInitialized,
    #[msg("Unknown or out-of-range hook callback kind")]
    InvalidCallback,
    #[msg("Unknown DEX venue (expected 0=orca, 1=raydium)")]
    InvalidDex,
    #[msg("Hook kind must be 0 (builtin) or 1 (community)")]
    InvalidHookKind,
    #[msg("Installation is not active")]
    NotActive,
    #[msg("Hook vetoed the lifecycle action")]
    HookVetoed,
}

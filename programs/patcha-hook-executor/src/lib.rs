//! Patcha Hook Executor — Anchor program for Solana CLMM hooks.
//!
//! A registry + installer + trigger surface that brings Uniswap-v4-style hooks
//! to Solana concentrated-liquidity pools (Orca Whirlpools, Raydium CLMM). Hooks
//! are registered once, installed per pool with a parameter blob, and triggered
//! at lifecycle points (before/after swap & liquidity) where the matching
//! builtin logic decides allow / deny / fee-override. The builtin hook logic is
//! a 1:1 port of `crates/hook-runtime` so a backend simulation and an on-chain
//! trigger agree.
//!
//! PDA seeds:
//!   ["hook_registry"]                            global registry
//!   ["hook", slug]                               per-hook metadata
//!   ["installation", pool, slug]                 per-pool install
//!   ["params", installation]                     hook params blob

use anchor_lang::prelude::*;

pub mod adapters;
pub mod error;
pub mod hooks;
pub mod state;

use error::PatchaError;
use hooks::{TriggerCtx, CB_MAX};
use state::*;

// `anchor keys sync` rewrites this + Anchor.toml after the first build; the real
// mainnet id is injected at deploy time.
declare_id!("EPcW7e8RxBNPpQK2XKoKG9maWH6QvmU3ejxifoU5rNRa");

#[program]
pub mod patcha_hook_executor {
    use super::*;

    /// Initialize the global hook registry (admin, once).
    pub fn initialize_registry(ctx: Context<InitializeRegistry>) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.admin = ctx.accounts.admin.key();
        registry.hook_count = 0;
        registry.install_count = 0;
        registry.bump = ctx.bumps.registry;
        registry._padding = [0u8; 7];
        Ok(())
    }

    /// Register a hook (builtin or community). Author = signer.
    pub fn register_hook(
        ctx: Context<RegisterHook>,
        slug: String,
        kind: u8,
        code_hash: [u8; 32],
    ) -> Result<()> {
        require!(
            !slug.is_empty() && slug.len() <= MAX_SLUG_LEN,
            PatchaError::InvalidSlug
        );
        require!(
            kind == HOOK_KIND_BUILTIN || kind == HOOK_KIND_COMMUNITY,
            PatchaError::InvalidHookKind
        );

        let meta = &mut ctx.accounts.hook_meta;
        meta.slug = slug;
        meta.kind = kind;
        meta.code_hash = code_hash;
        meta.author = ctx.accounts.author.key();
        // Builtin hooks ship audited; community hooks start unaudited.
        meta.audited = kind == HOOK_KIND_BUILTIN;
        meta.install_count = 0;
        meta.bump = ctx.bumps.hook_meta;
        meta._padding = [0u8; 5];

        let registry = &mut ctx.accounts.registry;
        registry.hook_count = registry
            .hook_count
            .checked_add(1)
            .ok_or(PatchaError::Overflow)?;
        Ok(())
    }

    /// Install a registered hook on a CLMM pool with its parameter blob.
    pub fn install_hook(
        ctx: Context<InstallHook>,
        pool: Pubkey,
        slug: String,
        dex: u8,
        params_blob: Vec<u8>,
    ) -> Result<()> {
        require!(
            !slug.is_empty() && slug.len() <= MAX_SLUG_LEN,
            PatchaError::InvalidSlug
        );
        require!(dex == DEX_ORCA || dex == DEX_RAYDIUM, PatchaError::InvalidDex);
        require!(
            params_blob.len() <= MAX_PARAMS_LEN,
            PatchaError::InvalidParams
        );
        // Reject a params blob that does not match the hook's schema.
        hooks::validate_params(&slug, &params_blob)?;

        let clock = Clock::get()?;

        let installation = &mut ctx.accounts.installation;
        installation.pool = pool;
        installation.slug = slug;
        installation.installer = ctx.accounts.installer.key();
        installation.dex = dex;
        installation.active = true;
        installation.installed_at = clock.unix_timestamp;
        installation.trigger_count = 0;
        installation.bump = ctx.bumps.installation;
        installation._padding = [0u8; 5];

        let params = &mut ctx.accounts.params;
        params.installation = installation.key();
        params.params_blob = params_blob;
        params.bump = ctx.bumps.params;
        params._padding = [0u8; 3];

        let meta = &mut ctx.accounts.hook_meta;
        meta.install_count = meta
            .install_count
            .checked_add(1)
            .ok_or(PatchaError::Overflow)?;

        let registry = &mut ctx.accounts.registry;
        registry.install_count = registry
            .install_count
            .checked_add(1)
            .ok_or(PatchaError::Overflow)?;
        Ok(())
    }

    /// Update an installation's parameter blob (installer only).
    pub fn update_params(ctx: Context<UpdateParams>, params_blob: Vec<u8>) -> Result<()> {
        require!(
            params_blob.len() <= MAX_PARAMS_LEN,
            PatchaError::InvalidParams
        );
        hooks::validate_params(&ctx.accounts.installation.slug, &params_blob)?;
        let params = &mut ctx.accounts.params;
        params.params_blob = params_blob;
        Ok(())
    }

    /// Trigger an installed hook at a CLMM lifecycle point. Called by the
    /// integration boundary (router program / keeper) that maps a venue event
    /// into a neutral context. Vetoes in a `before*` callback revert the tx.
    pub fn trigger_hook(
        ctx: Context<TriggerHook>,
        callback_kind: u8,
        amount_in: u64,
        tick: i32,
    ) -> Result<()> {
        require!(callback_kind <= CB_MAX, PatchaError::InvalidCallback);

        let clock = Clock::get()?;
        let installation = &mut ctx.accounts.installation;
        require!(installation.active, PatchaError::NotActive);

        let tctx = TriggerCtx {
            callback: callback_kind,
            dex: installation.dex,
            sender: ctx.accounts.caller.key(),
            amount_in,
            tick,
            timestamp: clock.unix_timestamp,
        };

        let outcome = hooks::dispatch(
            installation.slug.as_str(),
            &tctx,
            &ctx.accounts.params.params_blob,
        )?;

        installation.trigger_count = installation
            .trigger_count
            .checked_add(1)
            .ok_or(PatchaError::Overflow)?;

        emit!(HookTriggered {
            pool: installation.pool,
            slug: installation.slug.clone(),
            dex: installation.dex,
            callback: callback_kind,
            caller: tctx.sender,
            allow: outcome.allow,
            fee_override_bps: outcome.fee_override_bps,
            has_fee_override: outcome.has_fee_override,
            mev_bps_saved: outcome.mev_bps_saved,
            reason_code: outcome.reason_code,
        });

        if !outcome.allow && hooks::is_before(callback_kind) {
            msg!(
                "patcha: hook '{}' vetoed callback {} (reason {})",
                installation.slug,
                callback_kind,
                outcome.reason_code
            );
            return err!(PatchaError::HookVetoed);
        }
        Ok(())
    }

    /// Uninstall a hook from a pool (soft delete — keeps the PDA, saves rent).
    pub fn uninstall_hook(ctx: Context<UninstallHook>) -> Result<()> {
        let installation = &mut ctx.accounts.installation;
        require!(installation.active, PatchaError::NotActive);
        installation.active = false;

        let registry = &mut ctx.accounts.registry;
        registry.install_count = registry.install_count.saturating_sub(1);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Contexts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
pub struct InitializeRegistry<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + HookRegistry::LEN,
        seeds = [b"hook_registry"],
        bump,
    )]
    pub registry: Box<Account<'info, HookRegistry>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(slug: String)]
pub struct RegisterHook<'info> {
    #[account(mut)]
    pub author: Signer<'info>,
    #[account(
        mut,
        seeds = [b"hook_registry"],
        bump = registry.bump,
    )]
    pub registry: Box<Account<'info, HookRegistry>>,
    #[account(
        init,
        payer = author,
        space = 8 + HookMeta::LEN,
        seeds = [b"hook", slug.as_bytes()],
        bump,
    )]
    pub hook_meta: Box<Account<'info, HookMeta>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(pool: Pubkey, slug: String)]
pub struct InstallHook<'info> {
    #[account(mut)]
    pub installer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"hook_registry"],
        bump = registry.bump,
    )]
    pub registry: Box<Account<'info, HookRegistry>>,
    #[account(
        mut,
        seeds = [b"hook", slug.as_bytes()],
        bump = hook_meta.bump,
    )]
    pub hook_meta: Box<Account<'info, HookMeta>>,
    #[account(
        init,
        payer = installer,
        space = 8 + Installation::LEN,
        seeds = [b"installation", pool.as_ref(), slug.as_bytes()],
        bump,
    )]
    pub installation: Box<Account<'info, Installation>>,
    #[account(
        init,
        payer = installer,
        space = 8 + Params::LEN,
        seeds = [b"params", installation.key().as_ref()],
        bump,
    )]
    pub params: Box<Account<'info, Params>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateParams<'info> {
    pub installer: Signer<'info>,
    #[account(
        has_one = installer @ PatchaError::NotInstaller,
    )]
    pub installation: Box<Account<'info, Installation>>,
    #[account(
        mut,
        seeds = [b"params", installation.key().as_ref()],
        bump = params.bump,
        has_one = installation @ PatchaError::InvalidParams,
    )]
    pub params: Box<Account<'info, Params>>,
}

#[derive(Accounts)]
pub struct TriggerHook<'info> {
    /// Integration-boundary caller (router program / keeper). Also used as the
    /// acting sender for gating hooks in this reference path.
    pub caller: Signer<'info>,
    #[account(mut)]
    pub installation: Box<Account<'info, Installation>>,
    #[account(
        seeds = [b"params", installation.key().as_ref()],
        bump = params.bump,
        has_one = installation @ PatchaError::InvalidParams,
    )]
    pub params: Box<Account<'info, Params>>,
}

#[derive(Accounts)]
pub struct UninstallHook<'info> {
    pub installer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"hook_registry"],
        bump = registry.bump,
    )]
    pub registry: Box<Account<'info, HookRegistry>>,
    #[account(
        mut,
        has_one = installer @ PatchaError::NotInstaller,
    )]
    pub installation: Box<Account<'info, Installation>>,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct HookTriggered {
    pub pool: Pubkey,
    pub slug: String,
    pub dex: u8,
    pub callback: u8,
    pub caller: Pubkey,
    pub allow: bool,
    pub fee_override_bps: u32,
    pub has_fee_override: bool,
    pub mev_bps_saved: u32,
    pub reason_code: u8,
}

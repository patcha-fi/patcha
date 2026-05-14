//! KYCGate — require a verified-credential attestation before acting.
//! Category: kyc. Callbacks: beforeSwap, beforeAddLiquidity. Cable: grey.
//!
//! Port of `crates/hook-runtime/src/builtin/kyc_gate.rs`. Deny-by-default: an
//! action is permitted only if the sender holds a valid attestation. The runtime
//! models attestations with an explicit set; the on-chain params carry both the
//! attestation authority (informational) and the attested set so the decision is
//! identical. A production deployment can verify an attestation account / SAS
//! credential issued by `attestation_authority`.

use super::{reason, HookOutcome, TriggerCtx};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct KycGateParams {
    /// Authority whose attestation is required (32-byte pubkey).
    pub attestation_authority: [u8; 32],
    /// Senders holding a valid attestation.
    pub attested: Vec<[u8; 32]>,
}

pub fn is_attested(p: &KycGateParams, who: &[u8; 32]) -> bool {
    p.attested.iter().any(|a| a == who)
}

pub fn evaluate(ctx: &TriggerCtx, blob: &[u8]) -> Result<HookOutcome> {
    let p = super::decode_params::<KycGateParams>(blob)?;
    if is_attested(&p, &ctx.sender.to_bytes()) {
        Ok(HookOutcome::allow())
    } else {
        Ok(HookOutcome::deny(reason::KYC_NOT_ATTESTED))
    }
}

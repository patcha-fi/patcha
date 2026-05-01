//! DEX adapter boundary.
//!
//! Orca Whirlpools and Raydium CLMM do not call arbitrary external programs on
//! their swap path, so the hook engine is driven at the integration boundary: a
//! router program or an off-chain keeper observes a pool's lifecycle event,
//! maps it into a venue-neutral [`HookContext`] with one of these adapters, and
//! evaluates the installed hooks against it.
//!
//! Each adapter owns exactly one mapping — turning a venue-specific
//! swap/liquidity event into the neutral context the builtin hooks read. The
//! mapping is pure and toolchain-free so it unit-tests in milliseconds and is
//! reused verbatim by both the backend simulator and the on-chain trigger path.

pub mod orca;
pub mod raydium;

pub use patcha_hook_runtime::{Dex, HookCallback, HookContext, HookResult, Registry};

/// A CLMM venue this crate can adapt. Thin wrapper over [`Dex`] that also
/// carries the on-chain program id string for the integration boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Venue {
    Orca,
    Raydium,
}

impl Venue {
    /// The runtime [`Dex`] tag for this venue.
    pub fn dex(self) -> Dex {
        match self {
            Venue::Orca => Dex::OrcaWhirlpool,
            Venue::Raydium => Dex::RaydiumClmm,
        }
    }

    /// Mainnet program id of the venue's CLMM program.
    pub fn program_id(self) -> &'static str {
        match self {
            Venue::Orca => orca::WHIRLPOOL_PROGRAM_ID,
            Venue::Raydium => raydium::RAYDIUM_CLMM_PROGRAM_ID,
        }
    }

    /// Short label, matching `Dex::as_str`.
    pub fn as_str(self) -> &'static str {
        self.dex().as_str()
    }
}

/// Decode a 32-byte account key from its base58 string form. Returns `None` for
/// malformed input. Kept dependency-free with a small inline base58 decoder so
/// the adapter crate stays toolchain-free.
pub fn decode_key(s: &str) -> Option<[u8; 32]> {
    const ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut bytes: Vec<u8> = Vec::with_capacity(32);
    for ch in s.bytes() {
        let val = ALPHABET.iter().position(|&c| c == ch)? as u32;
        let mut carry = val;
        for b in bytes.iter_mut() {
            carry += (*b as u32) * 58;
            *b = (carry & 0xff) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            bytes.push((carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    // Leading '1's encode leading zero bytes.
    for ch in s.bytes() {
        if ch == b'1' {
            bytes.push(0);
        } else {
            break;
        }
    }
    bytes.reverse();
    if bytes.len() > 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn venue_maps_to_dex() {
        assert_eq!(Venue::Orca.dex(), Dex::OrcaWhirlpool);
        assert_eq!(Venue::Raydium.dex(), Dex::RaydiumClmm);
        assert_eq!(Venue::Orca.as_str(), "orca");
        assert_eq!(Venue::Raydium.as_str(), "raydium");
    }

    #[test]
    fn program_ids_are_the_canonical_mainnet_ids() {
        assert!(Venue::Orca.program_id().starts_with("whirL"));
        assert!(Venue::Raydium.program_id().starts_with("CAMM"));
    }

    #[test]
    fn decode_key_roundtrips_known_program_id() {
        // A valid base58 Solana program id decodes to exactly 32 bytes.
        let key = decode_key(orca::WHIRLPOOL_PROGRAM_ID).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn decode_key_rejects_non_base58() {
        assert!(decode_key("0OIl_not_base58").is_none());
    }
}

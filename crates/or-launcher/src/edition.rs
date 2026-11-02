//! Which `OR2006C2C.exe` the game folder holds, by SHA-256.
//!
//! The 2006 EXE is wrapped in disc DRM that breaks DLL wrappers, so
//! OutRun2006Tweaks ships its own DRM-free replacement EXE. The launcher
//! recognises both: an original install (it swaps in the replacement) and an
//! install already patched by Tweaks (it refreshes the files it manages).

use sha2::{Digest, Sha256};

pub const EXE: &str = "OR2006C2C.exe";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edition {
    pub name: &'static str,
    /// The replacement EXE is already in place.
    pub patched: bool,
}

/// The original EXE from the 2006 retail/repack install (DRM-wrapped,
/// 14,950,400 bytes, PE timestamp 1148663097 = 2006-05-26).
pub const ORIGINAL: Edition = Edition {
    name: "OutRun 2006 (original EXE; the launcher replaces it in its copy)",
    patched: false,
};

/// The DRM-free replacement EXE from OutRun2006Tweaks (3,674,112 bytes,
/// same PE timestamp; already Large Address Aware).
pub const TWEAKS: Edition = Edition {
    name: "OutRun 2006 + OutRun2006Tweaks EXE",
    patched: true,
};

fn digests() -> impl Iterator<Item = ([u8; 32], Edition)> {
    let original: [u8; 32] =
        hex("bdafa88a5abdd2a9743f6bdcc5e2189288c0203790412fce10b9bb649933482f");
    let tweaks: [u8; 32] = hex("68ceb386829066f8455b9d027320af962584321f3e2e8a79c72841495a6134c3");
    [(original, ORIGINAL), (tweaks, TWEAKS)].into_iter()
}

/// Identifies an EXE image; `Err` carries its SHA-256 for the report we ask
/// the player to send.
pub fn identify(bytes: &[u8]) -> Result<Edition, [u8; 32]> {
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    digests()
        .find(|(known, _)| *known == digest)
        .map(|(_, edition)| edition)
        .ok_or(digest)
}

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex(value: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (slot, pair) in out.iter_mut().zip(value.as_bytes().chunks(2)) {
        *slot = u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_digest_table_decodes_and_is_distinct() {
        let table: Vec<_> = digests().collect();
        assert_eq!(
            to_hex(&table[0].0),
            "bdafa88a5abdd2a9743f6bdcc5e2189288c0203790412fce10b9bb649933482f"
        );
        assert_eq!(
            to_hex(&table[1].0),
            "68ceb386829066f8455b9d027320af962584321f3e2e8a79c72841495a6134c3"
        );
        assert_ne!(table[0].0, table[1].0);
        assert!(!table[0].1.patched && table[1].1.patched);
    }

    #[test]
    fn unknown_exes_report_their_sha256() {
        let digest = Sha256::digest(b"not the game");
        assert_eq!(identify(b"not the game"), Err(digest.into()));
        assert_eq!(to_hex(&digest).len(), 64);
    }

    #[test]
    fn hex_parses_the_pinned_digests() {
        assert_eq!(
            to_hex(&hex(
                "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
            )),
            "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
        );
        for (digest, _) in digests() {
            assert_eq!(hex(&to_hex(&digest)), digest);
        }
    }
}

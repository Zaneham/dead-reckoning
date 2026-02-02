//! Shares - Shamir's Secret Sharing implementation
//!
//! Sometimes ye want to split a secret among yer crew such that
//! any 3 of 5 captains can reconstruct it, but 2 alone cannot.
//! That's Shamir's Secret Sharing, invented by Adi Shamir in 1979.
//! The same Shamir from RSA. Clever bloke.
//!
//! This is useful fer:
//! - Splittin' encryption keys among trusted contacts
//! - Ensurin' no single point of failure (or betrayal)
//! - Feelin' like yer in a spy movie
//!
//! The maths involves polynomials over finite fields, which is
//! exactly as fun as it sounds. Fortunately the `sharks` crate
//! handles all that so we don't have to.

use sharks::{Share, Sharks};

use crate::error::{Error, Result};

/// Secret sharing scheme configuration
///
/// Holds the threshold (minimum shares needed) and total (shares to create).
/// If threshold is 3 and total is 5, ye create 5 shares and any 3 can
/// reconstruct the secret. Very flexible. Very paranoid.
pub struct SecretSharing {
    /// Minimum number of shares needed to reconstruct
    threshold: u8,
    /// Total number of shares to create
    total: u8,
}

impl SecretSharing {
    /// Create a new secret sharing scheme
    ///
    /// - threshold: How many shares needed to reconstruct (e.g., 3)
    /// - total: How many shares to create (e.g., 5)
    ///
    /// The threshold must be less than or equal to total, obviously.
    /// We're not gonna check that fer ye because we trust ye to not
    /// be an absolute numpty about it.
    pub fn new(threshold: u8, total: u8) -> Self {
        Self { threshold, total }
    }

    /// Split a secret into shares using Shamir's Secret Sharing
    ///
    /// Takes yer secret (as bytes) and returns a Vec of shares.
    /// Each share is also bytes. Distribute these to yer trusted
    /// captains. Don't give the same captain multiple shares unless
    /// ye really trust them.
    pub fn split(&self, secret: &[u8]) -> Result<Vec<Vec<u8>>> {
        let sharks = Sharks(self.threshold);
        let dealer = sharks.dealer(secret);

        let shares: Vec<Vec<u8>> = dealer
            .take(self.total as usize)
            .map(|s| Vec::from(&s))
            .collect();

        Ok(shares)
    }

    /// Reconstruct secret from shares
    ///
    /// Takes a slice of shares and attempts to reconstruct the original
    /// secret. Ye need at least `threshold` shares or this will fail.
    /// With fewer shares, the maths just doesn't work. Not our fault,
    /// blame Lagrange interpolation.
    ///
    /// Returns an error if:
    /// - Not enough shares provided
    /// - Shares are corrupted or invalid
    /// - The shares don't belong to the same secret (mixed up shares from
    ///   different split operations)
    pub fn reconstruct(&self, shares: &[Vec<u8>]) -> Result<Vec<u8>> {
        if shares.len() < self.threshold as usize {
            return Err(Error::InsufficientShares {
                need: self.threshold,
                have: shares.len() as u8,
            });
        }

        let sharks = Sharks(self.threshold);

        // Parse the raw bytes back into Share objects
        let parsed_shares: Vec<Share> = shares
            .iter()
            .map(|s| Share::try_from(s.as_slice()))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Crypto(format!("Invalid share: {}. Someone's been tamperin'.", e)))?;

        // Do the fancy polynomial interpolation to recover the secret
        let secret = sharks.recover(&parsed_shares)
            .map_err(|e| Error::Crypto(format!("Recovery failed: {}. Shares might be corrupted.", e)))?;

        Ok(secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_recover() {
        // The classic treasure map scenario
        let secret = b"the treasure is buried at skull island";
        let sharing = SecretSharing::new(3, 5);

        let shares = sharing.split(secret).unwrap();
        assert_eq!(shares.len(), 5);

        // Recover with exactly threshold shares (3 of 5)
        let recovered = sharing.reconstruct(&shares[0..3]).unwrap();
        assert_eq!(recovered, secret);

        // Recover with more than threshold (all 5)
        let recovered = sharing.reconstruct(&shares[0..5]).unwrap();
        assert_eq!(recovered, secret);

        // Different subset of 3 should also work
        let recovered = sharing.reconstruct(&shares[2..5]).unwrap();
        assert_eq!(recovered, secret);
    }

    #[test]
    fn test_insufficient_shares() {
        // Can't reconstruct with fewer shares than the threshold
        // Maths doesn't work that way. Sorry.
        let secret = b"not enough pieces of eight";
        let sharing = SecretSharing::new(3, 5);

        let shares = sharing.split(secret).unwrap();

        // Try with only 2 shares when we need 3
        let result = sharing.reconstruct(&shares[0..2]);
        assert!(result.is_err());

        // The error should tell us what we need vs what we have
        if let Err(Error::InsufficientShares { need, have }) = result {
            assert_eq!(need, 3);
            assert_eq!(have, 2);
        } else {
            panic!("Expected InsufficientShares error");
        }
    }
}

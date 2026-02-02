//! Cargo - The encrypted treasure chest
//!
//! This module handles all the cryptographic heavy liftin' fer storing
//! yer sensitive files. Uses AES-256-GCM because we ain't landlubbers
//! who use DES or some other ancient cipher from the 1970s.
//!
//! The key derivation uses Argon2 because bcrypt is fer boomers and
//! PBKDF2 is fer people who think 10,000 iterations is "enough".

use std::io::{Read, Write};
use std::path::Path;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use rand::RngCore;

use crate::error::{Error, Result};

// Crypto constants - don't touch these unless ye know what yer doin'
const SALT_LEN: usize = 32;   // 256 bits of salt, plenty salty
const NONCE_LEN: usize = 12;  // 96 bits as AES-GCM demands
const KEY_LEN: usize = 32;    // 256 bits because we're not messin' around

/// The treasure chest itself. Holds the derived encryption key.
///
/// Note: The key is derived from yer password using Argon2. If ye forget
/// the password, the treasure is lost forever. No "forgot password" here, matey.
pub struct Cargo {
    key: [u8; KEY_LEN],
}

impl Cargo {
    /// Create a new cargo vault with the given password
    ///
    /// This generates a fresh salt and derives a key. Use this when
    /// creating NEW encrypted cargo, not when opening existing stuff.
    pub fn new(password: &str) -> Result<Self> {
        let mut salt = [0u8; SALT_LEN];
        rand::thread_rng().fill_bytes(&mut salt);

        let key = Self::derive_key(password, &salt)?;
        Ok(Self { key })
    }

    /// Open an existing cargo vault with password and known salt
    ///
    /// Use this when ye already have encrypted cargo and need to decrypt it.
    /// The salt should have been stored alongside the ciphertext.
    pub fn open(password: &str, salt: &[u8]) -> Result<Self> {
        let key = Self::derive_key(password, salt)?;
        Ok(Self { key })
    }

    /// Seal the cargo - encrypt data with AES-256-GCM
    ///
    /// Returns nonce + ciphertext. The nonce is prepended so ye don't
    /// have to track it seperately. Very convienent, if I do say so meself.
    pub fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| Error::Crypto(e.to_string()))?;

        // Generate a random nonce. NEVER reuse nonces with the same key!
        // That would be catastrophic. Like, "your crypto is broken" catastrophic.
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| Error::Crypto(e.to_string()))?;

        // Prepend nonce to ciphertext so it's all one blob
        let mut result = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Unseal the cargo - decrypt that precious booty
    ///
    /// Expects nonce + ciphertext format (as produced by seal()).
    /// If decryption fails, either the password is wrong or the data
    /// has been tampered with. Either way, no treasure fer ye.
    pub fn unseal(&self, sealed: &[u8]) -> Result<Vec<u8>> {
        if sealed.len() < NONCE_LEN {
            return Err(Error::Crypto("Data too short - this ain't valid cargo!".into()));
        }

        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| Error::Crypto(e.to_string()))?;

        let nonce = Nonce::from_slice(&sealed[..NONCE_LEN]);
        let ciphertext = &sealed[NONCE_LEN..];

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| Error::Crypto(e.to_string()))
    }

    /// Pack multiple files into a single encrypted archive
    ///
    /// Takes a list of paths, reads 'em all, bundles 'em together,
    /// and encrypts the whole lot. Like a pirate's treasure chest
    /// but with better security than a padlock.
    pub fn pack_files(&self, paths: &[impl AsRef<Path>]) -> Result<Vec<u8>> {
        let mut archive = Vec::new();

        for path in paths {
            let path = path.as_ref();

            // Skip files that don't exist - we're not gonna panic over it
            if !path.exists() {
                continue;
            }

            if path.is_file() {
                let mut file = std::fs::File::open(path)?;
                let mut contents = Vec::new();
                file.read_to_end(&mut contents)?;

                let entry = CargoEntry {
                    path: path.to_string_lossy().into_owned(),
                    is_dir: false,
                    data: contents,
                };

                // Length-prefixed encoding so we can unpack later
                let encoded = serde_json::to_vec(&entry)?;
                archive.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
                archive.extend_from_slice(&encoded);
            } else if path.is_dir() {
                self.pack_directory(path, &mut archive)?;
            }
        }

        self.seal(&archive)
    }

    /// Recursively pack a directory's contents
    ///
    /// Walks the directory tree and adds all files to the archive.
    /// Clippy complains about &self only being used in recursion but
    /// we're keepin' it fer API consistency. Fight me, clippy.
    #[allow(clippy::only_used_in_recursion)]
    fn pack_directory(&self, dir: &Path, archive: &mut Vec<u8>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let mut file = std::fs::File::open(&path)?;
                let mut contents = Vec::new();
                file.read_to_end(&mut contents)?;

                let entry = CargoEntry {
                    path: path.to_string_lossy().into_owned(),
                    is_dir: false,
                    data: contents,
                };

                let encoded = serde_json::to_vec(&entry)?;
                archive.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
                archive.extend_from_slice(&encoded);
            } else if path.is_dir() {
                // Yo dawg, I heard ye like recursion...
                self.pack_directory(&path, archive)?;
            }
        }

        Ok(())
    }

    /// Unpack encrypted cargo to a directory
    ///
    /// Decrypts the archive and extracts all files to output_dir.
    /// Returns list of extracted file paths so ye know what ye got.
    pub fn unpack(&self, sealed: &[u8], output_dir: &Path) -> Result<Vec<String>> {
        let archive = self.unseal(sealed)?;
        let mut extracted = Vec::new();
        let mut cursor = 0;

        // Parse the length-prefixed entries
        while cursor + 4 <= archive.len() {
            let len_bytes: [u8; 4] = archive[cursor..cursor + 4]
                .try_into()
                .map_err(|_| Error::Crypto("Corrupted archive: invalid length field".into()))?;
            let len = u32::from_le_bytes(len_bytes) as usize;
            cursor += 4;

            // Sanity check - don't read past the end
            if cursor + len > archive.len() {
                break;
            }

            let entry: CargoEntry = serde_json::from_slice(&archive[cursor..cursor + len])?;
            cursor += len;

            // Write the file to disk
            let out_path = output_dir.join(&entry.path);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut file = std::fs::File::create(&out_path)?;
            file.write_all(&entry.data)?;

            extracted.push(entry.path);
        }

        Ok(extracted)
    }

    /// Derive encryption key from password using Argon2
    ///
    /// Argon2 is memory-hard which means it's expensive to brute force.
    /// Take that, GPU crackers! This ain't bcrypt's grandma's hash function.
    fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LEN]> {
        let mut key = [0u8; KEY_LEN];
        Argon2::default()
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| Error::Crypto(e.to_string()))?;
        Ok(key)
    }
}

/// Internal struct fer storing file entries in the archive
///
/// is_dir is currently unused but kept fer potential future use
/// (like maybe restoring empty directories or somethin')
#[derive(serde::Serialize, serde::Deserialize)]
struct CargoEntry {
    path: String,
    #[allow(dead_code)]  // Might use this later, who knows
    is_dir: bool,
    data: Vec<u8>,
}

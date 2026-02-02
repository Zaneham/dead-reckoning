//! Error types fer when things go horribly wrong
//!
//! Every voyage has its storms. These be the ways this ship can sink.

use thiserror::Error;

/// All the ways this cursed software can fail ye
///
/// Some of these are recoverable, some ain't. Such is life on the high seas.
#[derive(Error, Debug)]
pub enum Error {
    /// The ship's manifest (config) be corrupted or invalid
    #[error("Config error: {0}")]
    Config(String),

    /// Ye haven't even started yer voyage yet, landlubber!
    /// Run `dead-reckoning init` to begin
    #[error("No config found. Run `dead-reckoning init` first.")]
    NoConfig,

    /// The captain didn't check in. The kraken has been released.
    /// This is... kind of the whole point of the software, innit?
    #[error("Watch expired {0} ago. The fleet has been notified.")]
    WatchExpired(String),

    /// Someone used the duress code. They're in trouble but pretendin' they ain't.
    /// Fleet has been silently alerted. Godspeed.
    #[error("Duress detected. Silent alert sent to fleet.")]
    DuressDetected,

    /// Wrong passphrase, ye scallywag. Try again.
    #[error("Invalid check-in code")]
    InvalidCode,

    /// The crypto gremlins struck. AES didn't AES, Argon didn't Argon.
    /// Probably means corrupted data or wrong password.
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Ye need more pieces of the treasure map to reconstruct the secret!
    /// Shamir's scheme requires a minimum threshold. Math is unforgiving.
    #[error("Not enough captains to reconstruct secret. Need {need}, have {have}.")]
    InsufficientShares { need: u8, have: u8 },

    /// Failed to send the message in a bottle. Email bounced, webhook 404'd, etc.
    #[error("Notification failed: {0}")]
    Notification(String),

    /// Generic IO error - file not found, permission denied, disk full, the usual
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization failed. The data be cursed.
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// TOML parsing failed. Check yer config file fer typos, matey.
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Result type fer this crate. Either ye succeed or ye get an Error.
/// There is no try, only do or do not. Wait, wrong franchise.
pub type Result<T> = std::result::Result<T, Error>;

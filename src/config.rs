//! Config - Where we keep the ship's manifest
//!
//! All the configuration fer Dead Reckoning lives here. It's like
//! the captain's log but fer software settings. Very organized.
//! Very nautical. Uses TOML because JSON is fer web developers
//! and YAML is fer people who enjoy sufferin'.
//!
//! The config includes:
//! - Watch settings (check-in interval, grace period, hashed passphrases)
//! - Fleet settings (trusted contacts, Shamir threshold)
//! - SMTP settings (fer sendin' emails like it's 1995)
//! - Cargo settings (encrypted files to release)
//! - Scuttle settings (files to destroy in an emergency)

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// The main config struct - holds everything
///
/// This is the top-level configuration. All the other config structs
/// hang off this one like barnacles on a ship's hull. But useful barnacles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Watch configuration (the dead man's switch itself)
    pub watch: WatchConfig,
    /// Fleet configuration (yer trusted contacts)
    pub fleet: FleetConfig,
    /// SMTP configuration (fer email, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smtp: Option<SmtpConfig>,
    /// Cargo configuration (encrypted files to release, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo: Option<CargoConfig>,
    /// Scuttle configuration (files to destroy, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scuttle: Option<ScuttleConfig>,
}

/// SMTP configuration fer sendin' emails
///
/// Email: the cockroach of communication protocols. Will survive
/// the nuclear apocalypse. Still requires authentication though.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    /// SMTP server hostname (e.g., "smtp.gmail.com")
    pub host: String,
    /// SMTP port (587 for STARTTLS, 465 for SSL, 25 fer the reckless)
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    /// Username for authentication (usually yer email address)
    pub username: String,
    /// Password for authentication (use an app password, not yer main one!)
    pub password: String,
    /// From address for outgoing mail
    pub from: String,
    /// Use STARTTLS (true) or implicit TLS (false)
    #[serde(default = "default_true")]
    pub starttls: bool,
}

/// Default SMTP port is 587 (STARTTLS)
fn default_smtp_port() -> u16 {
    587
}

/// Watch configuration - the heart of the dead man's switch
///
/// This controls when ye need to check in, how much grace period ye get,
/// and what the secret passphrases are (hashed, not plaintext, we're not savages).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    /// Check-in interval (e.g., "24h", "12h", "7d")
    /// How often ye need to prove yer still alive
    pub interval: String,
    /// Grace period before alerting fleet (e.g., "6h")
    /// Extra time after interval expires before we release the kraken
    pub grace_period: String,
    /// Normal check-in phrase (hashed, not stored plaintext)
    /// SHA-256 of the normalized passphrase
    #[serde(default)]
    pub safe_word_hash: String,
    /// Duress phrase (hashed) - looks normal but triggers silent alert
    /// Use this when someone's forcin' ye to check in
    #[serde(default)]
    pub duress_word_hash: String,
    /// Last successful check-in timestamp
    /// None means ye've never checked in (tsk tsk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checkin: Option<DateTime<Utc>>,
}

/// Fleet configuration - yer trusted contacts
///
/// These are the people who get notified when ye go silent.
/// Choose wisely. Choose people who'll actually do somethin' useful.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetConfig {
    /// List of trusted contacts (captains)
    #[serde(default)]
    pub captains: Vec<Captain>,
    /// How many captains needed to reconstruct secrets (Shamir threshold)
    /// If ye have 5 captains and threshold is 3, any 3 can reconstruct the secret
    #[serde(default = "default_threshold")]
    pub threshold: u8,
}

/// Default threshold is 2 (need at least 2 captains to agree)
fn default_threshold() -> u8 {
    2
}

/// A single trusted contact (captain)
///
/// Each captain has a name and a way to contact them.
/// We call them captains because in this metaphor, everyone's
/// a captain of their own ship. Very democratic. Very pirate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Captain {
    /// Human-readable name (fer yer reference)
    pub name: String,
    /// How to contact this captain
    pub contact: ContactMethod,
}

/// How to contact a captain
///
/// Multiple options because not everyone uses email and some people
/// are paranoid enough to only use encrypted messengers. Fair enough.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ContactMethod {
    /// Email address (the classic)
    Email(String),
    /// Signal phone number (end-to-end encrypted, very cloak-and-dagger)
    Signal(String),
    /// Matrix room ID (federated, fer the decentralization enthusiasts)
    Matrix(String),
    /// Telegram chat ID (popular, reasonably secure)
    Telegram(String),
    /// Webhook URL (fer the DevOps pirates)
    Webhook(String),
}

impl std::fmt::Display for ContactMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Email(e) => write!(f, "email:{}", e),
            Self::Signal(s) => write!(f, "signal:{}", s),
            Self::Matrix(m) => write!(f, "matrix:{}", m),
            Self::Telegram(t) => write!(f, "telegram:{}", t),
            Self::Webhook(w) => write!(f, "webhook:{}", w),
        }
    }
}

/// Cargo configuration - the treasure chest
///
/// Files to encrypt and release to the fleet when the watch expires.
/// Think of it as yer final message in a bottle. Make it count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoConfig {
    /// Files/directories to include in the encrypted cargo
    #[serde(default)]
    pub manifest: Vec<PathBuf>,
    /// Path to the encrypted cargo file (once created)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chest_path: Option<PathBuf>,
    /// Whether to release cargo on watch expiry (usually true, that's the point)
    #[serde(default = "default_true")]
    pub release_on_expiry: bool,
    /// Also release to public endpoints? (use with caution, or don't, yer choice)
    #[serde(default)]
    pub public_release: bool,
}

/// Default to true fer boolean options that should usually be on
fn default_true() -> bool {
    true
}

/// Scuttle configuration - fer when ye need to sink the evidence
///
/// Paths to securely delete when scuttle is triggered.
/// Choose carefully. There's no undo. Gone means gone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScuttleConfig {
    /// Paths to securely delete on scuttle
    #[serde(default)]
    pub paths: Vec<String>,
    /// Wipe method: "zero", "random", "dod", "gutmann"
    /// Higher methods are more thorough but slower
    #[serde(default = "default_wipe_method")]
    pub method: String,
}

/// Default wipe method is DoD 5220.22-M (3-pass)
/// Good balance of security and speed. Government approved!
fn default_wipe_method() -> String {
    "dod".into()
}

impl Default for Config {
    /// Create a default config
    ///
    /// Sensible defaults fer someone who hasn't configured anything yet.
    /// 24h interval, 6h grace, no contacts, no cargo, no scuttle.
    /// Basically useless until ye configure it, but at least it won't crash.
    fn default() -> Self {
        Self {
            watch: WatchConfig {
                interval: "24h".into(),
                grace_period: "6h".into(),
                safe_word_hash: String::new(),
                duress_word_hash: String::new(),
                last_checkin: None,
            },
            fleet: FleetConfig {
                captains: Vec::new(),
                threshold: 2,
            },
            smtp: None,
            cargo: None,
            scuttle: None,
        }
    }
}

impl Config {
    /// Get the default config directory
    ///
    /// Uses the platform-appropriate location. On Windows that's
    /// somewhere in AppData, on Linux it's ~/.config, on Mac it's
    /// ~/Library/something. Very civilized.
    pub fn config_dir() -> PathBuf {
        directories::ProjectDirs::from("com", "deadreckoning", "dead-reckoning")
            .map(|p| p.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Get the default config file path
    ///
    /// It's called voyage.toml because we're pirates and
    /// "config.toml" is borin'. Arr.
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("voyage.toml")
    }

    /// Load config from the default location
    ///
    /// If there's no config file, returns NoConfig error.
    /// That means ye need to run `dead-reckoning init` first, ye landlubber.
    pub fn load() -> Result<Self> {
        Self::load_from(&Self::config_path())
    }

    /// Load config from a specific path
    ///
    /// Useful fer testin' or if ye want to keep multiple configs.
    /// Maybe one fer work and one fer personal? Live yer best life.
    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(crate::error::Error::NoConfig);
        }
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to the default location
    ///
    /// Creates the directory if it doesn't exist. Very helpful.
    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::config_path())
    }

    /// Save config to a specific path
    ///
    /// Creates parent directories if needed. We're thorough like that.
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::Error::Config(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

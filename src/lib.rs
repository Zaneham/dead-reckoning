//! # Dead Reckoning - A dead man's switch for the digital age
//!
//! Ahoy! When the captain goes silent, the fleet gets notified.
//!
//! This here be a tool for those who sail dangerous waters - journalists,
//! activists, whistleblowers, and anyone else who might dissapear without
//! warning. If ye stop checkin' in, yer trusted crew gets the message.
//!
//! ## Features
//!
//! - **Watch**: Periodic check-in system. Miss it and the countdown begins, savvy?
//! - **Fleet**: Yer trusted contacts - they get notified when things go south
//! - **Cargo**: Encrypted treasure released to the fleet (AES-256-GCM, not some landlubber cipher)
//! - **Lookout**: Watches fer suspicious behavoir in yer check-in patterns
//! - **Scuttle**: When ye need to sink the evidence. Davy Jones style.
//! - **Sentinel**: Scans fer spyware and government barnacles on yer hull
//! - **Shares**: Shamir's Secret Sharing - split the treasure map among the crew
//!
//! ## A Word of Caution
//!
//! This software deals with mortality. Not in a fun pirate way where ye come
//! back in the sequel, but the real kind. Use it wisely, test it thoroughly,
//! and fer the love of all that's holy, don't forget yer check-in phrase.

pub mod cargo;      // The encrypted treasure chest
pub mod config;     // Where we keep the ship's manifest
pub mod daemon;     // The eternal watchman (never sleeps, like a good lookout)
pub mod error;      // When things go wrong (and they will, matey)
pub mod fleet;      // Yer trusted crew of contacts
pub mod lookout;    // Watches fer suspicious behavor
pub mod scuttle;    // Sink the ship! DELETE EVERYTHING!
pub mod sentinel;   // Scans fer spyware, RATs, and other digital vermin
pub mod watch;      // The dead man's switch itself

pub use config::Config;
pub use error::{Error, Result};

/// Version string - fer knowin' which version of this cursed software yer runnin'
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// ASCII art banner - every good CLI needs one, it's the law
pub const BANNER: &str = r#"
    ⚓ DEAD RECKONING ⚓

    "When the captain goes silent,
     the fleet gets the message."

"#;

/// The Kraken! Released when watch expires.
/// Fun fact: krakens are public domain, unlike certain mouse-eared corporations.
pub const KRAKEN: &str = r#"
       ___
    .-'   `'.
   /         \
  |  (o) (o)  |
  |     ^     |
  |  '-----'  |
   \  `===`  /
    '-.....-'
   /|       |\
  / |  |||  | \
 /  |  |||  |  \
    |__|||__|
     KRAKEN RELEASED
"#;

/// Calm seas - all clear, the captain lives another day
pub const CLEAR_SKIES: &str = r#"
      ~  ~
   ~        ~
  ~    ⛵    ~
 ~~~~~~~~~~~~~~~~
   Seas are calm
"#;

/// Storm's a-brewin' - check-in due soon, ye scurvy dog
pub const STORM_BREWING: &str = r#"
     ⛈️  ⛈️
   ~~~~~~~~~
  ~~~~~~~~~~~
    ⚠️ STORM
    BREWING
"#;

//! The Watch - The heart of the dead man's switch
//!
//! This here be the core mechanism, savvy? The captain checks in with a
//! secret phrase, and if they don't... well, let's just say Davy Jones
//! gets a new crewmate and the fleet gets the news.
//!
//! Fun fact: The duress word feature means ye can pretend everything's fine
//! while secretly sendin' an SOS. Very sneaky. Very pirate. Quite clever
//! really, like hiding a submarine in a duck pond.

use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};

use crate::config::Config;
use crate::error::Result;

/// The Watch itself - keeper of the check-in schedule
///
/// This struct is like the ship's hourglass, but instead of sand
/// it's countin' down to either "all clear" or "RELEASE THE KRAKEN".
/// Rather like a very aggressive egg timer, if you think about it.
pub struct Watch {
    config: Config,
}

/// What happened when ye tried to check in, matey?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckInResult {
    /// The passphrase be correct! Live another day, ye scurvy dog.
    Accepted,
    /// Duress detected - ye said the secret panic word. Help is comin' silently.
    /// Like a very concerned ninja.
    Duress,
    /// Wrong phrase, landlubber! Try again or walk the plank.
    Invalid,
}

/// How's the watch lookin', captain?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchStatus {
    /// All clear, within check-in window. Smooth sailin', cup of tea time.
    Secure,
    /// Check-in due soon. Storm clouds gatherin' on the horizon, bit worrying really.
    Warning,
    /// Grace period active. THIS IS NOT A DRILL. Put down the biscuits.
    Critical,
    /// Watch expired. The kraken stirs. Fleet gets notified. Terribly sorry about this.
    Expired,
}

impl Watch {
    /// Create a new watch from yer config
    ///
    /// This don't start the countdown, it just prepares the mechanism.
    /// Like loadin' a flintlock but not pullin' the trigger. Yet.
    /// Very suspenseful. Very nautical.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Check in with yer passphrase
    ///
    /// Use the safe word to reset the timer and live another day.
    /// Use the duress word if someone's got a cutlass to yer throat
    /// and ye need to look like yer cooperatin' while secretly alertin' the fleet.
    ///
    /// Get it wrong and... well, try again I suppose. We ain't monsters.
    /// Just mildly disappointed, like a British parent.
    pub fn check_in(&mut self, phrase: &str) -> Result<CheckInResult> {
        let hash = Self::hash_phrase(phrase);

        // Duress word check - silent alarm, looks normal to the bad guys
        if hash == self.config.watch.duress_word_hash {
            return Ok(CheckInResult::Duress);
        }

        // Normal safe word - all clear, reset the watch
        if hash == self.config.watch.safe_word_hash {
            self.config.watch.last_checkin = Some(Utc::now());
            self.config.save()?;
            return Ok(CheckInResult::Accepted);
        }

        // Neither word matches - ye absolute muppet
        Ok(CheckInResult::Invalid)
    }

    /// Check the current status of the watch
    ///
    /// Returns whether yer in calm seas or about to release the kraken.
    /// This don't modify anything, just peeks at the hourglass like a
    /// nosy neighbor peekin' through the curtains.
    pub fn status(&self) -> WatchStatus {
        let Some(last) = self.config.watch.last_checkin else {
            // Never checked in? That's expired, matey. Bit of a cock-up really.
            return WatchStatus::Expired;
        };

        let interval = Self::parse_duration(&self.config.watch.interval)
            .unwrap_or(Duration::hours(24));
        let grace = Self::parse_duration(&self.config.watch.grace_period)
            .unwrap_or(Duration::hours(6));

        let now = Utc::now();
        let elapsed = now - last;

        if elapsed < interval - Duration::hours(2) {
            // Plenty of time left. Put the kettle on.
            WatchStatus::Secure
        } else if elapsed < interval {
            // Getting close. Better check in soon, savvy?
            WatchStatus::Warning
        } else if elapsed < interval + grace {
            // In the grace period. THIS BE SERIOUS. Terribly serious.
            WatchStatus::Critical
        } else {
            // Time's up. The kraken awakens. How unfortunate.
            WatchStatus::Expired
        }
    }

    /// How long until the watch expires?
    ///
    /// Returns None if already expired (too late, matey, awfully sorry)
    /// Returns Some(duration) if ye still have time to save yerself
    pub fn time_until_expiry(&self) -> Option<Duration> {
        let last = self.config.watch.last_checkin?;
        let interval = Self::parse_duration(&self.config.watch.interval)
            .unwrap_or(Duration::hours(24));
        let grace = Self::parse_duration(&self.config.watch.grace_period)
            .unwrap_or(Duration::hours(6));

        let expiry = last + interval + grace;
        let now = Utc::now();

        if expiry > now {
            Some(expiry - now)
        } else {
            None // Already expired. Yer goose is cooked. Medium rare.
        }
    }

    /// Set the safe and duress phrases
    ///
    /// The safe word resets the timer. The duress word triggers a silent alert.
    /// Choose somethin' ye can remember but others can't guess.
    /// "password123" is not a good choice. Neither is "arr". Nor "qwerty".
    /// Honestly the bar is quite low here and people still trip over it.
    pub fn set_phrases(config: &mut Config, safe_word: &str, duress_word: &str) {
        config.watch.safe_word_hash = Self::hash_phrase(safe_word);
        config.watch.duress_word_hash = Self::hash_phrase(duress_word);
    }

    /// Hash a phrase using SHA-256
    ///
    /// We normalize first (trim whitespace, lowercase) so "Hello World"
    /// and "hello world  " both work. We're strict about security but
    /// forgivin' about typos. British politeness meets pirate paranoia.
    fn hash_phrase(phrase: &str) -> String {
        let normalized = phrase.trim().to_lowercase();
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Parse a duration string like "24h" or "7d"
    ///
    /// Supports:
    /// - "Xh" for hours (e.g., "24h" = 24 hours)
    /// - "Xd" for days (e.g., "7d" = 7 days)
    /// - "Xm" for minutes (e.g., "30m" = 30 minutes, fer the terrifically paranoid)
    fn parse_duration(s: &str) -> Option<Duration> {
        let s = s.trim().to_lowercase();

        if let Some(h) = s.strip_suffix('h') {
            return h.parse().ok().map(Duration::hours);
        }
        if let Some(d) = s.strip_suffix('d') {
            return d.parse().ok().map(Duration::days);
        }
        if let Some(m) = s.strip_suffix('m') {
            return m.parse().ok().map(Duration::minutes);
        }

        None // Unrecognized format. We don't speak that language, terribly sorry.
    }
}

/// Format a duration fer human consumption
///
/// Because "86400 seconds" is techincally correct but nobody wants
/// to do maths when they're stressed about deadlines. Or dead men. Or switches.
/// Or the ever-present spectre of mortality lurkin' in the corner like
/// an unwanted house guest who won't take the hint.
pub fn format_duration(d: Duration) -> String {
    let total_secs = d.num_seconds();

    if total_secs < 0 {
        return "expired".into(); // Too late, matey. Chin up though.
    }

    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let mins = (total_secs % 3600) / 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

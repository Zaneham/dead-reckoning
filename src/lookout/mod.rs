//! Lookout - Watches fer suspicious behavor in check-in patterns
//!
//! This module is like havin' a very observant parrot on yer shoulder,
//! except instead of repeatin' swear words, it notices when yer check-in
//! patterns are off.
//!
//! If ye normally check in at 9am every day and suddenly ye check in at
//! 3am, that's a bit odd, innit? The lookout tracks these patterns and
//! raises the alarm when things don't add up.
//!
//! Useful fer detecting:
//! - Coerced check-ins (someone forcin' ye to check in at gunpoint)
//! - Compromised accounts (someone else checkin' in fer ye)
//! - Time zone kidnapping (ye've been moved to a different continent)
//! - General weirdness

use chrono::{DateTime, Datelike, Timelike, Utc};

use crate::config::Config;

/// The Lookout - pattern analysis fer paranoid pirates
///
/// Keeps track of when ye check in and notices when somethin's amiss.
/// Like a very suspicious grandmother who notices ye've changed yer haircut.
pub struct Lookout {
    config: Config,
    checkin_history: Vec<CheckinEvent>,
}

/// Record of a single check-in event
///
/// We track the time, hour of day, and day of week so we can spot
/// patterns. Humans are creatures of habit. Deviations are suspicious.
#[derive(Debug, Clone)]
pub struct CheckinEvent {
    pub timestamp: DateTime<Utc>,
    pub hour_of_day: u32,
    pub day_of_week: u32,
}

/// How suspicious is this check-in?
///
/// Ranges from "perfectly normal" to "somethin' ain't right here".
/// The higher the suspicion, the more likely someone's being coerced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuspicionLevel {
    /// Everything looks normal. Carry on.
    Normal,
    /// Slightly off. Worth notin' but not alarmin'.
    Unusual,
    /// This is gettin' weird. Maybe investigate?
    Suspicious,
    /// Somethin' is very wrong. Red flags everywhere.
    HighlyAnomalous,
}

impl Lookout {
    /// Create a new lookout
    ///
    /// Fresh lookout, no history yet. Like a new detective on their
    /// first day. Eager but inexperienced.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            checkin_history: Vec::new(),
        }
    }

    /// Record a check-in event
    ///
    /// Adds this check-in to our history so we can build up a pattern.
    /// We keep the last 30 check-ins because that's enough to establish
    /// patterns without usin' too much memory. Very efficient.
    pub fn record_checkin(&mut self, timestamp: DateTime<Utc>) {
        self.checkin_history.push(CheckinEvent {
            timestamp,
            hour_of_day: timestamp.hour(),
            day_of_week: timestamp.weekday().num_days_from_monday(),
        });

        // Keep last 30 check-ins for pattern analysis
        // Like a goldfish but with slightly better memory
        if self.checkin_history.len() > 30 {
            self.checkin_history.remove(0);
        }
    }

    /// Analyze a check-in fer anomalies
    ///
    /// Compares the given timestamp against historical patterns and
    /// returns a suspicion level. Higher is worse. Like golf scores
    /// but fer paranoia.
    pub fn analyze_checkin(&self, timestamp: DateTime<Utc>) -> SuspicionLevel {
        if self.checkin_history.len() < 5 {
            // Not enough history to establish pattern
            // Can't be suspicious of somethin' we don't understand yet
            return SuspicionLevel::Normal;
        }

        let mut anomaly_score = 0;

        // Check time of day
        let current_hour = timestamp.hour();
        let avg_hour = self.average_checkin_hour();
        let hour_diff = (current_hour as i32 - avg_hour as i32).abs();

        if hour_diff > 8 {
            // More than 8 hours off from usual? That's a paddlin'.
            anomaly_score += 2;
        } else if hour_diff > 4 {
            // 4-8 hours off? Bit weird but maybe ye had a lie-in.
            anomaly_score += 1;
        }

        // Check if it's way too early (might be under pressure)
        let expected_next = self.expected_next_checkin();
        if let Some(expected) = expected_next {
            let time_until = expected - timestamp;
            if time_until.num_hours() > 12 {
                // Checking in way before needed? That's suspicious.
                // Either yer very organized or someone's got a gun to yer head.
                anomaly_score += 1;
            }
        }

        // Convert score to suspicion level
        match anomaly_score {
            0 => SuspicionLevel::Normal,
            1 => SuspicionLevel::Unusual,
            2 => SuspicionLevel::Suspicious,
            _ => SuspicionLevel::HighlyAnomalous,
        }
    }

    /// Calculate the average hour of day fer check-ins
    ///
    /// If ye usually check in at 9am, this returns 9.
    /// Simple maths but useful fer pattern detection.
    fn average_checkin_hour(&self) -> u32 {
        if self.checkin_history.is_empty() {
            return 12; // Default to noon if no history
        }
        let sum: u32 = self.checkin_history.iter().map(|e| e.hour_of_day).sum();
        sum / self.checkin_history.len() as u32
    }

    /// Calculate when the next check-in is expected
    ///
    /// Based on the last check-in and the configured interval.
    /// Returns None if there's no last check-in recorded.
    fn expected_next_checkin(&self) -> Option<DateTime<Utc>> {
        self.config.watch.last_checkin.map(|last| {
            let interval = parse_duration(&self.config.watch.interval)
                .unwrap_or(chrono::Duration::hours(24));
            last + interval
        })
    }
}

/// Parse a duration string like "24h" or "7d"
///
/// Same as the one in watch module but we need it here too.
/// Could probably refactor this into a shared utility but
/// that sounds like work and this is a pirate-themed project.
fn parse_duration(s: &str) -> Option<chrono::Duration> {
    let s = s.trim().to_lowercase();

    if let Some(h) = s.strip_suffix('h') {
        return h.parse().ok().map(chrono::Duration::hours);
    }
    if let Some(d) = s.strip_suffix('d') {
        return d.parse().ok().map(chrono::Duration::days);
    }

    None
}

impl std::fmt::Display for SuspicionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "Normal"),
            Self::Unusual => write!(f, "Unusual timing"),
            Self::Suspicious => write!(f, "Suspicious pattern"),
            Self::HighlyAnomalous => write!(f, "HIGHLY ANOMALOUS"),
        }
    }
}

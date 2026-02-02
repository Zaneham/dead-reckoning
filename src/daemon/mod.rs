//! Daemon - The eternal watchman
//!
//! This module runs in the background like a very concerned ghost,
//! constantly checkin' if the captain is still alive (metaphorically)
//! and sendin' out alerts if they're not.
//!
//! It's like havin' a butler who's sole job is to check if ye've died
//! and notify yer solicitor. Very morbid. Very useful.
//!
//! The daemon:
//! - Runs continuously in the background
//! - Periodically checks the watch status
//! - Sends alerts when the watch expires
//! - Handles graceful shutdown (Ctrl+C)
//! - Tries not to be too annoying about it all

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use colored::*;

use crate::config::Config;
use crate::fleet::{DistressSignal, Fleet};
use crate::watch::{Watch, WatchStatus};

/// The Daemon - background watchman extraordinaire
///
/// Sits there quietly, occasionally checkin' the time, ready to
/// release the kraken at a moment's notice. A very responsible
/// piece of software, if a bit gloomy about its purpose in life.
pub struct Daemon {
    config_path: std::path::PathBuf,
    check_interval: Duration,
    running: Arc<AtomicBool>,
    alert_sent: bool,
}

impl Daemon {
    /// Create a new daemon
    ///
    /// Doesn't start runnin' immediately. Ye gotta call run() fer that.
    /// Like a car, ye need to turn the key before it goes anywhere.
    /// Unless it's a Tesla, but that's a different kind of dystopia.
    pub fn new(config_path: impl AsRef<Path>, check_interval_secs: u64) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            check_interval: Duration::from_secs(check_interval_secs),
            running: Arc::new(AtomicBool::new(true)),
            alert_sent: false,
        }
    }

    /// Get a handle to stop the daemon
    ///
    /// Returns an Arc<AtomicBool> that can be set to false to stop
    /// the daemon gracefully. Like a safeword but fer software.
    pub fn stop_handle(&self) -> Arc<AtomicBool> {
        self.running.clone()
    }

    /// Run the daemon loop
    ///
    /// This is where the magic happens. The daemon sits here, checkin'
    /// the watch status periodically, until either:
    /// 1. Someone tells it to stop
    /// 2. The heat death of the universe
    ///
    /// Place yer bets on which comes first.
    pub async fn run(&mut self) -> crate::error::Result<()> {
        eprintln!(
            "{} Daemon started. Monitoring {}",
            "[DAEMON]".cyan(),
            self.config_path.display()
        );
        eprintln!(
            "{} Check interval: {} seconds",
            "[DAEMON]".cyan(),
            self.check_interval.as_secs()
        );

        while self.running.load(Ordering::Relaxed) {
            if let Err(e) = self.check_once().await {
                eprintln!("{} Error during check: {}", "[DAEMON]".red(), e);
            }

            // Sleep in small increments so we can respond to stop signal promptly
            // Like a light sleeper who wakes up when someone coughs
            let mut remaining = self.check_interval;
            while remaining > Duration::ZERO && self.running.load(Ordering::Relaxed) {
                let sleep_time = remaining.min(Duration::from_secs(1));
                tokio::time::sleep(sleep_time).await;
                remaining = remaining.saturating_sub(sleep_time);
            }
        }

        eprintln!("{} Daemon stopped. Cheerio!", "[DAEMON]".yellow());
        Ok(())
    }

    /// Check the watch status once
    ///
    /// This is called periodically by the main loop. Loads the config,
    /// checks the watch, and takes appropriate action. Very responsible.
    async fn check_once(&mut self) -> crate::error::Result<()> {
        let config = match Config::load_from(&self.config_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{} Failed to load config: {}", "[DAEMON]".red(), e);
                return Ok(());
            }
        };

        let watch = Watch::new(config.clone());
        let status = watch.status();
        let now = Utc::now();

        match status {
            WatchStatus::Secure => {
                eprintln!(
                    "{} [{}] Status: SECURE. All is well. Cup of tea time.",
                    "[DAEMON]".green(),
                    now.format("%Y-%m-%d %H:%M:%S")
                );
                self.alert_sent = false; // Reset alert flag
            }

            WatchStatus::Warning => {
                eprintln!(
                    "{} [{}] Status: WARNING. Check-in due soon! Chop chop!",
                    "[DAEMON]".yellow(),
                    now.format("%Y-%m-%d %H:%M:%S")
                );
            }

            WatchStatus::Critical => {
                eprintln!(
                    "{} [{}] Status: CRITICAL. Grace period active! This is gettin' serious!",
                    "[DAEMON]".red().bold(),
                    now.format("%Y-%m-%d %H:%M:%S")
                );
            }

            WatchStatus::Expired => {
                eprintln!(
                    "{} [{}] Status: EXPIRED! The kraken stirs!",
                    "[DAEMON]".red().bold(),
                    now.format("%Y-%m-%d %H:%M:%S")
                );

                if !self.alert_sent {
                    self.send_alerts(&config).await?;
                    self.alert_sent = true;
                } else {
                    eprintln!("{} Alerts already sent, not re-sending. We're not spammers.", "[DAEMON]".yellow());
                }
            }
        }

        Ok(())
    }

    /// Send alerts to the fleet
    ///
    /// This is the big one. The captain hasn't checked in, so we're
    /// releasing the kraken (metaphorically) and notifying everyone.
    async fn send_alerts(&self, config: &Config) -> crate::error::Result<()> {
        eprintln!("{} Sending distress signals to fleet...", "[DAEMON]".red().bold());

        let fleet = Fleet::new(config.clone());
        let signal = DistressSignal {
            message: "DEAD RECKONING: Watch has expired. Captain has not checked in. This is not a drill.".into(),
            cargo_attached: false,
            is_duress: false,
        };

        let results = fleet.send_distress(&signal).await?;

        for result in results {
            if result.success {
                eprintln!("{} Notified: {}", "[DAEMON]".green(), result.captain);
            } else {
                eprintln!(
                    "{} Failed to notify {}: {}",
                    "[DAEMON]".red(),
                    result.captain,
                    result.error.unwrap_or_else(|| "Unknown error. How mysterious.".to_string())
                );
            }
        }

        Ok(())
    }
}

/// Run as a background daemon with signal handling
///
/// This is the main entry point fer runnin' the daemon. Sets up
/// the Ctrl+C handler and starts the main loop. Very user-friendly.
/// Won't keep runnin' if ye ask it to stop. Good software manners.
pub async fn run_daemon(config_path: impl AsRef<Path>, check_interval_secs: u64) -> crate::error::Result<()> {
    let mut daemon = Daemon::new(config_path, check_interval_secs);
    let stop_handle = daemon.stop_handle();

    // Set up Ctrl+C handler
    // Because nobody wants software that ignores yer desperate pleas to stop
    let stop_handle_clone = stop_handle.clone();
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            eprintln!("\n{} Received shutdown signal. Wrappin' things up...", "[DAEMON]".yellow());
            stop_handle_clone.store(false, Ordering::Relaxed);
        }
    });

    daemon.run().await
}

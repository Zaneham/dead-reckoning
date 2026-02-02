use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use colored::*;

use dead_reckoning::{
    cargo::Cargo,
    config::{Captain, Config, ContactMethod, FleetConfig, SmtpConfig, WatchConfig},
    daemon,
    fleet::{shares::SecretSharing, DistressSignal, Fleet},
    lookout::{Lookout, SuspicionLevel},
    scuttle::Scuttle,
    sentinel::{Sentinel, ThreatLevel},
    watch::{format_duration, Watch, WatchStatus},
    BANNER, CLEAR_SKIES, KRAKEN, STORM_BREWING,
};

#[derive(Parser)]
#[command(name = "dead-reckoning")]
#[command(about = "A dead man's switch for the digital age")]
#[command(version)]
struct Cli {
    /// Path to config file
    #[arg(short, long, default_value = "voyage.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new voyage configuration
    Init {
        /// Check-in interval (e.g., "24h", "7d")
        #[arg(short, long, default_value = "24h")]
        interval: String,

        /// Grace period after missed check-in
        #[arg(short, long, default_value = "6h")]
        grace: String,
    },

    /// Check in with your safe word (keeps the watch running)
    Parley {
        /// Your check-in phrase
        phrase: String,
    },

    /// Show current watch status
    Status,

    /// Set your safe and duress phrases
    SetPhrases {
        /// The safe word for normal check-ins
        #[arg(long)]
        safe: String,

        /// The duress word that triggers immediate alert
        #[arg(long)]
        duress: String,
    },

    /// Manage your fleet of trusted contacts
    Fleet {
        #[command(subcommand)]
        action: FleetAction,
    },

    /// Manage encrypted cargo
    Cargo {
        #[command(subcommand)]
        action: CargoAction,
    },

    /// Split a secret using Shamir's Secret Sharing
    Split {
        /// The secret to split
        secret: String,

        /// Minimum shares needed to reconstruct
        #[arg(short, long, default_value = "3")]
        threshold: u8,

        /// Total number of shares to create
        #[arg(short = 'n', long, default_value = "5")]
        total: u8,
    },

    /// Reconstruct a secret from shares
    Reconstruct {
        /// Share files (or raw hex shares)
        shares: Vec<String>,

        /// Threshold for reconstruction
        #[arg(short, long, default_value = "3")]
        threshold: u8,
    },

    /// Emergency: securely delete configured files
    Scuttle {
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },

    /// Manually trigger distress signal (for testing)
    Distress {
        /// Custom message
        #[arg(short, long)]
        message: Option<String>,

        /// Simulate duress condition
        #[arg(long)]
        duress: bool,
    },

    /// Run as a background daemon monitoring the watch
    Daemon {
        /// Check interval in seconds
        #[arg(short, long, default_value = "60")]
        interval: u64,
    },

    /// Configure SMTP for email notifications
    Smtp {
        /// SMTP server hostname
        #[arg(long)]
        host: String,

        /// SMTP port
        #[arg(long, default_value = "587")]
        port: u16,

        /// SMTP username
        #[arg(long)]
        username: String,

        /// SMTP password
        #[arg(long)]
        password: String,

        /// From email address
        #[arg(long)]
        from: String,

        /// Use STARTTLS (default) or implicit TLS
        #[arg(long, default_value = "true")]
        starttls: bool,
    },

    /// Scan system for signs of compromise
    Sentinel {
        /// Run continuously, scanning at interval
        #[arg(short, long)]
        watch: bool,

        /// Scan interval in seconds (for --watch mode)
        #[arg(short, long, default_value = "300")]
        interval: u64,

        /// Don't flag remote access tools (TeamViewer, AnyDesk, etc.)
        #[arg(long)]
        allow_remote: bool,

        /// Don't flag screen capture tools (OBS, etc.)
        #[arg(long)]
        allow_capture: bool,

        /// Trigger alert if threat level >= this (low, medium, high, critical)
        #[arg(long, default_value = "high")]
        alert_threshold: String,
    },
}

#[derive(Subcommand)]
enum FleetAction {
    /// List all captains in the fleet
    List,

    /// Add a captain to the fleet
    Add {
        /// Captain's name
        name: String,

        /// Contact method (email:addr, webhook:url, signal:number, matrix:room, telegram:chat)
        contact: String,
    },

    /// Remove a captain from the fleet
    Remove {
        /// Captain's name
        name: String,
    },

    /// Set the threshold for secret sharing
    Threshold {
        /// Number of captains needed
        count: u8,
    },
}

#[derive(Subcommand)]
enum CargoAction {
    /// Pack files into encrypted cargo
    Pack {
        /// Files to pack
        files: Vec<PathBuf>,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,

        /// Encryption password
        #[arg(short, long)]
        password: String,
    },

    /// Unpack encrypted cargo
    Unpack {
        /// Cargo file
        file: PathBuf,

        /// Output directory
        #[arg(short, long)]
        output: PathBuf,

        /// Decryption password
        #[arg(short, long)]
        password: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { interval, grace } => {
            cmd_init(&cli.config, &interval, &grace)?;
        }

        Commands::Parley { phrase } => {
            cmd_parley(&cli.config, &phrase)?;
        }

        Commands::Status => {
            cmd_status(&cli.config)?;
        }

        Commands::SetPhrases { safe, duress } => {
            cmd_set_phrases(&cli.config, &safe, &duress)?;
        }

        Commands::Fleet { action } => {
            cmd_fleet(&cli.config, action)?;
        }

        Commands::Cargo { action } => {
            cmd_cargo(action)?;
        }

        Commands::Split {
            secret,
            threshold,
            total,
        } => {
            cmd_split(&secret, threshold, total)?;
        }

        Commands::Reconstruct { shares, threshold } => {
            cmd_reconstruct(&shares, threshold)?;
        }

        Commands::Scuttle { force } => {
            cmd_scuttle(&cli.config, force)?;
        }

        Commands::Distress { message, duress } => {
            cmd_distress(&cli.config, message, duress).await?;
        }

        Commands::Daemon { interval } => {
            cmd_daemon(&cli.config, interval).await?;
        }

        Commands::Smtp {
            host,
            port,
            username,
            password,
            from,
            starttls,
        } => {
            cmd_smtp(&cli.config, host, port, username, password, from, starttls)?;
        }

        Commands::Sentinel {
            watch,
            interval,
            allow_remote,
            allow_capture,
            alert_threshold,
        } => {
            cmd_sentinel(&cli.config, watch, interval, allow_remote, allow_capture, &alert_threshold).await?;
        }
    }

    Ok(())
}

fn cmd_init(config_path: &Path, interval: &str, grace: &str) -> anyhow::Result<()> {
    println!("{}", BANNER.cyan());

    if config_path.exists() {
        eprintln!(
            "{} Config already exists at {}",
            "Warning:".yellow(),
            config_path.display()
        );
        return Ok(());
    }

    let config = Config {
        watch: WatchConfig {
            interval: interval.to_string(),
            grace_period: grace.to_string(),
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
    };

    config.save_to(config_path)?;

    println!("{}", "Voyage initialized!".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. Set your phrases:    dead-reckoning set-phrases --safe \"...\" --duress \"...\"");
    println!("  2. Add your fleet:      dead-reckoning fleet add \"Alice\" email:alice@example.com");
    println!("  3. Start checking in:   dead-reckoning parley \"your safe phrase\"");
    println!();

    Ok(())
}

fn cmd_parley(config_path: &Path, phrase: &str) -> anyhow::Result<()> {
    let mut config = Config::load_from(config_path)?;

    // Check for anomalies first
    let mut lookout = Lookout::new(config.clone());
    let now = chrono::Utc::now();

    if let Some(last) = config.watch.last_checkin {
        lookout.record_checkin(last);
    }

    let suspicion = lookout.analyze_checkin(now);

    let mut watch = Watch::new(config.clone());
    let result = watch.check_in(phrase)?;

    match result {
        dead_reckoning::watch::CheckInResult::Accepted => {
            println!("{}", CLEAR_SKIES.green());
            println!("{}", "Check-in accepted. Seas remain calm.".green().bold());

            // Warn about suspicious behavior even on accepted check-in
            match suspicion {
                SuspicionLevel::Unusual => {
                    println!();
                    println!("{}", "Note: Unusual check-in timing detected.".yellow());
                }
                SuspicionLevel::Suspicious => {
                    println!();
                    println!(
                        "{}",
                        "Warning: Suspicious check-in pattern.".yellow().bold()
                    );
                }
                SuspicionLevel::HighlyAnomalous => {
                    println!();
                    println!(
                        "{}",
                        "ALERT: Highly anomalous behavior detected!".red().bold()
                    );
                }
                _ => {}
            }

            // Reload config to get updated timestamp
            config = Config::load_from(config_path)?;
            let watch = Watch::new(config);
            if let Some(remaining) = watch.time_until_expiry() {
                println!();
                println!("Next check-in due in: {}", format_duration(remaining));
            }
        }

        dead_reckoning::watch::CheckInResult::Duress => {
            // Don't reveal duress detection
            println!("{}", CLEAR_SKIES.green());
            println!("{}", "Check-in accepted.".green());

            // But internally, this would trigger silent alerts
            eprintln!(
                "{}",
                "[INTERNAL: Duress code detected - alerts suppressed for safety]"
                    .red()
                    .dimmed()
            );
        }

        dead_reckoning::watch::CheckInResult::Invalid => {
            println!("{}", "Invalid phrase. Check-in rejected.".red());
        }
    }

    Ok(())
}

fn cmd_status(config_path: &Path) -> anyhow::Result<()> {
    let config = Config::load_from(config_path)?;
    let watch = Watch::new(config.clone());

    let status = watch.status();

    match status {
        WatchStatus::Secure => {
            println!("{}", CLEAR_SKIES.green());
            println!("{}", "Status: SECURE".green().bold());
        }
        WatchStatus::Warning => {
            println!("{}", STORM_BREWING.yellow());
            println!("{}", "Status: CHECK-IN DUE SOON".yellow().bold());
        }
        WatchStatus::Critical => {
            println!("{}", STORM_BREWING.red());
            println!("{}", "Status: CRITICAL - Grace period active!".red().bold());
        }
        WatchStatus::Expired => {
            println!("{}", KRAKEN.red());
            println!("{}", "Status: EXPIRED - Fleet notified!".red().bold());
        }
    }

    if let Some(remaining) = watch.time_until_expiry() {
        println!();
        println!("Time until expiry: {}", format_duration(remaining));
    }

    println!();
    println!("Fleet size: {} captains", config.fleet.captains.len());
    println!("Threshold: {} shares needed", config.fleet.threshold);

    Ok(())
}

fn cmd_set_phrases(config_path: &Path, safe: &str, duress: &str) -> anyhow::Result<()> {
    let mut config = Config::load_from(config_path)?;

    Watch::set_phrases(&mut config, safe, duress);
    config.save_to(config_path)?;

    println!("{}", "Phrases updated.".green());
    println!("Safe phrase hash:   {}...", &config.watch.safe_word_hash[..16]);
    println!(
        "Duress phrase hash: {}...",
        &config.watch.duress_word_hash[..16]
    );

    Ok(())
}

fn cmd_fleet(config_path: &Path, action: FleetAction) -> anyhow::Result<()> {
    let mut config = Config::load_from(config_path)?;

    match action {
        FleetAction::List => {
            println!("{}", "Fleet Roster".cyan().bold());
            println!("============");

            if config.fleet.captains.is_empty() {
                println!("No captains enlisted.");
            } else {
                for (i, captain) in config.fleet.captains.iter().enumerate() {
                    let contact = match &captain.contact {
                        ContactMethod::Email(e) => format!("email: {}", e),
                        ContactMethod::Webhook(w) => format!("webhook: {}", w),
                        ContactMethod::Signal(s) => format!("signal: {}", s),
                        ContactMethod::Matrix(m) => format!("matrix: {}", m),
                        ContactMethod::Telegram(t) => format!("telegram: {}", t),
                    };
                    println!("  {}. {} ({})", i + 1, captain.name, contact);
                }
            }

            println!();
            println!("Threshold: {} captains needed", config.fleet.threshold);
        }

        FleetAction::Add { name, contact } => {
            let contact_method = parse_contact(&contact)?;

            config.fleet.captains.push(Captain {
                name: name.clone(),
                contact: contact_method,
            });

            config.save_to(config_path)?;
            println!("{} enlisted in the fleet.", name.green());
        }

        FleetAction::Remove { name } => {
            let before = config.fleet.captains.len();
            config.fleet.captains.retain(|c| c.name != name);
            let after = config.fleet.captains.len();

            if before == after {
                println!("{} not found in fleet.", name.yellow());
            } else {
                config.save_to(config_path)?;
                println!("{} removed from fleet.", name.red());
            }
        }

        FleetAction::Threshold { count } => {
            config.fleet.threshold = count;
            config.save_to(config_path)?;
            println!("Threshold set to {} captains.", count);
        }
    }

    Ok(())
}

fn cmd_cargo(action: CargoAction) -> anyhow::Result<()> {
    match action {
        CargoAction::Pack {
            files,
            output,
            password,
        } => {
            let cargo = Cargo::new(&password)?;
            let sealed = cargo.pack_files(&files)?;

            std::fs::write(&output, &sealed)?;

            println!(
                "{} Cargo packed: {} ({} bytes)",
                "✓".green(),
                output.display(),
                sealed.len()
            );
        }

        CargoAction::Unpack {
            file,
            output,
            password,
        } => {
            let sealed = std::fs::read(&file)?;

            // For unpacking, we need to extract the salt from the sealed data
            // In this implementation, we regenerate the key from password
            let cargo = Cargo::new(&password)?;
            let extracted = cargo.unpack(&sealed, &output)?;

            println!("{} Cargo unpacked:", "✓".green());
            for path in extracted {
                println!("  - {}", path);
            }
        }
    }

    Ok(())
}

fn cmd_split(secret: &str, threshold: u8, total: u8) -> anyhow::Result<()> {
    let sharing = SecretSharing::new(threshold, total);
    let shares = sharing.split(secret.as_bytes())?;

    println!("{}", "Secret split into shares:".cyan().bold());
    println!("Threshold: {} of {} needed to reconstruct", threshold, total);
    println!();

    for (i, share) in shares.iter().enumerate() {
        println!("Share {}: {}", i + 1, hex::encode(share));
    }

    println!();
    println!(
        "{}",
        "Keep these shares separate and secure!".yellow().bold()
    );

    Ok(())
}

fn cmd_reconstruct(shares: &[String], threshold: u8) -> anyhow::Result<()> {
    let parsed: Vec<Vec<u8>> = shares
        .iter()
        .map(|s| hex::decode(s.trim()))
        .collect::<Result<Vec<_>, _>>()?;

    let sharing = SecretSharing::new(threshold, parsed.len() as u8);
    let secret = sharing.reconstruct(&parsed)?;

    println!("{}", "Secret reconstructed:".green().bold());
    println!("{}", String::from_utf8_lossy(&secret));

    Ok(())
}

fn cmd_scuttle(config_path: &Path, force: bool) -> anyhow::Result<()> {
    let config = Config::load_from(config_path)?;

    let Some(scuttle_config) = config.scuttle else {
        println!("{}", "No scuttle paths configured.".yellow());
        return Ok(());
    };

    if !force {
        println!("{}", KRAKEN.red());
        println!(
            "{}",
            "WARNING: This will PERMANENTLY DELETE the following:".red().bold()
        );
        for path in &scuttle_config.paths {
            println!("  - {}", path);
        }
        println!();
        println!("Run with --force to confirm.");
        return Ok(());
    }

    let scuttle = Scuttle::new(scuttle_config);
    let report = scuttle.scuttle_all()?;

    println!("{}", report);

    Ok(())
}

async fn cmd_distress(
    config_path: &Path,
    message: Option<String>,
    is_duress: bool,
) -> anyhow::Result<()> {
    let config = Config::load_from(config_path)?;
    let fleet = Fleet::new(config);

    let signal = DistressSignal {
        message: message.unwrap_or_else(|| {
            if is_duress {
                "DURESS SIGNAL - Captain may be compromised".into()
            } else {
                "Watch expired - Captain has not checked in".into()
            }
        }),
        cargo_attached: false,
        is_duress,
    };

    println!("{}", "Sending distress signal...".yellow());

    let results = fleet.send_distress(&signal).await?;

    for result in results {
        if result.success {
            println!("  {} {}", "✓".green(), result.captain);
        } else {
            println!(
                "  {} {} - {}",
                "✗".red(),
                result.captain,
                result.error.unwrap_or_default()
            );
        }
    }

    Ok(())
}

fn parse_contact(s: &str) -> anyhow::Result<ContactMethod> {
    let parts: Vec<&str> = s.splitn(2, ':').collect();

    if parts.len() != 2 {
        anyhow::bail!("Invalid contact format. Use: type:value (e.g., email:alice@example.com)");
    }

    let method = match parts[0].to_lowercase().as_str() {
        "email" => ContactMethod::Email(parts[1].to_string()),
        "webhook" => ContactMethod::Webhook(parts[1].to_string()),
        "signal" => ContactMethod::Signal(parts[1].to_string()),
        "matrix" => ContactMethod::Matrix(parts[1].to_string()),
        "telegram" => ContactMethod::Telegram(parts[1].to_string()),
        _ => anyhow::bail!("Unknown contact type: {}", parts[0]),
    };

    Ok(method)
}

async fn cmd_daemon(config_path: &Path, interval_secs: u64) -> anyhow::Result<()> {
    println!("{}", BANNER.cyan());
    println!("{}", "Starting daemon mode...".yellow().bold());
    println!("Config: {}", config_path.display());
    println!("Check interval: {} seconds", interval_secs);
    println!();
    println!("{}", "Press Ctrl+C to stop.".dimmed());
    println!();

    daemon::run_daemon(config_path, interval_secs).await?;

    Ok(())
}

fn cmd_smtp(
    config_path: &Path,
    host: String,
    port: u16,
    username: String,
    password: String,
    from: String,
    starttls: bool,
) -> anyhow::Result<()> {
    let mut config = Config::load_from(config_path)?;

    config.smtp = Some(SmtpConfig {
        host: host.clone(),
        port,
        username,
        password: password.clone(),
        from: from.clone(),
        starttls,
    });

    config.save_to(config_path)?;

    println!("{}", "SMTP configured!".green().bold());
    println!("  Host: {}:{}", host, port);
    println!("  From: {}", from);
    println!("  TLS:  {}", if starttls { "STARTTLS" } else { "Implicit" });
    println!();
    println!(
        "{}",
        "Note: Password stored in config file. Secure the file appropriately."
            .yellow()
    );

    Ok(())
}

async fn cmd_sentinel(
    config_path: &Path,
    watch_mode: bool,
    interval: u64,
    allow_remote: bool,
    allow_capture: bool,
    alert_threshold: &str,
) -> anyhow::Result<()> {
    println!("{}", "=== SENTINEL - System Compromise Detection ===".cyan().bold());
    println!();

    let threshold = match alert_threshold.to_lowercase().as_str() {
        "low" => ThreatLevel::Low,
        "medium" => ThreatLevel::Medium,
        "high" => ThreatLevel::High,
        "critical" => ThreatLevel::Critical,
        _ => ThreatLevel::High,
    };

    let mut sentinel = Sentinel::new();
    sentinel.set_flag_remote_access(!allow_remote);
    sentinel.set_flag_screen_capture(!allow_capture);

    if watch_mode {
        println!("Running in watch mode (interval: {}s)", interval);
        println!("Alert threshold: {}", alert_threshold.to_uppercase());
        println!("Press Ctrl+C to stop.");
        println!();

        loop {
            let report = sentinel.scan();
            print_sentinel_report(&report);

            // Check if we should alert
            if should_alert(&report.overall_threat, &threshold) {
                println!("{}", "THREAT DETECTED - ALERTING FLEET".red().bold());

                if let Ok(config) = Config::load_from(config_path) {
                    let fleet = Fleet::new(config);
                    let signal = DistressSignal {
                        message: format!(
                            "SENTINEL ALERT: {} threat level detected on system. Possible compromise.",
                            report.overall_threat
                        ),
                        cargo_attached: false,
                        is_duress: false,
                    };

                    if let Ok(results) = fleet.send_distress(&signal).await {
                        for result in results {
                            if result.success {
                                println!("  {} Notified: {}", "✓".green(), result.captain);
                            } else {
                                println!("  {} Failed: {}", "✗".red(), result.captain);
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            println!("\n{}\n", "=".repeat(50));
        }
    } else {
        // Single scan
        let report = sentinel.scan();
        print_sentinel_report(&report);

        if should_alert(&report.overall_threat, &threshold) {
            println!();
            println!(
                "{}",
                "WARNING: Threat level exceeds threshold. Consider investigating."
                    .yellow()
                    .bold()
            );
        }
    }

    Ok(())
}

fn print_sentinel_report(report: &dead_reckoning::sentinel::SentinelReport) {
    println!("Scan time: {}", report.scan_time.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Processes scanned: {}", report.process_count);
    println!();

    let threat_color = match report.overall_threat {
        ThreatLevel::Clear => "CLEAR".green(),
        ThreatLevel::Low => "LOW".yellow(),
        ThreatLevel::Medium => "MEDIUM".yellow().bold(),
        ThreatLevel::High => "HIGH".red().bold(),
        ThreatLevel::Critical => "CRITICAL".red().bold().on_white(),
    };

    println!("Overall Threat Level: {}", threat_color);
    println!();

    if report.threats.is_empty() {
        println!("{}", "No threats detected.".green());
    } else {
        println!("{} threat(s) found:", report.threats.len());
        println!("{}", "-".repeat(50));

        for threat in &report.threats {
            let level_str = match threat.level {
                ThreatLevel::Clear => "CLEAR".normal(),
                ThreatLevel::Low => "LOW".yellow(),
                ThreatLevel::Medium => "MEDIUM".yellow().bold(),
                ThreatLevel::High => "HIGH".red(),
                ThreatLevel::Critical => "CRITICAL".red().bold(),
            };

            println!();
            println!("[{}] {}", level_str, threat.category);
            println!("  Name: {}", threat.name);
            println!("  {}", threat.description);
            if let Some(pid) = threat.pid {
                println!("  PID: {}", pid);
            }
            if let Some(path) = &threat.path {
                println!("  Path: {}", path);
            }
        }
    }
}

fn should_alert(current: &ThreatLevel, threshold: &ThreatLevel) -> bool {
    let level_to_num = |l: &ThreatLevel| match l {
        ThreatLevel::Clear => 0,
        ThreatLevel::Low => 1,
        ThreatLevel::Medium => 2,
        ThreatLevel::High => 3,
        ThreatLevel::Critical => 4,
    };

    level_to_num(current) >= level_to_num(threshold)
}

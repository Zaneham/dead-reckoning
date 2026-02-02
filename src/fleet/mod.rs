//! Fleet - Yer trusted crew of contacts
//!
//! When the captain goes silent, the fleet gets the message. This module
//! handles all the ways we can scream into the void hopin' someone hears us.
//!
//! Supported channels:
//! - Email (via SMTP, like it's 1982 but with TLS)
//! - Webhooks (fer the modern pirate with a Discord server)
//! - Signal (end-to-end encrypted, very cloak-and-dagger)
//! - Matrix (federated, fer the decentralization enthusiasts)
//! - Telegram (fer when ye need to reach people quickly, allegedly)
//!
//! It's like havin' a fleet of carrier pigeons, except some of them
//! are encrypted and one of them is just a webhook in a trenchcoat.

pub mod shares;

use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::config::{Captain, Config, ContactMethod, SmtpConfig};
use crate::error::Result;

/// The Fleet - all yer trusted contacts bundled together
///
/// These be the people who'll know somethin's wrong when ye stop respondin'.
/// Choose wisely. Ye wouldn't want to give the treasure map to just anyone.
/// That would be terribly awkward at parties.
pub struct Fleet {
    config: Config,
}

/// The distress signal - what we send when things go south
///
/// Can be a regular "captain's missing" alert or a duress "captain's been
/// captured but is pretendin' everything's fine" alert. The latter is
/// rather more dramatic, like a spy film but with worse catering.
pub struct DistressSignal {
    /// The actual message content
    pub message: String,
    /// Whether encrypted cargo is attached to this message
    pub cargo_attached: bool,
    /// Is this a duress alert? (silent alarm, don't tip off the kidnappers)
    pub is_duress: bool,
}

impl Fleet {
    /// Create a new fleet from config
    ///
    /// The fleet don't do nothin' until ye tell it to send a signal.
    /// It's like a bunch of carrier pigeons sittin' in their cages. Waitin'.
    /// Judgmentally. Pigeons are very judgmental birds, as it turns out.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Get the list of captains (trusted contacts)
    ///
    /// Why "captains"? Because in this metaphor everyone's a captain of
    /// their own ship. Democracy of the seas, if ye will. Very egalitarian.
    /// Marx would approve, probably. Or he'd write a very long book about it.
    pub fn captains(&self) -> &[Captain] {
        &self.config.fleet.captains
    }

    /// Get the threshold fer secret sharing
    ///
    /// This is how many captains need to work together to reconstruct
    /// the secret. It's like needin' 3 out of 5 keys to open the vault.
    /// Very Ocean's Eleven. Very pirate. Very mathematically sound.
    pub fn threshold(&self) -> u8 {
        self.config.fleet.threshold
    }

    /// SEND THE DISTRESS SIGNAL! ALL HANDS ON DECK!
    ///
    /// This attempts to notify every captain in the fleet. Some might
    /// fail (email bounced, webhook 404'd, carrier pigeon got eaten by
    /// a seagull) but we try our best. That's all anyone can ask really.
    pub async fn send_distress(&self, signal: &DistressSignal) -> Result<Vec<NotifyResult>> {
        let mut results = Vec::new();

        for captain in &self.config.fleet.captains {
            let result = self.notify_captain(captain, signal).await;
            results.push(NotifyResult {
                captain: captain.name.clone(),
                success: result.is_ok(),
                error: result.err().map(|e| e.to_string()),
            });
        }

        Ok(results)
    }

    /// Notify a single captain through their preferred method
    ///
    /// Each captain can choose how they want to be bothered when ye die.
    /// Some prefer email, some prefer encrypted messengers. We don't judge.
    /// Well, we judge a little if they choose email. But quietly.
    async fn notify_captain(&self, captain: &Captain, signal: &DistressSignal) -> Result<()> {
        match &captain.contact {
            ContactMethod::Email(addr) => {
                self.send_email(addr, &captain.name, signal).await
            }
            ContactMethod::Webhook(url) => {
                self.send_webhook(url, signal).await
            }
            ContactMethod::Signal(number) => {
                self.send_signal(number, signal).await
            }
            ContactMethod::Matrix(room) => {
                self.send_matrix(room, signal).await
            }
            ContactMethod::Telegram(chat) => {
                self.send_telegram(chat, signal).await
            }
        }
    }

    /// Send an email via SMTP
    ///
    /// The most tried-and-true method of communication. Invented before
    /// the internet was cool. Still works when everything else is down.
    /// Except when Gmail marks it as spam. Then yer absolutely stuffed.
    async fn send_email(&self, addr: &str, name: &str, signal: &DistressSignal) -> Result<()> {
        let Some(smtp_config) = &self.config.smtp else {
            // No SMTP configured - log and succeed (fer testing)
            // In production ye really should set this up, ye absolute walnut
            eprintln!("[EMAIL] SMTP not configured. Would send to {}: {}", addr, signal.message);
            return Ok(());
        };

        let email = build_email(smtp_config, addr, name, signal)?;
        let mailer = build_mailer(smtp_config)?;

        mailer
            .send(email)
            .await
            .map_err(|e| crate::error::Error::Notification(format!("SMTP error: {}", e)))?;

        Ok(())
    }

    /// Send via webhook
    ///
    /// POST a JSON payload to a URL. Works great with Discord, Slack,
    /// and any other chat system that supports incoming webhooks.
    /// Very modern. Very DevOps. Surprisingly piratey if ye squint.
    async fn send_webhook(&self, url: &str, signal: &DistressSignal) -> Result<()> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "type": if signal.is_duress { "DURESS" } else { "DISTRESS" },
            "message": signal.message,
            "cargo_attached": signal.cargo_attached,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        client.post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::error::Error::Notification(e.to_string()))?;

        Ok(())
    }

    /// Send via Signal messenger
    ///
    /// End-to-end encrypted. The government hates this one wierd trick.
    /// Requires signal-cli to be set up seperately. A bit fiddly but
    /// worth it fer the privacy. Like assembling IKEA furniture but
    /// fer yer human rights.
    async fn send_signal(&self, _number: &str, signal: &DistressSignal) -> Result<()> {
        // Signal requires signal-cli or similar
        // TODO: Implement via signal-cli subprocess or API
        // Fer now we just log it like a responsible pirate
        eprintln!("[SIGNAL] Would send: {}", signal.message);
        Ok(())
    }

    /// Send via Matrix
    ///
    /// Federated, decentralized, open source. It's like email but
    /// fer people who think email isn't complicated enough. Bless 'em.
    async fn send_matrix(&self, _room: &str, signal: &DistressSignal) -> Result<()> {
        // TODO: Implement Matrix client
        // Need to add matrix-sdk dependency and do the whole auth dance
        eprintln!("[MATRIX] Would send: {}", signal.message);
        Ok(())
    }

    /// Send via Telegram
    ///
    /// Popular with journalists, activists, and yer nan who somehow
    /// figured out how to use it before ye did. Impressive really.
    async fn send_telegram(&self, _chat: &str, signal: &DistressSignal) -> Result<()> {
        // TODO: Implement Telegram bot API
        // Need a bot token and chat ID, pretty straightforward actually
        eprintln!("[TELEGRAM] Would send: {}", signal.message);
        Ok(())
    }
}

/// Build the email message
///
/// Constructs a proper email with headers and body. Nothing fancy,
/// just plain text. We ain't sendin' HTML newsletters here. This
/// isn't a bloody furniture catalogue.
fn build_email(
    smtp: &SmtpConfig,
    to_addr: &str,
    to_name: &str,
    signal: &DistressSignal,
) -> Result<Message> {
    let subject = if signal.is_duress {
        "URGENT: Dead Reckoning - DURESS ALERT"
    } else {
        "Dead Reckoning - Watch Expired"
    };

    let body = format!(
        r#"{}

---
This is an automated message from Dead Reckoning.
The watch has expired and the captain has not checked in.

Timestamp: {}
Type: {}
Cargo attached: {}

If you received this message, please follow the pre-arranged protocol.
May fair winds guide yer actions. Cheerio.
"#,
        signal.message,
        chrono::Utc::now().to_rfc3339(),
        if signal.is_duress { "DURESS" } else { "EXPIRED" },
        if signal.cargo_attached { "Yes" } else { "No" },
    );

    let email = Message::builder()
        .from(smtp.from.parse().map_err(|e| {
            crate::error::Error::Notification(format!("Invalid from address: {}", e))
        })?)
        .to(format!("{} <{}>", to_name, to_addr).parse().map_err(|e| {
            crate::error::Error::Notification(format!("Invalid to address: {}", e))
        })?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body)
        .map_err(|e| crate::error::Error::Notification(format!("Failed to build email: {}", e)))?;

    Ok(email)
}

/// Build the SMTP transport
///
/// Supports both STARTTLS (port 587, upgrade connection to TLS) and
/// implicit TLS (port 465, TLS from the start). Pick yer poison.
/// Or rather, pick yer encryption. Same thing really.
fn build_mailer(smtp: &SmtpConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let creds = Credentials::new(smtp.username.clone(), smtp.password.clone());

    let mailer = if smtp.starttls {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host)
            .map_err(|e| crate::error::Error::Notification(format!("SMTP relay error: {}", e)))?
            .port(smtp.port)
            .credentials(creds)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp.host)
            .map_err(|e| crate::error::Error::Notification(format!("SMTP relay error: {}", e)))?
            .port(smtp.port)
            .credentials(creds)
            .build()
    };

    Ok(mailer)
}

/// The result of trying to notify a single captain
///
/// Did the message get through? Did the pigeon make it?
/// Is the captain aware that things have gone terribly pear-shaped?
/// These are the questions that keep us up at night.
pub struct NotifyResult {
    /// Which captain we tried to notify
    pub captain: String,
    /// Did it work? (fingers crossed)
    pub success: bool,
    /// If it didn't work, why not? (probably DNS. It's always DNS.)
    pub error: Option<String>,
}

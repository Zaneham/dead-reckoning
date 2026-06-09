# Dead Reckoning

A dead man's switch for the digital age. Because eventually, all of us stop checking in.

---

> **WORK IN PROGRESS**
>
> This software is in active development and should be considered beta quality. Before trusting your life to it:
>
> - **Test thoroughly** in a non-critical environment
> - **Signal, Matrix, and Telegram notifications are stubs** (they just print to console)
> - **Email and webhooks work**, but verify with your actual SMTP provider
> - **Sentinel catches common threats** but won't stop nation-state APTs
> - **The daemon runs in foreground** (no systemd/launchd service installer yet)
>
> If you find bugs, please [open an issue](https://github.com/Zaneham/dead-reckoning/issues). This is the kind of software where bugs could have real consequences, so report them early and often.

---

## What Is This?

Dead Reckoning is a tool that monitors whether you're still alive (or at least, still typing). If you stop checking in, it assumes the worst and notifies your designated contacts with whatever secrets, files, or dramatic farewell messages you've prepared.

It's like a will, but for people who don't trust lawyers and think "what if I got hit by a bus" should have a software solution.

## Why "Dead Reckoning"?

In navigation, dead reckoning is estimating your current position based on your last known position plus time elapsed. Sailors used it when they couldn't see the stars. We use it when we can't see you.

Also it has "dead" in the name. Seemed appropriate.

## Features

This thing does more than it probably should:

- **Watch System** - Set a check-in interval. Miss it, and the countdown begins.
- **Safe & Duress Codes** - One phrase means "I'm fine." Another phrase *looks* like "I'm fine" but secretly means "I'm typing this with a gun to my head, please send help."
- **Fleet Notifications** - Email, webhooks, Signal, Matrix, Telegram. Your trusted contacts get the message.
- **Shamir Secret Sharing** - Split your secrets into pieces. Need 3 of 5 people to reconstruct. Very "The Italian Job" but with math.
- **Encrypted Cargo** - AES-256-GCM encryption for your sensitive files. Argon2 key derivation because we're not animals.
- **Secure Deletion** - When you absolutely need files to not exist anymore. Multiple overwrite methods up to Gutmann 35-pass for the truly paranoid.
- **Daemon Mode** - Runs in the background, watching, waiting, judging your check-in habits.
- **Behavioral Analysis** - Checks in at 3am when you usually check in at 9am? That's suspicious. The system notices.
- **Sentinel** - Scans your system for spyware, RATs, keyloggers, and government malware. Because if Pegasus is on your phone, checking in won't help.

## Installation

### From Source (The Way Of Pain)

```bash
git clone https://github.com/Zaneham/dead-reckoning
cd dead-reckoning
cargo build --release
```

The binary will be at `target/release/dead-reckoning`. Copy it somewhere in your PATH and try not to think about why you need this.

### Requirements

- Rust (stable)
- The acceptance that mortality is real
- At least one person who would notice if you disappeared

## Usage

### The Quick Start (Optimist's Version)

```bash
# Initialize your voyage
dead-reckoning init

# Set your phrases
dead-reckoning set-phrases --safe "the eagle has landed" --duress "everything is fine"

# Add people who care about you
dead-reckoning fleet add "Alice" email:alice@example.com
dead-reckoning fleet add "Bob" webhook:https://bobs-server.com/oh-no

# Check in (do this regularly, or else)
dead-reckoning parley "the eagle has landed"

# Run the daemon (it watches)
dead-reckoning daemon
```

### The Paranoid Setup (Realist's Version)

```bash
# Initialize with custom timing
dead-reckoning init --interval 12h --grace 6h

# Set phrases (the duress one should sound normal)
dead-reckoning set-phrases \
  --safe "purple monkey dishwasher" \
  --duress "I am perfectly safe and happy"

# Configure actual email delivery
dead-reckoning smtp \
  --host smtp.gmail.com \
  --port 587 \
  --username you@gmail.com \
  --password "your-app-password" \
  --from "Dead Reckoning <you@gmail.com>"

# Add your fleet
dead-reckoning fleet add "Lawyer" email:lawyer@firm.com
dead-reckoning fleet add "Best Friend" email:bestie@email.com
dead-reckoning fleet add "That One Person" signal:+1234567890

# Split your master password among trusted people
dead-reckoning split "correct-horse-battery-staple" -t 3 -n 5
# Now give each share to a different person
# Any 3 of them can reconstruct it, but not 2

# Pack sensitive files into encrypted cargo
dead-reckoning cargo pack \
  secret-diary.txt \
  bitcoin-wallet.dat \
  embarrassing-fanfiction.docx \
  -o cargo.bin \
  -p "a]whoLe}different{password"

# Run as daemon
dead-reckoning daemon --interval 30
```

## Commands

| Command | What It Does |
|---------|--------------|
| `init` | Start a new voyage. Creates config file. |
| `parley` | Check in with your safe phrase. Resets the timer. |
| `status` | See how much time you have left. Existentially uncomfortable. |
| `set-phrases` | Configure your safe and duress codes. |
| `fleet add` | Add a trusted contact to notify. |
| `fleet list` | See who gets your final message. |
| `fleet remove` | For when friendships end. |
| `smtp` | Configure email settings. |
| `cargo pack` | Encrypt files for later release. |
| `cargo unpack` | Decrypt files (requires password). |
| `split` | Split a secret using Shamir's scheme. |
| `reconstruct` | Reassemble a secret from shares. |
| `scuttle` | Emergency secure deletion. There is no undo. |
| `distress` | Manually trigger alerts. For testing. Please. |
| `daemon` | Run in background, monitoring forever. |
| `sentinel` | Scan for spyware, RATs, and compromise indicators. |

## The Duress Code

This is the clever bit.

You set two phrases. One means "I'm fine, reset the timer." The other *also* resets the timer, but silently flags that something is wrong.

From the outside, both check-ins look identical. But internally, the duress code triggers a silent alert to your fleet. The idea is: if someone forces you to check in at gunpoint, you use the duress code. They think everything is normal. Your contacts know it isn't.

```bash
# Normal check-in (you're fine)
dead-reckoning parley "the eagle has landed"
# Output: "Check-in accepted. Seas remain calm."

# Duress check-in (you're not fine)
dead-reckoning parley "everything is fine"
# Output: "Check-in accepted." (but fleet is secretly notified)
```

Choose your duress phrase carefully. It should sound natural. "Everything is fine" is classic. "I am not being coerced" is... less subtle.

## Shamir Secret Sharing

Adi Shamir (the 'S' in RSA) invented this in 1979. The idea: split a secret into N pieces where any K pieces can reconstruct it, but K-1 pieces reveal nothing.

```bash
# Split your bitcoin wallet password
dead-reckoning split "correct-horse-battery-staple" -t 3 -n 5

# Output:
# Share 1: 01a4b3c2d1e0f9...
# Share 2: 02b5c4d3e2f1a0...
# Share 3: 03c6d5e4f3a2b1...
# Share 4: 04d7e6f5a4b3c2...
# Share 5: 05e8f7a6b5c4d3...
#
# Give each share to a different trusted person.
# Any 3 can recover the secret. 2 cannot.

# Later, to reconstruct:
dead-reckoning reconstruct <share1> <share3> <share5> -t 3
# Output: "correct-horse-battery-staple"
```

This is useful for:
- Master passwords
- Cryptocurrency seed phrases
- The location of where you buried the gold
- Anything you want accessible only after you're gone, and only if enough people agree

## The Scuttle

Sometimes you don't want files released. You want them *gone*.

```bash
# Configure scuttle targets in voyage.toml:
# [scuttle]
# paths = ["~/sensitive", "~/definitely-legal-documents"]
# method = "dod"  # or "zero", "random", "gutmann"

# Preview what would be deleted
dead-reckoning scuttle
# "WARNING: This will PERMANENTLY DELETE..."

# Actually do it (requires --force because we're not monsters)
dead-reckoning scuttle --force
```

Wipe methods:
- `zero` - Single pass of zeros. Fast. Basic.
- `random` - Single pass of random data. Slightly better.
- `dod` - DoD 5220.22-M standard. Three passes. What the US government uses.
- `gutmann` - 35 passes. For when you need plausible deniability against nation-states with electron microscopes.

Note: On SSDs, secure deletion is complicated due to wear leveling. If you're *that* paranoid, use full-disk encryption and destroy the key.

## Behavioral Analysis

The system learns your patterns. If you usually check in at 9am and suddenly check in at 3am, that's flagged as unusual. Multiple anomalies increase suspicion level.

```
Normal         - Nothing weird
Unusual        - Slightly off pattern
Suspicious     - Multiple anomalies
HighlyAnomalous - "Should we be worried about you?"
```

This helps detect coerced check-ins even without the duress code. If someone forces you to check in but doesn't know your usual schedule, the timing itself becomes a signal.

## Sentinel - Compromise Detection

Here's the thing about end-to-end encryption: it doesn't help if someone owns your device.

Pegasus. Predator. FinFisher. Government spyware that costs millions and can read your Signal messages by just... reading your screen. They don't break the encryption. They don't need to. They're already inside.

Sentinel scans your system for signs of compromise:

```bash
# Single scan
dead-reckoning sentinel

# Output:
# === SENTINEL - System Compromise Detection ===
#
# Scan time: 2025-01-15 09:00:00 UTC
# Processes scanned: 342
#
# Overall Threat Level: CLEAR
#
# No threats detected.
```

### What It Looks For

| Category | Examples | Threat Level |
|----------|----------|--------------|
| **Known Malware** | Pegasus, Cobalt Strike, Meterpreter, njRAT, DarkComet | CRITICAL |
| **Corporate Surveillance** | Teramind, Veriato, mSpy, FlexiSpy, Hubstaff | HIGH |
| **Remote Access Tools** | TeamViewer, AnyDesk, RustDesk (if you didn't install them) | MEDIUM |
| **Screen Capture** | OBS, Bandicam, screen recorders (if unexpected) | LOW |
| **Suspicious Processes** | Running from temp dirs, no executable path, mimic names (svch0st.exe) | MEDIUM |

### Continuous Monitoring

```bash
# Watch mode - scan every 5 minutes
dead-reckoning sentinel --watch

# Faster scanning (every 60 seconds)
dead-reckoning sentinel --watch --interval 60

# Alert your fleet if HIGH threat or above detected
dead-reckoning sentinel --watch --alert-threshold high

# You legitimately use TeamViewer? Allow it
dead-reckoning sentinel --allow-remote

# You're a streamer? Allow OBS
dead-reckoning sentinel --allow-capture
```

### When Sentinel Finds Something

If it detects a CRITICAL threat (known malware signatures), it can automatically alert your fleet:

```
=== SENTINEL - System Compromise Detection ===

Scan time: 2025-01-15 09:00:00 UTC
Processes scanned: 342

Overall Threat Level: CRITICAL

1 threat(s) found:
--------------------------------------------------

[CRITICAL] MALWARE
  Name: totally-legit-app.exe
  Known malware pattern detected: cobaltstrike
  PID: 1337
  Path: C:\Users\you\AppData\Local\Temp\totally-legit-app.exe

THREAT DETECTED - ALERTING FLEET
  ✓ Notified: Alice
  ✓ Notified: Bob
```

### Limitations

Sentinel is not antivirus software. It's a tripwire.

- It scans process names and paths against known patterns
- It won't detect zero-day exploits or novel malware
- Sophisticated attackers can evade detection
- On Windows, many system processes don't expose their paths (this is normal)

Think of it as a smoke detector, not a fire suppression system. If it goes off, investigate. If it doesn't, you're probably fine. Probably.

### The "Am I Compromised?" Flowchart

```
Is Sentinel showing CRITICAL threats?
    │
    ├── Yes ──→ Assume device is compromised
    │           Don't use it for anything sensitive
    │           Your fleet has been notified
    │           Get a new device
    │
    └── No ──→ Is Sentinel showing HIGH threats?
                    │
                    ├── Yes ──→ Corporate surveillance detected
                    │           Your employer is watching
                    │           Don't job search on this computer
                    │
                    └── No ──→ Probably fine
                               But that's what they want you to think
```

## Deployment

### Running as a Service

The daemon needs to run continuously to monitor your check-ins. Options:

**Windows Task Scheduler:**
```
Action: Start a program
Program: C:\path\to\dead-reckoning.exe
Arguments: daemon --config C:\path\to\voyage.toml
Trigger: At startup
```

**Linux systemd:**
```ini
[Unit]
Description=Dead Reckoning Daemon
After=network.target

[Service]
ExecStart=/usr/local/bin/dead-reckoning daemon
Restart=always
User=yourusername

[Install]
WantedBy=multi-user.target
```

**Docker:**
```dockerfile
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/dead-reckoning /usr/local/bin/
COPY voyage.toml /etc/dead-reckoning/
CMD ["dead-reckoning", "daemon", "-c", "/etc/dead-reckoning/voyage.toml"]
```

### The Cloud Option (Most Reliable)

Running locally has a problem: if your computer is off, seized, or destroyed, the daemon stops running.

For maximum reliability, run the daemon on a cloud server (free tier AWS/GCP/Fly.io). That way:
- Your devices being compromised doesn't stop the watch
- Power outages don't matter
- You can check in from anywhere via SSH or a web endpoint

## Configuration File

Everything lives in `voyage.toml`:

```toml
[watch]
interval = "24h"
grace_period = "6h"
safe_word_hash = "a1b2c3..."      # SHA-256, not plaintext
duress_word_hash = "d4e5f6..."    # SHA-256, not plaintext
last_checkin = "2025-01-15T09:00:00Z"

[fleet]
threshold = 3  # For secret sharing

[[fleet.captains]]
name = "Alice"
[fleet.captains.contact]
type = "Email"
value = "alice@example.com"

[[fleet.captains]]
name = "Bob"
[fleet.captains.contact]
type = "Webhook"
value = "https://bobs-server.com/alert"

[smtp]
host = "smtp.gmail.com"
port = 587
username = "you@gmail.com"
password = "app-password-here"
from = "you@gmail.com"
starttls = true

[scuttle]
paths = ["~/sensitive-stuff"]
method = "dod"
```

## FAQ

**Q: Is this legal?**
A: It's software that sends emails when you don't press a button. Yes, it's legal. What you *do* with it is your business.

**Q: What if I just forget to check in?**
A: That's what the grace period is for. You get warnings. If you still don't check in, well, the system works as designed.

**Q: Can I test this without alerting everyone?**
A: Yes. Use `dead-reckoning distress --message "this is a test"` to manually trigger. Tell your fleet first.

**Q: What happens to the daemon if my computer dies?**
A: It dies too. Run it on a cloud server if this concerns you.

**Q: Is the encryption actually secure?**
A: AES-256-GCM with Argon2id key derivation. Unless you're being targeted by the NSA and they've broken AES (they haven't publicly), you're fine.

**Q: Why nautical terminology?**
A: The original theme was going to be Pirates of the Caribbean but Disney has lawyers. "Fleet," "cargo," "scuttle," and "parley" are all public domain nautical terms. The kraken is also public domain. Take that, Mouse.

**Q: This is morbid.**
A: That's not a question. But yes. We're all going to die. Might as well have a plan.

**Q: Can Sentinel detect Pegasus?**
A: It looks for known process names and patterns. NSO Group's Pegasus is sophisticated and actively evades detection. If a nation-state is targeting you specifically, assume they'll get in. Sentinel is more useful for detecting commodity malware, corporate spyware, and script kiddie RATs. It's a tripwire, not a fortress.

**Q: Sentinel flagged something. Am I hacked?**
A: Maybe. Check the threat level. CRITICAL means known malware patterns - investigate immediately. MEDIUM might be legitimate software running from a weird location (like a VS Code installer in temp). Use your judgment. When in doubt, nuke from orbit.

## Why Does This Exist?

Legitimate reasons:
- Journalists in dangerous situations
- Activists in authoritarian regimes
- Cryptocurrency holders who want heirs to access funds
- Anyone who thinks "what happens to my digital life when I'm gone?"
- People with abusive situations who need a dead man's switch for safety
- Whistleblowers who want insurance
- People who saw the BBC article about governments reading encrypted messages and thought "well, shit"

Less legitimate but valid reasons:
- Paranoia (is it paranoia if they're actually watching?)
- Too many spy movies
- "It seemed like a fun project"
- The NSO Group exists and that makes you uncomfortable


```

## Related Projects

Because apparently I have a type:

| Project | What It Is | Concerning Level |
|---------|------------|------------------|
| [jovial](https://github.com/Zaneham/jovial) | JOVIAL J73 compiler | Moderate |
| [jovial-lsp](https://github.com/Zaneham/jovial-lsp) | LSP for bombing things | Elevated |
| [minuteman-computer-emulator](https://github.com/Zaneham/minuteman-computer-emulator) | ICBM guidance emulator | FBI watchlist |
| [voyager-fds-emulator](https://github.com/Zaneham/voyager-fds-emulator) | Space probe emulator | Wholesome actually |

## Contributing

Found a bug? Want to add Telegram bot support? Have opinions about secure deletion algorithms?

Open an issue or PR. I'm unreasonably interested in edge cases involving mortality and cryptography.

## License

Apache 2.0. Do whatever you want. If this software ends up saving someone's life, or helping a family access a deceased relative's cryptocurrency, or just giving someone peace of mind, that's the point.

## Acknowledgments

- **Adi Shamir** - For inventing secret sharing in 1979
- **The AES designers** - Joan Daemen and Vincent Rijmen
- **The Argon2 team** - For winning the Password Hashing Competition
- **Lettre maintainers** - For making Rust email not terrible
- **Low Level** - His Rust videos and courses helped with the project. 
- **You** - For reading a README about software for when you're dead

---

```
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

  RELEASE THE KRAKEN
  (when the captain stops responding)
```

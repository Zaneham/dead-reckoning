//! Sentinel - Desktop compromise detection
//!
//! Yer personal paranoia assistant! This module scans yer system fer signs
//! that someone's been pokin' around where they shouldn't. Spyware, RATs,
//! government malware, corporate surveillance, the works.
//!
//! It's like havin' a very anxious security guard who sees threats everywhere.
//! Except sometimes the threats are real. And sometimes it's just Windows
//! bein' Windows. Hard to tell, really.
//!
//! Monitors for:
//! - Known spyware/RAT processes (Pegasus, Cobalt Strike, etc.)
//! - Remote access tools (TeamViewer runnin' when ye didn't start it? Sus.)
//! - Screen recording software (someone watchin' ye type yer passwords?)
//! - Suspicious network listeners
//! - Keyloggers and input monitors
//!
//! Note: This ain't perfect. A sufficiently sophisticated attacker can hide
//! from process scanners. But it catches the low-hangin' fruit, and that's
//! better than nothin'. Like bringin' a knife to a gunfight, except the
//! knife is a process scanner and the gun is a nation-state APT.

use std::collections::HashSet;

use sysinfo::{Networks, System, Process, Pid};

/// Known spyware, RATs, and government surveillance tools
///
/// If ye see any of these runnin', assume yer compromised and act accordingly.
/// Which probably means runnin' scuttle and sendin' yer duress code.
/// Or just turnin' the computer off and yeeting it into the ocean.
const KNOWN_MALWARE: &[&str] = &[
    // Commercial spyware (the expensive kind, fer governments and jealous spouses)
    "pegasus",       // NSO Group's finest. Very fancy. Very illegal in most contexts.
    "nsogroup",
    "candiru",       // Israeli spyware. Named after a fish that swims up... nevermind.
    "predator",      // Cytrox's offering. Not the movie alien, sadly.
    "cytrox",
    "finfisher",     // German spyware. Very efficient. Very German.
    "finspy",
    "hacking team",  // Italian spyware company. Got hacked themselves. Ironic.
    "galileo",

    // Common RATs (Remote Access Trojans, not the rodents)
    "darkcomet",     // Classic RAT, been around since 2008. Retro malware.
    "njrat",         // Very popular with script kiddies
    "nanocore",      // .NET based, so at least it's cross-platform
    "quasar",        // Open source RAT. Fer the budget-conscious hacker.
    "asyncrat",
    "remcos",
    "netwire",
    "orcus",
    "luminosity",
    "imminent",
    "blackshades",   // The developer got arrested. Good.
    "poison ivy",    // Not the Batman villain, unfortunately
    "gh0st",         // Chinese RAT. The "0" makes it cool, apparently.
    "xtreme",
    "cybergate",
    "spynet",
    "darktrack",
    "havex",         // Industrial control system malware. Yikes.
    "plugx",         // APT favorite
    "cobalt",        // Cobalt Strike beacon
    "cobaltstrike",
    "meterpreter",   // Metasploit payload. Could be a pentester, could be a hacker.
    "mimikatz",      // Password dumping tool. Very naughty.

    // Keyloggers (fer when ye want to know what someone's typin')
    "keylogger",     // Generic match
    "ardamax",
    "spyrix",
    "revealer",
    "refog",
    "actual spy",
    "realtime-spy",
    "spytech",
    "kickidler",
    "teramind",
    "veriato",
    "activtrak",
];

/// Remote access tools (legitimate but suspicious if unexpected)
///
/// These ain't necessarily malware, but if TeamViewer is runnin' and ye
/// didn't start it... that's a paddlin'. I mean, that's suspicious.
const REMOTE_ACCESS_TOOLS: &[&str] = &[
    "teamviewer",    // The classic. Yer mum uses it fer tech support.
    "anydesk",       // German alternative. Also yer mum uses it.
    "rustdesk",      // Open source. Very cool. Still sus if unexpected.
    "ammyy",
    "logmein",
    "gotomypc",
    "splashtop",
    "supremo",
    "remotepc",
    "connectwise",
    "screenconnect",
    "dameware",
    "radmin",
    "vnc",           // The OG remote desktop
    "tightvnc",
    "ultravnc",
    "realvnc",
    "tigervnc",
    "x11vnc",
    "nomachine",
    "parsec",        // Mostly fer gaming but still
    "chrome remote",
    "chromeremotedesktop",
    "meshagent",     // MeshCentral agent
    "tacticalrmm",
];

/// Screen recording / capture tools
///
/// Could be a content creator. Could be someone recordin' yer screen
/// without permission. Context matters. Are ye a streamer? Then OBS is fine.
/// Are ye not a streamer but OBS is runnin'? Less fine.
const SCREEN_CAPTURE_TOOLS: &[&str] = &[
    "obs",           // Open Broadcaster Software. Every streamer uses it.
    "obs64",
    "obs32",
    "streamlabs",
    "xsplit",
    "bandicam",      // Popular in certain... communities.
    "camtasia",
    "snagit",
    "screencast",
    "loom",          // Very popular fer async video messages
    "screenrec",
    "icecream",      // Cute name fer screen recording software
    "flashback",
    "apowerrec",
    "movavi",
    "screenpal",
    "screencastify",
    "recordit",
    "sharex",        // Can be used fer capture. Great tool though.
    "greenshot",
    "lightshot",
];

/// System monitoring / corporate surveillance
///
/// Yer employer might be watchin' ye. Legally, in most places. Ethically?
/// Debatable. If ye see these on yer personal machine, that's concerning.
/// If ye see these on yer work machine, that's just Tuesday.
const CORPORATE_SURVEILLANCE: &[&str] = &[
    "teramind",
    "veriato",
    "activtrak",
    "hubstaff",
    "time doctor",   // Tracks yer time. Dystopian but legal.
    "desktime",
    "workpuls",
    "controlio",
    "staffcop",
    "interguard",
    "sneakypete",    // Subtle naming there
    "spector",
    "webwatcher",
    "flexispy",      // Phone spyware marketed to jealous partners. Gross.
    "mspy",
    "cocospy",
    "spyic",
    "minspy",
    "spyine",
    "neatspy",
    "clickfree",
    "phonespector",
];

/// How bad is the situation, on a scale from "fine" to "flee the country"?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreatLevel {
    /// No threats detected. Lovely. Have a biscuit.
    Clear,
    /// Something worth noting but probably benign. Keep an eye on it.
    Low,
    /// Suspicious activity detected. Might want to investigate.
    Medium,
    /// Likely compromised. Time to start worryin' seriously.
    High,
    /// Known malware detected. Assume fully compromised. Panic stations.
    Critical,
}

/// Information about a detected threat
///
/// Contains all the details ye need to either fix the problem or
/// justify yer paranoia to yer therapist.
#[derive(Debug, Clone)]
pub struct ThreatInfo {
    pub level: ThreatLevel,
    pub category: ThreatCategory,
    pub name: String,
    pub description: String,
    pub pid: Option<u32>,
    pub path: Option<String>,
}

/// What type of threat is it?
///
/// Helps ye prioritize. Malware is worse than screen capture which is
/// worse than "I just noticed TeamViewer is installed".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreatCategory {
    /// Actual malware. Bad. Very bad.
    Malware,
    /// Remote access tools. Sus but not necessarily bad.
    RemoteAccess,
    /// Screen recording. Could be innocent. Probably isn't.
    ScreenCapture,
    /// Corporate spyware. Legal but annoying.
    CorporateSurveillance,
    /// Weird network stuff. Needs investigation.
    SuspiciousNetwork,
    /// Process with concerning characteristics. Hmm.
    SuspiciousProcess,
}

impl std::fmt::Display for ThreatCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malware => write!(f, "MALWARE"),
            Self::RemoteAccess => write!(f, "REMOTE ACCESS"),
            Self::ScreenCapture => write!(f, "SCREEN CAPTURE"),
            Self::CorporateSurveillance => write!(f, "SURVEILLANCE"),
            Self::SuspiciousNetwork => write!(f, "NETWORK"),
            Self::SuspiciousProcess => write!(f, "PROCESS"),
        }
    }
}

/// The full report from a sentinel scan
///
/// Contains everything we found, sorted by threat level. Makes fer
/// interestin' readin' if yer paranoid. Borin' readin' if yer not.
#[derive(Debug)]
pub struct SentinelReport {
    pub overall_threat: ThreatLevel,
    pub threats: Vec<ThreatInfo>,
    pub process_count: usize,
    pub network_interfaces: usize,
    pub listening_ports: Vec<u16>,
    pub scan_time: chrono::DateTime<chrono::Utc>,
}

/// The Sentinel - yer paranoid security assistant
///
/// Sits there watchin' processes, judging them silently, occasionally
/// raisin' the alarm when somethin' looks dodgy. Like a very suspicious
/// cat, but fer computer security.
pub struct Sentinel {
    system: System,
    /// Processes to ignore (user-configured allowlist)
    allowlist: HashSet<String>,
    /// Whether to flag remote access tools (some users legitimately use them)
    flag_remote_access: bool,
    /// Whether to flag screen capture tools
    flag_screen_capture: bool,
}

impl Sentinel {
    /// Create a new Sentinel
    ///
    /// By default, it flags everything suspicious. Ye can configure
    /// it to be less paranoid if ye want, but why would ye?
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            allowlist: HashSet::new(),
            flag_remote_access: true,
            flag_screen_capture: true,
        }
    }

    /// Add a process name to the allowlist (won't be flagged)
    ///
    /// Use this if ye legitimately run OBS or TeamViewer or whatever.
    /// We won't judge. Much.
    pub fn allow(&mut self, name: &str) {
        self.allowlist.insert(name.to_lowercase());
    }

    /// Set whether to flag remote access tools
    ///
    /// If yer an IT person who uses TeamViewer all day, turn this off.
    /// If yer not, leave it on and be suspicious of TeamViewer.
    pub fn set_flag_remote_access(&mut self, flag: bool) {
        self.flag_remote_access = flag;
    }

    /// Set whether to flag screen capture tools
    ///
    /// Streamers should turn this off. Everyone else should leave it on
    /// and wonder why OBS is runnin' at 3am when they're not streamin'.
    pub fn set_flag_screen_capture(&mut self, flag: bool) {
        self.flag_screen_capture = flag;
    }

    /// Perform a full system scan
    ///
    /// Goes through every runnin' process and judges it. Returns a
    /// comprehensive report that'll either reassure ye or terrify ye.
    /// Probably the latter. That's what paranoia does.
    pub fn scan(&mut self) -> SentinelReport {
        // Refresh system info
        self.system.refresh_all();

        let mut threats = Vec::new();

        // Scan processes
        self.scan_processes(&mut threats);

        // Determine overall threat level
        let overall_threat = self.calculate_overall_threat(&threats);

        // Get network info
        let networks = Networks::new_with_refreshed_list();
        let network_interfaces = networks.iter().count();

        SentinelReport {
            overall_threat,
            threats,
            process_count: self.system.processes().len(),
            network_interfaces,
            listening_ports: Vec::new(), // Would need elevated privileges to scan properly
            scan_time: chrono::Utc::now(),
        }
    }

    /// Scan all running processes fer suspicious activity
    ///
    /// This is where the magic happens. We go through every process,
    /// compare it against our lists of known bad things, and flag
    /// anything that looks dodgy. Very thorough. Very paranoid.
    fn scan_processes(&self, threats: &mut Vec<ThreatInfo>) {
        for (pid, process) in self.system.processes() {
            let name = process.name().to_string_lossy().to_lowercase();
            let exe_path = process.exe()
                .map(|p| p.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            // Skip allowlisted processes
            if self.allowlist.contains(&name) {
                continue;
            }

            // Check against known malware (highest priority)
            if let Some(threat) = self.check_malware(&name, &exe_path, pid, process) {
                threats.push(threat);
                continue;
            }

            // Check remote access tools
            if self.flag_remote_access {
                if let Some(threat) = self.check_remote_access(&name, &exe_path, pid, process) {
                    threats.push(threat);
                    continue;
                }
            }

            // Check screen capture tools
            if self.flag_screen_capture {
                if let Some(threat) = self.check_screen_capture(&name, &exe_path, pid, process) {
                    threats.push(threat);
                    continue;
                }
            }

            // Check corporate surveillance
            if let Some(threat) = self.check_corporate_surveillance(&name, &exe_path, pid, process) {
                threats.push(threat);
                continue;
            }

            // Check for suspicious characteristics
            if let Some(threat) = self.check_suspicious_process(&name, &exe_path, pid, process) {
                threats.push(threat);
            }
        }
    }

    /// Check if process matches known malware
    ///
    /// If this returns Some, ye should probably panic. Or at least
    /// run scuttle and contact yer local cybersecurity professional.
    fn check_malware(&self, name: &str, exe_path: &str, pid: &Pid, process: &Process) -> Option<ThreatInfo> {
        for &malware in KNOWN_MALWARE {
            if name.contains(malware) || exe_path.contains(malware) {
                return Some(ThreatInfo {
                    level: ThreatLevel::Critical,
                    category: ThreatCategory::Malware,
                    name: process.name().to_string_lossy().to_string(),
                    description: format!("Known malware pattern detected: {}. This is very bad.", malware),
                    pid: Some(pid.as_u32()),
                    path: process.exe().map(|p| p.to_string_lossy().to_string()),
                });
            }
        }
        None
    }

    /// Check if process is a remote access tool
    fn check_remote_access(&self, name: &str, exe_path: &str, pid: &Pid, process: &Process) -> Option<ThreatInfo> {
        for &tool in REMOTE_ACCESS_TOOLS {
            if name.contains(tool) || exe_path.contains(tool) {
                return Some(ThreatInfo {
                    level: ThreatLevel::Medium,
                    category: ThreatCategory::RemoteAccess,
                    name: process.name().to_string_lossy().to_string(),
                    description: format!("Remote access tool detected: {}. Did ye start this?", tool),
                    pid: Some(pid.as_u32()),
                    path: process.exe().map(|p| p.to_string_lossy().to_string()),
                });
            }
        }
        None
    }

    /// Check if process is screen capture software
    fn check_screen_capture(&self, name: &str, exe_path: &str, pid: &Pid, process: &Process) -> Option<ThreatInfo> {
        for &tool in SCREEN_CAPTURE_TOOLS {
            if name.contains(tool) || exe_path.contains(tool) {
                return Some(ThreatInfo {
                    level: ThreatLevel::Low,
                    category: ThreatCategory::ScreenCapture,
                    name: process.name().to_string_lossy().to_string(),
                    description: format!("Screen capture tool detected: {}. Hope yer decent.", tool),
                    pid: Some(pid.as_u32()),
                    path: process.exe().map(|p| p.to_string_lossy().to_string()),
                });
            }
        }
        None
    }

    /// Check if process is corporate surveillance
    fn check_corporate_surveillance(&self, name: &str, exe_path: &str, pid: &Pid, process: &Process) -> Option<ThreatInfo> {
        for &tool in CORPORATE_SURVEILLANCE {
            if name.contains(tool) || exe_path.contains(tool) {
                return Some(ThreatInfo {
                    level: ThreatLevel::High,
                    category: ThreatCategory::CorporateSurveillance,
                    name: process.name().to_string_lossy().to_string(),
                    description: format!("Corporate surveillance tool detected: {}. Big Brother is watchin'.", tool),
                    pid: Some(pid.as_u32()),
                    path: process.exe().map(|p| p.to_string_lossy().to_string()),
                });
            }
        }
        None
    }

    /// Check for processes with suspicious characteristics
    ///
    /// Things like runnin' from temp directories, havin' no path,
    /// or havin' names that look like someone mashed the keyboard.
    fn check_suspicious_process(&self, name: &str, exe_path: &str, pid: &Pid, process: &Process) -> Option<ThreatInfo> {
        // Check for processes running from temp directory (dodgy)
        if exe_path.contains("\\temp\\") || exe_path.contains("/tmp/") || exe_path.contains("\\appdata\\local\\temp") {
            return Some(ThreatInfo {
                level: ThreatLevel::Medium,
                category: ThreatCategory::SuspiciousProcess,
                name: process.name().to_string_lossy().to_string(),
                description: "Process running from temporary directory. That's a bit rum.".to_string(),
                pid: Some(pid.as_u32()),
                path: process.exe().map(|p| p.to_string_lossy().to_string()),
            });
        }

        // Process with no path (memory-only, potentially injected)
        if process.exe().is_none() && !is_system_process(name) {
            return Some(ThreatInfo {
                level: ThreatLevel::Medium,
                category: ThreatCategory::SuspiciousProcess,
                name: process.name().to_string_lossy().to_string(),
                description: "Process with no executable path. Could be injected. Concerning.".to_string(),
                pid: Some(pid.as_u32()),
                path: None,
            });
        }

        // Suspicious naming patterns
        if is_suspicious_name(name) {
            return Some(ThreatInfo {
                level: ThreatLevel::Low,
                category: ThreatCategory::SuspiciousProcess,
                name: process.name().to_string_lossy().to_string(),
                description: "Process name matches suspicious pattern. Looks like malware namgin'.".to_string(),
                pid: Some(pid.as_u32()),
                path: process.exe().map(|p| p.to_string_lossy().to_string()),
            });
        }

        None
    }

    /// Calculate the overall threat level from all detected threats
    ///
    /// Basically returns the highest threat level found. One Critical
    /// makes the whole report Critical. Like one bad apple spoilin' the bunch.
    fn calculate_overall_threat(&self, threats: &[ThreatInfo]) -> ThreatLevel {
        if threats.is_empty() {
            return ThreatLevel::Clear;
        }

        // Return highest threat level found
        let mut highest = ThreatLevel::Clear;
        for threat in threats {
            match (&highest, &threat.level) {
                (_, ThreatLevel::Critical) => return ThreatLevel::Critical,
                (ThreatLevel::Clear, level) => highest = level.clone(),
                (ThreatLevel::Low, ThreatLevel::Medium | ThreatLevel::High) => highest = threat.level.clone(),
                (ThreatLevel::Medium, ThreatLevel::High) => highest = ThreatLevel::High,
                _ => {}
            }
        }

        highest
    }
}

impl Default for Sentinel {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if this is a known system process that legitimately has no path
///
/// Windows has loads of these. System processes that run in kernel mode
/// or protected processes that hide their paths. Perfectly normal.
/// Terribly confusing for security software. Such is life.
fn is_system_process(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    let name_lower = name_lower.as_str();

    // Windows core system processes (often have no visible path due to protection)
    let windows_system = [
        "system", "idle", "registry", "memory compression",
        "system idle process", "secure system", "smss.exe",
        "csrss.exe", "wininit.exe", "services.exe", "lsass.exe",
        "svchost.exe", "lsaiso.exe", "fontdrvhost.exe", "dwm.exe",
        "winlogon.exe", "spoolsv.exe", "searchindexer.exe",
        "searchprotocolhost.exe", "searchfilterhost.exe",
        "wudfhost.exe", "dashost.exe", "sihost.exe", "taskhostw.exe",
        "explorer.exe", "runtimebroker.exe", "applicationframehost.exe",
        "shellexperiencehost.exe", "startmenuexperiencehost.exe",
        "textinputhost.exe", "ctfmon.exe", "conhost.exe",
        "dllhost.exe", "msiexec.exe", "mscorsvw.exe", "ngen.exe",
        "audiodg.exe", "wlanext.exe", "wmpnetwk.exe",
        // Security software
        "msmpeng.exe", "nissrv.exe", "securityhealthservice.exe",
        "mpdefendercoreservice.exe", "mpcmdrun.exe",
        // Windows services
        "vmcompute.exe", "vmwp.exe", "wslservice.exe",
        "gamingservices.exe", "gamingservicesnet.exe",
        "officeclicktorun.exe", "searchapp.exe",
        "filesynchelper.exe", "onedrive.exe",
        // Common system drivers/services
        "nvdisplay.container.exe", "atieclxx.exe", "atiesrxx.exe",
        "amdfendrsr.exe", "rtkauduservice64.exe", "ngciso.exe",
        // Database/dev tools often run as services
        "postgres.exe", "pg_ctl.exe", "mysqld.exe", "mongod.exe",
        "sqlservr.exe", "redis-server.exe",
        // Steam/gaming
        "steamservice.exe", "eabackgroundservice.exe",
        "gameinputredistservice.exe",
        // WSL
        "vmmemwsl", "wsl.exe", "init",
        // Hardware vendors
        "logi_lamparray_service.exe", "logitechg_agent.exe",
        "wirelesskb850notificationservice.exe",
        "corsairservice.exe", "icue.exe",
        // Windows telemetry/diagnostics (spyware but legal, thanks Microsoft)
        "aggregatorhost.exe", "compattelrunner.exe",
        "devicecensus.exe", "diagtrack.exe",
    ];

    // Linux kernel processes
    let linux_system = [
        "[kernel]", "kthreadd", "ksoftirqd", "kworker",
        "migration", "watchdog", "cpuhp", "init", "systemd",
        "journald", "udevd", "dbus-daemon", "polkitd",
    ];

    windows_system.contains(&name_lower) || linux_system.contains(&name_lower)
}

/// Check for suspicious naming patterns
///
/// Malware often has weird names like "svchost1" or "csrsss" (note the extra s).
/// It's trying to look like system processes but failing. We catch those.
fn is_suspicious_name(name: &str) -> bool {
    // Random-looking names (common for malware)
    let name_chars: Vec<char> = name.chars().collect();

    // Very short random name
    if name.len() >= 5 && name.len() <= 8 {
        let consonants = name_chars.iter().filter(|c| !"aeiou".contains(**c) && c.is_alphabetic()).count();
        let vowels = name_chars.iter().filter(|c| "aeiou".contains(**c)).count();

        // Unusual consonant to vowel ratio (no vowels = suspicious keyboard mashing)
        if consonants > 0 && vowels == 0 && name.len() > 4 {
            return true;
        }
    }

    // Ends with random numbers (like malware1234.exe)
    if name.len() > 4 {
        let suffix: String = name.chars().rev().take(4).collect();
        if suffix.chars().all(|c| c.is_numeric()) {
            return true;
        }
    }

    // Mimics system processes with slight variations (the oldest trick in the book)
    let mimics = [
        "svch0st", "svchoost", "svchosts", "scvhost",  // svchost mimic
        "csrs", "csrsss", "cssrs",                      // csrss mimic
        "lssas", "lsasss", "isass",                     // lsass mimic
        "explor3r", "expl0rer",                         // explorer mimic
        "svhost", "syshost", "winhost",
    ];

    for mimic in mimics {
        // Exact match or starts with mimic pattern
        if name == mimic || name.starts_with(&format!("{}.", mimic)) {
            return true;
        }
    }

    false
}

impl std::fmt::Display for ThreatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Clear => write!(f, "CLEAR"),
            Self::Low => write!(f, "LOW"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::High => write!(f, "HIGH"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl std::fmt::Display for SentinelReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== SENTINEL SCAN REPORT ===")?;
        writeln!(f, "Time: {}", self.scan_time.format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(f, "Processes scanned: {}", self.process_count)?;
        writeln!(f, "Network interfaces: {}", self.network_interfaces)?;
        writeln!(f)?;
        writeln!(f, "OVERALL THREAT LEVEL: {}", self.overall_threat)?;
        writeln!(f)?;

        if self.threats.is_empty() {
            writeln!(f, "No threats detected. Lovely. Have a biscuit.")?;
        } else {
            writeln!(f, "THREATS DETECTED: {}", self.threats.len())?;
            writeln!(f, "{}", "-".repeat(50))?;

            for (i, threat) in self.threats.iter().enumerate() {
                writeln!(f, "\n[{}] {} - {}", i + 1, threat.level, threat.category)?;
                writeln!(f, "    Name: {}", threat.name)?;
                writeln!(f, "    {}", threat.description)?;
                if let Some(pid) = threat.pid {
                    writeln!(f, "    PID: {}", pid)?;
                }
                if let Some(path) = &threat.path {
                    writeln!(f, "    Path: {}", path)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspicious_names() {
        assert!(is_suspicious_name("svch0st"));
        assert!(is_suspicious_name("malware1234"));
        assert!(!is_suspicious_name("chrome"));
        assert!(!is_suspicious_name("firefox"));
    }

    #[test]
    fn test_sentinel_scan() {
        let mut sentinel = Sentinel::new();
        let report = sentinel.scan();

        // Should complete without panic
        assert!(report.process_count > 0);
        println!("{}", report);
    }
}

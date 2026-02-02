//! Scuttle - Sink the ship! DELETE EVERYTHING!
//!
//! When ye need to destroy evidence faster than ye can say "the authorities
//! are at the door", this module has yer back. Secure file deletion that
//! would make even the most paranoid cryptographer shed a tear of joy.
//!
//! Wipe methods:
//! - Zero: One pass of zeros. Fast but basic. Fer when ye trust SSDs.
//! - Random: One pass of random data. Slightly better. Still a bit rubbish.
//! - DoD 5220.22-M: Three passes. The US Department of Defense thinks this is enough.
//! - Gutmann: 35 passes. Fer when yer adversary has an electron microscope
//!   and unlimited time. Overkill? Probably. Satisfying? Absolutely.
//!
//! Note: On SSDs, none of this matters due to wear leveling. But it makes
//! us feel better, and that's what really counts. Like horoscopes.

use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

use rand::RngCore;

use crate::config::ScuttleConfig;
use crate::error::Result;

/// The Scuttle module - fer when ye need to sink the evidence
///
/// Named after the naval practice of intentionally sinking yer own ship
/// to prevent it from fallin' into enemy hands. Very dramatic. Very pirate.
/// Also a good name for a crab in a children's movie, but that's neither
/// here nor there.
pub struct Scuttle {
    config: ScuttleConfig,
}

/// How thoroughly do ye want to destroy the evidence?
///
/// Each method represents a different level of paranoia. Choose wisely
/// based on yer threat model and how much time ye have before the
/// door gets kicked in. No pressure.
#[derive(Debug, Clone, Copy)]
pub enum WipeMethod {
    /// Single pass of zeros (fast, basic). The "I'm in a hurry" option.
    Zero,
    /// Single pass of random data. Slightly fancier zeros. Still naff.
    Random,
    /// DoD 5220.22-M (3-pass). Government approved destruction. How reassuring.
    Dod3Pass,
    /// Gutmann (35-pass). Weapons-grade paranoia fer the discerning criminal.
    Gutmann,
}

impl Scuttle {
    /// Create a new scuttle system with the given config
    ///
    /// Don't worry, this doesn't delete anything yet. Ye gotta
    /// explicitly call scuttle_all() fer the fireworks.
    /// Very responsible design, if I do say so meself.
    pub fn new(config: ScuttleConfig) -> Self {
        Self { config }
    }

    /// SCUTTLE EVERYTHING! This is not a drill!
    ///
    /// Securely deletes all configured paths. Returns a report of
    /// what got deleted, what was skipped, and what failed.
    ///
    /// There's no undo. There's no "are you sure?". This is it.
    /// The files are gone. Into Davy Jones' locker with 'em.
    /// Cheerio to yer data. It's off to a better place now.
    pub fn scuttle_all(&self) -> Result<ScuttleReport> {
        let mut report = ScuttleReport::default();

        for path_str in &self.config.paths {
            let path = Path::new(path_str);

            if !path.exists() {
                // Can't delete what ain't there. Philosophy 101.
                report.skipped.push(path_str.clone());
                continue;
            }

            match self.secure_delete(path) {
                Ok(_) => report.deleted.push(path_str.clone()),
                Err(e) => report.failed.push((path_str.clone(), e.to_string())),
            }
        }

        report.method = self.config.method.clone();
        Ok(report)
    }

    /// Securely delete a single file or directory
    ///
    /// First overwrites the data, THEN deletes. Just deletin' a file
    /// doesn't actually erase the data, it just removes the pointer.
    /// Like takin' the label off a tin. The beans are still in there.
    /// We're more thorogh than that. We destroy the beans.
    pub fn secure_delete(&self, path: &Path) -> Result<()> {
        let method = Self::parse_method(&self.config.method);

        if path.is_file() {
            self.wipe_file(path, method)?;
            fs::remove_file(path)?;
        } else if path.is_dir() {
            self.wipe_directory(path, method)?;
            fs::remove_dir_all(path)?;
        }

        Ok(())
    }

    /// Recursively wipe a directory
    ///
    /// Goes through every file in the directory (and subdirectories)
    /// and wipes 'em all. Like a digital plague. But the good kind.
    /// If plagues can be good. This metaphor is getting away from me.
    fn wipe_directory(&self, dir: &Path, method: WipeMethod) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                self.wipe_file(&path, method)?;
            } else if path.is_dir() {
                // Yo dawg, I heard ye like recursion so I put recursion in yer recursion
                self.wipe_directory(&path, method)?;
            }
        }

        Ok(())
    }

    /// Wipe a single file with the specified method
    ///
    /// This is where the magic happens. We overwrite the file with
    /// garbage data multiple times, then sync to disk to make sure
    /// the writes actually happen and aren't just sittin' in a buffer
    /// havin' a nice cup of tea.
    fn wipe_file(&self, path: &Path, method: WipeMethod) -> Result<()> {
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len() as usize;

        if file_size == 0 {
            // Empty file, nothing to wipe. Moving on. Nothing to see here.
            return Ok(());
        }

        let passes = match method {
            WipeMethod::Zero => vec![WipePass::Zero],
            WipeMethod::Random => vec![WipePass::Random],
            WipeMethod::Dod3Pass => vec![
                WipePass::Pattern(0x00),  // All zeros
                WipePass::Pattern(0xFF),  // All ones
                WipePass::Random,         // Random chaos, very exciting
            ],
            WipeMethod::Gutmann => Self::gutmann_passes(),
        };

        let mut file = OpenOptions::new()
            .write(true)
            .open(path)?;

        for pass in passes {
            self.write_pass(&mut file, file_size, pass)?;
            file.sync_all()?;  // Make sure it hits the disk, not just the buffer
            file.seek(SeekFrom::Start(0))?;  // Back to the beginning fer the next pass
        }

        Ok(())
    }

    /// Write a single pass of data to the file
    ///
    /// We do this in chunks because allocatin' a gigabyte buffer
    /// fer a gigabyte file would be... unwise. Bit like orderin'
    /// a pint glass of hot sauce. 64KB chunks it is.
    fn write_pass(&self, file: &mut File, size: usize, pass: WipePass) -> Result<()> {
        const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks, very sensible
        let mut buffer = vec![0u8; CHUNK_SIZE.min(size)];
        let mut remaining = size;

        while remaining > 0 {
            let to_write = remaining.min(buffer.len());

            match pass {
                WipePass::Zero => buffer[..to_write].fill(0x00),
                WipePass::Pattern(byte) => buffer[..to_write].fill(byte),
                WipePass::Random => rand::thread_rng().fill_bytes(&mut buffer[..to_write]),
            }

            file.write_all(&buffer[..to_write])?;
            remaining -= to_write;
        }

        Ok(())
    }

    /// Generate the 35 passes fer the Gutmann method
    ///
    /// Named after Peter Gutmann who wrote a paper in 1996 about secure
    /// deletion. Some say 35 passes is overkill. Those people probably
    /// don't have state-level adversaries with electron microscopes.
    /// Must be nice. Living without that level of paranoia, I mean.
    fn gutmann_passes() -> Vec<WipePass> {
        let mut passes = Vec::with_capacity(35);

        // Passes 1-4: Random (to mask the data, very mysterious)
        for _ in 0..4 {
            passes.push(WipePass::Random);
        }

        // Passes 5-31: Specific patterns designed to defeat magnetic
        // force microscopy on old MFM/RLL drives. Mostly useless on
        // modern drives but included fer completeness and paranoia.
        // Also makes excellent small talk at parties. "Did you know
        // about the Gutmann wipe patterns?" No? Just me then.
        let patterns: [u8; 27] = [
            0x55, 0xAA, 0x92, 0x49, 0x24, 0x00, 0x11, 0x22,
            0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA,
            0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x92, 0x49, 0x24,
            0x6D, 0xB6, 0xDB,
        ];

        for &p in &patterns {
            passes.push(WipePass::Pattern(p));
        }

        // Passes 32-35: More random (fer good measure, can't be too careful)
        for _ in 0..4 {
            passes.push(WipePass::Random);
        }

        passes
    }

    /// Parse the wipe method from a string
    ///
    /// Defaults to DoD if the string is unrecognized. Better to be
    /// too secure than not secure enough, says the paranoid pirate.
    fn parse_method(s: &str) -> WipeMethod {
        match s.to_lowercase().as_str() {
            "zero" => WipeMethod::Zero,
            "random" => WipeMethod::Random,
            "dod" | "dod3" => WipeMethod::Dod3Pass,
            "gutmann" => WipeMethod::Gutmann,
            _ => WipeMethod::Dod3Pass, // Default to DoD. When in doubt, be paranoid.
        }
    }
}

/// Internal enum fer the different types of wipe passes
///
/// Zero = all 0x00, Pattern = specific byte repeated, Random = entropy!
/// Very exciting stuff if yer into that sort of thing. And clearly ye are,
/// since yer readin' the source code of a secure deletion module.
#[derive(Debug, Clone, Copy)]
enum WipePass {
    Zero,
    Pattern(u8),
    Random,
}

/// Report of what happened during the scuttle operation
///
/// Because ye probably want to know if yer sensitive files actually
/// got deleted or if they're still sittin' there incriminatin' ye.
/// Knowledge is power. Also plausible deniability.
#[derive(Debug, Default)]
pub struct ScuttleReport {
    /// Which wipe method was used
    pub method: String,
    /// Files/dirs that were successfully obliterated. Gone. Poof. Bye.
    pub deleted: Vec<String>,
    /// Paths that didn't exist (can't delete what ain't there, mate)
    pub skipped: Vec<String>,
    /// Paths that failed to delete (with error messages explaining the cock-up)
    pub failed: Vec<(String, String)>,
}

impl std::fmt::Display for ScuttleReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Scuttle Report (method: {})", self.method)?;
        writeln!(f, "=================================")?;

        if !self.deleted.is_empty() {
            writeln!(f, "\nDeleted ({}):", self.deleted.len())?;
            for path in &self.deleted {
                writeln!(f, "  [GONE] {}", path)?;
            }
        }

        if !self.skipped.is_empty() {
            writeln!(f, "\nSkipped ({}):", self.skipped.len())?;
            for path in &self.skipped {
                writeln!(f, "  [MISSING] {} (wasn't there anyway)", path)?;
            }
        }

        if !self.failed.is_empty() {
            writeln!(f, "\nFailed ({}):", self.failed.len())?;
            for (path, err) in &self.failed {
                writeln!(f, "  [OOPS] {}: {}", path, err)?;
            }
        }

        Ok(())
    }
}

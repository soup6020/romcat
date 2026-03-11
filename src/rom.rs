use std::fmt;
use std::fs;
use std::path::Path;

use crate::formats::{self, Endianness, ParsedRom, RomFormat};

/// Complete information about a ROM file.
pub struct RomInfo {
    pub file_name: String,
    pub file_size: u64,
    pub format: RomFormat,
    pub game_name: Option<String>,
    pub endianness: Endianness,
    pub mapper: Option<String>,
    pub expansion_chip: Option<String>,
    pub md5: String,
}

/// Human-readable file size.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Error type for ROM analysis failures.
#[derive(Debug)]
pub enum RomError {
    Io(std::io::Error),
}

impl fmt::Display for RomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl From<std::io::Error> for RomError {
    fn from(e: std::io::Error) -> Self {
        RomError::Io(e)
    }
}

/// Analyze a ROM file and return its metadata.
pub fn analyze(path: &Path) -> Result<RomInfo, RomError> {
    let data = fs::read(path)?;
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());
    let file_size = data.len() as u64;

    // Compute MD5
    let md5 = format!("{:x}", md5::compute(&data));

    // Try each format parser in order of specificity.
    // NES and N64 have clear magic bytes so they go first.
    // GBA has a fixed byte + logo check.
    // GB/GBC has a logo check.
    // SNES is heuristic-based and goes last.
    let parsers: &[fn(&[u8]) -> Option<ParsedRom>] = &[
        formats::nes::parse,
        formats::n64::parse,
        formats::gba::parse,
        formats::gb::parse,
        formats::snes::parse,
    ];

    let parsed = parsers.iter().find_map(|parser| parser(&data));

    match parsed {
        Some(p) => Ok(RomInfo {
            file_name,
            file_size,
            format: p.format,
            game_name: p.game_name,
            endianness: p.endianness,
            mapper: p.mapper,
            expansion_chip: p.expansion_chip,
            md5,
        }),
        None => Ok(RomInfo {
            file_name,
            file_size,
            format: RomFormat::Unknown,
            game_name: None,
            endianness: Endianness::NotApplicable,
            mapper: None,
            expansion_chip: None,
            md5,
        }),
    }
}

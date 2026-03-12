use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use md5::{Digest, Md5};

use crate::formats::{self, Endianness, ParsedRom, RomFormat};

/// Maximum number of bytes to read for header detection.
/// 1 MB is more than enough for all supported formats — the largest header
/// requirements are SNES HiROM (~64 KB), PS1 filesystem traversal (~52 KB),
/// and 3DS NCSD/NCCH/ExeFS/SMDH chains (~100 KB+).
const MAX_HEADER_READ: u64 = 1024 * 1024;

/// Buffer size for streaming MD5 computation.
const MD5_BUF_SIZE: usize = 64 * 1024;

/// Complete information about a ROM file.
pub struct RomInfo {
    pub file_name: String,
    pub file_size: u64,
    pub format: RomFormat,
    pub game_name: Option<String>,
    pub cart_id: Option<String>,
    pub endianness: Endianness,
    pub mapper: Option<String>,
    pub expansion_chip: Option<String>,
    pub md5: Option<String>,
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
    Io(io::Error),
}

impl fmt::Display for RomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl From<io::Error> for RomError {
    fn from(e: io::Error) -> Self {
        RomError::Io(e)
    }
}

/// Compute the MD5 hash of a file by streaming it in chunks.
fn compute_md5(file: &mut File) -> io::Result<String> {
    file.seek(SeekFrom::Start(0))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Md5::new();
    let mut buf = [0u8; MD5_BUF_SIZE];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Analyze a ROM file and return its metadata.
/// If `compute_hash` is false, the MD5 field will be None (much faster for large files).
pub fn analyze(path: &Path, compute_hash: bool) -> Result<RomInfo, RomError> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    // Read only the header portion needed for format detection.
    let header_read_size = file_size.min(MAX_HEADER_READ) as usize;
    let mut header = vec![0u8; header_read_size];
    file.read_exact(&mut header)?;

    // Try each format parser in order of specificity.
    // CHD, NES, N64, Wii (RVZ/WIA/WBFS/ISO), and 3DS have clear magic bytes.
    // NDS and GBA share the same Nintendo logo — NDS is tried first
    // because it has additional validation (unit code, ARM9 offset).
    // GB/GBC has a different logo check.
    // PS1 relies on ISO 9660 + PLAYSTATION system identifier.
    // SNES is heuristic-based and goes last.
    let parsers: &[fn(&[u8]) -> Option<ParsedRom>] = &[
        formats::chd::parse,
        formats::nes::parse,
        formats::n64::parse,
        formats::wii::parse,
        formats::n3ds::parse,
        formats::nds::parse,
        formats::gba::parse,
        formats::gb::parse,
        formats::ps1::parse,
        formats::snes::parse,
    ];

    let parsed = parsers.iter().find_map(|parser| parser(&header));

    // Stream the MD5 hash (does not load the full file into memory)
    let md5 = if compute_hash {
        Some(compute_md5(&mut file)?)
    } else {
        None
    };

    match parsed {
        Some(p) => Ok(RomInfo {
            file_name,
            file_size,
            format: p.format,
            game_name: p.game_name,
            cart_id: p.cart_id,
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
            cart_id: None,
            endianness: Endianness::NotApplicable,
            mapper: None,
            expansion_chip: None,
            md5,
        }),
    }
}

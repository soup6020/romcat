pub mod chd;
pub mod gb;
pub mod gba;
pub mod n64;
pub mod n3ds;
pub mod nds;
pub mod nes;
pub mod ps1;
pub mod snes;
pub mod wii;

use std::fmt;

/// The detected ROM format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomFormat {
    Nes,
    Nes2,
    N64,
    Gba,
    GameBoy,
    GameBoyColor,
    Snes,
    Nds,
    Ps1,
    Wii,
    WiiWbfs,
    WiiRvz,
    WiiWia,
    N3ds,
    Chd,
    Unknown,
}

impl fmt::Display for RomFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomFormat::Nes => write!(f, "NES"),
            RomFormat::Nes2 => write!(f, "NES 2.0"),
            RomFormat::N64 => write!(f, "N64"),
            RomFormat::Gba => write!(f, "GBA"),
            RomFormat::GameBoy => write!(f, "Game Boy"),
            RomFormat::GameBoyColor => write!(f, "Game Boy Color"),
            RomFormat::Snes => write!(f, "SNES"),
            RomFormat::Nds => write!(f, "NDS"),
            RomFormat::Ps1 => write!(f, "PS1"),
            RomFormat::Wii => write!(f, "Wii"),
            RomFormat::WiiWbfs => write!(f, "Wii (WBFS)"),
            RomFormat::WiiRvz => write!(f, "Wii (RVZ)"),
            RomFormat::WiiWia => write!(f, "Wii (WIA)"),
            RomFormat::N3ds => write!(f, "3DS"),
            RomFormat::Chd => write!(f, "CHD"),
            RomFormat::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Byte order of the ROM data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endianness {
    Big,
    Little,
    ByteSwapped,
    NotApplicable,
}

impl fmt::Display for Endianness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Endianness::Big => write!(f, "Big-Endian"),
            Endianness::Little => write!(f, "Little-Endian"),
            Endianness::ByteSwapped => write!(f, "Byte-Swapped"),
            Endianness::NotApplicable => write!(f, "N/A"),
        }
    }
}

/// Result of successfully parsing a ROM header.
#[derive(Debug, Clone)]
pub struct ParsedRom {
    pub format: RomFormat,
    pub game_name: Option<String>,
    pub cart_id: Option<String>,
    pub endianness: Endianness,
    pub mapper: Option<String>,
    pub expansion_chip: Option<String>,
}

/// Clean up a title string extracted from a ROM header.
/// Trims null bytes, non-ASCII bytes, and whitespace.
pub fn clean_title(bytes: &[u8]) -> Option<String> {
    let s: String = bytes
        .iter()
        .take_while(|&&b| b != 0)
        .filter(|&&b| b.is_ascii_graphic() || b == b' ')
        .map(|&b| b as char)
        .collect();
    let trimmed = s.trim();
    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
}

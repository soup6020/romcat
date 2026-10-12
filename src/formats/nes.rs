use super::{Endianness, ParsedRom, RomFormat};

/// iNES magic: "NES\x1A"
const INES_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

/// NES mapper names for the most common mappers.
fn mapper_name(id: u16) -> String {
    let name = match id {
        0 => "NROM",
        1 => "MMC1 (SxROM)",
        2 => "UxROM",
        3 => "CNROM",
        4 => "MMC3 (TxROM)",
        5 => "MMC5 (ExROM)",
        7 => "AxROM",
        9 => "MMC2 (PxROM)",
        10 => "MMC4 (FxROM)",
        11 => "Color Dreams",
        16 => "Bandai FCG",
        18 => "Jaleco SS 88006",
        19 => "Namco 129/163",
        21 => "VRC4a/VRC4c",
        22 => "VRC2a",
        23 => "VRC2b/VRC4e",
        24 => "VRC6a",
        25 => "VRC4b/VRC4d",
        26 => "VRC6b",
        34 => "BNROM/NINA-001",
        64 => "RAMBO-1",
        66 => "GxROM",
        69 => "Sunsoft FME-7",
        71 => "Codemasters",
        85 => "VRC7",
        118 => "MMC3 (TxSROM)",
        119 => "MMC3 (TQROM)",
        206 => "Namco 108/MIMIC-1",
        210 => "Namco 175/340",
        _ => "",
    };
    if name.is_empty() {
        format!("Mapper {id}")
    } else {
        format!("{name} ({id})")
    }
}

/// Try to parse an iNES or NES 2.0 ROM.
pub fn parse(data: &[u8]) -> Option<ParsedRom> {
    if data.len() < 16 {
        return None;
    }
    if data[0..4] != INES_MAGIC {
        return None;
    }

    let flags6 = data[6];
    let flags7 = data[7];

    // Mapper number: low nibble from flags6 high nibble, high nibble from flags7 high nibble
    let mapper_lo = (flags6 >> 4) as u16;
    let mapper_hi = (flags7 >> 4) as u16;
    let mut mapper_id = mapper_hi << 4 | mapper_lo;

    // NES 2.0 detection: bits 3-2 of flags7 == 0b10
    let is_nes2 = (flags7 & 0x0C) == 0x08;

    if is_nes2 && data.len() >= 9 {
        // NES 2.0 has extended mapper bits in byte 8
        let mapper_ext = (data[8] as u16 & 0x0F) << 8;
        mapper_id |= mapper_ext;
    }

    let format = if is_nes2 {
        RomFormat::Nes2
    } else {
        RomFormat::Nes
    };

    Some(ParsedRom {
        format,
        game_name: None, // iNES has no title field
        cart_id: None,   // iNES has no cart ID field
        endianness: Endianness::Little,
        mapper: Some(mapper_name(mapper_id)),
        expansion_chip: None,
    })
}

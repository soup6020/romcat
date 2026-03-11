use super::{Endianness, ParsedRom, RomFormat, clean_title};

/// The first 16 bytes of the Nintendo logo at offset 0x04.
/// Used as a quick-check for GBA identification.
const NINTENDO_LOGO_PREFIX: [u8; 16] = [
    0x24, 0xFF, 0xAE, 0x51, 0x69, 0x9A, 0xA2, 0x21,
    0x3D, 0x84, 0x82, 0x0A, 0x84, 0xE4, 0x09, 0xAD,
];

/// Minimum bytes needed: through the end of the header at 0xC0.
const MIN_HEADER_SIZE: usize = 0xC0;

/// Try to parse a GBA ROM header.
pub fn parse(data: &[u8]) -> Option<ParsedRom> {
    if data.len() < MIN_HEADER_SIZE {
        return None;
    }

    // Check the fixed byte at 0xB2 (must be 0x96)
    if data[0xB2] != 0x96 {
        return None;
    }

    // Verify the Nintendo logo prefix at 0x04
    if data[0x04..0x14] != NINTENDO_LOGO_PREFIX {
        return None;
    }

    // Game title: 12 bytes at 0xA0
    let title = clean_title(&data[0xA0..0xAC]);

    // Game code: 4 bytes at 0xAC (e.g. "AXVE" for Pokemon Ruby US)
    let game_code = clean_title(&data[0xAC..0xB0]);

    // Combine title and game code for a more informative name
    let game_name = match (title, game_code) {
        (Some(t), Some(c)) => Some(format!("{t} ({c})")),
        (Some(t), None) => Some(t),
        (None, Some(c)) => Some(c),
        (None, None) => None,
    };

    Some(ParsedRom {
        format: RomFormat::Gba,
        game_name,
        endianness: Endianness::Little,
        mapper: None, // GBA doesn't use mappers
        expansion_chip: None,
    })
}

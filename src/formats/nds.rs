use super::{Endianness, ParsedRom, RomFormat, clean_title};

/// The Nintendo logo at offset 0xC0 in the NDS header.
/// First 16 bytes used as a quick-check for identification.
const NINTENDO_LOGO_PREFIX: [u8; 16] = [
    0x24, 0xFF, 0xAE, 0x51, 0x69, 0x9A, 0xA2, 0x21,
    0x3D, 0x84, 0x82, 0x0A, 0x84, 0xE4, 0x09, 0xAD,
];

/// Minimum header size: we need at least through the Nintendo logo area.
const MIN_HEADER_SIZE: usize = 0x200;

/// Try to read the full game title from the Icon/Title structure.
/// The English title is at offset +0x340 within the structure, encoded as UTF-16LE.
fn read_banner_title(data: &[u8], banner_offset: usize) -> Option<String> {
    // Need at least the English title: banner_offset + 0x340 + 256 bytes
    let title_offset = banner_offset + 0x340;
    if data.len() < title_offset + 256 {
        return None;
    }

    // Read UTF-16LE code units (128 chars = 256 bytes)
    let mut chars: Vec<u16> = Vec::with_capacity(128);
    for i in (0..256).step_by(2) {
        let lo = data[title_offset + i] as u16;
        let hi = data[title_offset + i + 1] as u16;
        let code_unit = lo | (hi << 8);
        if code_unit == 0 {
            break;
        }
        chars.push(code_unit);
    }

    let s = String::from_utf16_lossy(&chars);
    // The banner title often has newlines (used for multi-line display on the DS).
    // Replace them with spaces for a single-line table display.
    let cleaned: String = s
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Try to parse a Nintendo DS ROM header.
pub fn parse(data: &[u8]) -> Option<ParsedRom> {
    if data.len() < MIN_HEADER_SIZE {
        return None;
    }

    // Verify the Nintendo logo at 0xC0
    if data[0xC0..0xD0] != NINTENDO_LOGO_PREFIX {
        return None;
    }

    // Game code at 0x0C (4 bytes ASCII) — this is the cart ID
    let cart_id = clean_title(&data[0x0C..0x10]);

    // Validate that game code looks like ASCII letters/digits
    // (helps avoid false positives against GBA ROMs which share the same logo)
    if let Some(ref id) = cart_id {
        if !id.chars().all(|c| c.is_ascii_alphanumeric()) {
            return None;
        }
    }

    // Unit code at 0x12: 0x00 = NDS, 0x02 = NDS+DSi, 0x03 = DSi only
    let unit_code = data[0x12];
    if unit_code != 0x00 && unit_code != 0x02 && unit_code != 0x03 {
        return None;
    }

    // ARM9 ROM offset at 0x20 (LE u32) — should be >= 0x4000 for official ROMs
    let arm9_offset = u32::from_le_bytes([data[0x20], data[0x21], data[0x22], data[0x23]]);
    if arm9_offset < 0x1000 {
        return None;
    }

    // Try to read the full banner title first, fall back to the short 12-byte title
    let banner_offset =
        u32::from_le_bytes([data[0x68], data[0x69], data[0x6A], data[0x6B]]) as usize;
    let game_name = read_banner_title(data, banner_offset)
        .or_else(|| clean_title(&data[0x00..0x0C]));

    Some(ParsedRom {
        format: RomFormat::Nds,
        game_name,
        cart_id,
        endianness: Endianness::Little,
        mapper: None,
        expansion_chip: None,
    })
}

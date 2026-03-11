use super::{Endianness, ParsedRom, RomFormat, clean_title};

/// First 8 bytes of the Nintendo logo at 0x0104.
const NINTENDO_LOGO_PREFIX: [u8; 8] = [0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B];

/// Minimum header size needed: 0x0150 (end of GB header).
const MIN_HEADER_SIZE: usize = 0x0150;

/// MBC / cartridge type names from byte 0x0147.
fn cartridge_type_name(code: u8) -> &'static str {
    match code {
        0x00 => "ROM Only",
        0x01 => "MBC1",
        0x02 => "MBC1+RAM",
        0x03 => "MBC1+RAM+Battery",
        0x05 => "MBC2",
        0x06 => "MBC2+Battery",
        0x08 => "ROM+RAM",
        0x09 => "ROM+RAM+Battery",
        0x0B => "MMM01",
        0x0C => "MMM01+RAM",
        0x0D => "MMM01+RAM+Battery",
        0x0F => "MBC3+Timer+Battery",
        0x10 => "MBC3+Timer+RAM+Battery",
        0x11 => "MBC3",
        0x12 => "MBC3+RAM",
        0x13 => "MBC3+RAM+Battery",
        0x19 => "MBC5",
        0x1A => "MBC5+RAM",
        0x1B => "MBC5+RAM+Battery",
        0x1C => "MBC5+Rumble",
        0x1D => "MBC5+Rumble+RAM",
        0x1E => "MBC5+Rumble+RAM+Battery",
        0x20 => "MBC6",
        0x22 => "MBC7+Sensor+Rumble+RAM+Battery",
        0xFC => "Pocket Camera",
        0xFD => "Bandai TAMA5",
        0xFE => "HuC3",
        0xFF => "HuC1+RAM+Battery",
        _ => "Unknown",
    }
}

/// Try to parse a Game Boy or Game Boy Color ROM.
pub fn parse(data: &[u8]) -> Option<ParsedRom> {
    if data.len() < MIN_HEADER_SIZE {
        return None;
    }

    // Verify the Nintendo logo at 0x0104
    if data[0x0104..0x010C] != NINTENDO_LOGO_PREFIX {
        return None;
    }

    // CGB flag at 0x0143
    let cgb_flag = data[0x0143];
    let is_gbc = cgb_flag == 0x80 || cgb_flag == 0xC0;

    let format = if is_gbc {
        RomFormat::GameBoyColor
    } else {
        RomFormat::GameBoy
    };

    // Title: for GBC, only 11 bytes (0x0134-0x013E); for GB, 16 bytes (0x0134-0x0143)
    let title_end = if is_gbc { 0x013F } else { 0x0144 };
    let title = clean_title(&data[0x0134..title_end]);

    // Cartridge type (mapper) at 0x0147
    let cart_type = data[0x0147];
    let mapper = cartridge_type_name(cart_type);

    Some(ParsedRom {
        format,
        game_name: title,
        endianness: Endianness::Little,
        mapper: Some(mapper.to_string()),
        expansion_chip: None,
    })
}

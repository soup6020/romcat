use super::{Endianness, ParsedRom, RomFormat, clean_title};

/// SNES header candidate at a given base offset.
struct SnesHeader {
    title: Option<String>,
    map_mode: u8,
    rom_type: u8,
    chipset_subtype: u8,
    score: u32,
}

/// Expansion chip / coprocessor names derived from the ROM type byte and chipset subtype.
///
/// The ROM type byte at header+0x16 (relative to title start at +0x15 offset from header base)
/// encodes the chipset. Bits 0-3 indicate the base type and bits 4-7 indicate coprocessor presence.
///
/// Reference: https://snes.nesdev.org/wiki/ROM_header
fn expansion_chip_name(rom_type: u8, chipset_subtype: u8) -> Option<&'static str> {
    // High nibble indicates coprocessor category
    let coprocessor = rom_type & 0xF0;
    // Low nibble indicates base feature set
    let base = rom_type & 0x0F;

    match coprocessor {
        0x00 => {
            // No coprocessor
            // base: 0=ROM, 1=ROM+RAM, 2=ROM+RAM+Battery
            None
        }
        0x10 => {
            // DSP coprocessor family
            match chipset_subtype {
                0x00 => Some("DSP-1"),
                0x01 => Some("DSP-2"),
                0x02 => Some("DSP-3"),
                0x03 => Some("DSP-4"),
                _ => Some("DSP"),
            }
        }
        0x20 => {
            // SuperFX / GSU
            if base <= 0x05 {
                Some("SuperFX (GSU)")
            } else {
                Some("SuperFX")
            }
        }
        0x30 => {
            // OBC1
            Some("OBC-1")
        }
        0x40 => {
            // SA-1
            Some("SA-1")
        }
        0x50 => {
            // S-DD1 or S-RTC
            match chipset_subtype {
                0x00 => Some("S-DD1"),
                0x01 => Some("S-RTC"),
                _ => Some("S-DD1"),
            }
        }
        0xE0 => {
            // Other / special
            match chipset_subtype {
                0x00 => Some("Other"),
                0x01 => Some("SPC7110"),
                0x02 => Some("ST010/ST011"),
                0x03 => Some("ST018"),
                0x04 => Some("Cx4"),
                _ => Some("Other"),
            }
        }
        0xF0 => {
            // Custom chipset (used by a few special carts)
            match chipset_subtype {
                0x00 => Some("Custom"),
                0x01 => Some("SPC7110+RTC"),
                0x02 => Some("S-DD1+OBC1"),
                _ => Some("Custom"),
            }
        }
        _ => None,
    }
}

/// Map mode name from the map mode byte at header+0x15.
fn map_mode_name(map_mode: u8) -> &'static str {
    match map_mode & 0x2F {
        0x20 => "LoROM",
        0x21 => "HiROM",
        0x23 => "SA-1 ROM",
        0x25 => "ExHiROM",
        _ => "Unknown",
    }
}

/// Try to read and score a SNES header at the given offset.
fn try_header(data: &[u8], offset: usize) -> Option<SnesHeader> {
    // We need at least offset + 0x30 bytes (header is 0x30 bytes from the title start)
    if data.len() < offset + 0x30 {
        return None;
    }

    // Title: 21 bytes at offset
    let title_bytes = &data[offset..offset + 21];
    let title = clean_title(title_bytes);

    // Map mode at offset + 0x15
    let map_mode = data[offset + 0x15];

    // ROM type at offset + 0x16
    let rom_type = data[offset + 0x16];

    // Chipset subtype at offset + 0x1A (only meaningful for NES 2.0-style extended headers,
    // but the byte is present in the SNES header)
    let chipset_subtype = if data.len() > offset + 0x1A {
        data[offset + 0x1A]
    } else {
        0
    };

    // Checksum complement at offset + 0x1C (2 bytes, little-endian)
    let complement = u16::from_le_bytes([data[offset + 0x1C], data[offset + 0x1D]]);
    // Checksum at offset + 0x1E (2 bytes, little-endian)
    let checksum = u16::from_le_bytes([data[offset + 0x1E], data[offset + 0x1F]]);

    // Score this header candidate
    let mut score: u32 = 0;

    // Checksum complement test: checksum XOR complement should be 0xFFFF
    if checksum ^ complement == 0xFFFF {
        score += 100;
    }

    // Title should be printable ASCII
    let printable_count = title_bytes
        .iter()
        .filter(|&&b| (0x20..=0x7E).contains(&b))
        .count();
    score += printable_count as u32 * 2;

    // Map mode should have bit 5 set (all valid map modes do)
    if map_mode & 0x20 != 0 {
        score += 20;
    }

    // ROM type base nibble should be 0-2 typically
    if (rom_type & 0x0F) <= 0x02 {
        score += 5;
    }

    // ROM size byte at offset + 0x17 should be reasonable (0x08-0x0D = 256KB-8MB)
    let rom_size = data[offset + 0x17];
    if (0x05..=0x0D).contains(&rom_size) {
        score += 10;
    }

    // RAM size byte at offset + 0x18 should be reasonable (0x00-0x07)
    let ram_size = data[offset + 0x18];
    if ram_size <= 0x07 {
        score += 5;
    }

    Some(SnesHeader {
        title,
        map_mode,
        rom_type,
        chipset_subtype,
        score,
    })
}

/// Try to parse a SNES ROM.
pub fn parse(data: &[u8]) -> Option<ParsedRom> {
    // Determine if there's a 512-byte copier header
    let has_copier_header = data.len() % 1024 == 512;
    let base = if has_copier_header { 512 } else { 0 };

    // Minimum useful size: need at least LoROM header location + header size
    if data.len() < base + 0x8000 {
        return None;
    }

    // Try LoROM header at 0x7FC0 and HiROM header at 0xFFC0
    let lorom_header = try_header(data, base + 0x7FC0);
    let hirom_header = if data.len() >= base + 0xFFC0 + 0x30 {
        try_header(data, base + 0xFFC0)
    } else {
        None
    };

    // Pick the best candidate by score
    let header = match (lorom_header, hirom_header) {
        (Some(lo), Some(hi)) => {
            if hi.score > lo.score {
                hi
            } else {
                lo
            }
        }
        (Some(lo), None) => lo,
        (None, Some(hi)) => hi,
        (None, None) => return None,
    };

    // Require a minimum score to avoid false positives
    // A valid SNES ROM should at least pass the checksum complement test
    if header.score < 50 {
        return None;
    }

    let mapper = map_mode_name(header.map_mode);
    let expansion = expansion_chip_name(header.rom_type, header.chipset_subtype);

    Some(ParsedRom {
        format: RomFormat::Snes,
        game_name: header.title,
        cart_id: None, // SNES has no standardised cart ID
        endianness: Endianness::Little,
        mapper: Some(mapper.to_string()),
        expansion_chip: expansion.map(|s| s.to_string()),
    })
}

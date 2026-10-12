use super::{Endianness, ParsedRom, RomFormat, clean_title};

/// NCSD magic at offset 0x100.
const NCSD_MAGIC: [u8; 4] = [0x4E, 0x43, 0x53, 0x44]; // "NCSD"

/// NCCH magic at partition header + 0x100.
const NCCH_MAGIC: [u8; 4] = [0x4E, 0x43, 0x43, 0x48]; // "NCCH"

/// SMDH magic.
const SMDH_MAGIC: [u8; 4] = [0x53, 0x4D, 0x44, 0x48]; // "SMDH"

/// Media unit size (all NCSD/NCCH offsets are in media units of 0x200 bytes).
const MEDIA_UNIT: usize = 0x200;

/// Minimum size for detection: need at least past the NCSD header.
const MIN_SIZE: usize = 0x200;

/// Read the English short title from an SMDH block.
/// SMDH application titles start at offset 0x08, with 16 language entries of 0x200 bytes each.
/// English is entry index 1 (offset 0x08 + 0x200 = 0x208).
/// Each entry: 0x80 bytes short desc (UTF-16LE) + 0x100 bytes long desc + 0x80 bytes publisher.
fn read_smdh_title(smdh: &[u8]) -> Option<String> {
    if smdh.len() < 4 || smdh[0..4] != SMDH_MAGIC {
        return None;
    }

    // English short title at 0x208, 0x80 bytes (64 UTF-16LE chars)
    let short_offset = 0x208;
    // English long title at 0x288, 0x100 bytes (128 UTF-16LE chars)
    let long_offset = 0x288;

    // Try long title first, fall back to short
    read_utf16le_string(smdh, long_offset, 0x100)
        .or_else(|| read_utf16le_string(smdh, short_offset, 0x80))
}

/// Read a null-terminated UTF-16LE string from a buffer.
fn read_utf16le_string(data: &[u8], offset: usize, max_bytes: usize) -> Option<String> {
    if data.len() < offset + max_bytes {
        return None;
    }

    let mut chars: Vec<u16> = Vec::new();
    for i in (0..max_bytes).step_by(2) {
        let lo = data[offset + i] as u16;
        let hi = data[offset + i + 1] as u16;
        let code_unit = lo | (hi << 8);
        if code_unit == 0 {
            break;
        }
        chars.push(code_unit);
    }

    let s = String::from_utf16_lossy(&chars);
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Try to find and read the SMDH (icon) from the ExeFS of partition 0.
/// ExeFS has a header with 10 file entries of 16 bytes each (8-byte name + 4-byte offset + 4-byte size),
/// followed by the file data. The "icon" file contains the SMDH.
fn read_exefs_smdh(data: &[u8], ncch_offset: usize) -> Option<String> {
    // ExeFS offset is at NCCH + 0x1A0 (LE u32 in media units)
    if data.len() < ncch_offset + 0x1A4 {
        return None;
    }
    let exefs_mu = u32::from_le_bytes([
        data[ncch_offset + 0x1A0],
        data[ncch_offset + 0x1A1],
        data[ncch_offset + 0x1A2],
        data[ncch_offset + 0x1A3],
    ]) as usize;

    if exefs_mu == 0 {
        return None;
    }

    let exefs_offset = ncch_offset + exefs_mu * MEDIA_UNIT;

    // ExeFS header: 10 entries of 16 bytes
    // Each entry: 8-byte name (ASCII null-padded), 4-byte offset (LE), 4-byte size (LE)
    // File data offsets are relative to the end of the ExeFS header (0x200 bytes)
    if data.len() < exefs_offset + 0x200 {
        return None;
    }

    for i in 0..10 {
        let entry_offset = exefs_offset + i * 16;
        let name_bytes = &data[entry_offset..entry_offset + 8];
        let name = String::from_utf8_lossy(name_bytes)
            .trim_end_matches('\0')
            .to_string();

        if name == "icon" {
            let file_offset_raw = u32::from_le_bytes([
                data[entry_offset + 8],
                data[entry_offset + 9],
                data[entry_offset + 10],
                data[entry_offset + 11],
            ]) as usize;
            let file_size = u32::from_le_bytes([
                data[entry_offset + 12],
                data[entry_offset + 13],
                data[entry_offset + 14],
                data[entry_offset + 15],
            ]) as usize;

            // File data starts after the ExeFS header (0x200 bytes)
            let smdh_offset = exefs_offset + 0x200 + file_offset_raw;
            if file_size > 0 && data.len() >= smdh_offset + file_size {
                return read_smdh_title(&data[smdh_offset..smdh_offset + file_size]);
            }
        }
    }

    None
}

/// Try to parse a 3DS cartridge dump (NCSD / .3ds / .cci).
pub fn parse(data: &[u8]) -> Option<ParsedRom> {
    if data.len() < MIN_SIZE {
        return None;
    }

    // NCSD magic at 0x100
    if data[0x100..0x104] != NCSD_MAGIC {
        return None;
    }

    // Partition table at 0x120: first entry is partition 0
    // Offset (LE u32 in media units) at 0x120, size at 0x124
    let part0_offset_mu =
        u32::from_le_bytes([data[0x120], data[0x121], data[0x122], data[0x123]]) as usize;

    if part0_offset_mu == 0 {
        return None;
    }

    let ncch_offset = part0_offset_mu * MEDIA_UNIT;

    // Verify NCCH magic at partition 0
    if data.len() < ncch_offset + 0x200 {
        return None;
    }
    if data[ncch_offset + 0x100..ncch_offset + 0x104] != NCCH_MAGIC {
        return None;
    }

    // Product code at NCCH + 0x150 (16 bytes ASCII, e.g. "CTR-P-AMCE")
    let product_code = clean_title(&data[ncch_offset + 0x150..ncch_offset + 0x160]);

    // Try to extract a short cart ID from the product code
    // Format is typically "CTR-P-XXXX" — we want the last 4 chars
    let cart_id = product_code.as_ref().map(|code| {
        if code.len() >= 10 && code.contains('-') {
            code.rsplit('-').next().unwrap_or(code).to_string()
        } else {
            code.clone()
        }
    });

    // Try to read the game title from the SMDH in ExeFS
    let game_name = read_exefs_smdh(data, ncch_offset);

    // Fall back to the product code as game name if SMDH isn't readable
    // (e.g. encrypted dumps)
    let game_name = game_name.or_else(|| product_code.clone());

    Some(ParsedRom {
        format: RomFormat::N3ds,
        game_name,
        cart_id,
        endianness: Endianness::Little,
        mapper: None,
        expansion_chip: None,
    })
}

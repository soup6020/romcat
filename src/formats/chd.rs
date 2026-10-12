use super::{Endianness, ParsedRom, RomFormat};

/// CHD magic: "MComprHD"
const CHD_MAGIC: [u8; 8] = [0x4D, 0x43, 0x6F, 0x6D, 0x70, 0x72, 0x48, 0x44];

/// Minimum header size for any CHD version.
const MIN_HEADER_SIZE: usize = 16;

/// Read a big-endian u32 from a byte slice.
fn read_be_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

/// Read a big-endian u64 from a byte slice.
fn read_be_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ])
}

/// Get the metadata offset for the given CHD version.
fn meta_offset_for_version(data: &[u8], version: u32) -> Option<u64> {
    match version {
        3 => {
            // v3: meta offset is at byte 36 (BE u64)
            if data.len() < 44 {
                return None;
            }
            Some(read_be_u64(data, 36))
        }
        4 => {
            // v4: meta offset is at byte 36 (BE u64)
            if data.len() < 44 {
                return None;
            }
            Some(read_be_u64(data, 36))
        }
        5 => {
            // v5: meta offset is at byte 48 (0x30, BE u64)
            if data.len() < 56 {
                return None;
            }
            Some(read_be_u64(data, 0x30))
        }
        _ => None,
    }
}

/// Walk the CHD metadata linked list and collect all entries.
/// Each metadata entry: 4-byte tag, 4-byte length+flags, 8-byte next pointer, then value.
fn read_metadata(data: &[u8], start_offset: u64) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut offset = start_offset as usize;

    // Safety limit to prevent infinite loops on corrupt files
    let max_entries = 256;

    for _ in 0..max_entries {
        if offset == 0 || data.len() < offset + 16 {
            break;
        }

        // Tag: 4 bytes ASCII
        let tag = String::from_utf8_lossy(&data[offset..offset + 4]).to_string();

        // Length + flags: BE u32. Lower 24 bits = length.
        let length_flags = read_be_u32(data, offset + 4);
        let length = (length_flags & 0x00FF_FFFF) as usize;

        // Next pointer: BE u64
        let next = read_be_u64(data, offset + 8);

        // Value: `length` bytes starting at offset + 16
        let value_start = offset + 16;
        if data.len() >= value_start + length {
            let value = String::from_utf8_lossy(&data[value_start..value_start + length])
                .trim_end_matches('\0')
                .to_string();
            entries.push((tag, value));
        }

        if next == 0 {
            break;
        }
        offset = next as usize;
    }

    entries
}

/// Try to identify the disc system from CHD metadata track info.
/// Returns a descriptive string if we can determine what kind of disc this is.
fn identify_disc_type(metadata: &[(String, String)]) -> Option<String> {
    for (tag, value) in metadata {
        // CD-ROM track metadata (CHTR or CHT2) indicates a CD-based system
        if tag == "CHTR" || tag == "CHT2" {
            let upper = value.to_uppercase();
            if upper.contains("MODE2") {
                return Some("CD-ROM (Mode 2)".to_string());
            } else if upper.contains("MODE1") {
                return Some("CD-ROM (Mode 1)".to_string());
            }
            return Some("CD-ROM".to_string());
        }
        // GD-ROM (Dreamcast)
        if tag == "CHGD" {
            return Some("GD-ROM".to_string());
        }
        // Hard disk
        if tag == "GDDD" {
            return Some("Hard Disk".to_string());
        }
    }
    None
}

/// CHD version string for the mapper/info column.
fn version_string(version: u32) -> String {
    format!("CHD v{version}")
}

/// Try to parse a CHD file.
pub fn parse(data: &[u8]) -> Option<ParsedRom> {
    if data.len() < MIN_HEADER_SIZE {
        return None;
    }

    // Check magic
    if data[0..8] != CHD_MAGIC {
        return None;
    }

    // Version at offset 0x0C (BE u32)
    let version = read_be_u32(data, 0x0C);
    if !(1..=5).contains(&version) {
        return None;
    }

    // Read metadata if we can find the offset
    let metadata = if let Some(meta_off) = meta_offset_for_version(data, version) {
        if meta_off > 0 {
            read_metadata(data, meta_off)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let disc_type = identify_disc_type(&metadata);

    // Build the mapper string: "CHD v5" optionally with disc type
    let mapper = match disc_type {
        Some(dt) => format!("{}, {dt}", version_string(version)),
        None => version_string(version),
    };

    Some(ParsedRom {
        format: RomFormat::Chd,
        game_name: None, // CHD doesn't store game title in the header
        cart_id: None,   // CHD doesn't store a serial
        endianness: Endianness::Big,
        mapper: Some(mapper),
        expansion_chip: None,
    })
}

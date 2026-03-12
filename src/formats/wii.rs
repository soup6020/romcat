use super::{Endianness, ParsedRom, RomFormat, clean_title};

/// Wii disc magic at offset 0x18 (big-endian u32).
const WII_MAGIC: [u8; 4] = [0x5D, 0x1C, 0x9E, 0xA3];

/// WBFS container magic: "WBFS"
const WBFS_MAGIC: [u8; 4] = [0x57, 0x42, 0x46, 0x53];

/// RVZ magic: "RVZ\x01"
const RVZ_MAGIC: [u8; 4] = [0x52, 0x56, 0x5A, 0x01];

/// WIA magic: "WIA\x01"
const WIA_MAGIC: [u8; 4] = [0x57, 0x49, 0x41, 0x01];

/// Parse the standard Wii disc header fields (game ID and title) from a buffer
/// that starts at the beginning of the disc header.
fn parse_disc_header(header: &[u8]) -> (Option<String>, Option<String>) {
    // Game ID: 6 bytes at offset 0x00
    let cart_id = clean_title(&header[0x00..0x06]);

    // Game title: 64 bytes at offset 0x20, null-terminated ASCII
    let game_name = if header.len() >= 0x60 {
        clean_title(&header[0x20..0x60])
    } else {
        None
    };

    (cart_id, game_name)
}

/// Try to parse a standard Wii ISO disc image.
fn parse_wii_iso(data: &[u8]) -> Option<ParsedRom> {
    if data.len() < 0x60 {
        return None;
    }

    // Check Wii magic at 0x18
    if data[0x18..0x1C] != WII_MAGIC {
        return None;
    }

    let (cart_id, game_name) = parse_disc_header(data);

    Some(ParsedRom {
        format: RomFormat::Wii,
        game_name,
        cart_id,
        endianness: Endianness::Big,
        mapper: None,
        expansion_chip: None,
    })
}

/// Try to parse a WBFS container.
fn parse_wbfs(data: &[u8]) -> Option<ParsedRom> {
    if data.len() < 12 {
        return None;
    }

    // Check WBFS magic at 0x00
    if data[0x00..0x04] != WBFS_MAGIC {
        return None;
    }

    // HD sector shift at 0x08
    let hd_sector_shift = data[0x08] as u32;
    if hd_sector_shift > 24 {
        return None; // Sanity check
    }
    let hd_sector_size = 1usize << hd_sector_shift;

    // The Wii disc header starts at one HD sector in
    let disc_header_offset = hd_sector_size;
    if data.len() < disc_header_offset + 0x60 {
        return None;
    }

    // Verify the Wii magic in the embedded disc header
    if data[disc_header_offset + 0x18..disc_header_offset + 0x1C] != WII_MAGIC {
        return None;
    }

    let (cart_id, game_name) = parse_disc_header(&data[disc_header_offset..]);

    Some(ParsedRom {
        format: RomFormat::WiiWbfs,
        game_name,
        cart_id,
        endianness: Endianness::Big,
        mapper: None,
        expansion_chip: None,
    })
}

/// Try to parse an RVZ or WIA compressed disc image.
fn parse_rvz_wia(data: &[u8]) -> Option<ParsedRom> {
    if data.len() < 0x58 + 0x80 {
        return None;
    }

    let magic = &data[0x00..0x04];
    let format = if magic == RVZ_MAGIC {
        RomFormat::WiiRvz
    } else if magic == WIA_MAGIC {
        RomFormat::WiiWia
    } else {
        return None;
    };

    // The first 0x80 bytes of the original disc header are embedded at offset 0x58
    // (Header2 starts at 0x48, disc header is at Header2 + 0x10 = 0x58)
    let disc_header = &data[0x58..0x58 + 0x80];

    // Verify Wii magic within the embedded disc header at +0x18
    if disc_header[0x18..0x1C] != WII_MAGIC {
        return None;
    }

    let (cart_id, game_name) = parse_disc_header(disc_header);

    Some(ParsedRom {
        format,
        game_name,
        cart_id,
        endianness: Endianness::Big,
        mapper: None,
        expansion_chip: None,
    })
}

/// Try to parse a Wii disc image in any supported container format.
/// Tries RVZ/WIA first (clear 4-byte magic), then WBFS (clear 4-byte magic),
/// then raw ISO (magic at offset 0x18).
pub fn parse(data: &[u8]) -> Option<ParsedRom> {
    parse_rvz_wia(data)
        .or_else(|| parse_wbfs(data))
        .or_else(|| parse_wii_iso(data))
}

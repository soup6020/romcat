use super::{Endianness, ParsedRom, RomFormat, clean_title};

/// Magic bytes for the three N64 ROM byte orderings.
const Z64_MAGIC: [u8; 4] = [0x80, 0x37, 0x12, 0x40]; // Big-endian (native)
const N64_MAGIC: [u8; 4] = [0x40, 0x12, 0x37, 0x80]; // Little-endian
const V64_MAGIC: [u8; 4] = [0x37, 0x80, 0x40, 0x12]; // Byte-swapped (word-swapped)

/// Minimum header size needed: 0x3B (cart ID offset) + 4 (cart ID length) = 63 bytes.
const MIN_HEADER_SIZE: usize = 0x3B + 4;

/// Swap bytes from little-endian (.n64) layout to big-endian (.z64) layout.
fn le_to_be(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; data.len()];
    for i in (0..data.len() - 3).step_by(4) {
        out[i] = data[i + 3];
        out[i + 1] = data[i + 2];
        out[i + 2] = data[i + 1];
        out[i + 3] = data[i];
    }
    out
}

/// Swap bytes from byte-swapped (.v64) layout to big-endian (.z64) layout.
fn v64_to_be(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; data.len()];
    for i in (0..data.len() - 1).step_by(2) {
        out[i] = data[i + 1];
        out[i + 1] = data[i];
    }
    out
}

/// Try to parse an N64 ROM header.
pub fn parse(data: &[u8]) -> Option<ParsedRom> {
    if data.len() < MIN_HEADER_SIZE {
        return None;
    }

    let magic = &data[0..4];
    let (endianness, header_bytes) = if magic == Z64_MAGIC {
        (Endianness::Big, data[..MIN_HEADER_SIZE].to_vec())
    } else if magic == N64_MAGIC {
        (Endianness::Little, le_to_be(&data[..MIN_HEADER_SIZE]))
    } else if magic == V64_MAGIC {
        (Endianness::ByteSwapped, v64_to_be(&data[..MIN_HEADER_SIZE]))
    } else {
        return None;
    };

    let title = clean_title(&header_bytes[0x20..0x20 + 20]);

    // Cartridge ID: 4 bytes at 0x3B (e.g. "NSME" for Super Mario 64 US)
    let cart_id = clean_title(&header_bytes[0x3B..0x3F]);

    Some(ParsedRom {
        format: RomFormat::N64,
        game_name: title,
        cart_id,
        endianness,
        mapper: None, // N64 cartridges don't use mappers
        expansion_chip: None,
    })
}

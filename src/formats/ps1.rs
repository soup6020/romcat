use super::{Endianness, ParsedRom, RomFormat};

/// ISO 9660 magic at the start of a Primary Volume Descriptor: "\x01CD001"
const ISO_MAGIC: [u8; 5] = [0x01, 0x43, 0x44, 0x30, 0x30]; // \x01CD00 (we check 5 bytes)

/// Sector sizes for the two common PS1 image formats.
const SECTOR_COOKED: usize = 2048; // .iso
const SECTOR_RAW: usize = 2352; // .bin (raw CD sector)

/// Offset of the data payload within a raw 2352-byte sector.
/// 12 bytes sync + 4 bytes header + 8 bytes subheader = 24 bytes before data.
const RAW_DATA_OFFSET: usize = 24;

/// CD-ROM sync pattern at the start of each raw sector.
const CD_SYNC: [u8; 12] = [
    0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
];

/// Read 2048 bytes of data from a given sector number, handling both cooked and raw formats.
/// Returns None if the sector is out of bounds.
fn read_sector<'a>(data: &'a [u8], sector: usize, sector_size: usize) -> Option<&'a [u8]> {
    let offset = sector * sector_size;
    let data_offset = if sector_size == SECTOR_RAW {
        offset + RAW_DATA_OFFSET
    } else {
        offset
    };
    if data.len() < data_offset + 2048 {
        return None;
    }
    Some(&data[data_offset..data_offset + 2048])
}

/// Detect whether this is a raw (2352) or cooked (2048) disc image.
fn detect_sector_size(data: &[u8]) -> Option<usize> {
    // Check for raw CD sync pattern at offset 0
    if data.len() >= SECTOR_RAW && data[0..12] == CD_SYNC {
        return Some(SECTOR_RAW);
    }
    // Try cooked: check for ISO PVD at sector 16
    if data.len() >= 16 * SECTOR_COOKED + 2048 {
        let pvd = &data[16 * SECTOR_COOKED..16 * SECTOR_COOKED + 5];
        if pvd == ISO_MAGIC {
            return Some(SECTOR_COOKED);
        }
    }
    // Try raw: check for ISO PVD at sector 16 within raw sectors
    if data.len() >= 16 * SECTOR_RAW + RAW_DATA_OFFSET + 2048 {
        let pvd_offset = 16 * SECTOR_RAW + RAW_DATA_OFFSET;
        if data[pvd_offset..pvd_offset + 5] == ISO_MAGIC {
            return Some(SECTOR_RAW);
        }
    }
    None
}

/// Check if the disc's System Identifier contains "PLAYSTATION".
fn is_playstation(data: &[u8], sector_size: usize) -> bool {
    if let Some(pvd) = read_sector(data, 16, sector_size) {
        // System Identifier is at PVD offset 8, 32 bytes
        let sys_id = &pvd[8..40];
        let sys_str = String::from_utf8_lossy(sys_id);
        sys_str.contains("PLAYSTATION")
    } else {
        false
    }
}

/// Extract the volume identifier (game title) from the ISO PVD.
fn read_volume_id(data: &[u8], sector_size: usize) -> Option<String> {
    let pvd = read_sector(data, 16, sector_size)?;
    // Volume Identifier at PVD offset 40, 32 bytes
    let vol_id = &pvd[40..72];
    let s: String = vol_id
        .iter()
        .take_while(|&&b| b != 0)
        .filter(|&&b| b.is_ascii_graphic() || b == b' ')
        .map(|&b| b as char)
        .collect();
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Try to find SYSTEM.CNF in the root directory and extract the boot path (game serial).
/// The serial is encoded in the executable filename, e.g. "cdrom:\SLUS_001.23;1" -> "SLUS-00123".
fn read_serial(data: &[u8], sector_size: usize) -> Option<String> {
    let pvd = read_sector(data, 16, sector_size)?;

    // Root directory record is at PVD offset 156, 34 bytes.
    // The extent location (sector) of the root directory is at record offset 2 (LE u32).
    let root_extent =
        u32::from_le_bytes([pvd[156 + 2], pvd[156 + 3], pvd[156 + 4], pvd[156 + 5]]) as usize;
    let root_size =
        u32::from_le_bytes([pvd[156 + 10], pvd[156 + 11], pvd[156 + 12], pvd[156 + 13]]) as usize;

    // Read root directory sectors and search for SYSTEM.CNF
    let sectors_needed = (root_size + 2047) / 2048;
    for s in 0..sectors_needed {
        let dir_data = read_sector(data, root_extent + s, sector_size)?;
        if let Some(serial) = find_system_cnf_serial(data, dir_data, sector_size) {
            return Some(serial);
        }
    }
    None
}

/// Search a directory sector for a SYSTEM.CNF entry and extract the serial from it.
fn find_system_cnf_serial(
    image: &[u8],
    dir_data: &[u8],
    sector_size: usize,
) -> Option<String> {
    let mut offset = 0;
    while offset + 33 < dir_data.len() {
        let record_len = dir_data[offset] as usize;
        if record_len == 0 {
            break;
        }
        if offset + record_len > dir_data.len() {
            break;
        }
        let name_len = dir_data[offset + 32] as usize;
        if name_len > 0 && offset + 33 + name_len <= dir_data.len() {
            let name = &dir_data[offset + 33..offset + 33 + name_len];
            let name_str = String::from_utf8_lossy(name).to_uppercase();
            // ISO 9660 filenames may have ";1" version suffix
            if name_str == "SYSTEM.CNF" || name_str == "SYSTEM.CNF;1" {
                // Read the file content
                let file_extent = u32::from_le_bytes([
                    dir_data[offset + 2],
                    dir_data[offset + 3],
                    dir_data[offset + 4],
                    dir_data[offset + 5],
                ]) as usize;
                let file_size = u32::from_le_bytes([
                    dir_data[offset + 10],
                    dir_data[offset + 11],
                    dir_data[offset + 12],
                    dir_data[offset + 13],
                ]) as usize;
                return read_system_cnf(image, file_extent, file_size, sector_size);
            }
        }
        offset += record_len;
    }
    None
}

/// Read SYSTEM.CNF content and extract the game serial from the BOOT line.
fn read_system_cnf(
    image: &[u8],
    extent: usize,
    size: usize,
    sector_size: usize,
) -> Option<String> {
    // Read up to the first sector (SYSTEM.CNF is always < 2048 bytes)
    let read_size = size.min(2048);
    let sector_data = read_sector(image, extent, sector_size)?;
    let content = String::from_utf8_lossy(&sector_data[..read_size]);

    // Look for the BOOT line, e.g.:
    //   BOOT = cdrom:\SLUS_001.23;1
    //   BOOT2 = cdrom0:\SLUS_001.23;1
    for line in content.lines() {
        let upper = line.to_uppercase();
        if upper.starts_with("BOOT") {
            // Extract the executable path after '='
            if let Some(path) = line.split('=').nth(1) {
                return parse_serial_from_path(path.trim());
            }
        }
    }
    None
}

/// Parse a PS1 serial from an executable path like "cdrom:\SLUS_001.23;1".
/// Returns something like "SLUS-00123".
fn parse_serial_from_path(path: &str) -> Option<String> {
    // Strip cdrom prefix and version suffix
    let name = path
        .rsplit(['\\', '/'])
        .next()?
        .split(';')
        .next()?
        .trim();

    // Expected format: XXXX_NNN.NN (e.g. SLUS_001.23)
    // Convert to: XXXX-NNNNN (e.g. SLUS-00123)
    if name.len() >= 11 && name.as_bytes()[4] == b'_' && name.as_bytes()[8] == b'.' {
        let prefix = &name[0..4];
        let num1 = &name[5..8];
        let num2 = &name[9..];
        Some(format!("{prefix}-{num1}{num2}"))
    } else {
        // Return as-is if it doesn't match the expected pattern
        Some(name.to_string())
    }
}

/// Try to parse a PS1 disc image (.bin or .iso).
pub fn parse(data: &[u8]) -> Option<ParsedRom> {
    let sector_size = detect_sector_size(data)?;

    if !is_playstation(data, sector_size) {
        return None;
    }

    let game_name = read_volume_id(data, sector_size);
    let cart_id = read_serial(data, sector_size);

    Some(ParsedRom {
        format: RomFormat::Ps1,
        game_name,
        cart_id,
        endianness: Endianness::Little,
        mapper: None,
        expansion_chip: None,
    })
}

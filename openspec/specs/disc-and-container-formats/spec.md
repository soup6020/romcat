# Disc and Container Formats

## Purpose

Define content detection and metadata extraction for Wii disc images and containers, PlayStation 1 disc images, and MAME CHD containers.

## Requirements

### Requirement: Parse raw Wii ISO images

The Wii parser SHALL recognize a raw image by Wii magic `5D 1C 9E A3` at offset `0x18` and extract the standard disc header fields.

#### Scenario: Valid raw Wii header

- **WHEN** the Wii magic is present and at least `0x60` bytes are available
- **THEN** the format is `Wii`
- **AND** the cartridge ID comes from the first six bytes
- **AND** the game name comes from the null-terminated field at offsets `0x20` through `0x5F`
- **AND** endianness is `Big-Endian`

### Requirement: Parse WBFS containers

The Wii parser SHALL recognize `WBFS` magic, validate the HD sector shift, locate the embedded Wii disc header one HD sector from the start, and verify its Wii magic.

#### Scenario: Valid WBFS image

- **WHEN** `WBFS` magic is present, the sector shift is at most 24, and the embedded disc header contains Wii magic
- **THEN** the format is `Wii (WBFS)`
- **AND** disc ID and title are read from the embedded header

#### Scenario: Implausible sector shift

- **WHEN** the WBFS HD sector shift exceeds 24
- **THEN** the WBFS parser rejects the input without attempting an unbounded offset

### Requirement: Parse RVZ and WIA containers

The Wii parser SHALL recognize `RVZ\x01` and `WIA\x01`, inspect the embedded original disc header at offset `0x58`, and require Wii magic within that header.

#### Scenario: Valid RVZ container

- **WHEN** `RVZ\x01` magic and embedded Wii magic are present
- **THEN** the format is `Wii (RVZ)`
- **AND** disc ID and title come from the embedded header

#### Scenario: Valid WIA container

- **WHEN** `WIA\x01` magic and embedded Wii magic are present
- **THEN** the format is `Wii (WIA)`
- **AND** disc ID and title come from the embedded header

#### Scenario: Container magic without embedded Wii magic

- **WHEN** RVZ, WIA, or WBFS container magic is present but its expected disc header lacks Wii magic
- **THEN** the corresponding parser rejects the input

### Requirement: Parse cooked and raw PlayStation 1 data sectors

The PlayStation 1 parser SHALL support cooked 2048-byte sectors and raw 2352-byte sectors with a 24-byte data offset, and SHALL require ISO 9660 volume data identifying the system as `PLAYSTATION`.

#### Scenario: Cooked ISO image

- **WHEN** the primary volume descriptor is found at cooked sector 16 and identifies `PLAYSTATION`
- **THEN** the format is `PS1`
- **AND** sectors are read in 2048-byte units

#### Scenario: Raw BIN image

- **WHEN** raw CD sync or a raw-sector primary volume descriptor is detected and the descriptor identifies `PLAYSTATION`
- **THEN** the format is `PS1`
- **AND** sectors are read in 2352-byte units with their 2048-byte payload at offset 24

#### Scenario: Generic ISO 9660 image

- **WHEN** ISO 9660 structures are present but the system identifier does not contain `PLAYSTATION`
- **THEN** the PlayStation 1 parser rejects the input

### Requirement: Extract PlayStation 1 display name and serial

The PlayStation 1 parser SHALL use the ISO 9660 volume identifier as game name and SHALL search root-directory `SYSTEM.CNF` for a boot executable serial.

#### Scenario: Standard boot executable path

- **WHEN** `SYSTEM.CNF` contains a boot path such as `cdrom:\\SLUS_001.23;1`
- **THEN** the cartridge ID is normalized to `SLUS-00123`

#### Scenario: Nonstandard executable name

- **WHEN** a boot path is found but its filename does not match `XXXX_NNN.NN`
- **THEN** the filename without path and ISO version suffix is returned as the cartridge ID

#### Scenario: SYSTEM.CNF is unavailable

- **WHEN** the root directory or `SYSTEM.CNF` content cannot be reached in the bounded parser buffer
- **THEN** the image remains a valid PS1 result with an absent cartridge ID

### Requirement: Parse CHD versions 1 through 5

The CHD parser SHALL require `MComprHD` magic, accept header versions 1 through 5, report big-endian storage, and expose the version in `Mapper/MBC`.

#### Scenario: Supported CHD version

- **WHEN** CHD magic is present and the big-endian version is between 1 and 5 inclusive
- **THEN** the format is `CHD`
- **AND** endianness is `Big-Endian`
- **AND** `Mapper/MBC` begins with `CHD v<version>`

#### Scenario: Unsupported CHD version

- **WHEN** CHD magic is present but the version is outside 1 through 5
- **THEN** the CHD parser rejects the input

### Requirement: Read reachable CHD metadata safely

For CHD versions 3 through 5, the parser SHALL follow the version-specific metadata offset and a bounded linked list to identify generic media type when entries are available in the parser buffer.

#### Scenario: CD track metadata is present

- **WHEN** a reachable `CHTR` or `CHT2` entry describes MODE1 or MODE2 tracks
- **THEN** `Mapper/MBC` appends `CD-ROM (Mode 1)` or `CD-ROM (Mode 2)` respectively

#### Scenario: GD-ROM or hard-disk metadata is present

- **WHEN** a reachable `CHGD` or `GDDD` metadata entry is found
- **THEN** `Mapper/MBC` appends `GD-ROM` or `Hard Disk` respectively

#### Scenario: Metadata is absent, out of bounds, or cyclic

- **WHEN** no usable metadata can be read or 256 entries have been examined
- **THEN** parsing terminates without an unbounded walk
- **AND** the CHD result still reports its version without a game title or cartridge ID

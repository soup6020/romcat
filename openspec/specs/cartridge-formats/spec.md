# Cartridge Formats

## Purpose

Define content detection and metadata extraction for cartridge-oriented NES, SNES, Nintendo 64, Game Boy, Game Boy Color, Game Boy Advance, Nintendo DS, and Nintendo 3DS images.

## Requirements

### Requirement: Parse iNES and NES 2.0 images

The NES parser SHALL require the `NES\x1A` magic, distinguish NES 2.0 from iNES using header flag bits, and report the combined mapper number with a known name when available.

#### Scenario: Valid iNES header

- **WHEN** bytes 0 through 3 are `NES\x1A` and the NES 2.0 marker is absent
- **THEN** the format is `NES`
- **AND** the mapper combines the high nibbles of flags 6 and 7
- **AND** endianness is `Little-Endian`

#### Scenario: Valid NES 2.0 header

- **WHEN** bytes 0 through 3 are `NES\x1A` and bits 3 through 2 of flag 7 equal binary `10`
- **THEN** the format is `NES 2.0`
- **AND** mapper extension bits from header byte 8 are included

#### Scenario: Truncated or wrong-magic input

- **WHEN** fewer than 16 bytes are available or the iNES magic is absent
- **THEN** the NES parser rejects the input

### Requirement: Parse SNES images heuristically

The SNES parser SHALL score candidate headers at LoROM and HiROM locations, account for a 512-byte copier header when identified, choose the higher-scoring candidate, and reject candidates below the minimum confidence threshold.

#### Scenario: Candidate has a valid checksum complement

- **WHEN** a candidate's checksum XOR checksum-complement equals `0xFFFF` and its other header fields are plausible
- **THEN** the candidate receives sufficient confidence to be considered as a SNES header

#### Scenario: LoROM and HiROM candidates are both available

- **WHEN** both header locations can be read
- **THEN** the parser selects the candidate with the higher score
- **AND** selects LoROM on an equal score

#### Scenario: Header confidence is too low

- **WHEN** the selected candidate's score is below 50
- **THEN** the SNES parser rejects the input

#### Scenario: SNES metadata is accepted

- **WHEN** a candidate is accepted
- **THEN** the parser reports its cleaned 21-byte title, mapping mode, recognized coprocessor if present, and `Little-Endian`

### Requirement: Parse all standard Nintendo 64 byte orderings

The Nintendo 64 parser SHALL recognize big-endian `.z64`, little-endian `.n64`, and byte-swapped `.v64` magic values and normalize header fields before extracting title and cartridge ID.

#### Scenario: Big-endian image

- **WHEN** the header starts with `80 37 12 40`
- **THEN** the format is `N64`
- **AND** endianness is `Big-Endian`

#### Scenario: Little-endian image

- **WHEN** the header starts with `40 12 37 80`
- **THEN** four-byte words are normalized to big-endian before reading metadata
- **AND** endianness is `Little-Endian`

#### Scenario: Byte-swapped image

- **WHEN** the header starts with `37 80 40 12`
- **THEN** byte pairs are normalized to big-endian before reading metadata
- **AND** endianness is `Byte-Swapped`

#### Scenario: Metadata extraction

- **WHEN** a supported N64 header is accepted
- **THEN** the title comes from the normalized 20 bytes at offset `0x20`
- **AND** the cartridge ID comes from the normalized four bytes at offset `0x3B`

### Requirement: Parse Game Boy and Game Boy Color images

The Game Boy parser SHALL require the Nintendo logo prefix at offset `0x0104`, distinguish Game Boy Color using the CGB flag, and report title and cartridge type.

#### Scenario: CGB-compatible or CGB-only header

- **WHEN** the CGB flag at `0x0143` is `0x80` or `0xC0`
- **THEN** the format is `Game Boy Color`
- **AND** the title uses the 11-byte CGB title field

#### Scenario: Original Game Boy header

- **WHEN** the Nintendo logo matches and the CGB flag is neither `0x80` nor `0xC0`
- **THEN** the format is `Game Boy`
- **AND** the title uses the 16-byte original title field

#### Scenario: Cartridge type is unknown

- **WHEN** byte `0x0147` is not in the known cartridge-type table
- **THEN** `Mapper/MBC` is reported as `Unknown`

### Requirement: Parse Game Boy Advance images

The Game Boy Advance parser SHALL require at least `0xC0` bytes, the Nintendo logo prefix at offset `0x04`, and fixed byte `0x96` at offset `0xB2`.

#### Scenario: Valid GBA header

- **WHEN** the logo and fixed byte match
- **THEN** the format is `GBA`
- **AND** the title comes from the 12 bytes at `0xA0`
- **AND** the cartridge ID comes from the four-byte game code at `0xAC`
- **AND** endianness is `Little-Endian`

#### Scenario: Logo matches but fixed byte does not

- **WHEN** the Nintendo logo prefix matches but offset `0xB2` is not `0x96`
- **THEN** the GBA parser rejects the input

### Requirement: Parse Nintendo DS images with additional validation

The Nintendo DS parser SHALL require the Nintendo logo prefix at offset `0xC0`, a supported unit code, and a plausible ARM9 ROM offset before accepting an image.

#### Scenario: Valid NDS or DSi unit code

- **WHEN** the unit code is `0x00`, `0x02`, or `0x03`, the logo matches, and the ARM9 offset is at least `0x1000`
- **THEN** the format is `NDS`
- **AND** the four-byte alphanumeric game code is reported as the cartridge ID

#### Scenario: English banner title is reachable

- **WHEN** the banner offset and its English UTF-16LE title fit in the bounded parser buffer
- **THEN** the banner title is used and embedded line breaks are replaced with spaces

#### Scenario: Banner title is unavailable

- **WHEN** the English banner title is absent, empty, or outside the available buffer
- **THEN** the parser falls back to the cleaned 12-byte internal title

#### Scenario: Header validation fails

- **WHEN** the game code contains non-alphanumeric characters, the unit code is unsupported, or the ARM9 offset is below `0x1000`
- **THEN** the Nintendo DS parser rejects the input

### Requirement: Parse Nintendo 3DS cartridge images

The Nintendo 3DS parser SHALL validate an NCSD header and its first NCCH partition, report the product-code suffix as cartridge ID, and use SMDH title metadata when reachable.

#### Scenario: Valid NCSD and NCCH chain

- **WHEN** `NCSD` appears at offset `0x100`, partition 0 has a nonzero media-unit offset, and `NCCH` appears at partition offset plus `0x100`
- **THEN** the format is `3DS`
- **AND** endianness is `Little-Endian`

#### Scenario: Product code follows the standard pattern

- **WHEN** the NCCH product code resembles `CTR-P-XXXX`
- **THEN** the final hyphen-separated component is reported as the cartridge ID

#### Scenario: SMDH icon is reachable

- **WHEN** the ExeFS `icon` entry and valid `SMDH` data fit in the bounded parser buffer
- **THEN** the English long UTF-16LE title is preferred, with the English short title as fallback

#### Scenario: SMDH title is unavailable

- **WHEN** ExeFS or SMDH title data is unavailable, including encrypted or deep content
- **THEN** the full cleaned product code is used as the game name

### Requirement: Clean fixed-width ASCII metadata consistently

ASCII title and identifier fields shared by cartridge parsers SHALL stop at the first null byte, retain printable ASCII and spaces, trim surrounding whitespace, and return no value when nothing remains.

#### Scenario: Field contains padding and non-ASCII bytes

- **WHEN** a fixed-width field contains leading or trailing spaces, non-ASCII bytes, or null padding
- **THEN** non-ASCII bytes are omitted, parsing stops at the first null, and surrounding whitespace is removed

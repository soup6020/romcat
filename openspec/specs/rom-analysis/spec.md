# ROM Analysis

## Purpose

Define how romcat reads files, selects a format parser, normalizes parser results, computes hashes, and handles unreadable or unknown inputs.

## Requirements

### Requirement: Analyze each requested file independently

The system SHALL attempt each input path in command-line order and SHALL continue processing later paths after an individual file cannot be opened, inspected, or hashed.

#### Scenario: All files are readable

- **WHEN** the user supplies multiple readable files
- **THEN** the system produces one result for each file in the same order

#### Scenario: One file has an I/O failure

- **WHEN** an input cannot be opened, inspected, read, or fully hashed
- **THEN** the system reports an error for that path to standard error
- **AND** continues analyzing the other inputs
- **AND** does not produce a result row for the failed input

### Requirement: Bound format-detection reads

The system SHALL read no more than the first 1 MiB of each file for format detection and metadata extraction, regardless of total file size.

#### Scenario: Small input

- **WHEN** a file is smaller than 1 MiB
- **THEN** the complete file is available to format parsers

#### Scenario: Large input

- **WHEN** a file is larger than 1 MiB
- **THEN** only its first 1 MiB is available to format parsers
- **AND** analysis does not load the complete file solely for detection

### Requirement: Detect formats from file content

The system SHALL determine format from header and filesystem content rather than requiring a particular filename extension.

#### Scenario: Recognized content has an unusual extension

- **WHEN** a readable file contains a valid supported header but has an unexpected extension
- **THEN** the system reports the format identified from its content

#### Scenario: Extension suggests a format but content does not

- **WHEN** a readable file has a familiar ROM extension but does not satisfy any parser
- **THEN** the system reports the format as `Unknown`

### Requirement: Use deterministic first-match parser dispatch

The system SHALL invoke parsers in this order and use the first successful result: CHD, NES, N64, Wii, Nintendo 3DS, Nintendo DS, Game Boy Advance, Game Boy/Game Boy Color, PlayStation 1, then SNES.

#### Scenario: Multiple parsers could accept an input

- **WHEN** more than one parser would accept the bounded header buffer
- **THEN** the result comes from the earliest successful parser in the defined order

#### Scenario: No parser accepts an input

- **WHEN** every parser rejects a readable input
- **THEN** the system returns an `Unknown` result with no title, cartridge ID, mapper, or expansion chip and with endianness `N/A`

### Requirement: Normalize successful parser metadata

The system SHALL represent each readable input with its basename, byte size, detected format, endianness, and optional game name, cartridge ID, mapper/MBC, expansion chip, and MD5 fields.

#### Scenario: Parser omits metadata

- **WHEN** a format does not expose a metadata field or extraction cannot reach that field within the bounded buffer
- **THEN** the normalized result leaves that field absent without treating analysis as an error

#### Scenario: Input path includes directories

- **WHEN** a supplied path has a filename component
- **THEN** the result's file label is the filename rather than the complete path

### Requirement: Hash complete files by default using streaming I/O

The system SHALL compute a lowercase hexadecimal MD5 digest of the complete file by default and SHALL stream data in 64 KiB chunks rather than loading the complete file in memory.

#### Scenario: Default hashing

- **WHEN** analysis is requested without `--no-hash`
- **THEN** the result contains the MD5 digest of every byte in the file

#### Scenario: Hashing is disabled

- **WHEN** analysis is requested with `--no-hash`
- **THEN** the system does not perform the full-file MD5 pass
- **AND** the result's MD5 field is absent

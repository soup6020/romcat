# CLI Reporting

## Purpose

Define romcat's command-line interface, tabular output, formatting rules, diagnostics, and process status.

## Requirements

### Requirement: Accept one or more file operands

The `romcat` binary SHALL require at least one file path and SHALL accept multiple file paths in a single invocation.

#### Scenario: No file is supplied

- **WHEN** the user invokes `romcat` without a file operand
- **THEN** command-line parsing fails and displays clap-generated usage guidance

#### Scenario: Multiple files are supplied

- **WHEN** the user invokes `romcat first.rom second.rom`
- **THEN** both paths are queued for analysis in the supplied order

### Requirement: Provide the hash opt-out and standard informational flags

The CLI SHALL support `--no-hash`, `-h`/`--help`, and `-V`/`--version`.

#### Scenario: User disables hashing

- **WHEN** the user includes `--no-hash`
- **THEN** analysis is requested without MD5 computation

#### Scenario: User requests version information

- **WHEN** the user includes `--version` or `-V`
- **THEN** the CLI displays the package version and exits without analyzing files

### Requirement: Render successful results as one UTF-8 table

The system SHALL print one dynamically arranged UTF-8 table to standard output when at least one file was analyzed successfully, with one row per successful result in input order.

#### Scenario: At least one analysis succeeds

- **WHEN** one or more readable inputs produce results
- **THEN** the system prints a table containing those results

#### Scenario: Every analysis fails with I/O errors

- **WHEN** no input produces a result
- **THEN** the system does not print an empty result table

### Requirement: Always show core columns

The table SHALL always contain `File`, `Format`, `Game Name`, `Endianness`, and `Size` columns.

#### Scenario: A title is unavailable

- **WHEN** a result has no game name
- **THEN** its `Game Name` cell contains an em dash

#### Scenario: A byte size is displayed

- **WHEN** a file size is rendered
- **THEN** values below 1 KiB use bytes
- **AND** values from 1 KiB use binary KiB-based `KB`, `MB`, or `GB` units with one decimal place
- **AND** the size cell is right-aligned

### Requirement: Include optional columns only when useful

The table SHALL include `Cart ID`, `Mapper/MBC`, `Expansion Chip`, and `MD5` only when at least one displayed result has a value for the corresponding field.

#### Scenario: No result has a mapper

- **WHEN** every displayed result has an absent mapper field
- **THEN** the table omits the `Mapper/MBC` column

#### Scenario: Some results lack a globally enabled field

- **WHEN** at least one result enables an optional column but another result lacks that field
- **THEN** the missing non-MD5 value is rendered as an em dash in that row

#### Scenario: Hashing is disabled for the invocation

- **WHEN** `--no-hash` makes MD5 absent from every result
- **THEN** the table omits the `MD5` column

### Requirement: Visually distinguish formats and selected metadata

The table SHALL apply a stable per-format color to format names, cyan to game names, yellow to expansion-chip values, and dark grey to MD5 values.

#### Scenario: Results contain different systems

- **WHEN** the table contains rows for multiple detected formats
- **THEN** each format label uses its configured system color while the column layout remains shared

### Requirement: Report partial failure through diagnostics and exit status

The system SHALL print each file-specific I/O error to standard error in the form `Error reading <path>: <reason>` and SHALL exit with status 1 if any input failed, even when other rows were printed.

#### Scenario: Mixed success and failure

- **WHEN** at least one file is analyzed and at least one file has an I/O failure
- **THEN** successful rows are printed to standard output
- **AND** failed paths are reported to standard error
- **AND** the process exits with status 1

#### Scenario: No I/O failure occurs

- **WHEN** every requested file is readable and hashable, including files reported as `Unknown`
- **THEN** the process completes successfully

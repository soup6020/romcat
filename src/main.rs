mod formats;
mod rom;

use std::path::PathBuf;
use std::process;

use clap::Parser;
use comfy_table::{presets::UTF8_FULL, ContentArrangement, Table};

use rom::{format_size, RomInfo};

#[derive(Parser)]
#[command(
    name = "romcat",
    about = "Analyze video game ROM files and display their metadata",
    version
)]
struct Cli {
    /// ROM files to analyze
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn build_table(roms: &[RomInfo]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    // Determine which optional columns have data
    let has_mapper = roms.iter().any(|r| r.mapper.is_some());
    let has_expansion = roms.iter().any(|r| r.expansion_chip.is_some());

    // Build header row
    let mut headers: Vec<&str> = vec!["File", "Format", "Game Name", "Endianness", "Size"];
    if has_mapper {
        headers.push("Mapper/MBC");
    }
    if has_expansion {
        headers.push("Expansion Chip");
    }
    headers.push("MD5");
    table.set_header(headers);

    // Build data rows
    for rom in roms {
        let mut row: Vec<String> = vec![
            rom.file_name.clone(),
            rom.format.to_string(),
            rom.game_name.as_deref().unwrap_or("\u{2014}").to_string(),
            rom.endianness.to_string(),
            format_size(rom.file_size),
        ];
        if has_mapper {
            row.push(
                rom.mapper
                    .as_deref()
                    .unwrap_or("\u{2014}")
                    .to_string(),
            );
        }
        if has_expansion {
            row.push(
                rom.expansion_chip
                    .as_deref()
                    .unwrap_or("\u{2014}")
                    .to_string(),
            );
        }
        row.push(rom.md5.clone());
        table.add_row(row);
    }

    table
}

fn main() {
    let cli = Cli::parse();
    let mut roms: Vec<RomInfo> = Vec::new();
    let mut had_errors = false;

    for path in &cli.files {
        match rom::analyze(path) {
            Ok(info) => roms.push(info),
            Err(e) => {
                eprintln!("Error reading {}: {e}", path.display());
                had_errors = true;
            }
        }
    }

    if !roms.is_empty() {
        let table = build_table(&roms);
        println!("{table}");
    }

    if had_errors {
        process::exit(1);
    }
}

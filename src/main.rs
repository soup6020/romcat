mod formats;
mod rom;

use std::path::PathBuf;
use std::process;

use clap::Parser;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, CellAlignment, Color, ContentArrangement, Table};

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

    /// Skip MD5 hash computation (faster for large files)
    #[arg(long)]
    no_hash: bool,
}

/// Pick a colour for a given ROM format.
fn format_color(format: &formats::RomFormat) -> Color {
    match format {
        formats::RomFormat::Nes | formats::RomFormat::Nes2 => Color::Red,
        formats::RomFormat::N64 => Color::Green,
        formats::RomFormat::Gba => Color::Magenta,
        formats::RomFormat::GameBoy => Color::DarkYellow,
        formats::RomFormat::GameBoyColor => Color::Yellow,
        formats::RomFormat::Snes => Color::Blue,
        formats::RomFormat::Nds => Color::Cyan,
        formats::RomFormat::Ps1 => Color::DarkCyan,
        formats::RomFormat::Wii
        | formats::RomFormat::WiiWbfs
        | formats::RomFormat::WiiRvz
        | formats::RomFormat::WiiWia => Color::DarkBlue,
        formats::RomFormat::N3ds => Color::DarkGreen,
        formats::RomFormat::Chd => Color::DarkMagenta,
        formats::RomFormat::Unknown => Color::DarkGrey,
    }
}

fn build_table(roms: &[RomInfo]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    // Determine which optional columns have data
    let has_cart_id = roms.iter().any(|r| r.cart_id.is_some());
    let has_mapper = roms.iter().any(|r| r.mapper.is_some());
    let has_expansion = roms.iter().any(|r| r.expansion_chip.is_some());
    let has_md5 = roms.iter().any(|r| r.md5.is_some());

    // Build header row
    let mut headers: Vec<Cell> = vec![
        Cell::new("File").fg(Color::White),
        Cell::new("Format").fg(Color::White),
        Cell::new("Game Name").fg(Color::White),
    ];
    if has_cart_id {
        headers.push(Cell::new("Cart ID").fg(Color::White));
    }
    headers.push(Cell::new("Endianness").fg(Color::White));
    headers.push(Cell::new("Size").fg(Color::White));
    if has_mapper {
        headers.push(Cell::new("Mapper/MBC").fg(Color::White));
    }
    if has_expansion {
        headers.push(Cell::new("Expansion Chip").fg(Color::White));
    }
    if has_md5 {
        headers.push(Cell::new("MD5").fg(Color::White));
    }
    table.set_header(headers);

    let dash = "\u{2014}";

    // Build data rows
    for rom in roms {
        let color = format_color(&rom.format);

        let mut row: Vec<Cell> = vec![
            Cell::new(&rom.file_name),
            Cell::new(rom.format.to_string()).fg(color),
            Cell::new(rom.game_name.as_deref().unwrap_or(dash)).fg(Color::Cyan),
        ];
        if has_cart_id {
            row.push(Cell::new(rom.cart_id.as_deref().unwrap_or(dash)));
        }
        row.push(Cell::new(rom.endianness.to_string()));
        row.push(
            Cell::new(format_size(rom.file_size)).set_alignment(CellAlignment::Right),
        );
        if has_mapper {
            row.push(Cell::new(
                rom.mapper.as_deref().unwrap_or(dash),
            ));
        }
        if has_expansion {
            row.push(
                Cell::new(rom.expansion_chip.as_deref().unwrap_or(dash)).fg(Color::Yellow),
            );
        }
        if has_md5 {
            row.push(
                Cell::new(rom.md5.as_deref().unwrap_or("")).fg(Color::DarkGrey),
            );
        }
        table.add_row(row);
    }

    table
}

fn main() {
    let cli = Cli::parse();
    let mut roms: Vec<RomInfo> = Vec::new();
    let mut had_errors = false;

    for path in &cli.files {
        match rom::analyze(path, !cli.no_hash) {
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

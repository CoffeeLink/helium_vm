use clap::Parser;

use owo_colors::OwoColorize;

use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, Read};
use std::process::exit;

use crate::helium::prelude::*;
mod helium;
mod devices;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the file containing the rom image, the file must be less than 255 bytes
    #[arg(value_name = "ROM file")]
    rom_file: PathBuf,

    /// Enables stepping and shows register states
    #[arg(short, long)]
    debug: bool,
}


fn main() {
    let config = Cli::parse();
    let rom_file = File::open(config.rom_file).unwrap_or_else(|e| {
        eprintln!("{}", format!("Could not open rom-file: {}", e).red());
        exit(-1);
    });

    let rom_file_metadata = rom_file.metadata().unwrap_or_else(|e| {
        eprintln!("Somehow failed to get metadata about the rom file");
        exit(-1);
    });

    let size = rom_file_metadata.len();
    if size > u8::MAX as u64 {
        eprintln!("Rom-file exceeds size limit(255 bytes)");
        exit(-1);
    }

    let reader = BufReader::new(rom_file);
    let mut rom: Vec<u8> = Vec::with_capacity(size as usize);

    for byte in reader.bytes() {
        if byte.is_err() {
            eprintln!("Failed to read rom file.");
            exit(-1);
        }

        rom.push(byte.unwrap());
    }

    let device_mounter = IOController;
    let mut cpu = CPU::new(device_mounter, rom);

    cpu.start();

    while cpu.is_on {
        cpu.next();
    }

    dbg!(cpu.registers);
}

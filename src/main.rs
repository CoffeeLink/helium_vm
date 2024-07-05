#![feature(bigint_helper_methods)]
#![feature(ascii_char)]

use clap::Parser;

use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, Read};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use ansi_escapes::CursorUp;

use crate::helium::prelude::*;
mod helium;
mod devices;
mod utils;
mod system_manager;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the file containing the rom image, the file must be less than 255 bytes
    #[arg(value_name = "ROM file")]
    rom_file: PathBuf,

    /// Defines how many instructions the CPU should complete every second.
    #[arg(short, long, value_name = "Step rate(f)", default_value = "2")]
    step_rate: f32,
    
    /// Enables debug controls
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let config = Cli::parse();

    let rom = load_rom(&config)
        .map_err(|msg|{
        eprintln!("{}", msg);
        exit(-1)
    }).unwrap();

    let mut device_mounter = IOController::new();

    let mut cpu = CPU::new(device_mounter, rom);
    cpu.start();
    update_state_ui(&cpu, true);

    while cpu.is_on {
        sleep(Duration::from_millis(500)); // 1 step per sec
        cpu.next();
        update_state_ui(&cpu, false);
    }
}

fn update_state_ui(cpu: &CPU, first_update: bool) {
    let cpu_state_ui = cpu.generate_state_ui();
    let memory_state_ui = cpu.memory.draw_hexdump();
    
    let out = format!("{}\n{}", cpu_state_ui, memory_state_ui);
    let line_count = out.lines().count();
    
    let mut final_out = String::new();
    
    if !first_update {
        final_out = format!("{}", CursorUp(line_count as u16));
    }
    
    final_out.push_str(&out);
    println!("{}", final_out);
}

fn load_rom(config: &Cli) -> Result<Vec<u8>, String> {
    let rom_file = File::open(&config.rom_file)
        .map_err(|e| format!("Could not open rom-file: {}", e))?;

    let rom_meta = rom_file.metadata()
        .map_err(|e| format!("Failed to read the metadata of the rom-file: {}", e))?;

    let size = rom_meta.len();
    if size > 256 {
        return Err(format!("Rom file exceeds the 256 byte limit ({})", size));
    }

    let reader = BufReader::new(rom_file);
    let mut rom: Vec<u8> = Vec::with_capacity(size as usize);

    for byte in reader.bytes() {
        rom.push(byte
            .map_err(|e| format!("Failed to read byte: {}", e))
            ?
        );
    }
    Ok(rom)
}
#![feature(bigint_helper_methods)]
#![feature(ascii_char)]

use clap::Parser;

use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, Read};
use std::process::exit;
use std::time::{Duration, Instant};
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
    
    /// Disables the UI for the CPU state
    #[arg(long, default_value = "false")]
    no_gui: bool,

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

    let millis_per_iter = 1000f32 / config.step_rate;
    let per_iter_duration = Duration::from_millis(millis_per_iter.round() as u64);

    #[allow(unused_mut)]
    let mut device_mounter = IOController::new();
    
    let mut cpu = CPU::new(device_mounter, rom);
    
    
    cpu.start();
    if !config.no_gui {
        update_state_ui(&cpu, true);
    }

    let mut start = Instant::now();

    while cpu.is_on {
        let elapsed = start.elapsed();

        // If enough time has passed, run it again.
        if elapsed >= per_iter_duration {
            cpu.next();
            
            if !config.no_gui {
                update_state_ui(&cpu, false);
            }

            start = Instant::now();
        }
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
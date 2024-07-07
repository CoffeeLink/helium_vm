#![feature(bigint_helper_methods)]
#![feature(ascii_char)]

use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufReader, Read};
use std::process::exit;
use std::time::{Duration, Instant};
use ansi_escapes::{ClearScreen, CursorHide, CursorShow};
use crate::devices::stdout_ascii_buffer::CharIOBuffer;
use crate::devices::telnet_terminal::TelnetTerminal;

use crate::helium::prelude::*;

/// Contains the "Core" of Helium, the CPU, IO-CTL and the memory.
pub mod helium;
/// Holds all devices and the Device Trait.
pub mod devices;
/// Some utility stuff
pub mod utils;

/// Holds all command line arguments.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the file containing the rom image, the file must be less than 255 bytes
    #[arg(value_name = "ROM file")]
    rom_file: PathBuf,

    /// Defines how many instructions the CPU should complete every second.
    #[arg(short, long, value_name = "Step rate(f)", default_value = "100")]
    step_rate: f32,

    /// What Devices to Link
    #[clap(short, long, value_enum)]
    #[arg(default_value = "term-link")]
    devices: Vec<DeviceType>,

    /// Opens/Creates an interrupts.log file where every interrupt of every device is logged, by name, by code.
    #[arg(long, default_value = "false")]
    interrupt_logging: bool,

    /// Disables the UI for the CPU state
    #[arg(long, default_value = "false")]
    no_gui: bool,

    /// Enables debug controls
    #[arg(long)]
    debug: bool,

    /// The Port for the TermLink server hosted on 127.0.0.1:??? (only when TermLink is enabled tho)
    #[arg(short, long, value_name = "Terminal Port", default_value = "5555")]
    port: u16
}

/// This enum holds all available devices for use
#[derive(ValueEnum, Copy, Clone, Debug, PartialOrd, PartialEq)]
enum DeviceType {
    TermLink,
    CharBuffer,
}

impl DeviceType {
    /// Used for mounting the device automatically, each device has their own impl of this and this.
    /// makes mounting stuff easy by just iterating over a list of these and calling mount.
    pub fn mount(&self, cli: &Cli, mounter: &mut IOController) {
        match self {
            DeviceType::TermLink => {
                let tt = TelnetTerminal::new(1, cli.port);
                mounter.mount_device(51..54, tt);
            }
            
            DeviceType::CharBuffer => {
                let ch_io_buffer = CharIOBuffer::new();
                mounter.mount_device(0..51, ch_io_buffer);
            }
        }
    }
}

fn main() {
    let config = Cli::parse();

    let rom = load_rom(&config.rom_file)
        .map_err(|msg|{
        eprintln!("{}", msg);
        exit(-1)
    }).unwrap();

    let micros_per_iter = 1_000_000f64 / config.step_rate as f64;
    let per_iter_duration = Duration::from_micros(micros_per_iter.round() as u64);

    #[allow(unused_mut)]
    let mut device_mounter = IOController::new(config.interrupt_logging);
    
    // Load devices dynamically.
    for device_type in config.devices.clone() {
        device_type.mount(&config, &mut device_mounter)
    }

    let mut cpu = CPU::new(device_mounter, rom);

    cpu.start();
    if !config.no_gui {
        update_state_ui(&cpu);
    }

    let mut start = Instant::now();
    let mut elapsed = per_iter_duration;

    while cpu.is_on {
        // If enough time has passed, run it again.
        if elapsed >= per_iter_duration {
            cpu.next();

            print!("{}", CursorHide);
            
            if !config.no_gui {
                update_state_ui(&cpu);
                println!(); // Separation from the UI
            }
            
            print!("{}", cpu.io_ctl.draw_ui(config.no_gui, config.debug));

            start = Instant::now();
        }
        // Update elapsed
        elapsed = start.elapsed();
    }
    
    // end of execution
    
    print!("{}", CursorShow);
}

/// Draws the UI for the CPU and the memory, also clears the screen.
fn update_state_ui(cpu: &CPU) {
    let cpu_state_ui = cpu.generate_state_ui();
    let memory_state_ui = cpu.memory.draw_hexdump();

    let out = format!("{}\n{}", cpu_state_ui, memory_state_ui);
    // let line_count = out.lines().count();

    println!("{}{}",ClearScreen, out);
}


/// Takes a Path to a file which will be loaded into a 256 long vec, returns error messages if something goes wrong. 
fn load_rom(path: &Path) -> Result<Vec<u8>, String> {
    let rom_file = File::open(&path)
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
            .map_err(|e| format!("Failed to read byte: {}", e))?
        );
    }
    Ok(rom)
}
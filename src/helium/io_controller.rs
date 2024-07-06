use std::fmt::Debug;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::ops::{Range};
use owo_colors::OwoColorize;
use crate::devices::device::Device;

#[derive(Debug)]
struct RangedDevice {
    pub range: Range<u8>,
    pub device: Box<dyn Device>
}

/// Handles all "Hardware components" / Devices connected to the CPUs IO Bus
/// Has generalized functions for everything and allows the mounting of Devices in a colliding way (the address ranges can match/intersect)
#[derive(Debug)]
pub struct IOController {
    devices: Vec<RangedDevice>,
    interrupt_log: Option<File>,
}
// should probably work with callbacks, like IO.mount(addr_range, callback: Fn(addr, data))
// also, it should give a warning if 2 "Devices" "Collide" in the address range, but it shouldn't crash.
// so it should iterate over the callbacks for each address when called.
// also ever IO device should have its "update" function mounted too on a add_device_update(UpdateFN)
// this way the CPU should just be able to call UpdateIO after it finishes a "Cycle" (an instruction).
// Maybe make an IO Device trait, idk, it's just a small project.

impl IOController {
    pub fn new(logging_enabled: bool) -> Self {
        let mut file: Option<File> = None;
        if logging_enabled {
            file = Some(File::create("interrupts.log").expect("Failed to open interrupt log file."));
        }

        Self { devices: Vec::new(), interrupt_log: file }
    }


    /// Takes a device and a given address range for it to use.
    pub fn mount_device<D>(&mut self, address: Range<u8>, mut device: D)
    where D: Device + 'static {

        let addr_space = device.get_address_space();

        if addr_space.is_some() && addr_space.unwrap() != address.len() as u8 {
            println!("Device address space does not match given range");
        }

        device.init_device();
        self.devices.push(RangedDevice { range: address, device: Box::new(device) });
    }

    /// Ran when the CPU starts up.
    pub fn startup(&mut self) {
        for device in &mut self.devices {
            device.device.startup();
        }
    }

    /// Resets all devices
    pub fn reset(&mut self) {
        for device in &mut self.devices {
            device.device.reset_device();
        }
    }

    /// Finds the given device and returns a reference to it.
    pub fn find_device<T: Device + 'static>(&self) -> Option<&T> {
        self.devices.iter().filter_map(|device| {
            // Attempt to downcast the device to the specific type T
            device.device.as_any().downcast_ref::<T>()
        }).next() // Return the first matching device found
    }

    /// Updates the CPU
    pub fn update(&mut self) {
        for device_data in &mut self.devices {
            device_data.device.update_device();
        }
    }

    /// Checks all devices if they would like to cause an interrupt.
    pub fn device_has_interrupt_request(&mut self) -> Option<u8> {
        let mut has_interrupt = false;
        let mut int_code: u8 = 0;

        for device in &mut self.devices {
            if let Some((code, int_log)) = device.device.has_interrupt_request() {
                // try logging this
                if self.interrupt_log.is_some() {
                    let log = self.interrupt_log.as_mut().unwrap();
                    let mut writer = BufWriter::new(log);

                    let log_msg = format!("{}: {}:\t\t{:08b}", device.device.get_name().bright_green(), int_log, code.yellow());
                    write!(writer, "{}\n", log_msg).expect("Failed to Log interrupt.")
                }

                has_interrupt = true;
                int_code = int_code | code;
            }
        }

        // try logging the end of an int seq
        if self.interrupt_log.is_some() && has_interrupt {
            let log = self.interrupt_log.as_mut().unwrap();
            let mut writer = BufWriter::new(log);

            write!(writer, "\n",).expect("Failed to Log end interrupt.")
        }


        if !has_interrupt {
            None
        } else {
            Some(int_code)
        }
    }

    /// Asks every device to create their Strings for the UIs and separates them,
    /// the returned string will be a collective of the generated UIs.
    pub fn draw_ui(&mut self, no_gui: bool, debug: bool) -> String {
        let mut out_buffer = String::new();

        for dev in self.devices.iter_mut() {
            if let Some(ui_str) = dev.device.draw_ui(no_gui, debug) {
                out_buffer.push_str(&ui_str);
                out_buffer.push('\n')
            }
        }

        out_buffer
    }

    /// Read Data from a device on a given address, if no device is present, result is 0.
    /// ## Caution:
    /// If multiple devices are listening on this address: all results get Binary OR-d
    pub fn read(&mut self, address: u8) -> u8 {
        let mut out = 0;

        for device_data in &mut self.devices {
            if !device_data.range.contains(&address) { continue }

            let relative_address = address - device_data.range.start;
            out = out | device_data.device.read(relative_address);
        }
        return out
    }

    /// Write data to the give IO address,
    /// all devices listening on the given address will get the data.
    pub fn write(&mut self, address: u8, data: u8) {
        for device_data in &mut self.devices {
            if !device_data.range.contains(&address) { continue }

            let relative_address = address - device_data.range.start;
            device_data.device.write(relative_address, data);
        }
    }
}
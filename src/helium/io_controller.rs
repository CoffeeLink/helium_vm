use std::fmt::Debug;
use std::ops::{Range};
use crate::devices::device::Device;

#[derive(Debug)]
struct RangedDevice {
    pub range: Range<u8>,
    pub device: Box<dyn Device>
}

/// enables the "Mounting" of IO;
#[derive(Debug)]
pub struct IOController {
    devices: Vec<RangedDevice>
}
// should probably work with callbacks, like IO.mount(addr_range, callback: Fn(addr, data))
// also, it should give a warning if 2 "Devices" "Collide" in the address range, but it shouldn't crash.
// so it should iterate over the callbacks for each address when called.
// also ever IO device should have its "update" function mounted too on a add_device_update(UpdateFN)
// this way the CPU should just be able to call UpdateIO after it finishes a "Cycle" (an instruction).
// Maybe make an IO Device trait, idk, it's just a small project.

impl IOController {
    pub fn new() -> Self {
        Self { devices: Vec::new() }
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
    
    /// Updates the CPU
    pub fn update(&mut self) {
        for device_data in &mut self.devices {
            device_data.device.update_device();
        }
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
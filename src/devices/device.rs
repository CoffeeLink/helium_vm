use std::fmt::Debug;

/// A trait designed to hold all behaviour required for each device.
pub trait Device: Debug {
    /// Only gets called when the device is registered into the IO Controller
    fn init_device(&mut self);
    
    /// Gets called before the CPU starts its first cycle.
    fn startup(&mut self);
    
    /// Should handle all updates, like UI/state
    fn update_device(&mut self);

    /// Called when a system-wide reset occurs
    fn reset_device(&mut self);

    /// Called when the CPU wants to read from a given IO address.
    fn read(&mut self, address: u8) -> u8;

    /// Called when the CPU wants to write to the given IO address.
    fn write(&mut self, address: u8, value: u8);
    
    /// Called when the device gets mounted, 
    /// used to give warning when the range doesn't match in size
    /// If its unknown or there is no need for this, just leave it be.
    fn get_address_space(&self) -> Option<u8> { None }
}
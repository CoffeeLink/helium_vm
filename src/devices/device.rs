use std::any::Any;
use std::fmt::Debug;

/// A trait designed to hold all behaviour required for each device.
pub trait Device: Debug {
    /// Only gets called when the device is registered into the IO Controller
    fn init_device(&mut self);

    /// Gets called before the CPU starts its first cycle.
    fn startup(&mut self);

    /// Should handle all updates, like UI/state
    fn update_device(&mut self);
    
    /// After each iteration of the CPU a UI will be drawn, 
    /// this can be turned off but some components are UI so that's optional.
    /// basically if you want a cool UI you can have it.
    fn draw_ui(&mut self, no_gui: bool, debug: bool) -> Option<String>;

    /// Called after each update, checking if the device sent an INT signal,
    /// Interrupt signals also take an "interrupt code" which is just a byte of info.
    /// **Note:** interrupt codes get OR-ed into one byte,
    /// so if 2 devices interrupt their codes will be codeA | codeB.
    /// This should be taken into design consideration.
    ///
    /// a Log message can be given too.
    fn has_interrupt_request(&mut self) -> Option<(u8, String)>;

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

    /// Required for finding systems, just return self.
    fn as_any(&self) -> &dyn Any;

    /// Returns the name of the Device in a &str, used in interrupt logging.
    fn get_name(&self) -> &str {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .expect("Failed to get typename, somehow")
    }
}
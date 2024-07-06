use std::any::Any;
use crate::devices::device::Device;

const BUFFER_SIZE: usize = 50;

#[derive(Debug)]
pub struct CharIOBuffer {
    pub buffer: [u8; BUFFER_SIZE],
}

// Helper func stolen from stack overflow
fn get_ascii_str(buffer: &[u8]) -> Result<&str, ()> {
    for byte in buffer.into_iter() {
        if byte >= &128 {
            return Err(());
        }
    }
    Ok(unsafe {
        // This is safe because we verified above that it's a valid ASCII
        // string, and all ASCII strings are also UTF8 strings
        core::str::from_utf8_unchecked(buffer)
    })
}

impl CharIOBuffer {
    pub fn new() -> Self {
        Self {
            buffer: [0; BUFFER_SIZE]
        }
    }
    
    pub fn as_ascii_str(&self) -> String {
        get_ascii_str(&self.buffer).unwrap_or("ERROR STR").into()
    }
}

impl Device for CharIOBuffer {
    fn init_device(&mut self) {
        /* Pass */
    }
    fn startup(&mut self) {
        /* Pass */
    }

    fn update_device(&mut self) {
        /* Pass */
    }

    fn draw_ui(&mut self, _no_gui: bool, debug: bool) -> Option<String> {
        let mut out = String::from(self.as_ascii_str());
        
        if debug {
            out.push('\n');
            out.push_str(&format!("{:?}", self.buffer));
        }
        
        Some(out)
    }

    fn has_interrupt_request(&mut self) -> Option<(u8, String)> {
        None
    }

    fn reset_device(&mut self) {
        self.buffer = [0; BUFFER_SIZE];
    }

    fn read(&mut self, address: u8) -> u8 {
        return *self.buffer.get(address as usize).unwrap_or(&0);
    }

    fn write(&mut self, address: u8, value: u8) {
        if address + 1 > self.buffer.len() as u8 {
            return;
        }
        
        self.buffer[address as usize] = value;
    }

    fn get_address_space(&self) -> Option<u8> {
        Some(BUFFER_SIZE as u8)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
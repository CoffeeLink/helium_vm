/// Responsible for making sure there is a "ROM" block in the memory.
#[derive(Debug, Copy, Clone)]
pub struct MemoryControl {
    container: [u8; u8::MAX as usize],
    rom_limit: Option<u8>,
}
impl MemoryControl {
    pub fn new(mut rom: Vec<u8>) -> Self {
        assert!(rom.len() <= u8::MAX as usize);

        let mut container: [u8; u8::MAX as usize] = [0; u8::MAX as usize];
        let len = rom.len() as u8;

        let drain = rom.drain(..);
        for (i, val) in drain.enumerate() {
            container[i] = val;
        }

        let mut rom_limit: Option<u8> = None;

        if len != 0 {
            rom_limit = Some(len - 1);
        }

        Self {
            container,
            rom_limit,
        }
    }

    pub fn get(&self, index: u8) -> u8 {
        return self.container[index as usize];
    }

    /// Returns true if: the mem write was successful (not in ROM).
    pub fn set(&mut self, index: u8, value: u8) -> bool {
        if self.rom_limit.is_none() {
            self.container[index as usize] = value;
            return true;
        }

        let limit = self.rom_limit.unwrap();
        if index <= limit {
            return false;
        }
        // index > rom limit
        self.container[index as usize] = value;
        return true;
    }
}
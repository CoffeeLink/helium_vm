use owo_colors::{OwoColorize, Style};
use crate::utils::chars::*;
/// Responsible for making sure there is a "ROM" block in the memory.
/// Allows the reading and writing of memory, 
/// also has draw_ui which basically generates a styled hexdump of the memory.
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
    
    /// Creates a Hexdump lookalike UI
    pub fn draw_hexdump(&self) -> String {
        let mut out = String::new();

        const BYTES_PER_LINE: usize = 17;

        let mut index: usize = 0;
        let mut line_addr: usize = 0;

        // push the top of the thing
        // Corner, line, name of window, line, secondary name, line, corner
        out.push_str(&format!("{}{}| {} |{}{}{}| {} |{}{}\n",
                              CORNER_L,
                              format!("{}", H_LINE).repeat(21),
                              "Memory View",
                              format!("{}", H_LINE).repeat(22),
                              LINEBREAK_U,
                              format!("{}", H_LINE).repeat(2),
                              "ASCII view",
                              format!("{}", H_LINE).repeat(3),
                              CORNER_R
        ));

        while index < 255 {
            let mut hex_part = String::new();
            let mut ascii_part = String::new();

            for i in 0..BYTES_PER_LINE {
                if index > 255 {
                    break;
                }

                let value = match self.container.get(index){
                    Some(v) => *v,
                    None => break,
                };
                index += 1;

                let hex = format!("{:02X}", value);

                if value == 0 {
                    hex_part.push_str(&format!("{}", hex.dimmed()));
                    ascii_part.push_str(&format!("{}", '.'.dimmed()));

                } else {
                    let mut color: Style = Style::new().white();

                    let ch = value.as_ascii();
                    if ch.is_none() {
                        ascii_part.push_str(&format!("{}", '.'.green()));
                        color = color.green();
                    } else {
                        let ch = ch.unwrap().to_char();
                        if ch.is_whitespace() || ch.is_ascii_control() {
                            ascii_part.push_str(&format!("{}", '.'.green()));
                            color = color.green();
                        } else {
                            if ch.is_alphabetic() {
                                ascii_part.push_str(&format!("{}", ch.cyan()));
                                color = color.cyan();
                            } else {
                                ascii_part.push(ch);
                            }
                        }
                    }

                    hex_part.push_str(&format!("{}", hex.style(color)));
                }

                if i == 7 || i == 15 {
                    hex_part.push(' ');
                    //ascii_part.push(' ');
                }

                if i != BYTES_PER_LINE -1{ hex_part.push(' '); }
                if i == BYTES_PER_LINE -1{ ascii_part.push(' '); }
            }

            out.push_str(&format!("{} {}: ", V_LINE,
                                  format!("{:02X}", line_addr)
                                      .bold()
                                      .bright_green()));
            out.push_str(&hex_part);
            out.push_str(&format!(" {} ", V_LINE));
            out.push_str(&ascii_part);
            out.push_str(&format!("{}\n", V_LINE));

            line_addr += 15;
        }

        // Footer
        out.push_str(&format!(
            "{}{}{}{}{}",
            CORNEL_DL,
            format!("{}", H_LINE).repeat(58),
            LINEBREAK_D,
            format!("{}", H_LINE).repeat(19),
            CORNEL_DR
        ));

        out
    }
}
use bitmatch::bitmatch;
use owo_colors::{OwoColorize, Style};
use crate::helium::io_controller::IOController;
use crate::helium::memory::MemoryControl;
use crate::utils::chars::*;

#[derive(Debug)]
pub struct CPU {
    registers: [u8; 4], // A, B, C, D

    instruction_reg: u8,
    program_counter: u8,
    secondary_counter: u8,

    interrupt_addr: u8,
    interrupt_code: u8,
    interrupt_req: bool,
    interrupt_enabled: bool,

    in_interrupt: bool,

    carry: bool,
    overflow: bool,
    zero: bool,
    signed: bool,

    pub memory: MemoryControl,
    pub io_ctl: IOController,

    pub is_on: bool,
}

impl CPU {
    pub fn new(devices: IOController, rom_data: Vec<u8>) -> Self {
        let mem = MemoryControl::new(rom_data);
        Self {
            registers: [0; 4],
            instruction_reg: 0,
            program_counter: 0,
            secondary_counter: 0,

            interrupt_req: false,
            interrupt_enabled: false,
            interrupt_code: 0,
            interrupt_addr: 0,

            in_interrupt: false,

            carry: false,
            overflow: false,
            zero: false,
            signed: false,

            memory: mem,
            io_ctl: devices,

            is_on: false,
        }
    }

    /// Resets everything as if the cpu restarted.
    pub fn reset(&mut self) {
        self.registers = [0u8; 4];
        self.program_counter = 0;
        self.secondary_counter = 0;
        self.instruction_reg = 0;
        self.interrupt_addr = 0;
        self.interrupt_code = 0;

        self.interrupt_req = false;
        self.interrupt_enabled = false;

        self.in_interrupt = false;

        self.carry = false;
        self.overflow = false;
        self.zero = false;
        self.signed = false;

        //self.memory.reset_mem();
        self.io_ctl.reset();
    }

    pub fn start(&mut self) {
        self.io_ctl.startup();
        self.is_on = true;
    }

    /// Causes a interrupt request.
    pub fn interrupt(&mut self) { self.interrupt_req = true; }

    #[bitmatch]
    pub fn next(&mut self) {
        // CHECK FOR INTERRUPT
        if self.interrupt_enabled && !self.in_interrupt && self.interrupt_req {
            self.in_interrupt = true;
            self.interrupt_req = false;

            self.secondary_counter = self.program_counter;
            self.program_counter = self.interrupt_addr;
        }


        // load instruction
        self.instruction_reg = self.memory.get(self.program_counter);
        // increment program counter.
        self.program_counter = self.program_counter.overflowing_add(1).0; // doesn't panic when 255 + 1 causes an overflow

        // Decode instruction
        // These are for the liter

        #[allow(unused_variables)]
        let x: u8 = 0;
        #[allow(unused_variables)]
        let y: u8 = 0;

        #[bitmatch]
        match self.instruction_reg {
            "0000_0000" => { self.is_on = false } // halt
            "0000_01_xx" => {
                // LOAD IMM
                let reg = x as usize;
                let value = self.memory.get(self.program_counter);
                self.program_counter = self.program_counter.overflowing_add(1).0;

                self.registers[reg] = value;
            }

            "00_0010_xx" => {
                // Load (imm)
                let reg = x as usize;
                let addr = self.memory.get(self.program_counter);
                self.program_counter = self.program_counter.overflowing_add(1).0;

                self.registers[reg] = self.memory.get(addr);
            }

            "00_0011_xx" => {
                // Store (imm)
                let reg = x as usize;
                let addr = self.memory.get(self.program_counter);
                self.program_counter = self.program_counter.overflowing_add(1).0;

                self.memory.set(addr, self.registers[reg]);
            }

            "00_0100_xx" => {
                // IN (imm)
                let reg = x as usize;
                let io_addr = self.memory.get(self.program_counter);
                self.program_counter = self.program_counter.overflowing_add(1).0;

                self.registers[reg] = self.io_ctl.read(io_addr);
            }
            "00_0101_xx" => {
                // IN (reg(imm))
                let reg = x as usize;
                let mut io_addr_reg = self.memory.get(self.program_counter);
                self.program_counter = self.program_counter.overflowing_add(1).0;

                if io_addr_reg > 3 { io_addr_reg = 0; } // Bounds check

                let io_addr = self.registers[io_addr_reg as usize];
                self.registers[reg] = self.io_ctl.read(io_addr);

            }

            "00_0110_xx" => {
                // OUT (imm)
                let reg = x as usize;
                let io_addr = self.memory.get(self.program_counter);
                self.program_counter = self.program_counter.overflowing_add(1).0;

                self.io_ctl.write(io_addr, self.registers[reg]);
            }
            "00_0111_xx" => {
                //OUT reg(imm)
                let reg = x as usize;
                let mut io_addr_reg = self.memory.get(self.program_counter);
                self.program_counter = self.program_counter.overflowing_add(1).0;

                if io_addr_reg > 3 { io_addr_reg = 0; } // Bounds check

                let io_addr = self.registers[io_addr_reg as usize];
                self.io_ctl.write(io_addr, self.registers[reg]);
            }
            "00_1000_xx" => {
                // FSWAP
                let reg = x as usize;
                let flag_state = self.flags_into_u8();
                self.flags_from_u8(self.registers[reg]);
                self.registers[reg] = flag_state;
            }

            "00_1001_xx" => {
                // Shift Right (by 1)
                let reg = x as usize;
                let value = self.registers[reg];

                self.carry = value & 1 == 1;
                self.registers[reg] = value >> 1;
            }
            "00_1010_xx" => {
                // Shift Left (by 1)
                let reg = x as usize;
                let value = self.registers[reg];

                self.carry = value & 128 == 128;
                self.registers[reg] = value << 1;
            }

            // Interrupt instructions

            "00_1011_xx" => {
                // Set Interrupt addr
                let reg = x as usize;
                self.interrupt_addr = self.registers[reg];
            }
            "00_1100_00" => {
                // enable interrupt
                self.interrupt_enabled = true;
            }
            "00_1100_01" => {
                // Clear interrupt req
                self.interrupt_req = false;
            }
            "00_1100_10" => {
                // disable interrupt
                self.interrupt_enabled = false;
            }
            "00_1100_11" => {
                // Return from interrupt mode.
                if !self.in_interrupt { return; }

                self.program_counter = self.secondary_counter;
                self.in_interrupt = false;
            }
            "00_1101_00" => {
                // Call Interrupt
                self.interrupt_code = 0;
                self.interrupt_req = true;
            }
            "00_1101_01" => {
                // Get INT code
                self.registers[0] = self.interrupt_code;
            }
            // Some system commands
            "00_1101_10" => { self.reset(); }
            "00_1101_11" => { /* No Operation */ }

            // rotation
            "00_1110_xx" => {
                // Rotate right
                let reg = x as usize;
                let value = self.registers[reg];
                self.carry = (value & 1) == 1;
                self.registers[reg] = value.rotate_right(1);
            }
            "00_1111_xx" => {
                // Rotate left
                let reg = x as usize;
                let value = self.registers[reg];
                self.carry = (value & 128) == 128;
                self.registers[reg] = value.rotate_left(1);
            }

            //ALU ops
            "01_00_xx_yy" => {
                // add and save (uses carry, and updates flags)
                self.add(x as usize, y as usize, true);
            }
            "01_01_xx_yy" => {
                // sub and save, uses carry, updates flags
                self.sub(x as usize, y as usize, true);
            }

            "01_10_xx_yy" => {
                //add without save
                self.add(x as usize, y as usize, false);
            }
            "01_11_xx_yy" => {
                //sub without save
                self.sub(x as usize, y as usize, false);
            }

            // Logic ops & move

            "10_00_xx_yy" => {
                // AND a b
                let reg_a = x as usize;
                let reg_b = y as usize;

                let result = self.registers[reg_a] & self.registers[reg_b];
                self.zero = result == 0;
                self.signed = (result & 128) == 128;

                self.registers[reg_b] = result;
            }
            "10_01_xx_yy" => {
                // xor a b
                let reg_a = x as usize;
                let reg_b = y as usize;

                let result = self.registers[reg_a] ^ self.registers[reg_b];
                self.zero = result == 0;
                self.signed = (result & 128) == 128;

                self.registers[reg_b] = result;
            }
            "10_10_xx_yy" => {
                // or a b
                let reg_a = x as usize;
                let reg_b = y as usize;

                let result = self.registers[reg_a] | self.registers[reg_b];
                self.zero = result == 0;
                self.signed = (result & 128) == 128;

                self.registers[reg_b] = result;
            }

            "10_11_xx_yy" => {
                // Move A -> B
                let reg_a = x as usize;
                let reg_b = y as usize;

                self.registers[reg_b] = self.registers[reg_a];
            }

            // memory with regs
            "11_00_xxyy" => {
                // Load reg[xx] = mem[reg[yy]]
                let reg_a = x as usize;
                let reg_b = y as usize;

                self.registers[reg_a] = self.memory.get(self.registers[reg_b]);
            }
            "11_01_xxyy" => {
                // Store mem[reg[yy]] = reg[xx]

                let reg_a = x as usize;
                let reg_b = y as usize;

                self.memory.set(self.registers[reg_b], self.registers[reg_a]);
            }

            // Jumps
            "11_10_xxx_y" => {
                // JMP IF cond(x) to (if y: reg(imm8)?: imm8)
                let condition = x as u8;
                let is_reg = y == 1;

                let mut address = self.memory.get(self.program_counter);
                self.program_counter = self.program_counter.overflowing_add(1).0;

                if is_reg { address = self.registers[address as usize] }
                self.jmp_if(condition, address);

            }
            "11_11_xxx_y" => {
                // JMPR IF cond(x) to (if y: reg(imm8)?: imm8)
                let condition = x as u8;
                let is_reg = y == 1;

                let mut address = self.memory.get(self.program_counter);
                self.program_counter = self.program_counter.overflowing_add(1).0;

                if is_reg { address = self.registers[address as usize] }

                address = self.program_counter.overflowing_add(address).0;
                self.jmp_if(condition, address);
            }

            "aaaa_aaaa" => {
                println!("unknown instruction: {a:08b}");
            }
        }
        // After everything
        self.io_ctl.update();
    }

    fn flags_into_u8(&mut self) -> u8 {
        (self.signed as u8) << 3
            | (self.carry as u8) << 2
            | (self.overflow as u8) << 1
            | self.zero as u8
    }

    fn flags_from_u8(&mut self, flags: u8) {
        self.signed =   (flags & 8) == 8;
        self.carry =    (flags & 4) == 4;
        self.overflow = (flags & 2) == 2;
        self.zero =     (flags & 1) == 1;
    }

    fn add(&mut self, reg_a: usize, reg_b: usize, save: bool) {
        let both_same_sign = self.registers[reg_a] & 128 == self.registers[reg_b] & 128;
        let positive = self.registers[reg_a] & 128 == 0;

        let (result, carry) = self.registers[reg_a].carrying_add(self.registers[reg_b], self.carry);

        // Update flags
        let result_positive = result & 128 == 0;

        // Overflow
        if result_positive != positive && both_same_sign {
            self.overflow = true
        } else { self.overflow = false }

        self.carry = carry;
        self.signed = (result & 128) == 128;
        self.zero = result == 0;

        if save {
            self.registers[reg_b] = result;
        }
    }

    fn sub(&mut self, reg_a: usize, reg_b: usize, save: bool) {
        let (sum, carry) = self.registers[reg_b].overflowing_sub(self.registers[reg_a]);
        let (f_sum, f_carry) = sum.overflowing_add(1);

        self.carry = carry | f_carry;
        self.zero = f_sum == 0;
        self.overflow = self.registers[reg_b].checked_sub(self.registers[reg_a]).is_none(); // If it overflows, the function will return none.
        self.signed = (f_sum & 128) == 128;

        if save {
            self.registers[reg_b] = f_sum;
        }
    }

    fn jmp_if(&mut self, condition_code: u8, address: u8) {
        match condition_code {
            0 => self.program_counter = address,

            1 if self.carry => self.program_counter = address,
            2 if !self.carry => self.program_counter = address,

            3 if self.overflow => self.program_counter = address,
            4 if !self.overflow => self.program_counter = address,

            5 if self.zero => self.program_counter = address,
            6 if !self.zero => self.program_counter = address,

            7 if self.signed => self.program_counter = address,

            _ => panic!("INVALID CONDITION CODE GIVEN: {:08b}", condition_code)
        }
    }
    
    fn bin_repr(value: u8) -> String {
        let mut out_style = Style::new().white();
        let mut binary_repr = format!("{:08b}", value);
        
        binary_repr.insert(4, ' ');
        
        if value == 0 {
            out_style = out_style.dimmed();
        }
        
        format!("{} {}{:03}{} {}",
                              binary_repr.style(out_style),
                              "[".dimmed().bold(),
                              value,
                              "]".dimmed().bold(),
                              V_LINE
        )
    }
    
    fn hex_repr(value: u8) -> String {
        let rerp = format!("{:02X}", value);
        return if value == 0 {
            format!("{}", rerp.dimmed())
        } else {
            rerp
        }
    }
    
    fn flag_repr(name: &str, state: bool) -> String {
        let mut out_style = Style::new().bright_red();
        if state {
            out_style = out_style.bright_green();
        }
        format!("{}", name.style(out_style))
    }
    
    pub fn generate_state_ui(&self) -> String {
        let mut out = String::new();

        // UI Design

        // Length: 80 chars <- target pls hit.
        //  Formatting:
        // If a value is 0, .dimmed()
        // If a flag is 0,  .dimmed()/.red() [Subject to change]
        // If a flag is 1,  .bright_green()
        // A value name like r1 / PC must be Bold

        // ################################################################################
        // ┌────| Registers |────┬─────────────────────┬────────┬────────┬────| Flags |───┐
        // | r0: 0000 0000 [000] | r2: 0000 0000 [000] | PC: 00 | IA: 00 | ZE, SI, CA, OV |
        // | r1: 0000 0000 [000] | r3: 0000 0000 [000] | IR: 00 | IC: 00 | IR, IE, IQ, CK |
        // +---------------------+---------------------+--------+--------+----------------+
        // ┌─────────────────────| Memory View |──────────────────────┬──| ASCII view |───┐
        
        // Header
        out.push_str("┌────| Registers |────┬─────────────────────┬────────┬────────┬────| Flags |───┐\n");
        
        // r0 and r2
        out.push(V_LINE);
        out.push_str(&format!(" {} {}",
            "r0:".bold().green(), &Self::bin_repr(self.registers[0])
        ));
        out.push_str(&format!(" {} {}",
                              "r2:".bold().green(), &Self::bin_repr(self.registers[2])
        ));
        
        // PC
        out.push_str(&format!(" {} {} {}",
                              "PC:".bold().green(), Self::hex_repr(self.program_counter), V_LINE
        ));
        // IA
        out.push_str(&format!(" {} {} {}", 
                              "IA:".bold().green(), Self::hex_repr(self.interrupt_addr), V_LINE
        ));
        // Flags Z,S,C,O
        out.push_str(&format!(
            " {}, {}, {}, {} {}\n", // ZE, SI, OV |
            Self::flag_repr("ZE", self.zero),
            Self::flag_repr("SI", self.signed),
            Self::flag_repr("CA", self.carry),
            Self::flag_repr("OV", self.overflow),
            V_LINE
        ));
        
        // Line 2

        out.push(V_LINE);
        // R1, R3
        out.push_str(&format!(" {} {}",
                              "r1:".bold().green(), &Self::bin_repr(self.registers[1])
        ));
        out.push_str(&format!(" {} {}",
                              "r3:".bold().green(), &Self::bin_repr(self.registers[3])
        ));
        
        // IR
        out.push_str(&format!(" {} {} {}",
                              "PC:".bold().green(), Self::hex_repr(self.instruction_reg), V_LINE
        ));
        // IC
        out.push_str(&format!(" {} {} {}",
                              "IA:".bold().green(), Self::hex_repr(self.interrupt_code), V_LINE
        ));

        // Flags Z,S,C,O
        out.push_str(&format!(
            " {}, {}, {}, {} {}\n", // ZE, SI, OV |
            Self::flag_repr("IN", self.in_interrupt),
            Self::flag_repr("IE", self.interrupt_enabled),
            Self::flag_repr("IQ", self.interrupt_req),
            Self::flag_repr("ON", self.is_on),
            V_LINE
        ));
        
        // Footer
        out.push_str("└─────────────────────┴─────────────────────┴────────┴────────┴────────────────┘");
        out
    }
}
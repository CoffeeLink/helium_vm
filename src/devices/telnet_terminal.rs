use std::any::Any;
use std::io::{Read, Write};
use std::io::ErrorKind::WouldBlock;
use std::net::{TcpListener, TcpStream};
use crate::devices::device::Device;

/// The goal of this device is to have the ability to connect to a port and act like a terminal
/// The CPU will be able to get information about the connection, like is_connected or last char, etc
///
/// # Behaviour docs:
/// each address will act like a register, like last char and other like that.
/// ## Address space:
/// 00: connection_state (1 if someone is connected)
/// 01: last_char (contains the last char received by the server)
/// 02: write_char (will send this char to the client upon writing it)
///
/// ## Interrupts: The device will send an interrupt if the connection_state or last_char changes
/// The interrupt code will be the configured one.
#[derive(Debug)]
pub struct TelnetTerminal {
    interrupt_code: u8,
    interrupt_queued: bool,
    interrupt_log: String,

    tcp_listener: TcpListener,

    connection_state: u8, // 1: connected, 0: not
    last_char: u8,
    char_to_write: Option<u8>,

    active_connection: Option<TcpStream>
}

impl TelnetTerminal {
    pub fn new(code: u8, port: u16) -> Self {
        Self {
            interrupt_code: code,
            interrupt_queued: false,
            tcp_listener: TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap(),
            interrupt_log: String::new(),

            connection_state: 0,
            last_char: 0,
            char_to_write: None,

            active_connection: None

        }
    }
}

impl Device for TelnetTerminal {
    fn init_device(&mut self) {
        self.tcp_listener.set_nonblocking(true).expect("TCP Listener decided to not co-operate :/");
        self.update_device();
    }

    fn startup(&mut self) {
        self.update_device();
    }

    fn update_device(&mut self) {

        // IF no connections are present, try getting one
        if self.active_connection.is_none() {
            if let Some(Ok(conn)) = self.tcp_listener.incoming().next() {
                conn.set_nonblocking(true).expect("E");
                self.active_connection = Some(conn);
                self.connection_state = 1;
                self.interrupt_queued = true;

                self.interrupt_log.push_str("Connection Acquired ");
            }
        }

        if self.active_connection.is_some() {
            // Check if a char is queued to be sent.
            if self.char_to_write.is_some() {
                let conn = self.active_connection.as_mut().unwrap();

                let ch = self.char_to_write.unwrap();
                self.char_to_write = None;

                match conn.write(&[ch]) {
                    Err(e) => {
                        conn.shutdown(std::net::Shutdown::Both).unwrap();
                        self.active_connection = None;

                        self.interrupt_queued = true;
                        self.connection_state = 0;

                        self.interrupt_log.push_str(&format!("Failed To Write: {} ", e));
                    },
                    Ok(_) => {}
                }
            }
        }

        // Try reading a byte
        if self.active_connection.is_some() {
            let conn = self.active_connection.as_mut().unwrap();

            let mut buffer: [u8; 1] = [0];

            let num_bytes = match conn.read(&mut buffer) {
                Err(e) if e.kind() == WouldBlock => { 0 },
                Err(e) => {
                    // Connection err
                    conn.shutdown(std::net::Shutdown::Both).unwrap();
                    self.active_connection = None;

                    self.interrupt_queued = true;
                    self.connection_state = 0;

                    self.interrupt_log.push_str(&format!("Connection Error: {} ", e));

                    2 // means that the connection died
                }
                Ok(num) => num
            };

            if num_bytes == 1 {
                // This should be the normal case
                self.last_char = buffer[0];
                self.interrupt_queued = true;
                self.interrupt_log.push_str(&format!("Byte received: {} ", self.last_char));
            }
        }
    }

    fn draw_ui(&mut self, _no_gui: bool, debug: bool) -> Option<String> {
        if debug {
            Some(format!("{:?}", self))
        } else {
            None
        }
    }

    fn has_interrupt_request(&mut self) -> Option<(u8, String)> {
        if self.interrupt_queued {
            self.interrupt_queued = false;

            let log = self.interrupt_log.clone();
            self.interrupt_log = String::new(); // Erase Log

            Some((self.interrupt_code, log))
        } else {
            None
        }
    }

    fn reset_device(&mut self) {}

    fn read(&mut self, address: u8) -> u8 {
        match address {
            0 => self.connection_state,
            1 => self.last_char,
            2 => 0,

            _=> 0
        }
    }

    fn write(&mut self, address: u8, value: u8) {
        match address {
            2 => self.char_to_write = Some(value),
            _=> {  }
        }
    }

    fn get_address_space(&self) -> Option<u8> { Some(3) }

    fn as_any(&self) -> &dyn Any { self }
}
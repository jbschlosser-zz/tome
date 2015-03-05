pub const TELNET_SE: u8 = 240; // End of subnegotiation parameters.
pub const TELNET_NOP: u8 = 241; // No operation.
pub const TELNET_DATA_MARK: u8 = 242; // The data stream portion of a Synch. This should always be accompanied by a TCP Urgent notification.
pub const TELNET_BREAK: u8 = 243; // NVT character BRK.
pub const TELNET_IP: u8 = 244; // The function IP (interrupt process).
pub const TELNET_AO: u8 = 245; // The function AO (abort output).
pub const TELNET_AYT: u8 = 246; // The function AYT (are you there).
pub const TELNET_EC: u8 = 247; // The function EC (erase character).
pub const TELNET_EL: u8 = 248; // The function EL (erase line).
pub const TELNET_GA: u8 = 249; // The GA (go ahead) signal.
pub const TELNET_SB: u8 = 250; // Indicates that what follows is subnegotiation of the indicated option.
pub const TELNET_WILL: u8 = 251; // Indicates the desire to begin performing, or confirmation that you are now performing, the indicated option.
pub const TELNET_WONT: u8 = 252; // Indicates the refusal to perform, or continue performing, the indicated option.
pub const TELNET_DO: u8 = 253; // Indicates the request that the other party perform, or
                      // confirmation that you are expecting the other party to perform, the
                      // indicated option.
pub const TELNET_DONT: u8 = 254; // Indicates the demand that the other party stop performing,
                        // or confirmation that you are no longer expecting the other party
                        // to perform, the indicated option.
pub const TELNET_IAC: u8 = 255; // Interpret As Command. Indicates the start of a telnet option
                       // negotiation.
 
pub const TELNET_MAX_COMMAND_SIZE: usize = 64;

#[derive(Debug)]
pub struct Telnet {
    cmd: Vec<u8>
}

impl Telnet {
    pub fn new() -> Telnet {
        Telnet { cmd: Vec::new() }
    }
    pub fn update(&mut self, byte: u8) -> Option<Vec<u8>> {
        // No command starts until an IAC is seen.
        if self.cmd.len() == 0 && byte != TELNET_IAC {
            return None;
        }

        // Check if the command has reached the max allowed size.
        if self.cmd.len() == TELNET_MAX_COMMAND_SIZE {
            // Boom. Ran out of space for the command :(
            self.cmd.clear();
            return None;
        }

        // Add the byte to the command.
        self.cmd.push(byte);

        // The current length of the command determines how the byte
        // affects the state.
        if self.cmd.len() == 2 {
            match byte {
                // Two byte commands.
                TELNET_IAC|TELNET_NOP|TELNET_DATA_MARK|TELNET_BREAK|TELNET_IP|TELNET_AO|TELNET_AYT|TELNET_EC|TELNET_EL|TELNET_GA => {
                    let res = self.cmd.clone();
                    self.cmd.clear();
                    return Some(res);
                },
                // Three byte commands.
                TELNET_WILL|TELNET_WONT|TELNET_DO|TELNET_DONT|TELNET_SB => (),
                // Unknown command... ignore it and start over.
                _ => self.cmd.clear()
            }
        } else if self.cmd.len() == 3 {
            let prev_byte = self.cmd[self.cmd.len() - 2];
            match prev_byte {
                // Three byte commands.
                TELNET_WILL|TELNET_WONT|TELNET_DO|TELNET_DONT => {
                    let res = self.cmd.clone();
                    self.cmd.clear();
                    return Some(res);
                },
                // Sub-negotiation can span an arbitrary number of bytes.
                TELNET_SB => (),
                // Something bad happened... start over.
                _ => self.cmd.clear()
            }
        } else if self.cmd.len() > 3 {
            // Sub-negotiation is assumed, since that is the only command
            // that can be this long. Check if the most recent bytes are
            // IAC,SE. This ends sub-negotiation.
            if self.cmd[self.cmd.len() - 2] == TELNET_IAC &&
                self.cmd[self.cmd.len() - 1] == TELNET_SE
            {
                let res = self.cmd.clone();
                self.cmd.clear();
                return Some(res);
            }
        }

        return None;
    }
    pub fn in_progress(&self) -> bool {
        self.cmd.len() > 0
    }
}

use parse_state::ParseState;

pub const SE: u8 = 240; // End of subnegotiation parameters.
pub const NOP: u8 = 241; // No operation.
pub const DATA_MARK: u8 = 242; // The data stream portion of a Synch. This should always be accompanied by a TCP Urgent notification.
pub const BREAK: u8 = 243; // NVT character BRK.
pub const IP: u8 = 244; // The function IP (interrupt process).
pub const AO: u8 = 245; // The function AO (abort output).
pub const AYT: u8 = 246; // The function AYT (are you there).
pub const EC: u8 = 247; // The function EC (erase character).
pub const EL: u8 = 248; // The function EL (erase line).
pub const GA: u8 = 249; // The GA (go ahead) signal.
pub const SB: u8 = 250; // Indicates that what follows is subnegotiation of the indicated option.
pub const WILL: u8 = 251; // Indicates the desire to begin performing, or confirmation that you are now performing, the indicated option.
pub const WONT: u8 = 252; // Indicates the refusal to perform, or continue performing, the indicated option.
pub const DO: u8 = 253; // Indicates the request that the other party perform, or
                      // confirmation that you are expecting the other party to perform, the
                      // indicated option.
pub const DONT: u8 = 254; // Indicates the demand that the other party stop performing,
                        // or confirmation that you are no longer expecting the other party
                        // to perform, the indicated option.
pub const IAC: u8 = 255; // Interpret As Command. Indicates the start of a telnet option
                       // negotiation.
pub const GMCP: u8 = 0xC9;
 
pub fn parse(old_state: &ParseState, byte: u8) -> ParseState {
    match *old_state {
        ParseState::NotInProgress => {
            match byte {
                IAC => ParseState::InProgress(vec![byte]),
                _ => ParseState::NotInProgress
            }
        },
        ParseState::InProgress(ref b) => {
            let mut bytes = b.clone();
            bytes.push(byte);

            // Determine the next state.
            match bytes.len() {
                1 => ParseState::InProgress(bytes),
                2 => {
                    match byte {
                        // Two byte commands.
                        IAC | NOP | DATA_MARK | BREAK | IP | AO | AYT |
                        EC | EL | GA =>
                            ParseState::Success(bytes),

                        // Three byte commands.
                        WILL | WONT | DO | DONT | SB =>
                            ParseState::InProgress(bytes),

                        // Unknown command.
                        _ => ParseState::Error(bytes)
                    }
                },
                3 => {
                    let prev_byte = bytes[bytes.len() - 2];
                    match prev_byte {
                        // Three byte commands.
                        WILL | WONT | DO | DONT =>
                            ParseState::Success(bytes),

                        // Sub-negotiation can span an arbitrary number of
                        // bytes.
                        SB => ParseState::InProgress(bytes),

                        // Unexpected command.
                        _ => ParseState::Error(bytes)
                    }
                },
                _ => {
                    // Sub-negotiation is assumed, since that is the only
                    // command that can be this long. Check if the most recent
                    // bytes are IAC,SE. This ends sub-negotiation.
                    let prev_byte = bytes[bytes.len() - 2];
                    if prev_byte == IAC && byte == SE {
                        ParseState::Success(bytes)
                    } else {
                        ParseState::InProgress(bytes)
                    }
                }
            }
        },
        ParseState::Success(_) => parse(&ParseState::NotInProgress, byte),
        ParseState::Error(_) => parse(&ParseState::NotInProgress, byte)
    }
}

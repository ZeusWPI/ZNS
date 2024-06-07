use crate::structs::{Message, Opcode, RCODE};

impl Message {
    pub fn set_response(&mut self, rcode: RCODE) {
        self.header.flags = (self.header.flags | 0b1_0000_1_0_0_0_000_0000 | rcode as u16)
            & 0b1_1111_1_0_1_0_111_1111
    }

    pub fn get_opcode(&self) -> Result<Opcode, String> {
        Opcode::try_from((self.header.flags & 0b0111100000000000) >> 11)
    }
}

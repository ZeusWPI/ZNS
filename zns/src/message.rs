use crate::structs::{Message, Opcode, RCODE};

impl Message {
    pub fn set_response(&mut self, rcode: RCODE) {
        self.header.flags = (self.header.flags | 0b1_0000_1_0_0_0_000_0000 | rcode as u16)
            & 0b1_1111_1_0_1_0_111_1111
    }

    pub fn get_opcode(&self) -> Result<Opcode, String> {
        Opcode::try_from((self.header.flags & 0b0111100000000000) >> 11)
    }

    #[allow(dead_code)] // Used with tests
    pub fn get_rcode(&self) -> Result<RCODE, u16> {
        RCODE::try_from(self.header.flags & (!0 >> 12))
    }
}

#[cfg(test)]
mod tests {

    use crate::structs::Header;

    use super::*;

    #[test]
    fn test() {
        let mut message = Message {
            header: Header {
                id: 1,
                flags: 288,
                qdcount: 0,
                ancount: 0,
                nscount: 0,
                arcount: 0,
            },
            question: vec![],
            answer: vec![],
            authority: vec![],
            additional: vec![],
        };

        assert_eq!(message.get_opcode().unwrap() as u8, Opcode::QUERY as u8);

        message.set_response(RCODE::NOTIMP);

        assert!((message.header.flags & (1 << 15)) > 0);

        assert_eq!(message.get_rcode().unwrap(), RCODE::NOTIMP);
    }
}

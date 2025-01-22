use crate::{
    labelstring::LabelString,
    structs::{Message, Opcode, RCODE, RR},
};

impl Message {
    pub fn set_response(&mut self, rcode: RCODE) {
        self.header.flags =
            (self.header.flags | 0b1000_0100_0000_0000 | rcode as u16) & 0b1111_1101_0111_1111
    }

    pub fn get_opcode(&self) -> Result<Opcode, String> {
        Opcode::try_from((self.header.flags & 0b0111100000000000) >> 11)
    }

    #[cfg(feature = "test-utils")]
    pub fn get_rcode(&self) -> Result<RCODE, u16> {
        RCODE::try_from(self.header.flags & (!0 >> 12))
    }

    pub fn not_authoritative(&self, auth_zone: &LabelString) -> Option<String> {
        for question in &self.question {
            let zlen = question.qname.len();
            if !(zlen >= auth_zone.len()
                && &Into::<LabelString>::into(
                    question.qname.as_slice()[zlen - auth_zone.len()..].to_vec(),
                ) == auth_zone)
            {
                return Some(question.qname.to_string());
            }
        }
        None
    }

    pub fn extend_answer(&mut self, rrs: Vec<RR>) {
        self.header.ancount += rrs.len() as u16;
        self.answer.extend(rrs);
    }
}

#[cfg(test)]
mod tests {

    use crate::{structs::Header, test_utils::get_message};

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

    #[test]
    fn test_authoritative() {
        let name = LabelString::from("not.good.zone");

        let message = get_message(Some(name));

        assert!(message
            .not_authoritative(&LabelString::from("good"))
            .is_some());

        assert!(message
            .not_authoritative(&LabelString::from("Zone"))
            .is_none())
    }
}

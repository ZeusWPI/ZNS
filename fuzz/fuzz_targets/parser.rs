#![no_main]

use libfuzzer_sys::fuzz_target;
use zns::{
    parser::{FromBytes},
    reader::Reader,
    structs::Message,
};

fuzz_target!(|data: &[u8]| {
    let mut reader = Reader::new(data);
    let _ = Message::from_bytes(&mut reader);
});


#![no_main]

use libfuzzer_sys::fuzz_target;
use zns::{parser::ToBytes, structs::Message};

fuzz_target!(|message: Message| {
    let _ = Message::to_bytes(message);
});

use std::collections::VecDeque;

use crate::Message;

struct Worker {
    queue: VecDeque<Message>
}

impl Worker {
    pub fn new() -> Worker {
        Worker { queue: VecDeque::new() }
    }

    pub fn append() {

    }
}



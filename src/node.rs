use std::{io::Write, net::TcpStream};

use crate::resp::RESP;

pub struct Node {
    stream: TcpStream,
}

impl Node {
    pub fn new(stream: TcpStream) -> Self {
        Node { stream }
    }

    pub fn write(&mut self, resp: RESP) {
        self.stream.write(resp.to_string().as_bytes()).unwrap();
    }
}

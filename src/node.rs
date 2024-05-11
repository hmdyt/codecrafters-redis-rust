use std::{
    io::{Read, Write},
    net::TcpStream,
};

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

    pub fn read(&mut self) -> RESP {
        let mut buf: [u8; 1024] = [0; 1024];
        self.stream.read(&mut buf).unwrap();
        RESP::from_bytes(&buf)
    }
}

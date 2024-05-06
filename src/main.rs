use std::thread;
use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                loop {
                    let mut buf = [0; 512];
                    let read_count = stream.read(&mut buf).unwrap();
                    if read_count == 0 {
                        println!("connection closed");
                        break;
                    }
                    stream.write(b"+PONG\r\n").unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        });
    }
}

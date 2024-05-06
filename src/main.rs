use std::net::TcpStream;
use std::thread;
use std::{
    io::{Read, Write},
    net::TcpListener,
};

use redis_starter_rust::command::RedisCommand;
use redis_starter_rust::store;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        thread::spawn(move || match stream {
            Ok(stream) => {
                handle_stream(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        });
    }
}

fn handle_stream(mut stream: TcpStream) {
    println!("accepted new connection");
    loop {
        let mut buf = [0; 1024];
        let read_count = stream.read(&mut buf).unwrap();
        if read_count == 0 {
            println!("connection closed");
            break;
        } else {
            let ret = handle_redis_command(RedisCommand::from_binary(&buf[..read_count]));
            stream.write(format_returning_str(&ret).as_bytes()).unwrap();
        }
    }
}

fn handle_redis_command(command: RedisCommand) -> String {
    match command {
        RedisCommand::Echo(s) => s,
        RedisCommand::Ping => "PONG".to_string(),
        RedisCommand::Set { key, value } => {
            store::set(&key, &value);
            "OK".to_string()
        }
        RedisCommand::Get { key } => match store::get(&key) {
            Some(value) => value,
            None => "nil".to_string(),
        },
    }
}

fn format_returning_str(s: &str) -> String {
    if s == "OK" {
        return "+OK\r\n".to_string();
    }
    format!("${}\r\n{}\r\n", s.len(), s)
}

use std::net::TcpStream;
use std::thread;
use std::{
    io::{Read, Write},
    net::TcpListener,
};

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

#[derive(Debug, PartialEq)]
enum RedisCommand {
    Echo(String),
    PING,
}

impl RedisCommand {
    // *2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n
    pub fn from_binary(data: &[u8]) -> RedisCommand {
        let data_string = String::from_utf8(data.to_vec()).unwrap();
        let mut lines = data_string.lines();
        let _line_count = lines.next().unwrap();

        let _command_length = lines.next().unwrap();
        let command = lines.next().unwrap();

        match command {
            "ECHO" => RedisCommand::Echo(lines.skip(1).next().unwrap().to_string()),
            "PING" => RedisCommand::PING,
            _ => panic!("unknown command"),
        }
    }
}

fn handle_redis_command(command: RedisCommand) -> String {
    match command {
        RedisCommand::Echo(s) => s,
        RedisCommand::PING => "PONG".to_string(),
    }
}

fn format_returning_str(s: &str) -> String {
    format!("${}\r\n{}\r\n", s.len(), s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_command_from_binary_ping() {
        let data = b"*1\r\n$4\r\nPING\r\n";
        let cmd = RedisCommand::from_binary(data);
        assert_eq!(cmd, RedisCommand::PING);
    }

    #[test]
    fn test_redis_command_from_binary_echo() {
        let data = b"*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n";
        let cmd = RedisCommand::from_binary(data);
        assert_eq!(cmd, RedisCommand::Echo("hey".to_string()));
    }
}

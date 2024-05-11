use std::net::TcpStream;
use std::thread;
use std::{
    io::{Read, Write},
    net::TcpListener,
};

use redis_starter_rust::command::{InfoSection, RedisCommand, SetCommandOption};
use redis_starter_rust::{role, store};

const DEFAULT_PORT: &str = "6379";
const DEFAULT_HOST: &str = "127.0.0.1";

fn main() {
    let args = redis_starter_rust::cli::CliArgs::parse();
    role::set_role(args.role);
    let listener = TcpListener::bind(format!(
        "{}:{}",
        args.host.unwrap_or(DEFAULT_HOST.to_string()),
        args.port.unwrap_or(DEFAULT_PORT.to_string())
    ))
    .unwrap();

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
        RedisCommand::Set {
            key,
            value,
            options,
        } => {
            let px = options.iter().find_map(|option| match option {
                SetCommandOption::Px(px) => Some(*px),
            });
            store::set(&key, &value, px);
            "OK".to_string()
        }
        RedisCommand::Get { key } => match store::get(&key) {
            Some(value) => value,
            None => "nil".to_string(),
        },
        RedisCommand::Info { section } => match section {
            InfoSection::All => handle_redis_command_info_replication(),
            InfoSection::Replication => handle_redis_command_info_replication(),
        },
    }
}

fn handle_redis_command_info_replication() -> String {
    let role = role::get_role();
    match role {
        role::Role::Master => "role:master".to_string(),
        role::Role::Slave {
            master_host: _,
            master_port: _,
        } => "role:slave".to_string(),
    }
}

fn format_returning_str(s: &str) -> String {
    match s {
        "OK" => return "+OK\r\n".to_string(),
        "nil" => return "$-1\r\n".to_string(),
        s => format!("${}\r\n{}\r\n", s.len(), s),
    }
}

use redis_starter_rust::command::{InfoSection, RedisCommand, ReplconfCommand, SetCommandOption};
use redis_starter_rust::resp::RESP;
use redis_starter_rust::server_state::{Role, ServerState};
use redis_starter_rust::{server_state, store};
use std::net::TcpStream;
use std::thread;
use std::{
    io::{Read, Write},
    net::TcpListener,
};

const DEFAULT_PORT: &str = "6379";
const DEFAULT_HOST: &str = "127.0.0.1";

fn main() {
    let args = redis_starter_rust::cli::CliArgs::parse();
    ServerState::init(&args.role);
    handshake(args.role, args.port.as_deref().unwrap_or(DEFAULT_PORT));

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

fn handshake(role: Role, port: &str) {
    match role {
        Role::Master => {}
        Role::Slave {
            master_host,
            master_port,
        } => {
            let stream = TcpStream::connect(format!("{}:{}", master_host, master_port)).unwrap();
            let mut node = redis_starter_rust::node::Node::new(stream);
            node.write(RedisCommand::Ping.to_resp());
            let _ = node.read();

            node.write(
                RedisCommand::Replconf {
                    command: ReplconfCommand::ListeningPort(port.to_string()),
                }
                .to_resp(),
            );
            let _ = node.read();

            node.write(
                RedisCommand::Replconf {
                    command: ReplconfCommand::Capa("psync2".to_string()),
                }
                .to_resp(),
            );
            let _ = node.read();

            node.write(
                RedisCommand::Psync {
                    master_replid: "?".to_string(),
                    master_repl_offset: -1,
                }
                .to_resp(),
            );
        }
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
            let ret = handle_redis_command(RedisCommand::new(RESP::from_bytes(&buf[..read_count])));
            stream.write(ret.to_string().as_bytes()).unwrap();
        }
    }
}

fn handle_redis_command(command: RedisCommand) -> RESP {
    match command {
        RedisCommand::Echo(s) => RESP::bulk_strings(&s),
        RedisCommand::Ping => RESP::simple_string("PONG"),
        RedisCommand::Set {
            key,
            value,
            options,
        } => {
            let px = options.iter().find_map(|option| match option {
                SetCommandOption::Px(px) => Some(*px),
            });
            store::set(&key, &value, px);
            RESP::simple_string("OK")
        }
        RedisCommand::Get { key } => match store::get(&key) {
            Some(value) => RESP::bulk_strings(&value),
            None => RESP::NullBulkStrings,
        },
        RedisCommand::Info { section } => match section {
            InfoSection::All => handle_redis_command_info_replication(),
            InfoSection::Replication => handle_redis_command_info_replication(),
        },
        RedisCommand::Replconf { .. } => {
            // TODO: implement
            RESP::simple_string("OK")
        }
        RedisCommand::Psync { .. } => {
            let state = ServerState::get();
            RESP::SimpleString(format!(
                "FULLRESYNC {} {}\n",
                state.master_replid, state.master_repl_offset
            ))
        }
    }
}

fn handle_redis_command_info_replication() -> RESP {
    let state = ServerState::get();
    match state.role {
        server_state::Role::Master => RESP::BulkStrings(format!(
            "role:master\nmaster_replid:{}\nmaster_repl_offset:{}",
            state.master_replid, state.master_repl_offset
        )),
        server_state::Role::Slave {
            master_host: _,
            master_port: _,
        } => RESP::bulk_strings("role:slave"),
    }
}

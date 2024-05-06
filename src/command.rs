#[derive(Debug, PartialEq)]
pub enum RedisCommand {
    Echo(String),
    Ping,
    Set {
        key: String,
        value: String,
        options: Vec<SetCommandOption>,
    },
    Get {
        key: String,
    },
}

impl RedisCommand {
    // *2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n
    pub fn from_binary(data: &[u8]) -> RedisCommand {
        let data_string = String::from_utf8(data.to_vec()).unwrap();
        let mut lines = data_string.lines();
        let _line_count = lines.next().unwrap();

        let _command_length = lines.next().unwrap();
        let command = lines.next().unwrap();

        let mut args_iter = lines.skip(1).step_by(2);

        match command {
            "ECHO" => RedisCommand::Echo(args_iter.next().unwrap().to_string()),
            "PING" => RedisCommand::Ping,
            "SET" => {
                let key = args_iter.next().unwrap().to_string();
                let value = args_iter.next().unwrap().to_string();
                let mut options = vec![];
                loop {
                    match args_iter.next() {
                        Some(option) => {
                            let value = args_iter.next().unwrap();
                            options.push(SetCommandOption::new(option, value));
                        }
                        None => break,
                    }
                }
                RedisCommand::Set {
                    key,
                    value,
                    options,
                }
            }
            "GET" => RedisCommand::Get {
                key: args_iter.next().unwrap().to_string(),
            },
            _ => panic!("unknown command"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SetCommandOption {
    Px(u128), // milliseconds
}

impl SetCommandOption {
    pub fn new(option: &str, value: &str) -> SetCommandOption {
        match option {
            "px" => SetCommandOption::Px(value.parse().unwrap()),
            _ => panic!("unknown option"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_command_from_binary_ping() {
        let data = b"*1\r\n$4\r\nPING\r\n";
        let cmd = RedisCommand::from_binary(data);
        assert_eq!(cmd, RedisCommand::Ping);
    }

    #[test]
    fn test_redis_command_from_binary_echo() {
        let data = b"*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n";
        let cmd = RedisCommand::from_binary(data);
        assert_eq!(cmd, RedisCommand::Echo("hey".to_string()));
    }

    #[test]
    fn test_redis_command_from_binary_set() {
        let data = b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n$2\r\npx\r\n$4\r\n1000\r\n";
        let cmd = RedisCommand::from_binary(data);
        assert_eq!(
            cmd,
            RedisCommand::Set {
                key: "key".to_string(),
                value: "value".to_string(),
                options: vec![SetCommandOption::Px(1000)]
            }
        );
    }

    #[test]
    fn test_redis_command_from_binary_get() {
        let data = b"*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n";
        let cmd = RedisCommand::from_binary(data);
        assert_eq!(
            cmd,
            RedisCommand::Get {
                key: "key".to_string()
            }
        );
    }
}

#[derive(Debug, PartialEq)]
pub enum RedisCommand {
    Echo(String),
    Ping,
    Set { key: String, value: String },
    Get { key: String },
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
            "SET" => RedisCommand::Set {
                key: args_iter.next().unwrap().to_string(),
                value: args_iter.next().unwrap().to_string(),
            },
            "GET" => RedisCommand::Get {
                key: args_iter.next().unwrap().to_string(),
            },
            _ => panic!("unknown command"),
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
        let data = b"*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n";
        let cmd = RedisCommand::from_binary(data);
        assert_eq!(
            cmd,
            RedisCommand::Set {
                key: "key".to_string(),
                value: "value".to_string()
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

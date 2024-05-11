use crate::resp::RESP;

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
    Info {
        section: InfoSection,
    },
    Replconf {
        command: ReplconfCommand,
    },
    Psync {
        master_replid: String,
        master_repl_offset: i64,
    },
}

impl RedisCommand {
    pub fn new(resp: RESP) -> Self {
        if let RESP::Array(array) = resp {
            let mut iter = array.iter();
            match iter.next().unwrap() {
                RESP::BulkStrings(command) => match command.as_str() {
                    "PING" => RedisCommand::Ping,
                    "ECHO" => Self::new_echo(&mut iter),
                    "SET" => Self::new_set(&mut iter),
                    "GET" => Self::new_get(&mut iter),
                    "INFO" => Self::new_info(&mut iter),
                    "REPLCONF" => Self::new_replconf(&mut iter),
                    "PSYNC" => Self::new_psync(&mut iter),
                    _ => panic!("unknown command"),
                },
                _ => panic!("invalid command"),
            }
        } else {
            panic!("invalid command");
        }
    }

    fn new_echo(iter: &mut std::slice::Iter<RESP>) -> Self {
        let value = match iter.next().unwrap() {
            RESP::BulkStrings(value) => value,
            _ => panic!("invalid command"),
        };
        RedisCommand::Echo(value.to_string())
    }

    fn new_set(iter: &mut std::slice::Iter<RESP>) -> Self {
        let key = match iter.next().unwrap() {
            RESP::BulkStrings(key) => key,
            _ => panic!("invalid command"),
        };
        let value = match iter.next().unwrap() {
            RESP::BulkStrings(value) => value,
            _ => panic!("invalid command"),
        };
        let mut options = vec![];
        loop {
            match iter.next() {
                Some(option) => {
                    let value = match iter.next().unwrap() {
                        RESP::BulkStrings(value) => value,
                        _ => panic!("invalid command"),
                    };
                    let option = match option {
                        RESP::BulkStrings(option) => option,
                        _ => panic!("invalid command"),
                    };
                    options.push(SetCommandOption::new(option, value));
                }
                None => break,
            }
        }
        RedisCommand::Set {
            key: key.to_string(),
            value: value.to_string(),
            options,
        }
    }

    fn new_get(iter: &mut std::slice::Iter<RESP>) -> Self {
        let key = match iter.next().unwrap() {
            RESP::BulkStrings(key) => key,
            _ => panic!("invalid command"),
        };
        RedisCommand::Get {
            key: key.to_string(),
        }
    }

    fn new_info(iter: &mut std::slice::Iter<RESP>) -> Self {
        let section = match iter.next() {
            Some(RESP::BulkStrings(section)) => InfoSection::new(Some(section)),
            None => InfoSection::new(None),
            _ => panic!("invalid command"),
        };
        RedisCommand::Info { section }
    }

    fn new_replconf(iter: &mut std::slice::Iter<RESP>) -> RedisCommand {
        let command = match iter.next().unwrap() {
            RESP::BulkStrings(command) => command,
            _ => panic!("invalid command"),
        };
        let arg = match iter.next().unwrap() {
            RESP::BulkStrings(arg) => arg,
            _ => panic!("invalid command"),
        };
        RedisCommand::Replconf {
            command: ReplconfCommand::new(command, arg),
        }
    }

    fn new_psync(iter: &mut std::slice::Iter<RESP>) -> RedisCommand {
        let master_replid = match iter.next().unwrap() {
            RESP::BulkStrings(master_replid) => master_replid,
            _ => panic!("invalid command"),
        };
        let master_repl_offset = match iter.next().unwrap() {
            RESP::BulkStrings(master_repl_offset) => master_repl_offset,
            _ => panic!("invalid command"),
        };
        RedisCommand::Psync {
            master_replid: master_replid.to_string(),
            master_repl_offset: master_repl_offset.parse().unwrap(),
        }
    }

    pub fn to_resp(self) -> RESP {
        match self {
            RedisCommand::Ping => RESP::Array(vec![RESP::BulkStrings("PING".to_string())]),
            RedisCommand::Echo(s) => RESP::Array(vec![
                RESP::BulkStrings("ECHO".to_string()),
                RESP::BulkStrings(s),
            ]),
            RedisCommand::Set {
                key,
                value,
                options,
            } => {
                let mut ret = vec![
                    RESP::BulkStrings("SET".to_string()),
                    RESP::BulkStrings(key),
                    RESP::BulkStrings(value),
                ];
                for option in options {
                    match option {
                        SetCommandOption::Px(px) => {
                            ret.push(RESP::BulkStrings("px".to_string()));
                            ret.push(RESP::BulkStrings(px.to_string()));
                        }
                    }
                }
                RESP::Array(ret)
            }
            RedisCommand::Get { key } => RESP::Array(vec![
                RESP::BulkStrings("GET".to_string()),
                RESP::BulkStrings(key),
            ]),
            RedisCommand::Info { section } => match section {
                InfoSection::All => RESP::Array(vec![RESP::BulkStrings("INFO".to_string())]),
                InfoSection::Replication => RESP::Array(vec![
                    RESP::BulkStrings("INFO".to_string()),
                    RESP::BulkStrings("replication".to_string()),
                ]),
            },
            RedisCommand::Replconf { command } => match command {
                ReplconfCommand::ListeningPort(port) => RESP::Array(vec![
                    RESP::BulkStrings("REPLCONF".to_string()),
                    RESP::BulkStrings("listening-port".to_string()),
                    RESP::BulkStrings(port),
                ]),
                ReplconfCommand::Capa(capa) => RESP::Array(vec![
                    RESP::BulkStrings("REPLCONF".to_string()),
                    RESP::BulkStrings("capa".to_string()),
                    RESP::BulkStrings(capa),
                ]),
            },
            RedisCommand::Psync {
                master_replid,
                master_repl_offset,
            } => RESP::Array(vec![
                RESP::BulkStrings("PSYNC".to_string()),
                RESP::BulkStrings(master_replid),
                RESP::BulkStrings(master_repl_offset.to_string()),
            ]),
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

#[derive(Debug, PartialEq)]
pub enum InfoSection {
    All,
    Replication,
}

impl InfoSection {
    pub fn new(maybe_str: Option<&str>) -> Self {
        match maybe_str {
            Some("replication") => InfoSection::Replication,
            None => InfoSection::All,
            _ => panic!("unknown section"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ReplconfCommand {
    ListeningPort(String),
    Capa(String),
}

impl ReplconfCommand {
    pub fn new(command: &str, arg: &str) -> Self {
        match command {
            "listening-port" => ReplconfCommand::ListeningPort(arg.to_string()),
            "capa" => ReplconfCommand::Capa(arg.to_string()),
            _ => panic!("unknown command"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ping() {
        let resp = RESP::Array(vec![RESP::BulkStrings("PING".to_string())]);
        assert_eq!(RedisCommand::new(resp), RedisCommand::Ping);
    }

    #[test]
    fn test_new_echo() {
        let resp = RESP::Array(vec![
            RESP::BulkStrings("ECHO".to_string()),
            RESP::BulkStrings("hello".to_string()),
        ]);
        assert_eq!(
            RedisCommand::new(resp),
            RedisCommand::Echo("hello".to_string())
        );
    }

    #[test]
    fn test_new_set() {
        let resp = RESP::Array(vec![
            RESP::BulkStrings("SET".to_string()),
            RESP::BulkStrings("key".to_string()),
            RESP::BulkStrings("value".to_string()),
            RESP::BulkStrings("px".to_string()),
            RESP::BulkStrings("1000".to_string()),
        ]);
        assert_eq!(
            RedisCommand::new(resp),
            RedisCommand::Set {
                key: "key".to_string(),
                value: "value".to_string(),
                options: vec![SetCommandOption::Px(1000)]
            }
        );
    }

    #[test]
    fn test_new_get() {
        let resp = RESP::Array(vec![
            RESP::BulkStrings("GET".to_string()),
            RESP::BulkStrings("key".to_string()),
        ]);
        assert_eq!(
            RedisCommand::new(resp),
            RedisCommand::Get {
                key: "key".to_string()
            }
        );
    }

    #[test]
    fn test_new_info() {
        let resp = RESP::Array(vec![RESP::BulkStrings("INFO".to_string())]);
        assert_eq!(
            RedisCommand::new(resp),
            RedisCommand::Info {
                section: InfoSection::All
            }
        );

        let resp = RESP::Array(vec![
            RESP::BulkStrings("INFO".to_string()),
            RESP::BulkStrings("replication".to_string()),
        ]);
        assert_eq!(
            RedisCommand::new(resp),
            RedisCommand::Info {
                section: InfoSection::Replication
            }
        );
    }

    #[test]
    fn test_new_replconf() {
        let resp = RESP::Array(vec![
            RESP::BulkStrings("REPLCONF".to_string()),
            RESP::BulkStrings("listening-port".to_string()),
            RESP::BulkStrings("12345".to_string()),
        ]);
        assert_eq!(
            RedisCommand::new(resp),
            RedisCommand::Replconf {
                command: ReplconfCommand::ListeningPort("12345".to_string())
            }
        );

        let resp = RESP::Array(vec![
            RESP::BulkStrings("REPLCONF".to_string()),
            RESP::BulkStrings("capa".to_string()),
            RESP::BulkStrings("eof".to_string()),
        ]);
        assert_eq!(
            RedisCommand::new(resp),
            RedisCommand::Replconf {
                command: ReplconfCommand::Capa("eof".to_string())
            }
        );
    }

    #[test]
    fn test_new_psync() {
        let resp = RESP::Array(vec![
            RESP::BulkStrings("PSYNC".to_string()),
            RESP::BulkStrings("master_replid".to_string()),
            RESP::BulkStrings("1000".to_string()),
        ]);
        assert_eq!(
            RedisCommand::new(resp),
            RedisCommand::Psync {
                master_replid: "master_replid".to_string(),
                master_repl_offset: 1000
            }
        );
    }
}

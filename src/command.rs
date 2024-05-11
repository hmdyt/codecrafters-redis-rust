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
}

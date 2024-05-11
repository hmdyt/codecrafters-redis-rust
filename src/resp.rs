#[derive(Debug)]
pub enum RESP {
    SimpleString(String),
    BulkStrings(String),
    NullBulkStrings,
}

impl RESP {
    pub fn to_string(self) -> String {
        match self {
            RESP::SimpleString(s) => format!("+{}\r\n", s),
            RESP::BulkStrings(s) => format!("${}\r\n{}\r\n", s.len(), s),
            RESP::NullBulkStrings => "$-1\r\n".to_string(),
        }
    }

    pub fn simple_string(s: &str) -> RESP {
        RESP::SimpleString(s.to_string())
    }

    pub fn bulk_strings(s: &str) -> RESP {
        RESP::BulkStrings(s.to_string())
    }
}

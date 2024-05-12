#[derive(Debug, PartialEq, Clone)]
pub enum RESP {
    SimpleString(String),
    BulkStrings(String),
    NullBulkStrings,
    Array(Vec<RESP>),
    Rdb(Vec<u8>),
}

impl RESP {
    pub fn to_string(self) -> String {
        match self {
            Self::SimpleString(s) => format!("+{}\r\n", s),
            Self::BulkStrings(s) => format!("${}\r\n{}\r\n", s.len(), s),
            Self::NullBulkStrings => "$-1\r\n".to_string(),
            Self::Array(array) => {
                let mut ret = format!("*{}\r\n", array.len());
                for resp in array {
                    ret.push_str(&resp.to_string());
                }
                ret
            }
            Self::Rdb(data) => {
                let mut ret = format!("${}\r\n", data.len());
                for byte in data {
                    ret.push(byte as char);
                }
                ret.push_str("\r\n");
                ret
            }
        }
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let mut iter = data.iter().map(|&x| x as char);
        let ret = Self::parse(&mut iter);
        ret
    }

    fn parse(iter: &mut impl Iterator<Item = char>) -> Self {
        let ret = match iter.next() {
            Some('+') => Self::parse_simple_string(iter),
            Some('$') => Self::parse_bulk_strings(iter),
            Some('*') => Self::parse_array(iter),
            _ => panic!("unknown type"),
        };
        ret
    }

    fn parse_simple_string(iter: &mut impl Iterator<Item = char>) -> Self {
        // '+' is already consumed
        let s = iter.take_while(|&x| x != '\r').collect::<String>();
        assert_eq!(iter.next(), Some('\n'));
        Self::SimpleString(s)
    }

    fn parse_bulk_strings(iter: &mut impl Iterator<Item = char>) -> Self {
        // '$' is already consumed
        let n = iter.take_while(|&x| x != '\r').collect::<String>();
        assert_eq!(iter.next(), Some('\n'));

        if n == "-1" {
            return Self::NullBulkStrings;
        }

        let n = n.parse::<usize>().unwrap();
        let s = iter.take_while(|&x| x != '\r').collect::<String>();
        assert_eq!(iter.next(), Some('\n'));
        assert_eq!(s.len(), n);
        Self::BulkStrings(s)
    }

    fn parse_array(iter: &mut impl Iterator<Item = char>) -> Self {
        // '*' is already consumed
        let n = iter
            .take_while(|&x| x != '\r')
            .collect::<String>()
            .parse::<usize>()
            .unwrap();
        assert_eq!(iter.next(), Some('\n'));

        let mut array = vec![];
        for _ in 0..n {
            array.push(Self::parse(iter));
        }
        Self::Array(array)
    }

    pub fn simple_string(s: &str) -> Self {
        Self::SimpleString(s.to_string())
    }

    pub fn bulk_strings(s: &str) -> Self {
        Self::BulkStrings(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        assert_eq!(
            RESP::SimpleString("OK".to_string()).to_string(),
            "+OK\r\n".to_string()
        );
        assert_eq!(
            RESP::BulkStrings("value".to_string()).to_string(),
            "$5\r\nvalue\r\n".to_string()
        );
        assert_eq!(RESP::NullBulkStrings.to_string(), "$-1\r\n".to_string());
        assert_eq!(
            RESP::Array(vec![
                RESP::SimpleString("OK".to_string()),
                RESP::BulkStrings("value".to_string())
            ])
            .to_string(),
            "*2\r\n+OK\r\n$5\r\nvalue\r\n".to_string()
        );
        assert_eq!(
            RESP::Rdb(vec![0x01, 0x02, 0x03]).to_string(),
            "$3\r\n\x01\x02\x03\r\n".to_string()
        );
    }

    #[test]
    fn test_from_bytes() {
        assert_eq!(
            RESP::from_bytes(b"+OK\r\n"),
            RESP::SimpleString("OK".to_string())
        );
        assert_eq!(
            RESP::from_bytes(b"$5\r\nvalue\r\n"),
            RESP::BulkStrings("value".to_string())
        );
        assert_eq!(RESP::from_bytes(b"$-1\r\n"), RESP::NullBulkStrings);
        assert_eq!(
            RESP::from_bytes(b"*2\r\n+OK\r\n$5\r\nvalue\r\n"),
            RESP::Array(vec![
                RESP::SimpleString("OK".to_string()),
                RESP::BulkStrings("value".to_string())
            ])
        );
    }
}

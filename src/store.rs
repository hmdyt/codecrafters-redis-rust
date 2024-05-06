use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time;

lazy_static! {
    static ref STORE: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());
}

#[derive(Clone)]
struct Value {
    value: String,
    expires_at: Option<u128>,
}

pub fn set(key: &str, value: &str, px: Option<u128>) {
    let expires_at = match px {
        Some(px) => Some(now() + px),
        None => None,
    };
    let value = Value {
        value: value.to_string(),
        expires_at,
    };
    STORE.lock().unwrap().insert(key.to_string(), value);
}

pub fn get(key: &str) -> Option<String> {
    match STORE.lock().unwrap().get(key) {
        Some(value) => {
            if let Some(expires_at) = value.expires_at {
                if expires_at < now() {
                    return None;
                }
            }
            Some(value.value.clone())
        }
        None => None,
    }
}

// TODO: テスタブルな形にする
fn now() -> u128 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        set("key1", "value1", None);
        assert_eq!(get("key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_set_get_expired() {
        set("key2", "value2", Some(1000000000));
        assert_eq!(get("key2"), Some("value2".to_string()));
        set("key3", "value3", Some(0));
        // sleep
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(get("key3"), None);
    }
}

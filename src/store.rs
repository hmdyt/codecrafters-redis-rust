use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref STORE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub fn set(key: &str, value: &str) {
    STORE
        .lock()
        .unwrap()
        .insert(key.to_string(), value.to_string());
}

pub fn get(key: &str) -> Option<String> {
    STORE.lock().unwrap().get(key).cloned()
}

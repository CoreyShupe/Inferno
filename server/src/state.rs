use packets::value::ValueType;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Default, Clone)]
pub struct State {
    map: Arc<RwLock<HashMap<String, Arc<RwLock<ValueType>>>>>,
}

impl State {
    pub async fn get(&self, key: &str) -> Option<ValueType> {
        let read = self.map.read().expect("Failed to read global state");
        let value = read.get(key)?.clone();
        drop(read);
        let extracted = value.read().expect("Failed to read value");
        Some(extracted.clone())
    }

    pub async fn set(&self, key: &str, value: ValueType) {
        if matches!(value, ValueType::None) {
            self.del(key).await;
            return;
        }

        let mut write = self.map.write().expect("Failed to write global state");
        write.insert(key.to_string(), Arc::new(RwLock::new(value)));
    }

    pub async fn del(&self, key: &str) -> Option<ValueType> {
        let mut write = self.map.write().expect("Failed to write global state");
        let value = write.remove(key)?;
        drop(write);
        let extracted = value.read().expect("Failed to read value");
        Some(extracted.clone())
    }
}

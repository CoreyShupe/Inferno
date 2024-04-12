use crate::state::CompositeValue;
use dashmap::DashMap;
use packets::value::ValueType;
use std::sync::Arc;

pub trait Container {
    fn ct_get(&self, key: &str) -> Option<ValueType>;

    fn ct_insert(&self, key: &str, value: ValueType) -> Option<ValueType>;

    fn ct_remove(&self, key: &str) -> Option<ValueType>;
}

impl Container for Arc<DashMap<String, ValueType>> {
    fn ct_get(&self, key: &str) -> Option<ValueType> {
        DashMap::get(self, key).map(|value| value.value().clone())
    }

    fn ct_insert(&self, key: &str, value: ValueType) -> Option<ValueType> {
        DashMap::insert(self, key.to_string(), value).map(|value| value.clone())
    }

    fn ct_remove(&self, key: &str) -> Option<ValueType> {
        DashMap::remove(self, key).map(|(_, value)| value.clone())
    }
}

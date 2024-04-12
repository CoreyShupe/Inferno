use bztree::BzTree;
use dashmap::{DashMap, DashSet};
use errors::{Result, StateError};
use packets::{ClientCommandExecutor, ServerResponse};
use std::ops::Neg;
use std::sync::Arc;

use crate::data::list::ArcSwapLinkedList;
use packets::value::ValueType;

#[derive(Default, Clone)]
pub struct State {
    map: Arc<DashMap<String, CompositeValue>>,
}

impl ClientCommandExecutor for &State {
    async fn expire(self, key: String, expire: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn persist(self, key: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn ttl(self, key: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn del(self, keys: Vec<String>) -> Result<ServerResponse> {
        for key in keys {
            self.map.remove(&key);
        }
        Ok(ServerResponse::Ok)
    }

    async fn decr(self, key: String) -> Result<ServerResponse> {
        self.decr_by(key, 1).await
    }

    async fn decr_by(self, key: String, by: u32) -> Result<ServerResponse> {
        if by > i32::MAX as u32 {
            // todo: fix error name
            Err(StateError::BadState)?;
        }
        let by = by as i32;
        let value = self.map.get_mut(&key);
        let Some(mut value) = value else {
            self.map
                .insert(key, CompositeValue::Value(ValueType::Int(by.neg())));
            return Ok(ServerResponse::Single {
                value: ValueType::Int(1),
            });
        };

        match value.value().value() {
            Ok(ValueType::Int(v)) => {
                if v < i32::MIN + by {
                    // todo: fix error name
                    Err(StateError::BadState)?;
                }

                *value.value_mut() = CompositeValue::Value(ValueType::Int(v - by));
                Ok(ServerResponse::Single {
                    value: ValueType::Int(v - by),
                })
            }
            _ => Err(StateError::BadKeyType)?,
        }
    }

    async fn incr(self, key: String) -> Result<ServerResponse> {
        self.incr_by(key, 1).await
    }

    async fn incr_by(self, key: String, by: u32) -> Result<ServerResponse> {
        if by > i32::MAX as u32 {
            // todo: fix error name
            Err(StateError::BadState)?;
        }
        let by = by as i32;
        let value = self.map.get_mut(&key);
        let Some(mut value) = value else {
            self.map
                .insert(key, CompositeValue::Value(ValueType::Int(by)));
            return Ok(ServerResponse::Single {
                value: ValueType::Int(1),
            });
        };

        match value.value().value() {
            Ok(ValueType::Int(v)) => {
                if v > i32::MAX - by {
                    // todo: fix error name
                    Err(StateError::BadState)?;
                }

                *value.value_mut() = CompositeValue::Value(ValueType::Int(v + by));
                Ok(ServerResponse::Single {
                    value: ValueType::Int(v + by),
                })
            }
            _ => Err(StateError::BadKeyType)?,
        }
    }

    async fn get(self, key: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn get_del(self, keys: Vec<String>) -> Result<ServerResponse> {
        let mut response = Vec::new();
        for key in keys {
            let opt_val = self.map.remove(&key);
            let Some(value) = opt_val.and_then(|(_, value)| value.value().ok()) else {
                continue;
            };
            response.push(value);
        }

        Ok(ServerResponse::Bulk { values: response })
    }

    async fn get_ex(self, key: String, expire: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn get_set(self, key: String, value: ValueType) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn mget(self, keys: Vec<String>) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn set(self, key: String, value: ValueType) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn set_ex(self, key: String, value: ValueType, expire: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn set_nx(self, key: String, value: ValueType) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn mset(self, keys: Vec<String>, values: Vec<ValueType>) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn mset_nx(self, keys: Vec<String>, values: Vec<ValueType>) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hexpire(self, key: String, field: String, expire: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hdel(self, key: String, fields: Vec<String>) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hdel_get(self, key: String, fields: Vec<String>) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hpop_rand(self, key: String, count: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hexists(self, key: String, field: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hget(self, key: String, field: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hget_all(self, key: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hmget(self, key: String, fields: Vec<String>) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hkeys(self, key: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hvalues(self, key: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hlen(self, key: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hdecr(self, key: String, field: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hdecr_by(self, key: String, field: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hincr(self, key: String, field: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hincr_by(self, key: String, field: String, by: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hset(self, key: String, field: String, value: ValueType) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hset_nx(self, key: String, field: String, value: ValueType) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hset_ex(
        self,
        key: String,
        field: String,
        value: ValueType,
        expire: u32,
    ) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hmset(self, key: String, fields: Vec<(String, ValueType)>) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn hmset_nx(
        self,
        key: String,
        fields: Vec<(String, ValueType)>,
    ) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn zadd(self, key: String, score: u32, member: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn zadd_nx(self, key: String, score: u32, member: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn zincr_by(self, key: String, score: u32, member: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn zdecr_by(self, key: String, score: u32, member: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn zscore(self, key: String, member: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn zmscore(self, key: String, members: Vec<String>) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn zpop_min(self, key: String, count: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn zpop_max(self, key: String, count: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn zrem(self, key: String, member: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn zexpire(self, key: String, member: String, expire: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn llpush(self, key: String, value: ValueType) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn llpush_nx(self, key: String, value: ValueType) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn llpush_ex(self, key: String, value: ValueType, expire: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn lrpush(self, key: String, value: ValueType) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn lrpush_nx(self, key: String, value: ValueType) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn lrpush_ex(self, key: String, value: ValueType, expire: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn lexpire(self, key: String, index: u32, expire: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn llpop(self, key: String, count: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn lrpop(self, key: String, count: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn lrange(self, key: String, start: u32, end: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn sadd(self, key: String, members: Vec<String>) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn sadd_nx(self, key: String, members: Vec<String>) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn sadd_ex(self, key: String, member: String, expire: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn smember(self, key: String, member: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn smembers(self, key: String) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn sexpire(self, key: String, member: String, expire: u32) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn srem(self, key: String, members: Vec<String>) -> Result<ServerResponse> {
        unimplemented!()
    }

    async fn spop(self, key: String, count: u32) -> Result<ServerResponse> {
        unimplemented!()
    }
}

#[derive(Clone)]
pub enum CompositeValue {
    Value(ValueType),
    List(Arc<ArcSwapLinkedList<ValueType>>),
    Set(Arc<DashSet<ValueType>>),
    Map(Arc<DashMap<String, ValueType>>),
    OrdSet(Arc<BzTree<String, isize>>),
}

impl CompositeValue {
    pub fn value(&self) -> Result<ValueType> {
        match self {
            CompositeValue::Value(value) => Ok(value.clone()),
            _ => Err(StateError::BadKeyType)?,
        }
    }

    pub fn list(&self) -> Result<Arc<ArcSwapLinkedList<ValueType>>> {
        match self {
            CompositeValue::List(list) => Ok(list.clone()),
            _ => Err(StateError::BadKeyType)?,
        }
    }
    pub fn set(&self) -> Result<Arc<DashSet<ValueType>>> {
        match self {
            CompositeValue::Set(set) => Ok(set.clone()),
            _ => Err(StateError::BadKeyType)?,
        }
    }

    pub fn map(&self) -> Result<Arc<DashMap<String, ValueType>>> {
        match self {
            CompositeValue::Map(map) => Ok(map.clone()),
            _ => Err(StateError::BadKeyType)?,
        }
    }

    pub fn ord_set(&self) -> Result<Arc<BzTree<String, isize>>> {
        match self {
            CompositeValue::OrdSet(ord_set) => Ok(ord_set.clone()),
            _ => Err(StateError::BadKeyType)?,
        }
    }
}

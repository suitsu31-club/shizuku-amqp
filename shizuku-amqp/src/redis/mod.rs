use kanau::message::{MessageDe, MessageSer};
use thiserror::Error;

/// Trait for redis key that is indexed by a static key
pub trait StaticKeyIndexedValue {
    /// Get the key
    fn static_key() -> String;
}

/// Trait for redis key that is indexed by a dynamic key
pub trait KeyValue {
    /// Key type
    type Key: std::fmt::Display + Send + Sync + Sized;
    
    /// Value type
    type Value: Send + Sync + Sized;
    
    /// Get the key
    fn key(&self) -> &Self::Key;
    
    /// Get the value
    fn value(&self) -> &Self::Value;
    
    /// Get the value
    fn into_value(self) -> Self::Value;
    
    /// Create a new key value pair
    fn new(key: Self::Key, value: Self::Value) -> Self;
    
    fn delete(
        &self,
        con: &mut redis::aio::ConnectionManager,
        key: &Self::Key,
    ) -> impl Future<Output = Result<(), redis::RedisError>> + Send {
        async move {
            
        }
    }
}

pub trait KeyValueRead: KeyValue
    where Self::Value: MessageDe
{
    fn read(
        con: &mut redis::aio::ConnectionManager,
        key: &Self::Key,
    ) -> impl Future<Output = Result<Self::Value, KvReadError<Self::Value>>> + Send {
        async move {
        }
    }
}

#[derive(Debug, Error)]
pub enum KvReadError<V: MessageDe> {
    #[error("Failed to deserialize: {0}")]
    DeserializeError(V::DeError),
    #[error("Failed to read from redis: {0}")]
    RedisError(#[from] redis::RedisError),
}

pub trait KeyValueWrite: KeyValue
    where Self::Value: MessageSer
{
    fn write(
        &self,
        con: &mut redis::aio::ConnectionManager,
    ) -> impl Future<Output = Result<(), KvWriteError<Self::Value>>> + Send {
        async move {
        }
    }
}

#[derive(Debug, Error)]
pub enum KvWriteError<V: MessageSer> {
    #[error("Failed to serialize: {0}")]
    SerializeError(V::SerError),
    #[error("Failed to write to redis: {0}")]
    RedisError(#[from] redis::RedisError),
}
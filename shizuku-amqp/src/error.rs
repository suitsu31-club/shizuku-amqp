use thiserror::Error;

#[derive(Debug, Error)]
/// internal errors
pub enum Error {
    #[error("{0}")]
    /// Serialize Error
    SerializeError(#[from] kanau::message::SerializeError),
    
    /// AMQP Error
    #[error("{0}")]
    AmqpError(#[from] amqprs::error::Error),
    
    #[error("{0}")]
    /// Deserialize Error
    DeserializeError(#[from] kanau::message::DeserializeError),
    
    #[error("Business Login Error: {0}")]
    /// Error occurred in business logic. This kind of business error can be solved by retrying.
    BusinessError(anyhow::Error),
    
    #[error("Business Panic Error: {0}")]
    /// Error occurred in business logic. This kind of business error can not be solved by retrying.
    BusinessPanicError(anyhow::Error),
    
    #[error("Panic Error: {0}")]
    /// Panic in tokio spawn
    TokioPanicError(#[from] tokio::task::JoinError),
}
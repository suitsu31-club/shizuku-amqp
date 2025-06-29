use kanau::message::{MessageDe, MessageSer};
use kanau::processor::Processor;
use std::marker::PhantomData;
use std::pin::Pin;
use amqprs::channel::{BasicAckArguments, Channel};
use amqprs::consumer::AsyncConsumer;
use amqprs::{BasicProperties, Deliver};
use crate::error::{Error, MetaDecodeError};
use crate::rabbitmq::ack;

/// Trait for sending message to rabbitmq
pub trait AmqpMessageSend: MessageSer + Send + Sized + AmqpRouting {
    /// Message meta type
    type Meta: AmqpMessageMeta + Send + Sync;

    /// Send message to rabbitmq
    fn send(
        self,
        channel: &amqprs::channel::Channel,
        props: Self::Meta,
    ) -> impl Future<Output = Result<(), crate::error::Error>> + Send {
        async {
            let bytes = self.to_bytes().map_err(|e| e.into())?;
            channel
                .basic_publish(
                    props.into_meta(),
                    bytes.into_vec(),
                    amqprs::channel::BasicPublishArguments::new(Self::EXCHANGE, Self::ROUTING_KEY),
                )
                .await?;
            Ok(())
        }
    }
}

/// Trait for message meta
pub trait AmqpMessageMeta: Sized {
    /// Convert to amqp meta
    fn into_meta(self) -> BasicProperties;

    /// Try to decode from amqp meta
    fn try_from_meta(meta: &BasicProperties) -> Result<Self, MetaDecodeError>;
}

impl AmqpMessageMeta for BasicProperties {
    fn into_meta(self) -> BasicProperties {
        self
    }

    fn try_from_meta(meta: &BasicProperties) -> Result<Self, MetaDecodeError> {
        Ok(meta.clone())
    }
}

impl AmqpMessageMeta for () {
    fn into_meta(self) -> BasicProperties {
        BasicProperties::default()
    }

    fn try_from_meta(_meta: &BasicProperties) -> Result<Self, MetaDecodeError> {
        Ok(())
    }
}

/// Trait for routing message to rabbitmq
pub trait AmqpRouting {
    /// Exchange name
    const EXCHANGE: &'static str;

    /// Routing key
    const ROUTING_KEY: &'static str;
}

/// Trait for consuming message from rabbitmq
pub trait AmqpMessageProcessor<Message: AmqpMessageSend + MessageDe>:
    Processor<(Message, Message::Meta), Result<(), crate::error::Error>>
{
}

/// Consumer for rabbitmq
pub struct AmqpMessageConsumer<
    Message: AmqpMessageSend + MessageDe,
    Inner: AmqpMessageProcessor<Message>,
> {
    inner: Inner,
    _marker: PhantomData<Message>,
}

impl<Message: AmqpMessageSend + MessageDe, Inner: AmqpMessageProcessor<Message>>
    AmqpMessageConsumer<Message, Inner>
{
    /// Create a new consumer
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
    
    /// Process message
    pub async fn on_message(&self, prop: BasicProperties, content: Vec<u8>) -> Result<(), crate::error::Error> {
        let decoded_message = Message::from_bytes(&content).map_err(|e| e.into())?;
        let meta = Message::Meta::try_from_meta(&prop)?;
        self.inner.process((decoded_message, meta)).await
    }
}

impl<M,I> AsyncConsumer for AmqpMessageConsumer<M,I>
    where
        M: AmqpMessageSend + MessageDe + Send + Sync,
        I: AmqpMessageProcessor<M> + Send + Sync,
        M::Meta: AmqpMessageMeta,
        M::DeError: Send,
{
    fn consume<'life0, 'life1, 'async_trait>(
        &'life0 mut self,
        channel: &'life1 Channel,
        deliver: Deliver,
        basic_properties: BasicProperties,
        content: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where Self: 'async_trait,
          'life0: 'async_trait,
          'life1: 'async_trait,
    {
        Box::pin(async move {
            match self.on_message(basic_properties, content).await {
                Ok(_) => {
                    ack(channel, BasicAckArguments::new(deliver.delivery_tag(), false), 5).await;
                }
                Err(Error::BusinessError(e)) => {
                    tracing::error!("Business error: {}", e);
                }
                Err(e) => {
                    tracing::error!("Error: {}", e);
                    ack(channel, BasicAckArguments::new(deliver.delivery_tag(), false), 5).await;
                }
            }
        })
    }
}


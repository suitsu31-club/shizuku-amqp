use kanau::message::MessageSer;

/// Trait for sending message to rabbitmq
pub trait AmqpMessageSend: MessageSer + Send + Sized + AmqpRouting {
    /// Send message to rabbitmq
    fn send<P: AmqpMessageMeta + Send + Sync>(
        self, 
        channel: &amqprs::channel::Channel,
        props: P,
    ) -> impl Future<Output = Result<(), crate::error::Error>> + Send {
        async {
            let bytes = self.to_bytes().map_err(|e| e.into())?;
            channel.basic_publish(
                props.into_meta(),
                bytes.into_vec(),
                amqprs::channel::BasicPublishArguments::new(
                    Self::EXCHANGE,
                    Self::ROUTING_KEY,
                ),
            ).await?;
            Ok(())
        }
    }
}

/// Trait for message meta
pub trait AmqpMessageMeta {
    /// Convert to amqp meta
    fn into_meta(self) -> amqprs::BasicProperties;
}

impl AmqpMessageMeta for amqprs::BasicProperties {
    fn into_meta(self) -> amqprs::BasicProperties {
        self
    }
}

impl AmqpMessageMeta for () {
    fn into_meta(self) -> amqprs::BasicProperties {
        amqprs::BasicProperties::default()
    }
}

/// Trait for routing message to rabbitmq
pub trait AmqpRouting {
    /// Exchange name
    const EXCHANGE: &'static str;
    
    /// Routing key
    const ROUTING_KEY: &'static str;
}
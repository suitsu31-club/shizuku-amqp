
/// Tools that used to process AMQP messages
pub mod event;

/// Ack message with retry
/// 
/// **Will panic if failed to ack after max retries**
pub async fn ack(
    channel: &amqprs::channel::Channel,
    arg: amqprs::channel::BasicAckArguments,
    max_retries: u32,
) {
    let mut retries = 0;
    while retries < max_retries {
        match channel.basic_ack(arg.clone()).await {
            Ok(_) => return,
            Err(e) => {
                tracing::error!("Failed to ack message: {}", e);
                retries += 1;
            }
        }
    }
    
    #[allow(clippy::panic)]
    if retries == max_retries {
        tracing::error!("Failed to ack message after {} retries", max_retries);
        panic!("Failed to ack message after {} retries", max_retries);
    }
}
use std::sync::mpsc::Receiver;
use std::time::Duration;

use rdkafka::{
    config::ClientConfig,
    error::KafkaError,
    message::{Header, OwnedHeaders, OwnedMessage},
    producer::{FutureProducer, FutureRecord},
};

use chrono::prelude::*;

const OUTPUT_HEADERS: [&str; 1] = ["readied_at"];

// #[tokio::main]
pub async fn publish(
    producer: FutureProducer,
    message: String,
    // receiver: Receiver<String>,
) -> Result<(i32, i64), (KafkaError, OwnedMessage)> {
    let utc: DateTime<Utc> = Utc::now();

    // match receiver.recv().unwrap() {
    //     m => println!("the payload from sender thread {:?}", m),
    //     // Ok(m)=> println!("the payload from sender thread {:?}",m),
    //     // Err(e)=> println!("Error occure while receiving from the channel {:?}",e)
    // }
    producer
        .send(
            FutureRecord::to("outbox.topic")
                .payload(&format!("Message {}", message))
                .key(&format!("Key {}", String::from("test key")))
                .headers(OwnedHeaders::new().insert(Header {
                    key: &String::from("readied_at"),
                    value: Some(&utc.to_string()),
                })),
            Duration::from_secs(0),
        )
        .await
}

pub fn producer() -> FutureProducer {
    //->Result<(), Box<dyn std::error::Error>>  {
    println!("Hello from Producer");
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:29092")
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    return producer;
    // loop {
    // match publish(producer, &message).await {
    //     Ok(m) => println!("successful publish {:?}", m),
    //     Err(e) => {
    //         println!("Kafka error: {:?}", e);
    //     }
    // };
    // }
}

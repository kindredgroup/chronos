use std::time::Duration;

use rdkafka::{
    config::ClientConfig,
    error::KafkaError,
    message::{Header, OwnedHeaders, OwnedMessage},
    producer::{FutureProducer, FutureRecord},
};

use chrono::prelude::*;


async fn publish(
    producer: &FutureProducer,
    message: &String,
) -> Result<(i32, i64), (KafkaError, OwnedMessage)> {
    let utc: DateTime<Utc> = Utc::now();

    producer
        .send(
            FutureRecord::to("test")
                .payload(&format!("Message {}", message))
                .key(&format!("Key {}", String::from("test key")))
                .headers(OwnedHeaders::new().insert(Header {
                    key: "readied_at",
                    value: Some(&utc.to_string()),
                })),
            Duration::from_secs(0),
        )
        .await
}

#[tokio::main]
pub async fn producer(message: String) {
    //->Result<(), Box<dyn std::error::Error>>  {
    println!("Hello from Producer");
    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", "localhost:29092")
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    loop {
        match publish(producer, &message).await {
            Ok(m) => println!("successful publish {:?}", m),
            Err(e) => {
                println!("Kafka error: {:?}", e);
            }
        };
    }
}

use std::time::Duration;

use rdkafka::{
    config::ClientConfig,
    error::KafkaError,
    message::{Header, OwnedHeaders, OwnedMessage},
    producer::{FutureProducer, FutureRecord},
};

use async_trait::async_trait;
use chrono::prelude::*;
use rdkafka::producer::future_producer::OwnedDeliveryResult;

const OUTPUT_HEADERS: [&str; 1] = ["readied_at"];

#[async_trait]
pub trait ProducerMessages {
    async fn produce(m: &str, p: &FutureProducer)
        ->  Result<OwnedDeliveryResult, KafkaError> ;
}

pub struct KafkaPublisher {
    pub(crate) client: FutureProducer,
}

impl KafkaPublisher {
    pub async fn new() -> Self {
        Self { client: producer() }
    }
}

#[async_trait]
impl ProducerMessages for KafkaPublisher {
    async fn produce(
        m: &str,
        p: &FutureProducer,
    ) -> Result<OwnedDeliveryResult, KafkaError> {
        publish(&p, String::from(m)).await
    }
}

// #[tokio::main]
pub async fn publish(
    producer: &FutureProducer,
    message: String,
    // receiver: Receiver<String>,
) -> Result<OwnedDeliveryResult, KafkaError> {
    let utc: DateTime<Utc> = Utc::now();
    println!("publishing message {:?}", message);

    // match receiver.recv().unwrap() {
    //     m => println!("the payload from sender thread {:?}", m),
    //     // Ok(m)=> println!("the payload from sender thread {:?}",m),
    //     // Err(e)=> println!("Error occure while receiving from the channel {:?}",e)
    // }
    let delivery_status = producer
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
        .await;
    Ok(delivery_status)
}

pub fn producer() -> FutureProducer {
    //->Result<(), Box<dyn std::error::Error>>  {
    println!("Hello from Producer");
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9093")
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    return producer;
}

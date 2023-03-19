use std::time::Duration;

use rdkafka::{
    config::ClientConfig,
    error::KafkaError,
    message::{Header, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
};

use async_trait::async_trait;
use chrono::prelude::*;
use rdkafka::producer::future_producer::OwnedDeliveryResult;

const OUTPUT_HEADERS: [&str; 2] = ["chronosID", "chronosDeadline"];

#[async_trait]
pub trait ProducerMessages {
    async fn publish(
        producer: &FutureProducer,
        message: &str,
        headers: &str,
        key: &str
    ) -> Result<OwnedDeliveryResult, KafkaError>;
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
    async fn publish(
        producer: &FutureProducer,
        message: &str,
        headers: &str,
        key: &str
    ) -> Result<OwnedDeliveryResult, KafkaError> {
        let utc: DateTime<Utc> = Utc::now();
        println!("publishing message {:?} headers--{:?}", message, headers);


        let delivery_status = producer
            .send(
                FutureRecord::to("outbox.topic")
                    .payload( message)
                    .key(key)
                    .headers(OwnedHeaders::new()
                            .insert(Header {
                            key: "chronosID",
                            value: Some(headers),
                            })
                            // .insert(Header {
                            //     key: "chronosDeadline",
                            //     value:headers["chronosDeadline"].to_str(),
                            // })
                                    ),
                Duration::from_secs(0),
            )
            .await;
        Ok(delivery_status)
    }
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

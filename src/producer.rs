use std::collections::HashMap;
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
use crate::utils::into_headers;

const OUTPUT_HEADERS: [&str; 2] = ["chronosID", "chronosDeadline"];

#[async_trait]
pub trait ProducerMessages {
    async fn publish(
        &self,
        message: &str,
        headers: &HashMap<String,String>,
        key: &str
    ) -> Result<OwnedDeliveryResult, KafkaError> ;
}

pub struct KafkaPublisher {
    pub(crate) client: FutureProducer,
}

impl KafkaPublisher {
    pub fn new() -> Self {
        Self { client: producer() }
    }
}

#[async_trait]
impl ProducerMessages for KafkaPublisher {
    async fn publish(
        &self,
        message: &str,
        headers: &HashMap<String,String>,
        key: &str
    ) -> Result<OwnedDeliveryResult, KafkaError> {
        let client = &self.client;
        let utc: DateTime<Utc> = Utc::now();
        let o_header = into_headers(&headers);
        println!("headers {:?}",o_header );
        // println!("headers {:?} headers--{:?}", &headers["chronosID)"].to_string(), &headers["chronosDeadline)"].to_string());

        let delivery_status = client
            .send(
                FutureRecord::to("outbox.topic")
                    .payload( message)
                    .key(key)
                    .headers(o_header),
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

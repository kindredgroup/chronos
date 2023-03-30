use std::collections::HashMap;
use std::time::Duration;
use async_trait::async_trait;

use rdkafka::{
    config::ClientConfig,
    error::KafkaError,
    producer::{FutureProducer, FutureRecord},
};

use crate::utils::into_headers;
use chrono::prelude::*;
use rdkafka::producer::future_producer::OwnedDeliveryResult;

#[async_trait]
pub trait MessageProducer {
    async fn publish(
        &self,
        message: String,
        headers: Option<HashMap<String, String>>,
        key: String
    ) -> Result<OwnedDeliveryResult, KafkaError> ;
}

pub struct KafkaPublisher {
    pub(crate) client: FutureProducer,
    pub(crate) topic: String,
}

impl KafkaPublisher {
    pub fn new(topic: String) -> Self {
        Self {
            client: producer(),
            topic,
        }
    }
}

#[async_trait]
impl MessageProducer for KafkaPublisher {

    async fn publish(
        &self,
        message: String,
        headers: Option<HashMap<String, String>>,
        key: String,
    ) -> Result<OwnedDeliveryResult, KafkaError> {
        let client = &self.client;
        let o_header = into_headers(&headers.unwrap());
        // println!("headers {:?}", o_header);
        // println!("headers {:?} headers--{:?}", &headers["chronosId)"].to_string(), &headers["chronosDeadline)"].to_string());

        let delivery_status = client
            .send(
                FutureRecord::to(&self.topic.as_str())
                    .payload(message.as_str())
                    .key(key.as_str())
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
        // .set("security.protocol", "SASL_PLAINTEXT")
        //     ("sasl.mechanisms", "SCRAM-SHA-512"),
        //     ("sasl.username", ""),
        //     ("sasl.password", "")
        .create()
        .expect("Producer creation error");

    return producer;
}

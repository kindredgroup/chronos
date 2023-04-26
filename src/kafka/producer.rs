use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use log::debug;
use rdkafka::producer::{BaseRecord, DefaultProducerContext, FutureProducer, FutureRecord, ThreadedProducer};
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use crate::kafka::errors::KafkaAdapterError;
use crate::utils::util::into_headers;


use super::config::KafkaConfig;

// Kafka Producer
// #[derive(Clone)]
pub struct KafkaProducer {
    producer: FutureProducer,
    topic: String,
}

impl KafkaProducer {
    pub fn new(config: &KafkaConfig) -> Self {
        let producer = config.build_producer_config().create().expect("Failed to create producer");
        let topic = config.out_topic.to_owned();

        Self { producer, topic }
    }
    pub(crate) async fn publish(
        &self,
        message: String,
        headers: Option<HashMap<String, String>>,
        key: String,
    ) -> Result<(), KafkaAdapterError> {
        let o_header = into_headers(&headers.unwrap());
        // println!("headers {:?}", o_header);
        // println!("headers {:?} headers--{:?}", &headers["chronosId)"].to_string(), &headers["chronosDeadline)"].to_string());

        let delivery_status = self.producer
            .send(
                FutureRecord::to(&self.topic.as_str())
                    .payload(message.as_str())
                    .key(key.as_str())
                    .headers(o_header),
                Duration::from_secs(0),
            )
            .await
            .map_err(|(kafka_error, record)| KafkaAdapterError::PublishMessage(kafka_error, "message publishing failed".to_string()))?;
        Ok(())
    }
}


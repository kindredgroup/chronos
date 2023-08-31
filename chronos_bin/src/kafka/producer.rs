use std::collections::HashMap;
use std::time::Duration;

use crate::kafka::errors::KafkaAdapterError;
use crate::utils::util::into_headers;
use rdkafka::producer::{FutureProducer, FutureRecord};

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
        id: String,
    ) -> Result<String, KafkaAdapterError> {
        let o_header = into_headers(&headers.unwrap());
        // println!("headers {:?}", o_header);
        // println!("headers {:?} headers--{:?}", &headers["chronosId)"].to_string(), &headers["chronosDeadline)"].to_string());

        let _delivery_status = self
            .producer
            .send(
                FutureRecord::to(self.topic.as_str())
                    .payload(message.as_str())
                    .key(key.as_str())
                    .headers(o_header),
                Duration::from_secs(0),
            )
            .await
            .map_err(|(kafka_error, _record)| KafkaAdapterError::PublishMessage(kafka_error, "message publishing failed".to_string()))?;
        Ok(id)
    }
}

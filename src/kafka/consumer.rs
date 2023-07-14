use std::{num::TryFromIntError, time::Duration};

use async_trait::async_trait;
use log::{debug, info};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    Message, TopicPartitionList,
};
use rdkafka::message::BorrowedMessage;
use crate::kafka::errors::KafkaAdapterError;
use crate::core::MessageConsumer;

use super::{config::KafkaConfig};

// Kafka Consumer Client
// #[derive(Debug, Clone)]
pub struct KafkaConsumer {
    pub consumer: StreamConsumer,
    pub topic: String,
}

impl KafkaConsumer {
    pub fn new(config: &KafkaConfig) -> Self {
        let consumer = config.build_consumer_config().create().expect("Failed to create consumer");

        let topic = config.in_topic.clone();
        Self {
            consumer,
            topic,
        }
    }

    pub(crate) async fn subscribe(&self) {

        let _ = &self
            .consumer
            .subscribe(&[&self.topic])
            .expect("consumer Subscribe to topic failed");
    }
    pub(crate) async fn consume_message(&self) ->Result<BorrowedMessage, KafkaAdapterError> {

        self.consumer.recv().await.map_err(|e| KafkaAdapterError::ReceiveMessage(e))
    }
}

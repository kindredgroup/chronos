use crate::consumer::{KafkaConsumer, MessageConsumer};
use log::{debug, error, info};
use std::sync::Arc;
// use crate::core::{DataStore, MessageConsumer, MessageProducer};
use crate::message_processor::MessageProcessor;
use crate::message_receiver::MessageReceiver;
use crate::monitor::FailureDetector;
use crate::pg_client::PgDB;
use crate::producer::KafkaPublisher;

pub struct Runner {
    // pub data_store: Box<dyn DataStore>,
    // pub producer: Box<dyn MessageProducer>,
    // pub consumer: Box<dyn MessageConsumer>,
}

// Each thread has it own instance for kafka/db --> FIRST CUT
impl Runner {
    pub async fn run(&self) {
        debug!("Chronos Runner");

        let topics = vec!["input.topic"];

        let kafka_consumer = KafkaConsumer::new(topics, "amn.test.rust".to_string());
        let kafka_producer = KafkaPublisher::new("outbox.topic".to_string());

        let data_store = Arc::new(Box::new(PgDB{}));
        let producer = Arc::new(Box::new(kafka_producer));
        let consumer = Arc::new(Box::new(kafka_consumer));

        let monitor_ds = data_store.clone();

        let processor_producer = producer.clone();
        let processor_ds = data_store.clone();

        let rec_consumer = consumer.clone();
        let rec_producer = producer.clone();
        let rec_ds = data_store.clone();


        let monitor_handler = tokio::task::spawn(async {
            let monitor = FailureDetector {
                data_store: monitor_ds
            };
            monitor.run().await;
        });
        let message_processor_handler = tokio::task::spawn(async {

            let message_processor = MessageProcessor {
                producer: processor_producer,
                data_store: processor_ds
            };
            message_processor.run().await;
        });
        let message_receiver_handler = tokio::task::spawn(async {

            kafka_consumer.subscribe().await;

            let kafka_producer = KafkaPublisher::new("outbox.topic".to_string());

            let message_receiver = MessageReceiver {
                consumer: rec_consumer,
                producer: rec_producer,
                data_store: rec_ds,
            };

            message_receiver.run().await;
        });

        futures::future::join_all([
            monitor_handler,
            message_processor_handler,
            message_receiver_handler,
        ])
            .await;
    }
}

use crate::core::{MessageConsumer, MessageProducer};
use crate::kafka::consumer::KafkaConsumer;
use crate::kafka::producer::KafkaProducer;
use log::{debug, error, info};
use std::sync::Arc;
use std::thread;

use crate::message_processor::MessageProcessor;
use crate::message_receiver::MessageReceiver;
use crate::monitor::FailureDetector;
use crate::postgres::pg::Pg;

pub struct Runner {
    pub consumer: Arc<Box<KafkaConsumer>>,
    pub producer: Arc<Box<KafkaProducer>>,
    pub data_store: Arc<Box<Pg>>,
}

impl Runner {
    pub async fn run(&self) {
        debug!("Chronos Runner");

        let monitor_ds = Arc::clone(&self.data_store);

        let process_ds = Arc::clone(&self.data_store);
        let process_producer = self.producer.clone();

        let receiver_ds = Arc::clone(&self.data_store);
        let receiver_prod = self.producer.clone();
        let receiver_consumer = self.consumer.clone();

        let monitor_handler = tokio::task::spawn(async {
            let monitor = FailureDetector { data_store: monitor_ds };
            monitor.run().await;
        });
        let message_processor_handler = tokio::task::spawn(async {
            let message_processor = MessageProcessor {
                producer: process_producer,
                data_store: process_ds,
            };
            message_processor.run().await;
        });
        let message_receiver_handler = tokio::spawn(async {
            let message_receiver = MessageReceiver {
                consumer: receiver_consumer,
                producer: receiver_prod,
                data_store: receiver_ds,
            };

            message_receiver.run().await;
        });

        futures::future::join_all([monitor_handler, message_processor_handler, message_receiver_handler]).await;
    }
}

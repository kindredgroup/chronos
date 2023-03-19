// use crate::core::{DataStore, MessageConsumer, MessageProducer};
use crate::message_processor::MessageProcessor;
use crate::message_receiver::MessageReceiver;
use crate::monitor::FailureDetector;

pub struct Runner {
    // pub data_store: Box<dyn DataStore>,
    // pub producer: Box<dyn MessageProducer>,
    // pub consumer: Box<dyn MessageConsumer>,
}

// Each thread has it own instance for kafka/db --> FIRST CUT
impl Runner {
    // pub async fn run(&self) {
    pub async fn run(&self) {
        let monitor_handler = tokio::task::spawn(async {
            let monitor = FailureDetector {
                // data_store: self.data_store.clone()
            };
            monitor.run().await;
        });
        let message_processor_handler = tokio::task::spawn(async {
            let message_processor = MessageProcessor {
                // data_store: self.data_store.clone(),
                // producer: self.producer.clone(),
            };
            message_processor.run().await;
        });
        let message_receiver_handler = tokio::task::spawn(async {
            let message_receiver = MessageReceiver {
                // consumer: self.consumer.clone(),
                // producer: self.producer.clone(),
                // data_store: self.data_store.clone()

            };
            message_receiver.run().await;
        });
        // loop {
        futures::future::join_all([
            // monitor_handler,
            // message_processor_handler,
            message_receiver_handler,
        ])
        .await;
        // }
    }
}

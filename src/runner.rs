use crate::consumer:: MessageConsumer;
use log::{debug, error, info};
use std::sync::Arc;

use crate::message_processor::MessageProcessor;
use crate::message_receiver::MessageReceiver;
use crate::monitor::FailureDetector;
use crate::persistence_store::PersistenceStore;
use crate::pg_client::PgDB;
use crate::producer:: MessageProducer;

// pub struct Runner {
//     // pub data_store: Box<dyn DataStore>,
//     // pub producer: Box<dyn MessageProducer>,
//     // pub consumer: Box<dyn MessageConsumer>,
// }

pub struct Runner {
    pub consumer: Arc< Box<dyn MessageConsumer + Sync + Send>>,
    pub producer: Arc<Box<dyn MessageProducer + Sync + Send>>,
    pub data_store: Arc<Box<dyn PersistenceStore + Sync + Send>>,
}

impl Runner {
    pub async fn run(&self) {
        debug!("Chronos Runner");

        let monitor_ds = self.data_store.clone();

        let process_ds = self.data_store.clone();
        let process_producer = self.producer.clone();

        let receiver_ds = self.data_store.clone();
        let receiver_prod = self.producer.clone();
        let receiver_consumer = self.consumer.clone();

        let monitor_handler = tokio::task::spawn(async  {
            let monitor = FailureDetector {
                data_store: monitor_ds
            };
            monitor.run().await;
        });
        let message_processor_handler = tokio::task::spawn(async {

            let message_processor = MessageProcessor {
                producer: process_producer,
                data_store: process_ds,
            };
            message_processor.run().await;
        });
        let message_receiver_handler = tokio::task::spawn(async {

            let message_receiver = MessageReceiver {
                consumer: receiver_consumer,
                producer: receiver_prod,
                data_store: receiver_ds,
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

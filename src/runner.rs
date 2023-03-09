use crate::core::{DataStore, MessageConsumer, MessageProducer};
use crate::message_processor::MessageProcessor;
use crate::message_reciever::MessageReciever;
use crate::monitor::Monitor;


pub struct Runner {
    pub data_store: Box<dyn DataStore>,
    pub producer: Box<dyn MessageProducer> ,
    pub consumer: Box<dyn MessageConsumer>
}

impl Runner {
    pub async fn run(&self) {

        let monitor_handler = tokio::task::spawn(async {
            let monitor = Monitor{ data_store: self.data_store.clone() };
            monitor.run().await;
        });
        let message_processor_handler = tokio::task::spawn(async {
            let message_processor = MessageProcessor{ data_store: self.data_store.clone(), producer: self.producer.clone() };
            message_processor.run().await;
        });
        let message_reciever_handler = tokio::task::spawn(async {
            let message_reciever = MessageReciever{
                consumer: self.consumer.clone(),
                producer: self.producer.clone(),
                data_store: self.data_store.clone()

            };
            message_reciever.run().await;
        });
        // loop {
            futures::future::join_all([monitor_handler, message_processor_handler, message_reciever_handler]).await;
        // }


    }
}



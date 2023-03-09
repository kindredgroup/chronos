use chrono::{DateTime, Utc};
use crate::core::{DataStore, MessageConsumer, MessageProducer};

pub struct MessageReciever {
    pub(crate) consumer: Box<dyn MessageConsumer>,
    pub(crate) data_store: Box<dyn DataStore>,
    pub(crate) producer: Box<dyn MessageProducer>

}

impl MessageReciever {

    pub async fn run(&self) {
        loop {
            if let Ok(message) = self.consumer.consume().await {
                if message.deadline < Utc::now() {
                    self.producer().produce().await.expect("message produce failed");
                } else {
                    self.data_store.insert(message.clone()).await.expect("message insertion failed");
                }
                self.consumer.commit(message.offset).await.expect("commit failed");
            }
        }
    }
}
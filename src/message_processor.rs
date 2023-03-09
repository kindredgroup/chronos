use std::time::Duration;
use chrono::{DateTime, Utc};
use crate::core::{ChronosDeliveryMessage, ChronosMessageStatus, DataStore, MessageProducer};

pub struct MessageProcessor {
    pub(crate) data_store: Box<dyn DataStore>,
    pub(crate) producer: Box<dyn MessageProducer>

}

impl MessageProcessor {

    pub async fn run(&self) {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if let Ok(messages) = self.data_store.get_messages(ChronosMessageStatus::Submitted, None, Some(1)) {
                for message in messages {
                    self.data_store.move_to_ready_state(message).await.expect("mart to ready state failed");
                    self.producer.produce_message(message.clone()).await.expect("produce message failed");
                    self.delete_record(message).await.expect("delete record failed");
                }
            }
        }
    }
}
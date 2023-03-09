use std::time::Duration;
use chrono::{DateTime, Utc};
use crate::core::{ChronosMessageStatus, DataStore};

pub struct Monitor {
    pub(crate) data_store: Box<dyn DataStore>

}

impl Monitor {

    pub async fn run(&self) {
        loop {
            let _ = tokio::time::sleep(Duration::from_secs(10)).await; // sleep for 10sec
            // find messages in ready status for more than 5 secs
            if let Ok(messages) = self.data_store.get_messages(ChronosMessageStatus::Ready, Utc::now() - Duration::from_secs(5)) {
                for message in messages {
                   self.data_store.move_to_initial_state(message).await.expect("initial state update failed");
                }
            }
        }
    }
}
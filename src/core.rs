use async_trait::async_trait;
use chrono::{DateTime, Utc};

pub enum ChronosError {
    ConsumerError,
    ProducerError,
    DBError,
}

pub enum ChronosMessageStatus {
    Submitted,
    Ready,
}

pub struct ChronosDeliveryMessage {
    pub(crate) deadline: DateTime<Utc>,
    pub(crate) offset: u64,
}

#[async_trait]
pub trait Runner {
    async fn run();
}

#[async_trait]
pub trait MessageConsumer {
    async fn consume(&self) -> Result<ChronosDeliveryMessage, ChronosError>;
    fn subscribe(&self);
    fn unsubscrbe(&self);
    async fn commit(&self, offset: u64);
}

#[async_trait]
pub trait MessageProducer {
    async fn produce(&self, message: ChronosDeliveryMessage) -> Result<(), ChronosError>;
}

#[async_trait]
pub trait DataStore {
    async fn insert(
        &self,
        message: ChronosDeliveryMessage,
    ) -> Result<ChronosDeliveryMessage, ChronosError>;
    async fn delete(
        &self,
        message: ChronosDeliveryMessage,
    ) -> Result<ChronosDeliveryMessage, ChronosError>;
    async fn move_to_initial_state(
        &self,
        message: ChronosDeliveryMessage,
    ) -> Result<ChronosDeliveryMessage, ChronosError>;
    async fn move_to_ready_state(
        &self,
        message: ChronosDeliveryMessage,
    ) -> Result<ChronosDeliveryMessage, ChronosError>;
    async fn get_messages(
        &self,
        status: ChronosMessageStatus,
        date_time: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<ChronosDeliveryMessage>, ChronosError>;
}

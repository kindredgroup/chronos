use std::sync::Arc;
use chronos::runner::Runner;
use env_logger::Env;
use log::{debug, error, info, warn};
use chronos::consumer::{KafkaConsumer, MessageConsumer};
use chronos::producer::{KafkaPublisher, MessageProducer};
use chronos::persistence_store::PersistenceStore;
use chronos::pg_client::PgDB;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::init();

    let topics = vec!["input.topic"];

    let kafka_consumer = KafkaConsumer::new(topics, "amn.test.rust".to_string());
    let kafka_producer = KafkaPublisher::new("outbox.topic".to_string());
    let pg = PgDB{};

    let data_store:Arc<Box<dyn PersistenceStore + Send + Sync >> = Arc::new(Box::new(pg));
    let producer:Arc<Box<dyn MessageProducer + Send + Sync >> = Arc::new(Box::new(kafka_producer));
    let consumer:Arc<Box<dyn MessageConsumer + Send + Sync >> = Arc::new(Box::new(kafka_consumer));


    let r = Runner {
        data_store,
        producer,
        consumer,
    };
    log::error!("starting chronos");
    debug!("debug logs starting chronos");
    info!("info logs starting chronos");

    r.run().await;
}

// struct MyDataStore {
//     data: Vec<ChronosDeliveryMessage>,
// }
//
// impl DataStore for MyDataStore {
//     async fn insert(
//         &self,
//         message: ChronosDeliveryMessage,
//     ) -> Result<ChronosDeliveryMessage, ChronosError> {
//         todo!()
//     }
//
//     async fn delete(
//         &self,
//         message: ChronosDeliveryMessage,
//     ) -> Result<ChronosDeliveryMessage, ChronosError> {
//         todo!()
//     }
//
//     async fn move_to_initial_state(
//         &self,
//         message: ChronosDeliveryMessage,
//     ) -> Result<ChronosDeliveryMessage, ChronosError> {
//         todo!()
//     }
//
//     async fn move_to_ready_state(
//         &self,
//         message: ChronosDeliveryMessage,
//     ) -> Result<ChronosDeliveryMessage, ChronosError> {
//         todo!()
//     }
//
//     async fn get_messages(
//         &self,
//         status: ChronosMessageStatus,
//         date_time: Option<DateTime<Utc>>,
//         limit: Option<u64>,
//     ) -> Result<Vec<ChronosDeliveryMessage>, ChronosError> {
//         todo!()
//     }
// }

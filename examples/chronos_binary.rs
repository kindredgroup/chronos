use std::sync::Arc;
use chronos::runner::Runner;
use env_logger::Env;
use log::{debug, error, info, warn};
use chronos::postgres::pg::Pg;
use chronos::kafka::producer::KafkaProducer;
use chronos::kafka::consumer::KafkaConsumer;
use chronos::kafka::config::KafkaConfig;
use chronos::postgres::config::PgConfig;

#[tokio::main]
async fn main() {
    env_logger::init();
    match dotenvy::dotenv() {
        Ok(path) => println!(".env read successfully from {}", path.display()),
        Err(e) => println!("Could not load .env file: {e}"),
    };

    let kafka_config = KafkaConfig::from_env();
    let pg_config = PgConfig::from_env();

    let kafka_consumer = KafkaConsumer::new(&kafka_config);
    let kafka_producer = KafkaProducer::new(&kafka_config);
    let data_store=Pg::new(pg_config).await.unwrap();



    let r = Runner {
        data_store: Arc::new(Box::new(data_store)),
        producer: Arc::new(Box::new(kafka_producer)),
        consumer: Arc::new(Box::new(kafka_consumer))
    };

    debug!("debug logs starting chronos");


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

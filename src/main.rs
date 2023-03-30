// use chrono::{DateTime, Utc};
// use chronos::core::{ChronosDeliveryMessage, ChronosError, ChronosMessageStatus, DataStore};
use chronos::runner::Runner;
use env_logger::Env;
use log::{debug, error, info, warn};

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::init();
    let r = Runner {
        // data_store: Box::new(MyDataStore { data: Vec::new() }),
        // producer: Box::new(()),
        // consumer: Box::new(()),
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

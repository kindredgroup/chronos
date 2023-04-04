use std::sync::Arc;
use crate::pg_client::{ PgDB};
use chrono::{Duration as chrono_duration, Utc};
use log::{error, info};
use std::time::Duration;
use crate::persistence_store::PersistenceStore;

pub struct FailureDetector {
    pub(crate) data_store: Arc<Box<dyn PersistenceStore + Sync + Send>>,
}

//Needs to accept the poll time
impl FailureDetector {
    pub async fn run(&self) {
        println!("Monitoring On!");
        loop {
            let _ = tokio::time::sleep(Duration::from_secs(10)).await; // sleep for 10sec


            let fetched_rows = &self.data_store.failed_to_fire(
                     Utc::now() + chrono_duration::seconds(100)
                )
                .await;

            let _id_list = &self.data_store.reset_to_init( &fetched_rows);

            //TODO Log the list of id's that failed to fire and were re-sent to the init state

        }
    }
}

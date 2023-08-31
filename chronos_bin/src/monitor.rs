use crate::postgres::pg::Pg;
use crate::utils::config::ChronosConfig;
use chrono::{Duration as chrono_duration, Utc};
use std::sync::Arc;
use std::time::Duration;

pub struct FailureDetector {
    pub(crate) data_store: Arc<Pg>,
}

//Needs to accept the poll time
impl FailureDetector {
    pub async fn run(&self) {
        println!("Monitoring On!");
        loop {
            let _ = tokio::time::sleep(Duration::from_secs(ChronosConfig::from_env().db_poll_interval)).await; // sleep for 10sec

            match &self
                .data_store
                .failed_to_fire(&(Utc::now() - Duration::from_secs(ChronosConfig::from_env().fail_detect_interval)))
                .await
            {
                Ok(fetched_rows) => {
                    if !fetched_rows.is_empty() {
                        if let Err(e) = &self.data_store.reset_to_init(fetched_rows).await {
                            println!("error in monitor reset_to_init {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("error in monitor {}", e);
                }
            }
        }
    }
}

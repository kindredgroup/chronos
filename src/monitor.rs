// use crate::core::{ChronosMessageStatus, DataStore};
use crate::pg_client::{DBOps, GetDelays, PgDB, TableRow};
use chrono::{Duration as chrono_duration, Utc};
use std::time::Duration;
use log::{error, info};
use tokio_postgres::Error;

pub struct FailureDetector {
    // pub(crate) data_store: Box<dyn DataStore>,
}

//Needs to accept the poll time
impl FailureDetector {
    pub async fn run(&self) {
        println!("Monitoring On!");
        loop {
            let _ = tokio::time::sleep(Duration::from_secs(10)).await; // sleep for 10sec
                                                                       // find messages in ready status for more than 5 secs
            let db_config = PgDB {
                connection_config: String::from(
                    "host=localhost user=admin password=admin dbname=chronos_db",
                ),
            };

            let data_store = PgDB::new(&db_config).await;
            let fetched_rows = PgDB::get_delayed(
                &data_store,
                &GetDelays {
                    delay_time: Utc::now() + chrono_duration::seconds(5),
                },
            )
            .await;

            let mut id_list: Vec<&str> = Vec::new();
            for row in &fetched_rows {
                let updated_row = TableRow {
                    id: row.get("id"),
                    deadline: row.get("deadline"),
                    readied_at: row.get("readied_at"),
                    readied_by: row.get("readied_by"),
                    message_headers: row.get("message_headers"),
                    message_key: row.get("message_key"),
                    message_value: row.get("message_value"),
                };

                println!("logging failed to fire messages {}", updated_row.id);
                id_list.push(updated_row.id);
                match  PgDB::reset_to_init(&data_store, &updated_row.id, 10).await {
                    Ok(m) => info!("successfully reset state to init "),
                    Err(e) => {
                        error!("error occurred while reset to init state {}", &e);
                        println!("error occurred while reset to init state {}", &e);}
                }
            }

        }
    }
}

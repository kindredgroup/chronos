// use crate::core::{ChronosMessageStatus, DataStore, MessageProducer};
use crate::pg_client::{DBOps, GetReady, PgDB, TableRow};
use crate::producer::{KafkaPublisher, ProducerMessages};
use chrono::Utc;
use std::time::Duration;
use uuid::Uuid;

pub struct MessageProcessor {
    // pub(crate) data_store: Box<dyn DataStore>,
    // pub(crate) producer: Box<dyn MessageProducer>,
}

impl MessageProcessor {
    pub async fn run(&self) {
        println!("Processor turned ON!");
        let k_producer = KafkaPublisher::new().await.client;

        let db_config = PgDB {
            connection_config: String::from(
                "host=localhost user=admin password=admin dbname=chronos_db",
            ),
        };
        let data_store = PgDB::new(&db_config).await;

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let deadline = Utc::now();
            let uuid = Uuid::new_v4();

            let param = GetReady {
                readied_at: deadline,
                readied_by: uuid,
                deadline, // Doesnt look
                limit: 3,
                // order: "asc",
            };

            let mut ready_params = Vec::new();
            ready_params.push(param);

            let publish_rows = PgDB::readying_update(&data_store, &ready_params).await;

            println!("Rows Needs Readying:: {:?}", publish_rows.len());

            let mut table_row = Vec::new();
            for row in &publish_rows {
                let updated_row = TableRow {
                    id: row.get("id"),
                    deadline: row.get("deadline"),
                    readied_at: row.get("readied_at"),
                    readied_by: row.get("readied_by"),
                    message_headers: row.get("message_headers"),
                    message_key: row.get("message_key"),
                    message_value: row.get("message_value"),
                };
                // println!("checking the rows {:?}", &updated_row);
                table_row.push(updated_row);
            }

            // TODO: change the capacity once we know the limit of how many pushed records
            let mut ids: Vec<&str> = Vec::with_capacity(table_row.len());

            for value in table_row {
                let mut outcome = "noop";
                let result =
                    KafkaPublisher::produce(&*value.message_value.to_string(), &k_producer).await;
                match result {
                    Ok(m) => {
                        ids.push(value.id);
                        println!("insert success with number changed {:?}", m);
                        outcome = "success";
                    }
                    Err(e) => {
                        println!("publish failed {:?}", e);
                        // failure detection needs to pick
                        outcome = "error"
                    }
                }
            }
            println!("finished the loop for publish");
            PgDB::delete_fired(&data_store, &ids).await;

            println!("after the delete statement");

            // if let Ok(messages) =
            // {
            //     for message in messages {
            //         self.data_store
            //             .move_to_ready_state(message)
            //             .await
            //             .expect("mart to ready state failed");
            //         self.producer
            //             .produce_message(message.clone())
            //             .await
            //             .expect("produce message failed");
            //         self.delete_record(message)
            //             .await
            //             .expect("delete record failed");
            //     }
            // }
        }
    }
}

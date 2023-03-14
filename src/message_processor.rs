// use crate::core::{ChronosMessageStatus, DataStore, MessageProducer};
use crate::pg_client::{DBOps, GetReady, PgDB};
use crate::producer::{KafkaPublisher, ProducerMessages};
use chrono::{ Utc};
use std::time::Duration;
use uuid::Uuid;

pub struct MessageProcessor {
    // pub(crate) data_store: Box<dyn DataStore>,
    // pub(crate) producer: Box<dyn MessageProducer>,
}

impl MessageProcessor {
    pub async fn run(&self) {
        let k_producer = KafkaPublisher::new().await.client;

        let db_config = PgDB {
            connection_config: String::from(
                "host=localhost user=admin password=admin dbname=chronos_db",
            ),
        };
        let data_store = PgDB::new(&db_config).await;

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let uuid = Uuid::new_v4();
            let deadline = Utc::now();
            /*
             TODO:
             run pipeline for fetching produce_message list - Done
              insert to be ready list
              delete published
             */
            let param = GetReady {
                readied_by: uuid,
                deadline,
                limit: 1,
                order: "asc",
            };

            let mut ready_params = Vec::new();
            ready_params.push(param);

            let Ok(publish_rows) = PgDB::readying_update(&data_store, ready_params).await;

            // TODO: change the capacity once we know the limit of how many pushed records
            let mut ids= Vec::with_capacity(publish_rows.len());
            for value in publish_rows {
                if value.readied_at.to_string().is_empty() {
                    let mut outcome = "noop";
                    let result =
                        KafkaPublisher::produce(&*value.message_value.to_string(), &k_producer)
                            .await;
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
            }
            PgDB::delete_fired(&data_store, &ids);

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

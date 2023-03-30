use std::collections::HashMap;
use std::sync::Arc;

use crate::pg_client::{ GetReady, PgDB, TableRow};
use crate::producer::{KafkaPublisher, MessageProducer};
use chrono::Utc;
use std::time::Duration;
use uuid::Uuid;
use crate::persistence_store::PersistenceStore;

pub struct MessageProcessor {
    pub(crate) data_store: Arc<Box<dyn PersistenceStore  + Sync + Send>>,
    pub(crate) producer: Arc<Box<dyn MessageProducer + Sync + Send>>,
}

impl MessageProcessor {
    pub async fn run(&self) {
        println!("Processor turned ON!");
        let kafka_producer = KafkaPublisher::new("outbox.topic".to_string());


        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            //TODO: fine tune this 1sec duration
            let deadline = Utc::now() + chrono::Duration::seconds(1);
            let uuid = Uuid::new_v4();

            let param = GetReady {
                readied_at: deadline,
                readied_by: uuid,
                deadline,
                limit: 10,
                // order: "asc",
            };

            let mut ready_params = Vec::new();
            ready_params.push(param);

            let publish_rows = &self.data_store.ready_to_fire( &ready_params).await;

            println!(
                "Rows Needs Readying:: {:?} @ {:?}",
                publish_rows.len(),
                Utc::now()
            );

            let mut ids: Vec<&str> = Vec::new();

            for row in publish_rows {
                let updated_row = TableRow {
                    id: row.get("id"),
                    deadline: row.get("deadline"),
                    readied_at: row.get("readied_at"),
                    readied_by: row.get("readied_by"),
                    message_headers: row.get("message_headers"),
                    message_key: row.get("message_key"),
                    message_value: row.get("message_value"),
                };

                let headers: HashMap<String, String> =
                    match serde_json::from_str(&updated_row.message_headers.to_string()) {
                        Ok(T) => T,
                        Err(E) => {
                            println!("error occurred while parsing");
                            HashMap::new()
                        }
                    };
                //TODO: handle empty headers
                // println!("checking {:?}",headers);

                let result = kafka_producer
                    .publish(
                        updated_row.message_value.to_string(),
                        Some(headers),
                        updated_row.message_key.to_string(),
                    )
                    .await;
                match result {
                    Ok(m) => {
                        ids.push(&updated_row.id);
                        println!(
                            "insert success with number changed {:?} @{:?}",
                            m,
                            Utc::now()
                        );
                    }
                    Err(e) => {
                        println!("publish failed {:?}", e);
                        // failure detection needs to pick
                    }
                }
            }

            println!("finished the loop for publish now delete published from DB");
            if ids.len() > 0 {
                let outcome = &self.data_store.delete_fired( &ids.join(",")).await;
                println!("delete fired id {:?} @{:?} outcome {outcome}", &ids, Utc::now());
            } else {
                println!("no more processing {}", Utc::now().to_rfc3339());
            }

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

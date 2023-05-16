use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use chrono::Utc;
use std::time::Duration;
use uuid::Uuid;
use crate::kafka::producer::KafkaProducer;
use tokio_postgres::{Row};
use serde_json::{json, Value};
use crate::postgres::pg::{GetReady, Pg, TableRow};
use crate::postgres::config::PgConfig;
use crate::postgres::errors::PgError;


pub struct MessageProcessor {
    pub(crate) data_store: Arc<Box<Pg>>,
    pub(crate) producer: Arc<Box<KafkaProducer>>,
}

impl MessageProcessor {
    pub async fn run(&self) {
        println!("MessageProcessor ON!");


        loop {
            tokio::time::sleep(Duration::from_millis(10)).await;
            // println!("MessageProcessor");
            let deadline = Utc::now();
            let uuid = Uuid::new_v4();

            let param = GetReady {
                readied_at: deadline,
                readied_by: uuid,
                deadline,
                limit: 1000,
                // order: "asc",
            };

            let mut ready_params = Vec::new();
            ready_params.push(param);

            let publish_rows = &self.data_store.ready_to_fire( &ready_params).await.unwrap();

            // println!(
            //     "Rows Needs Readying:: {:?} @ {:?}",
            //     publish_rows.len(),
            //     Utc::now()
            // );

            let mut ids: Vec<String> = Vec::with_capacity(publish_rows.len());
            let mut publish_futures = Vec::with_capacity(publish_rows.len());
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

               publish_futures.push(self.producer
                    .publish(
                        updated_row.message_value.to_string(),
                        Some(headers),
                        updated_row.message_key.to_string(),
                        updated_row.id.to_string()
                    ))
            }
            let results = futures::future::join_all(publish_futures).await;
            for result in results {
                match result {
                    Ok(m) => {
                        ids.push(m);
                        // println!(
                        //     "insert success with number changed {:?} @{:?}",
                        //     m,
                        //     Utc::now()
                        // );
                    }
                    Err(e) => {
                        println!("publish failed {:?}", e);
                        // failure detection needs to pick
                    }
                }
            }

        //    println!("finished the loop for publish now delete published from DB");
            if ids.len() > 0 {
                let outcome = &self.data_store.delete_fired( &ids).await.unwrap();
               // println!("delete fired id {:?} @{:?} outcome {outcome}", &ids, Utc::now());
            }

            //println!("after the delete statement");

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

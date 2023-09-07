use crate::kafka::producer::KafkaProducer;
use crate::postgres::pg::{GetReady, Pg, TableRow};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub struct MessageProcessor {
    pub(crate) data_store: Arc<Pg>,
    pub(crate) producer: Arc<KafkaProducer>,
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

            match &self.data_store.ready_to_fire(&ready_params).await {
                Ok(publish_rows) => {
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

                        let headers: HashMap<String, String> = match serde_json::from_str(&updated_row.message_headers.to_string()) {
                            Ok(t) => t,
                            Err(_e) => {
                                println!("error occurred while parsing");
                                HashMap::new()
                            }
                        };
                        //TODO: handle empty headers
                        // println!("checking {:?}",headers);

                        publish_futures.push(self.producer.publish(
                            updated_row.message_value.to_string(),
                            Some(headers),
                            updated_row.message_key.to_string(),
                            updated_row.id.to_string(),
                        ))
                    }
                    let results = futures::future::join_all(publish_futures).await;
                    for result in results {
                        match result {
                            Ok(m) => {
                                ids.push(m);
                            }
                            Err(e) => {
                                println!("publish failed {:?}", e);
                                // failure detection needs to pick
                            }
                        }
                    }

                    if !ids.is_empty() {
                        if let Err(outcome_error) = &self.data_store.delete_fired(&ids).await {
                            println!("error occurred in message processor delete_fired {}", outcome_error);
                        }
                    }
                }
                Err(e) => {
                    println!("error occurred in message processor {}", e);
                }
            }
        }
    }
}

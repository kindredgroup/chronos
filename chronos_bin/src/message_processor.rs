use crate::kafka::producer::KafkaProducer;
use crate::postgres::pg::{GetReady, Pg, TableRow};
use crate::utils::delay_controller::DelayController;
use chrono::Utc;
use log::{debug, error, info};
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
        info!("MessageProcessor ON!");

        //Get UUID for the node that deployed this thread
        let node_id: String = std::env::var("NODE_ID").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
        // info!("node_id {}", node_id);
        let mut delay_controller = DelayController::new(100);
        loop {
            tokio::time::sleep(Duration::from_millis(10)).await;
            // tokio::time::sleep(Duration::from_millis(ChronosConfig::from_env().db_poll_interval)).await;
            let deadline = Utc::now();

            let param = GetReady {
                readied_at: deadline,
                readied_by: Uuid::parse_str(&node_id).unwrap(),
                deadline,
                // limit: 1000,
                // order: "asc",
            };

            //retry loop
            loop {
                // thread::sleep(Duration::from_millis(100));
                let max_retry_count = 3;
                let mut retry_count = 0;

                let mut node_id: Option<String> = None;
                // let mut row_id: Option<String> = None;
                match &self.data_store.ready_to_fire(&param).await {
                    Ok(publish_rows) => {
                        let rdy_to_pblsh_count = publish_rows.len();
                        if rdy_to_pblsh_count > 0 {
                            let mut ids: Vec<String> = Vec::with_capacity(rdy_to_pblsh_count);
                            let mut publish_futures = Vec::with_capacity(rdy_to_pblsh_count);
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

                                let mut headers: HashMap<String, String> = match serde_json::from_str(&updated_row.message_headers.to_string()) {
                                    Ok(ser_headers) => ser_headers,
                                    Err(err) => {
                                        error!("error occurred while parsing {}", err);
                                        HashMap::new()
                                    }
                                };

                                node_id = Some(updated_row.readied_by.to_string());
                                // row_id = Some(updated_row.id.to_string());

                                headers.insert("readied_by".to_string(), node_id.unwrap());

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
                                        error!("Error: delayed message publish failed {:?}", e);
                                        break;
                                        // failure detection needs to pick
                                    }
                                }
                            }

                            if !ids.is_empty() {
                                if let Err(outcome_error) = &self.data_store.delete_fired(&ids).await {
                                    println!("Error: error occurred in message processor delete_fired {}", outcome_error);
                                    //add retry logic here
                                }
                                println!("delete ids {:?} and break", ids);
                                break;
                            }
                            debug!("number of rows published successfully and deleted from DB {}", ids.len());
                        } else {
                            debug!("no rows ready to fire for dealine {}", deadline);
                            break;
                        }
                    }
                    Err(e) => {
                        if e.contains("could not serialize access due to concurrent update") && retry_count < max_retry_count {
                            //retry goes here
                            eprintln!("retrying");
                            retry_count += 1;
                            if retry_count == max_retry_count {
                                error!(
                                    "Error: max retry count {} reached by node {} for row ",
                                    max_retry_count,
                                    node_id.unwrap(),
                                    // row_id.unwrap()
                                );
                                break;
                            }
                        }
                        error!("Error: error occurred in message processor while publishing {}", e);
                        break;
                    }
                }
            }
            delay_controller.sleep().await;
        }
    }
}

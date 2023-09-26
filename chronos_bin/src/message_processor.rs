use crate::kafka::producer::KafkaProducer;
use crate::postgres::pg::{GetReady, Pg, TableRow};
use crate::utils::config::ChronosConfig;
use crate::utils::delay_controller::DelayController;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{event, field, info_span, span, trace, Level};
use uuid::Uuid;

pub struct MessageProcessor {
    pub(crate) data_store: Arc<Pg>,
    pub(crate) producer: Arc<KafkaProducer>,
}

impl MessageProcessor {
    pub async fn run(&self) {
        // log::info!("MessageProcessor ON!");
        event!(tracing::Level::INFO, "Chronos Processor On!");

        //Get UUID for the node that deployed this thread
        let node_id: String = std::env::var("NODE_ID").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

        let mut delay_controller = DelayController::new(100);
        loop {
            tokio::time::sleep(Duration::from_millis(ChronosConfig::from_env().processor_db_poll)).await;

            let deadline = Utc::now() - Duration::from_secs(ChronosConfig::from_env().time_advance);

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

                let node_id_option: Option<String> = node_id.clone().into();
                // let mut row_id: Option<String> = None;
                let monitor_span = info_span!("processor_picked", node_id = field::Empty, errors = field::Empty);
                let _ = monitor_span.enter();
                match &self.data_store.ready_to_fire(&param).await {
                    Ok(publish_rows) => {
                        let rdy_to_pblsh_count = publish_rows.len();
                        if rdy_to_pblsh_count > 0 {
                            monitor_span.record("node_id", &node_id);
                            trace!("ready_to_publish_count {}", rdy_to_pblsh_count);
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
                                    Ok(t) => t,
                                    Err(_e) => {
                                        println!("error occurred while parsing");
                                        HashMap::new()
                                    }
                                };
                                //TODO: handle empty headers

                                let readied_by = updated_row.readied_by.to_string();

                                headers.insert("readied_by".to_string(), readied_by);

                                publish_futures.push(self.producer.publish(
                                    updated_row.message_value.to_string(),
                                    Some(headers),
                                    updated_row.message_key.to_string(),
                                    // updated_row.id.to_string(),
                                ))
                            }
                            let publish_kafka = info_span!("publish_kafka", node_id = &node_id, errors = field::Empty, published = field::Empty);
                            let _ = publish_kafka.enter();
                            let results = futures::future::join_all(publish_futures).await;
                            for result in results {
                                match result {
                                    Ok(m) => {
                                        publish_kafka.record("published", "success");
                                        ids.push(m);
                                    }
                                    Err(e) => {
                                        publish_kafka.record("published", "failure");
                                        publish_kafka.record("error", &e.to_string());

                                        log::error!("Error: delayed message publish failed {:?}", e);
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
                                // println!("delete ids {:?} and break", ids);
                                break;
                            }
                            log::debug!("number of rows published successfully and deleted from DB {}", ids.len());
                        } else {
                            log::debug!("no rows ready to fire for dealine {}", deadline);
                            break;
                        }
                    }
                    Err(e) => {
                        if e.contains("could not serialize access due to concurrent update") && retry_count < max_retry_count {
                            //retry goes here
                            eprintln!("retrying");
                            retry_count += 1;
                            if retry_count == max_retry_count {
                                log::error!(
                                    "Error: max retry count {} reached by node {} for row ",
                                    max_retry_count,
                                    node_id_option.unwrap(),
                                    // row_id.unwrap()
                                );
                                break;
                            }
                        }
                        log::error!("Error: error occurred in message processor while publishing {}", e);
                        break;
                    }
                }
            }
            delay_controller.sleep().await;
        }
    }
}

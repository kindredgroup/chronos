use crate::kafka::producer::KafkaProducer;
use crate::postgres::pg::{GetReady, Pg, TableRow};
use crate::utils::config::ChronosConfig;
use crate::utils::delay_controller::DelayController;
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
        //Get UUID for the node that deployed this thread
        let node_id: String = std::env::var("NODE_ID").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

        log::info!("MessageProcessor ON @ node_id: {}", node_id);

        let mut delay_controller = DelayController::new(100);
        loop {
            tokio::time::sleep(Duration::from_millis(ChronosConfig::from_env().processor_db_poll)).await;

            let deadline = Utc::now() - Duration::from_secs(ChronosConfig::from_env().time_advance);

            let params = GetReady {
                readied_at: deadline,
                readied_by: node_id,
                deadline,
                // limit: 1000,
                // order: "asc",
            };

            //retry loop
            let _ = &self.processor_message_ready(&params).await;

            delay_controller.sleep().await;
        }
    }

    #[tracing::instrument(skip_all, fields(node_id, correlationId, is_published, error))]
    async fn processor_message_ready(&self, params: &GetReady) {
        loop {
            let max_retry_count = 3;
            let mut retry_count = 0;

            match &self.data_store.ready_to_fire_db(params).await {
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
                                Ok(t) => t,
                                Err(_e) => {
                                    println!("error occurred while parsing");
                                    HashMap::new()
                                }
                            };
                            //TODO: handle empty headers

                            let readied_by = updated_row.readied_by.to_string();
                            tracing::Span::current().record("node_id", &readied_by);
                            headers.insert("readied_by".to_string(), readied_by);

                            tracing::Span::current().record("correlationId", updated_row.id.to_string());

                            publish_futures.push(self.producer.kafka_publish(
                                updated_row.message_value.to_string(),
                                Some(headers),
                                updated_row.message_key.to_string(),
                                // updated_row.id.to_string(),
                            ))
                        }
                        let results = futures::future::join_all(publish_futures).await;
                        for result in results {
                            match result {
                                Ok(m) => {
                                    tracing::Span::current().record("is_published", "true");
                                    ids.push(m);
                                }
                                Err(e) => {
                                    tracing::Span::current().record("is_published", "false");
                                    tracing::Span::current().record("error", &e.to_string());

                                    log::error!("Error: delayed message publish failed {:?}", e);
                                    break;
                                    // failure detection needs to pick
                                }
                            }
                        }

                        if !ids.is_empty() {
                            if let Err(outcome_error) = &self.data_store.delete_fired_db(&ids).await {
                                println!("Error: error occurred in message processor delete_fired {}", outcome_error);
                                //add retry logic here
                            }
                            // println!("delete ids {:?} and break", ids);
                            break;
                        }
                        log::debug!("number of rows published successfully and deleted from DB {}", ids.len());
                    } else {
                        log::debug!("no rows ready to fire for dealine ");
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
                                "node_id_option.unwrap()",
                                // row_id.unwrap()
                            );
                            break;
                        }
                        // &self.process_db_rows(&params).await;
                    }
                    log::error!("Error: error occurred in message processor while publishing {}", e);
                    break;
                }
            }
        }

        // let node_id_option: Option<String> = node_id.clone().into();
        // let mut row_id: Option<String> = None;
    }

    #[tracing::instrument(skip_all, fields(correlationId))]
    async fn clean_db(&self, ids: Vec<String>) {
        //rety in case delete fails
        let max_retries = 3;
        let mut retry_count = 0;
        loop {
            if retry_count < max_retries {
                match &self.data_store.delete_fired_db(&ids).await {
                    Ok(_) => {
                        tracing::Span::current().record("correlationId", ids.join(","));
                        break;
                    }
                    Err(e) => {
                        println!("Error: error occurred in message processor delete_fired {}", e);
                        retry_count += 1;
                        continue;
                    }
                }
            } else {
                log::error!("Error: max retry count {} reached by node {} for row ", max_retries, "node_id_option.unwrap()",);
                break;
            }
        }
    }
}

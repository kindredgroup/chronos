use chrono::{DateTime, Local};
use rdkafka::Message;
use serde_json::json;
use tracing::instrument;

use crate::kafka::consumer::KafkaConsumer;
use crate::kafka::producer::KafkaProducer;
use crate::postgres::pg::{Pg, TableInsertRow};
use crate::telemetry::metrics::metrics;
use crate::utils::util::{get_message_key, get_payload_utf8, required_headers, CHRONOS_ID, DEADLINE};
use rdkafka::message::BorrowedMessage;
use std::{collections::HashMap, str::FromStr, sync::Arc};

pub struct MessageReceiver {
    pub(crate) consumer: Arc<KafkaConsumer>,
    pub(crate) producer: Arc<KafkaProducer>,
    pub(crate) data_store: Arc<Pg>,
}

impl MessageReceiver {
    #[instrument(skip_all, fields(correlationId))]
    async fn insert_into_db(
        &self,
        new_message: &BorrowedMessage<'_>,
        reqd_headers: HashMap<String, String>,
        message_deadline: DateTime<Local>,
    ) -> Option<String> {
        let max_retry_count = 3;
        let mut retry_count = 0;
        //retry loop
        loop {
            if let Some(payload) = get_payload_utf8(new_message) {
                if let Ok(message_value) = &serde_json::from_slice(payload) {
                    if let Some(message_key) = get_message_key(new_message) {
                        let params = TableInsertRow {
                            id: &reqd_headers[CHRONOS_ID],
                            deadline: message_deadline,
                            message_headers: &json!(&reqd_headers),
                            message_key: message_key.as_str(),
                            message_value,
                        };

                        if let Err(e) = self.data_store.insert_to_delay_db(&params).await {
                            log::error!("insert to delay failed {}", e);
                            retry_count += 1;
                            if retry_count == max_retry_count {
                                return Some("max retry count reached for insert to delay query".to_string());
                            }
                            continue;
                        }
                        tracing::Span::current().record("correlationId", &message_key);
                    }
                    log::debug!("Message publish success {:?}", new_message);
                    return None;
                } else {
                    return Some("json conversion of payload failed".to_string());
                }
            } else {
                return Some("message payload is not utf8 encoded".to_string());
            }
        }
    }

    #[instrument(skip_all, fields(correlationId))]
    async fn prepare_and_publish(&self, message: &BorrowedMessage<'_>, reqd_headers: HashMap<String, String>) -> Option<String> {
        match get_payload_utf8(message) {
            Some(string_payload) => {
                if let Some(message_key) = get_message_key(message) {
                    let string_payload = String::from_utf8_lossy(string_payload).to_string();
                    tracing::Span::current().record("correlationId", &message_key);
                    if let Err(e) = &self.producer.kafka_publish(string_payload, Some(reqd_headers.clone()), message_key).await {
                        return Some(format!("publish failed for received message {:?} with error :: {}", message, e));
                    }
                } else {
                    return Some("message key not found".to_string());
                }
            }
            None => return None,
        };
        None
    }

    #[tracing::instrument(name = "receiver_handle_message", skip_all, fields(correlationId, error))]
    pub async fn handle_message(&self, message: &BorrowedMessage<'_>) {
        // Metrics
        // start instant for safe time recordings w no error handling
        let start_i = std::time::Instant::now();
        // We need the system TS to compare to the kafka timestamp
        let start_ts = chrono::Local::now();
        // Declare but don't set, this helps enumerate all
        // code paths for our recordings
        let dest: metrics::ConsumedMessageDestinations;
        let status: metrics::Status;
        // Check for headers
        match required_headers(message) {
            Some(reqd_headers) => {
                tracing::Span::current().record("correlationId", &reqd_headers[CHRONOS_ID]);
                // Get the deadline header
                let message_deadline = DateTime::<Local>::from_str(&reqd_headers[DEADLINE]);
                match message_deadline {
                    Ok(message_deadline) => {
                        if message_deadline <= start_ts {
                            dest = metrics::ConsumedMessageDestinations::KAFKA;
                            match self.prepare_and_publish(message, reqd_headers).await {
                                Some(err) => {
                                    log::error!("{}", err);
                                    tracing::Span::current().record("error", &err);
                                    status = metrics::Status::ERROR;
                                }
                                None => {
                                    status = metrics::Status::SUCCESS;
                                }
                            }
                        } else {
                            dest = metrics::ConsumedMessageDestinations::DATABASE;
                            match self.insert_into_db(message, reqd_headers, message_deadline).await {
                                Some(err) => {
                                    log::error!("{}", err);
                                    tracing::Span::current().record("error", &err);
                                    status = metrics::Status::ERROR;
                                }
                                None => {
                                    status = metrics::Status::SUCCESS;
                                }
                            };
                        }
                    }
                    Err(e) => {
                        // The user provided a bad time stamp
                        // If we see a TON of em, it could also indicate a bug in our
                        // time parsing or lots of messages with bad timestamps
                        log::warn!(
                            "message receiver: offset {} on partition {} caused time parser error {} ",
                            message.offset(),
                            message.partition(),
                            e
                        );
                        (dest, status) = (metrics::ConsumedMessageDestinations::DROPPED, metrics::Status::SUCCESS);
                    }
                }
            }
            None => {
                log::warn!(
                    "message receiver: required headers not found for offset {} on partition {}",
                    message.offset(),
                    message.partition(),
                );
                (dest, status) = (metrics::ConsumedMessageDestinations::DROPPED, metrics::Status::SUCCESS);
                // This is a success as the producer messed up, not us
            }
        }
        // We use an instant because no error handling
        let dur = std::time::Instant::now().duration_since(start_i);
        metrics::record_consumer_metrics(&start_ts, &dur, message, &dest, &status);
    }

    pub async fn run(&self) {
        log::info!("MessageReceiver ON!");
        let _ = &self.consumer.subscribe().await;
        loop {
            match &self.consumer.kafka_consume_message().await {
                Ok(message) => {
                    self.handle_message(message).await;
                }
                Err(e) => {
                    log::error!("error while consuming message {:?}", e);
                }
            }
        }
    }
}

use chrono::{DateTime, Utc};
use rdkafka::Message;
use serde_json::json;
use tracing::instrument;

use crate::kafka::consumer::KafkaConsumer;
use crate::kafka::producer::KafkaProducer;
use crate::postgres::pg::{Pg, TableInsertRow};
use crate::telemetry::metrics::metrics;
use crate::utils::util::{get_message_key, get_payload_utf8, required_headers, CHRONOS_ID, DEADLINE};

use rdkafka::message::BorrowedMessage;
use std::time::UNIX_EPOCH;
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
        message_deadline: DateTime<Utc>,
    ) -> Option<String> {
        let max_retry_count = 3;
        let mut retry_count = 0;
        //retry loop
        loop {
            if let Some(payload) = get_payload_utf8(new_message) {
                // This is a bug.
                // The README says:
                //
                // The `message_value` field will almost always be JSON in practice
                // but Chronos doesn't attempt to parse its contents — it simply forwards it on.
                // The value may be quite large — beyond the `varchar` limit — hence the use of a `blob`.
                //
                // This attempts to serialize the message body to JSON.
                // This makes Chronos incompatible with any non-json schemas
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
                // This check smells fishy
                // Nothing in the README requires that
                // the message has a key.
                // It only says we need the chronosMessageId and chronosDeadline.
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
        // Must be system time
        let start = std::time::SystemTime::now();
        let dest: metrics::ConsumedMessageDestinations;
        let status: metrics::Status;
        let new_message = &message;
        // Check for headers
        match required_headers(new_message) {
            Some(reqd_headers) => {
                tracing::Span::current().record("correlationId", &reqd_headers[CHRONOS_ID]);
                // Get the deadline
                let message_deadline = DateTime::<Utc>::from_str(&reqd_headers[DEADLINE]);
                match message_deadline {
                    Ok(message_deadline) => {
                        // I think this should also include the timing advance
                        // dl<=Utc::now()+timing
                        // In the worst case, a message will be delayed poll-1ns
                        // Lets use the numbers in the docs to
                        // In the README, the poll interval is set to 100ms, and the
                        // timing advance is set to 50ms.
                        // msg_dl_check: 23:59:59.99
                        // message_deadline: 00:00:00.00
                        // db_check: 00:00:00.00
                        // msg stored: 00:00:00.01
                        // msg_published: 00:00:00.10
                        // We should have sent the message before storing
                        if message_deadline <= Utc::now() {
                            dest = metrics::ConsumedMessageDestinations::KAFKA;
                            match self.prepare_and_publish(new_message, reqd_headers).await {
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
                            match self.insert_into_db(new_message, reqd_headers, message_deadline).await {
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
                        log::warn!("message receiver: time parser error {e}");
                        dest = metrics::ConsumedMessageDestinations::DROPPED;
                        status = metrics::Status::ERROR;
                    }
                }
            }
            None => {
                log::warn!("message receiver: required headers not found");
                dest = metrics::ConsumedMessageDestinations::DROPPED;
                status = metrics::Status::ERROR;
            }
        }
        let end = std::time::SystemTime::now();
        // Get the duration between
        let delta = end.duration_since(start);
        match delta {
            Ok(o) => {
                metrics::record_msg_consume(o.as_secs_f64(), dest, status);
            }
            Err(e) => {
                log::error!("system time error: {e}");
            }
        }
        let msg_ts = new_message.timestamp();
        match msg_ts.to_millis() {
            Some(msg_ts) => match end.duration_since(UNIX_EPOCH) {
                Ok(end) => {
                    let delta_sec = end.as_secs_f64() - (msg_ts / 1000) as f64;
                    metrics::record_msg_consume_latency(delta_sec as f64, new_message.partition());
                }
                Err(e) => log::error!("system time error: {}", e),
            },
            None => {
                log::error!(
                    "no message timestamp for message {} on partition {}",
                    new_message.offset(),
                    new_message.partition()
                );
            }
        }
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

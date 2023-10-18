use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde_json::json;
use tracing::instrument;

use crate::kafka::consumer::KafkaConsumer;
use crate::kafka::producer::KafkaProducer;
use crate::postgres::pg::{Pg, TableInsertRow};
use crate::utils::util::{get_message_key, get_payload_utf8, headers_check, required_headers, CHRONOS_ID, DEADLINE};
use rdkafka::message::{BorrowedMessage, Message};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

pub struct MessageReceiver {
    pub(crate) consumer: Arc<KafkaConsumer>,
    pub(crate) producer: Arc<KafkaProducer>,
    pub(crate) data_store: Arc<Pg>,
}

impl MessageReceiver {
    #[instrument(skip_all, fields(chronos_id))]
    pub async fn receiver_publish_to_kafka(&self, new_message: &BorrowedMessage<'_>, headers: HashMap<String, String>) {
        let string_payload = String::from_utf8_lossy(get_payload_utf8(new_message)).to_string();
        let message_key = get_message_key(new_message);
        tracing::Span::current().record("chronos_id", &message_key);
        let outcome = &self.producer.kafka_publish(string_payload, Some(headers), message_key.to_string()).await;
        match outcome {
            Ok(_) => {
                debug!("Published message to Kafka {}", &message_key);
            }
            Err(e) => {
                error!("Failed to publish message to Kafka: {:?}", e);
                // TODO check if needs to retry publishing
            }
        }
    }

    #[instrument(skip_all, fields(chronos_id))]
    pub async fn receiver_insert_to_db(&self, new_message: &BorrowedMessage<'_>, headers: HashMap<String, String>, deadline: DateTime<Utc>) {
        let result_value = &serde_json::from_slice(get_payload_utf8(new_message));
        let payload = match result_value {
            Ok(payload) => payload,
            Err(e) => {
                error!("de-ser failed for payload: {:?}", e);
                return;
            }
        };

        let message_key = get_message_key(new_message);
        tracing::Span::current().record("chronos_id", &message_key);

        let params = TableInsertRow {
            id: &headers[CHRONOS_ID],
            deadline,
            message_headers: &json!(&headers),
            message_key: message_key.as_str(),
            message_value: payload,
        };
        let _insert_time = Instant::now();

        //retry
        let total_retry_count = 3;
        let mut retry_count = 0;
        loop {
            match self.data_store.insert_to_delay_db(&params).await {
                Ok(_) => {
                    break;
                }
                Err(e) => {
                    error!("insert_to_delay failed: {:?} retrying again", e);
                    retry_count += 1;
                    if retry_count == total_retry_count {
                        error!("max retry count {} exceeded aborting insert_to_db for {}", total_retry_count, message_key);
                        break;
                    }
                }
            }
        }
    }

    #[tracing::instrument(name = "receiver_handle_message", skip_all, fields(chronos_id))]
    pub async fn handle_message(&self, message: &BorrowedMessage<'_>) {
        if headers_check(message.headers().unwrap()) {
            let new_message = &message;

            if let Some(headers) = required_headers(new_message) {
                tracing::Span::current().record("chronos_id", &headers[CHRONOS_ID]);
                let message_deadline: DateTime<Utc> = match DateTime::<Utc>::from_str(&headers[DEADLINE]) {
                    Ok(d) => d,
                    Err(e) => {
                        error!("failed to parse deadline: {}", e);
                        return;
                    }
                };

                if message_deadline <= Utc::now() {
                    debug!("message deadline is in the past, sending directly to out_topic");
                    // direct_sent_count += 1;
                    self.receiver_publish_to_kafka(new_message, headers).await
                } else {
                    debug!("message deadline is in the future, sending to kafka");
                    // db_insert_count += 1;

                    self.receiver_insert_to_db(new_message, headers, message_deadline).await
                    // println!("insert took: {:?}", insert_time.elapsed())
                }
            } else {
                warn!("message with improper headers on inbox.topic ");
            }
        } else {
            warn!("message with improper headers on inbox.topic ");
        }
        //  println!("{direct_sent_count} messages sent directly and {db_insert_count} added to db from total of {total_count} ");
    }

    pub async fn run(&self) {
        info!("MessageReceiver ON!");
        let _ = &self.consumer.subscribe().await;
        // let mut total_count = 0;
        // let mut direct_sent_count = 0;
        // let mut db_insert_count = 0;
        loop {
            if let Ok(message) = &self.consumer.kafka_consume_message().await {
                self.handle_message(message).await;
            }
        }
    }
}

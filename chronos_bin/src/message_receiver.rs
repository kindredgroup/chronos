use chrono::{DateTime, Utc};
use log::warn;
use serde_json::json;

use crate::kafka::consumer::KafkaConsumer;
use crate::kafka::producer::KafkaProducer;
use crate::postgres::pg::{Pg, TableInsertRow};
use crate::utils::util::{get_message_key, get_payload_utf8, headers_check, required_headers, CHRONOS_ID, DEADLINE};
use rdkafka::message::Message;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tracing::{event, field, info_span, trace_span};
use tracing_subscriber::fmt::FormatFields;

pub struct MessageReceiver {
    pub(crate) consumer: Arc<KafkaConsumer>,
    pub(crate) producer: Arc<KafkaProducer>,
    pub(crate) data_store: Arc<Pg>,
}

impl MessageReceiver {
    // pub fn new(consumer: Arc<KafkaConsumer>, producer: Arc<KafkaProducer>, data_store: Arc<Pg>) -> Self {
    //     Self {
    //         consumer,
    //         producer,
    //         data_store,
    //     }
    // }

    // #[tracing::instrument]
    pub async fn run(&self) {
        event!(tracing::Level::INFO, "Chronos Receiver On!");
        let _ = &self.consumer.subscribe().await;
        // for _n in 0..100 {
        let mut total_count = 0;
        let mut direct_sent_count = 0;
        let mut db_insert_count = 0;
        loop {
            if let Ok(message) = &self.consumer.consume_message().await {
                let receiver_span = info_span!(
                    "message_received",
                    errors = field::Empty,
                    message_key = get_message_key(message),
                    flow = field::Empty
                );
                let _ = receiver_span.enter();
                total_count += 1;
                if headers_check(message.headers().unwrap()) {
                    let new_message = &message;
                    let headers = required_headers(new_message).expect("parsing headers failed");
                    let message_deadline: DateTime<Utc> = DateTime::<Utc>::from_str(&headers[DEADLINE]).expect("String date parsing failed");

                    if message_deadline <= Utc::now() {
                        receiver_span.record("flow", "deadline passed, publish message!");
                        direct_sent_count += 1;
                        let string_payload = String::from_utf8_lossy(get_payload_utf8(new_message)).to_string();
                        let message_key = get_message_key(new_message);
                        let _outcome = &self
                            .producer
                            .publish(string_payload, Some(headers), message_key)
                            .await
                            .expect("Publish failed for received message");
                    } else {
                        receiver_span.record("flow", "deadline not passed, insert to db!");
                        db_insert_count += 1;
                        let chronos_message_id = &headers[CHRONOS_ID];

                        let payload = get_payload_utf8(new_message);

                        let message_key = get_message_key(new_message);

                        let params = TableInsertRow {
                            id: chronos_message_id,
                            deadline: message_deadline,
                            message_headers: &json!(&headers),
                            message_key: message_key.as_str(),
                            message_value: &serde_json::from_slice(payload).expect("de-ser failed for payload"),
                        };
                        let _insert_time = Instant::now();
                        self.data_store.insert_to_delay(&params).await.expect("insert to db failed");
                        // println!("insert took: {:?}", insert_time.elapsed())
                    }
                } else {
                    warn!("message with improper headers on inbox.topic ");
                    receiver_span.record("flow", "improper headers, ignore!");
                    //TODO: ignore
                }
                //  println!("{direct_sent_count} messages sent directly and {db_insert_count} added to db from total of {total_count} ");
            }

            // println!("commit received message {:?}", new_message);
            // if let Ok(m) = &kafka_consumer.client{
            //     m.commit_message(&message, CommitMode::Async).expect("commit message failed ");
            // }else{
            //     println!("Error Occurred");
            // }
        }
    }
    // }
}

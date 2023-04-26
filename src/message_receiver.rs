use std::collections::HashMap;

use chrono::{DateTime, Utc};
use log::{error, info, warn};
use serde_json::json;

use std::str::{from_utf8, FromStr};
use std::sync::Arc;
use rdkafka::message::{BorrowedMessage, Message};
use crate::kafka::consumer::KafkaConsumer;
use crate::kafka::producer::KafkaProducer;
use crate::postgres::pg::{Pg, TableInsertRow};
use crate::utils::util::{headers_check, required_headers, CHRONOS_ID, DEADLINE};

pub struct MessageReceiver {
    pub(crate) consumer: Arc< Box<KafkaConsumer>>,
    pub(crate) producer: Arc<Box<KafkaProducer>>,
    pub(crate) data_store: Arc<Box<Pg>>,
}


impl MessageReceiver {
    pub fn new(consumer: Arc< Box<KafkaConsumer>>,
               producer: Arc<Box<KafkaProducer>>,
               data_store: Arc<Box<Pg>>) -> Self {
        Self {
            consumer,
            producer,
            data_store
        }
    }
    pub async fn run(&self) {

        println!("Receiver ON!");
        &self.consumer.subscribe().await;
        // for _n in 0..100 {
        loop {
            if let Ok(message) = &self.consumer.consume_message().await {
                if headers_check(&message.headers().unwrap()) {
                    let new_message = &message;
                    let headers = required_headers(&new_message).expect("parsing headers failed");
                    let message_deadline: DateTime<Utc> =
                        DateTime::<Utc>::from_str(&headers[DEADLINE])
                            .expect("String date parsing failed");

                    if message_deadline <= Utc::now() {
                        //TODO: missing check the DB is the entry is present and mark it readied


                        let payload = new_message
                            .payload_view::<str>()
                            .expect("parsing payload failed")
                            .unwrap()
                            .to_string();
                        let message_key =
                            from_utf8(new_message.key().expect("no message Key found"))
                                .unwrap()
                                .to_string();
                        let _outcome = &self
                            .producer
                            .publish(payload, Some(headers), message_key)
                            .await
                            .expect("Publish failed for received message");
                    } else {
                        let chronos_message_id = &headers[CHRONOS_ID];

                        let payload = new_message
                            .payload_view::<str>()
                            .expect("parsing payload failed")
                            .unwrap()
                            .to_string();
                        let message_key =
                            from_utf8(new_message.key().expect("no message Key found")).unwrap();

                        let params = TableInsertRow {
                            id: &*chronos_message_id,
                            deadline: message_deadline,
                            message_headers: &json!(&headers),
                            message_key,
                            message_value: &json!(&payload),
                        };
                        self.data_store.insert_to_delay(&params).await.expect("TODO: panic message");

                    }
                } else {
                    warn!("message with improper headers on inbox.topic ");
                    //TODO: ignore
                }
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

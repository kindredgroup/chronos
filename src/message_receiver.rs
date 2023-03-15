// use chrono::prelude::*;
use chrono::{DateTime, Utc};
use rdkafka::consumer::{CommitMode, Consumer};
use std::str::{from_utf8, FromStr};

use crate::consumer::ConsumerClient;
use crate::pg_client::{DBOps, PgDB, TableInsertRow, TableRow};
use crate::producer::{KafkaPublisher, ProducerMessages};
use rdkafka::message::{Headers, Message};
use serde_json::json;
// use serde_json::{from_str, Value};

pub struct MessageReceiver {
    // pub(crate) consumer: Box<dyn MessageConsumer>,
    // pub(crate) data_store: Box<dyn DataStore>,
    // pub(crate) producer: Box<dyn MessageProducer>
}

impl MessageReceiver {
    pub async fn run(&self) {
        let topics = vec!["input.topic"];

        let k_consumer = ConsumerClient::new().await.client.unwrap();
        k_consumer.subscribe(&topics).expect("failed to subscribe");

        let k_producer = KafkaPublisher::new().await.client;

        let db_config = PgDB {
            connection_config: String::from(
                "host=localhost user=admin password=admin dbname=chronos_db",
            ),
        };
        let data_store = PgDB::new(&db_config).await;

        let deadline = Utc::now();
        loop {
            if let Ok(message) = k_consumer.recv().await {
                println!("headers :: {:?}", message);
                if let Some(headers) = message.headers() {
                    // let mut deadline_header = None ;
                    if headers.count() == 2 {
                        let mut message_headers = json!({});
                        for index in 0..headers.count() {
                            let key = headers.get(index).key;
                            let value = headers.get(index).value.unwrap();
                            let str_value = from_utf8(value).unwrap();
                            let header_ser = json!({ key: str_value });
                            message_headers = header_ser;
                        }

                        // TODO: iterate headers for deadline header
                        let header_chrono = headers.get(0).value.unwrap();

                        let chronosMessageId = headers.get(1).value.unwrap();
                        let chronosMessageId = from_utf8(chronosMessageId).unwrap();
                        println!("header_chrono header {:?} ", chronosMessageId);
                        let date_string = from_utf8(header_chrono).unwrap();
                        let utc_deadline = DateTime::<Utc>::from_str(date_string).unwrap();

                        println!("deadline header {:?} ", message_headers);

                        let message_key = message.key().expect("no message Key found");
                        let message_key = from_utf8(message_key).unwrap();

                        // if let Some(Ok(payload)) = message.payload_view::<str>() {
                        //     println!("payload for message {:?}",payload);
                        //     if utc_deadline > Utc::now() {
                        //         //TODO: check the DB is the entry is present and mark it readied
                        //         match KafkaPublisher::produce(payload, &k_producer).await {
                        //             Ok(m) => {
                        //                 println!("insert success with number changed {:?}", m);
                        //             }
                        //             Err(e) => {
                        //                 println!("publish failed {:?}", e);
                        //             }
                        //         }
                        //     } else {
                        //         let params = TableInsertRow {
                        //             id: &*chronosMessageId,
                        //             deadline,
                        //             message_headers,
                        //             message_key,
                        //             message_value: json!(&payload),
                        //         };
                        //         match PgDB::insert(&data_store, &params).await {
                        //             Ok(m) => {
                        //                 println!("insert success with number changed {:?}", m);
                        //             }
                        //             Err(e) => {
                        //                 println!("insert failed {:?}", e);
                        //             }
                        //         }
                        //     }
                        // }
                    }
                }

                println!("commit received message {:?}", message);
                k_consumer
                    .commit_message(&message, CommitMode::Async)
                    .expect("commit receiver message failed");
            }
        }
    }
}

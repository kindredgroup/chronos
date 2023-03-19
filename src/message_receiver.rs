use std::collections::HashMap;
// use chrono::prelude::*;
use chrono::{DateTime, Utc};
use rdkafka::consumer::{CommitMode, Consumer};
use std::str::{from_utf8, FromStr};
use log::{error, info, warn};

use crate::consumer::ConsumerClient;
use crate::pg_client::{DBOps, PgDB, TableInsertRow, TableRow};
use crate::producer::{KafkaPublisher, ProducerMessages};
use rdkafka::message::{Headers, Message};
use serde_json::{json, Value};
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

        loop {
            if let Ok(message) = k_consumer.recv().await {
                let new_message = &message;
                println!("consume new_message :: {:?}@ {:?}", new_message, Utc::now());
                if let Some(headers) = new_message.headers() {

                    if headers.count() == 2 {
                        //TODO: extract header type conversion into utils
                        let mut hash_headers = HashMap::new();
                        for index in 0..headers.count() {
                            let key = headers.get(index).key;
                            let value = headers.get(index).value.unwrap();
                            let str_value = from_utf8(value).unwrap();

                            hash_headers.insert(key, str_value);
                        }
                        let str_headers = serde_json::to_string(&hash_headers).unwrap();

                        let message_headers:Value = serde_json::from_str(&str_headers).unwrap();


                        //TODO: extract creation of payload for insert or publish to utils
                        let message_deadline = DateTime::<Utc>::from_str(
                            &message_headers["chronosDeadline"]
                                .as_str()
                                .expect("string from value failed")
                        )
                        .expect("String date parsing failed");

                        let chronos_message_id = &message_headers["chronosID"];
                        let chronos_message_id = &chronos_message_id.as_str().unwrap();

                        let message_key = from_utf8(new_message.key().expect("no message Key found")).unwrap();

                        if let Some(Ok(payload)) = new_message.payload_view::<str>() {

                            if message_deadline > Utc::now() {
                                //TODO: missing check the DB is the entry is present and mark it readied
                                match KafkaPublisher::publish( &k_producer,
                                                               &payload,
                                                               &str_headers.as_str(),
                                                               &message_key.to_string() ).await {
                                    Ok(m) => {
                                        println!("published message id {:?} @ {:?}", &chronos_message_id, Utc::now());
                                    }
                                    Err(e) => {
                                        println!("publish failed {:?}", e);
                                    }
                                }
                            } else {
                                let params = TableInsertRow {
                                    id: &*chronos_message_id,
                                    deadline: message_deadline,
                                    message_headers: &message_headers,
                                    message_key,
                                    message_value: &json!(&payload),
                                };
                                match PgDB::insert(&data_store, &params).await {
                                    Ok(m) => {
                                        println!("insert success with number changed {:?} @ {:?}", &params.id, Utc::now());
                                    }
                                    Err(e) => {
                                        println!("insert failed {:?}", e);
                                    }
                                }
                            }
                        }
                    }else{
                        warn!("message with improper headers on input.topic ");
                        //TODO: ignore

                    }
                }

                println!("commit received message {:?}", message);
                 match   k_consumer.commit_message(&message, CommitMode::Async){
                     Ok(commit_resp)=> {
                         info!("successfully committed offset for the message {:?}",&commit_resp);
                         commit_resp
                     }
                     Err(e)=> {
                         error!("commit offset for message failed {:?}",e);
                         println!("commit offset for message failed {:?}",e);
                     }
                 }

            }
        }
    }
}

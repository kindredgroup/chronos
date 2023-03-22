use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rdkafka::consumer::{CommitMode, Consumer};
use std::str::{from_utf8, FromStr};
use log::{error, info, warn};

use crate::consumer::ConsumerClient;
use crate::pg_client::{DBOps, PgDB, TableInsertRow, TableRow};
use crate::producer::{KafkaPublisher, ProducerMessages};
use rdkafka::message::{Header, Headers, Message};
use serde_json::{json, Value};
use crate::utils::required_headers;


pub struct MessageReceiver {
    // pub(crate) consumer: Box<dyn MessageConsumer>,
    // pub(crate) data_store: Box<dyn DataStore>,
    // pub(crate) producer: Box<dyn MessageProducer>
}

impl MessageReceiver {
    pub async fn run(&self) {
        let topics = vec!["input.topic"];

        let kafka_consumer = ConsumerClient::new(topics, "amn.test.rust".to_string());
        ConsumerClient::subsTopics(&kafka_consumer).await;
        
        let kafka_producer = KafkaPublisher::new();

        let db_config = PgDB {
            connection_config: String::from(
                "host=localhost user=admin password=admin dbname=chronos_db",
            ),
        };
        let data_store = PgDB::new(&db_config).await;

        loop {
            if let Ok(message) = ConsumerClient::consume_message(&kafka_consumer).await {
                let new_message = &message;
                println!("consume new_message :: {:?}@ {:?}", new_message, Utc::now());
                let headers: HashMap<String,String> = if let Some(reqd_headers) = required_headers(&new_message){
                    reqd_headers
                }else{
                    println!("exception occurred while parsing the headers");
                    break
                };

                // if let Some(headers) = new_message.headers() {
                    // let reqd_headers = headers.iter()
                    //     .filter(|header| header.key == "chronosID" || header.key =="chronosDeadline")
                    //     .collect::<HashMap<_,_>>();

                    if headers.len() == 2 {

                        let message_deadline:DateTime<Utc> = DateTime::<Utc>::from_str(
                            &headers["chronosDeadline"]
                        )
                        .expect("String date parsing failed");
                        let chronos_message_id = &headers["chronosID"];

                        let message_key = from_utf8(new_message.key().expect("no message Key found")).unwrap();

                        if let Some(Ok(payload)) = new_message.payload_view::<str>() {

                            if message_deadline <= Utc::now() {
                                //TODO: missing check the DB is the entry is present and mark it readied
                                match KafkaPublisher::publish( &kafka_producer,
                                                               &payload,
                                                               &headers,
                                                               &message_key.to_string() ).await {
                                    Ok(m) => {
                                        println!("published message id {:?} @ {:?}", &chronos_message_id, Utc::now());
                                    }
                                    Err(e) => {
                                        println!("publish failed {:?}", e);
                                    }
                                }
                            } else {
                                let chronos_message_id = &headers.get("chronosID").expect("chronos id missing");

                                let params = TableInsertRow {
                                    id: &*chronos_message_id,
                                    deadline: message_deadline,
                                    message_headers: &serde_json::json!(&headers),
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

                // println!("commit received message {:?}", new_message);
                // if let Ok(m) = &kafka_consumer.client{
                //     m.commit_message(&message, CommitMode::Async).expect("commit message failed ");
                // }else{
                //     println!("Error Occured");
                // }
               

            }
        }
    // }
}

// use chrono::prelude::*;
use chrono::{DateTime, Utc};
use rdkafka::consumer::{CommitMode, Consumer};
use std::str::{from_utf8, FromStr};

use crate::consumer::ConsumerClient;
use crate::pg_client::{DBOps, PgDB};
use crate::producer::{KafkaPublisher, ProducerMessages};
use rdkafka::message::{Headers, Message};
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
                        // TODO: iterate headers for deadline header
                        let header_chrono = headers.get(0).value.unwrap();

                        let date_string = from_utf8(header_chrono).unwrap();
                        let utc_deadline = DateTime::<Utc>::from_str(date_string).unwrap();

                        println!(
                            "deadline header {:?} {:?}",
                            utc_deadline,
                            utc_deadline < Utc::now()
                        );


                        if let Some(Ok(payload)) = message.payload_view::<str>() {
                            if utc_deadline < Utc::now() {
                                match KafkaPublisher::produce(payload, &k_producer).await {
                                    Ok(m) => {
                                        println!("insert success with number changed {:?}", m);
                                    }
                                    Err(e) => {
                                        println!("publish failed {:?}", e);
                                    }
                                }
                            } else {
                                match PgDB::insert(&data_store).await {
                                    Ok(m) => {
                                        println!("insert success with number changed {:?}", m);
                                    }
                                    Err(e) => {
                                        println!("insert failed {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                }

            }
        }
    }
}

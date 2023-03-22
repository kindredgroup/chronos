use std::collections::HashMap;
use std::time::Duration;

use anyhow::Error;
use log::info;

use chrono::prelude::*;
use rdkafka::config::ClientConfig;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;
use chronos::producer;

use clap::Parser;
use chronos::producer::{KafkaPublisher, ProducerMessages};



#[derive(Parser, Debug)]
#[command( version, about, long_about = None)]
struct Args {
    /// brokers
    #[arg(short, long, default_value_t =String::from("localhost:9093"))]
    brokers: String,

    ///  kafka topic name
    #[arg(short, long , default_value_t=String::from("input.topic"))]
    topic: String,

    ///  Number of messages
    #[arg(short, long, default_value_t = 5)]
    num: i32,

    ///  Number of messages
    #[arg(short, long, default_value_t = 0)]
    sec: i64,
}


#[tokio::main]
pub async fn main() {
    env_logger::init();
    println!("Hello from Producer {}", Utc::now().to_rfc3339());
    let matches  = Args::parse();

    let producer_client = KafkaPublisher::new();
    let message = "{message:test message}";
    let utc = Utc::now()+chrono::Duration::seconds(matches.sec);

    let headers=&HashMap::from(
       [
            ("chronosID".to_string(), Uuid::new_v4().to_string()),
            ("chronosDeadline".to_string(), utc.to_string())]
    );



    let mut futures:Vec<_>=Vec::new();
    for n in 0..matches.num {
        // println!("this is a test");
        let key = format!("Key {}", n).to_string();
        let param = "key";

        let outcome = KafkaPublisher::publish(&producer_client, &message, &headers, param);
        futures.push(outcome);
    };

    // let futures = for (0..matches.num) {
    //
    //
    // }).collect::<Vec<_>>();


    // This loop will wait until all delivery statuses have been received.
    for future in futures {
        println!("Future completed. Result: {:?}", future.await);
    }

    //TODO: brokers and topic from config
    // produce(&matches.brokers, &matches.topic,
    //     &matches.num,
    //     &matches.sec).await;
}

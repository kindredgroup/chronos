use std::time::Duration;

use log::info;

use chrono::prelude::*;
use rdkafka::config::ClientConfig;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use uuid::Uuid;

use clap::Parser;

// use crate::example_utils::setup_logger;

async fn produce(brokers: &str, topic_name: &str, num_of_messages: &i32) {
    let id = Uuid::new_v4(); // only because I like v1

    let utc: DateTime<Utc> = Utc::now();

    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    let num = num_of_messages.to_owned();
    println!("checking how many mess {}", num);

    // This loop is non blocking: all messages will be sent one after the other, without waiting
    // for the results.
    let futures = (0..num)
        .map(|i| async move {
            // The send operation on the topic returns a future, which will be
            // completed once the result or failure from Kafka is received.
            let delivery_status = producer
                .send(
                    FutureRecord::to(topic_name)
                        .payload(&format!("Message {}", i))
                        .key(&format!("Key {}", i))
                        .headers(
                            OwnedHeaders::new()
                                .insert(Header {
                                    key: "chronosDeadline",
                                    value: Some(&utc.to_string()),
                                })
                                .insert(Header {
                                    key: "chronosID",
                                    value: Some(&id.to_string()),
                                }),
                        ),
                    Duration::from_secs(0),
                )
                .await;

            // This will be executed when the result is received.
            info!("Delivery status for message {} received", i);
            println!("delivery status, {:?}", delivery_status);
            delivery_status
        })
        .collect::<Vec<_>>();

    // This loop will wait until all delivery statuses have been received.
    for future in futures {
        // info!("Future completed. Result: {:?}", future.await);
        println!("Future completed. Result: {:?}", future.await);
    }
}
#[derive(Parser, Debug)]
#[command( version, about, long_about = None)]
struct Args {
    /// brokers
    #[arg(short, long, default_value_t =String::from("localhost:9092"))]
    brokers: String,

    ///  logging config
    #[arg(short, long, default_value_t =String::from("localhost:9092"))]
    log_conf: String,

    ///  kafka topic name
    #[arg(short, long , default_value_t=String::from("rust"))]
    topic: String,

    ///  Number of messages
    #[arg(short, long, default_value_t = 5)]
    num: i32,
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    println!("Hello from Producer");
    let matches = Args::parse();

    //
    // let topic = &matches.topic;
    // let brokers = &matches.brokers;
    let num_of_messages = &matches.num;

    //TODO: brokers and topic from config
    produce("localhost:29092", "topic", num_of_messages).await;
}

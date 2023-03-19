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

    // let data = r#"
    //     {
    //         "name": "John Doe",
    //         "age": 43,
    //         "phones": [
    //             "+44 1234567",
    //             "+44 2345678"
    //         ]
    //     }"#;

    // Parse the string of data into serde_json::Value.
    // let v: Value = serde_json::from_str(data);

    #[derive(Debug, Clone)]
    struct InputMessage<'a> {
        deadline: DateTime<Utc>,
        message: &'a str,
    }

    // To use the `{}` marker, the trait `fmt::Display` must be implemented
    // manually for the type.
    impl fmt::Display for InputMessage<'_> {
        // This trait requires `fmt` with this exact signature.
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            // Write strictly the first element into the supplied output
            // stream: `f`. Returns `fmt::Result` which indicates whether the
            // operation succeeded or failed. Note that `write!` uses syntax which
            // is very similar to `println!`.
            write!(f, "{} {}", self.deadline, self.message)
        }
    }

    impl std::marker::Copy for InputMessage<'_> {}

    let message = InputMessage {
        deadline: utc,
        message: "Test Message",
    };

    // This loop is non blocking: all messages will be sent one after the other, without waiting
    // for the results.
    let futures = (0..num)
        .map(|i| async move {
            // The send operation on the topic returns a future, which will be
            // completed once the result or failure from Kafka is received.
            let delivery_status = producer
                .send(
                    FutureRecord::to(topic_name)
                        .payload(&format!("Message {}", message))
                        .key(&format!("Key {}", i))
                        .headers(
                            OwnedHeaders::new()
                                .insert(Header {
                                    key: "chronosDeadline",
                                    value: Some(&utc.to_string()),
                                })
                                .insert(Header {
                                    key: "chronosID",
                                    value: Some(&Uuid::new_v4().to_string()),
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
    println!("Hello from Producer {}", Utc::now().to_rfc3339());
    let matches = Args::parse();

    //
    // let topic = &matches.topic;
    // let brokers = &matches.brokers;
    let num_of_messages = &matches.num;

    //TODO: brokers and topic from config
    produce("localhost:9093", "input.topic", num_of_messages).await;
}

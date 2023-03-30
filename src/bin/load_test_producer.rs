use chrono::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

use chronos::producer::{KafkaPublisher, MessageProducer};
use chronos::utils::{CHRONOS_ID, DEADLINE};
use clap::Parser;
use tokio::time::Instant;
use uuid::Uuid;

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
    let matches = Args::parse();

    let producer_client = KafkaPublisher::new("input.topic".to_string());
    let message = "{message:test message}";

    let now_instance = Instant::now();
    let to_stop = now_instance.checked_add(Duration::from_secs(1)).unwrap();

    let batches = Vec::from([100]);

    for n in batches {
        let mut pub_futures: Vec<_> = Vec::new();

        for i in 0..n {
            let key = format!("Key {}", n + i).to_string();
            println!(" counter {:?}", key);
            let utc = Utc::now() + chrono::Duration::seconds(matches.sec);

            let headers = HashMap::from([
                (CHRONOS_ID.to_string(), Uuid::new_v4().to_string()),
                (DEADLINE.to_string(), utc.to_string()),
            ]);
            let outcome = producer_client.publish(message.to_string(), Some(headers), key);
            pub_futures.push(outcome);
        }
        futures::future::join_all(pub_futures).await;
        // println!("elapsed {:?}",now_instance.elapsed());

        // 100 messages take 14ms
        tokio::time::sleep(Duration::from_millis(40)).await;
    }
    // println!("number of messages {counter}");
    // println!("future length {:?}", pub_futures.len());

    // let futures = for (0..matches.num) {
    //
    //
    // }).collect::<Vec<_>>();

    // This loop will wait until all delivery statuses have been received.
    // pub_futures
    // for future in pub_futures {
    //     println!("Future completed. Result: {:?}", future.await);
    // }

    println!("elapsed {:?}", now_instance.elapsed());

    //TODO: brokers and topic from config
    // produce(&matches.brokers, &matches.topic,
    //     &matches.num,
    //     &matches.sec).await;
}

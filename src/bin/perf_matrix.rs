use chrono::prelude::*;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use chronos::consumer::{KafkaConsumer, MessageConsumer};
use chronos::runner::Runner;
use chronos::utils::{headers_check, required_headers};
use chronos::{
    producer::KafkaPublisher,
    utils::{CHRONOS_ID, DEADLINE},
};
use clap::Parser;
use rdkafka::Message;
use tokio::join;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use uuid::Uuid;
use chronos::producer::MessageProducer;

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
    let mut matrix_records: HashMap<String, String> = HashMap::new();

    let matches = Args::parse();

    let time_lapse = 3;
    // let producer_thread = tokio::task::spawn( async move {
    let producer_client = KafkaPublisher::new("input.topic".to_string());
    let message = "{message:test message}";

    let mut pub_futures: Vec<_> = Vec::new();

    let now_instance = Instant::now();
    let to_stop = now_instance.checked_add(Duration::from_secs(1)).unwrap();

    // for n in 0..matches.num {
    let mut counter = 0;
    while Duration::from_secs(time_lapse) > now_instance.elapsed() {
        counter = counter + 1;
        // println!("this is a test");
        let key = format!("Key {}", counter).to_string();
        println!(" counter {:?}", key);
        let utc = Utc::now() + chrono::Duration::seconds(matches.sec);

        let headers: HashMap<String, String> = HashMap::from([
            (CHRONOS_ID.to_string(), Uuid::new_v4().to_string()),
            (DEADLINE.to_string(), utc.to_string()),
        ]);
        let outcome = producer_client
            .publish(message.to_string(), Some(headers), key)
            .await
            .unwrap();
        pub_futures.push(outcome);
    }
    println!("number of messages {counter}");
    println!("future length {:?}", pub_futures.len());
    matrix_records.insert(
        "produced".to_string(),
        format!("{} per {time_lapse}", pub_futures.len()),
    );

    // This loop will wait until all delivery statuses have been received.
    for _future in pub_futures {
        // println!("Future completed. Result: {:?}", future.await);
    }

    // thread::sleep(Duration::from_secs(3));
    let runner = Runner {};
    runner.run().await;
    // });

    // let consumer_thread = tokio::task::spawn(async move  {

    // thread::sleep(Duration::from_secs(15));
    let topics = vec!["input.topic", "outbox.topic"];
    let kafka_consumer = KafkaConsumer::new(topics, "load.amn.test".to_string());
    kafka_consumer.subscribe().await;

    let mut input_time_records: HashMap<String, i64> = HashMap::new();
    let mut output_time_records: HashMap<String, i64> = HashMap::new();

    // for _n in 0..200 {
    loop {
        if let Ok(message) = kafka_consumer.consume_message().await {
            println!("this is from the consumer {:?}", message.topic());
            let m_headers = message.headers().expect("headers extraction failed");

            if headers_check(m_headers) {
                let m_timestamp = message
                    .timestamp()
                    .to_millis()
                    .expect("timestamp extract failed");
                let m_topic = message.topic();
                let headers = required_headers(&message).unwrap();

                let header = headers[CHRONOS_ID].to_owned();

                let key = format!("{}", &header);
                // println!("{:?} {:?} {:?}",m_topic, m_timestamp, headers);

                if &m_topic == &"input.topic" {
                    println!("{:?} - {:?}", key, m_topic);
                    input_time_records.insert(key, m_timestamp);
                } else {
                    println!("{:?} - {:?}", key, m_topic);
                    output_time_records.insert(key, m_timestamp);
                }
            } else {
                break;
            }
        } else {
            println!("message read from consumer failed")
        }
    }
    println!("collected data {:?}", input_time_records);
    // println!("number of Input messages::{:?} to number of output messages {}",input_time_records.len(), output_time_records.len());
    //
    // // sort_by_time(input_time_records)
    // let min_value = input_time_records.iter().min_by(|x, y| x.cmp(y)).unwrap();
    // let max_value = input_time_records.iter().max_by(|x, y| x.cmp(y)).unwrap();
    // let min_o_value = output_time_records.iter().min_by(|x, y| x.cmp(y)).unwrap();
    // let max_o_value = output_time_records.iter().max_by(|x, y| x.cmp(y)).unwrap();
    // println!(
    //     "\n Min{:?}- \n Max{:?} \n Min0{:?}- \n Max0{:?}",
    //     min_value, max_value, min_o_value, max_o_value
    // );
    // println!(
    //     "num of In {:?} num out {:?}",
    //     input_time_records.len(),
    //     output_time_records.len()
    // )
    // });

    //TODO: brokers and topic from config
    // produce(&matches.brokers, &matches.topic,
    //     &matches.num,
    //     &matches.sec).await;

    // producer_thread.await.unwrap();
    // consumer_thread.await.unwrap();
}

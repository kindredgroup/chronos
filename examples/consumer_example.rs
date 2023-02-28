// use clap::Parser;
use log::{info, warn};

use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::util::get_rdkafka_version;

// use crate::example_utils::setup_logger;

// mod example_utils;

// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        println!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        println!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        println!("Committing offsets: {:?}", result);
    }
}

// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;

async fn consume_and_print(brokers: &str, group_id: &str, topics: &[&str]) {
    let context = CustomContext;

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        //.set("statistics.interval.ms", "30000")
        .set("auto.offset.reset", "smallest")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)
        .expect("Consumer creation failed");

    consumer
        .subscribe(&topics.to_vec())
        .expect("Can't subscribe to specified topics");

    loop {
        match consumer.recv().await {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        warn!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                println!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                         m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                if let Some(headers) = m.headers() {
                    for header in headers.iter() {
                        println!("  Header {:#?}: {:?}", header.key, header.value);
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}

// #[derive(Parser, Debug)]
// #[command( version, about, long_about = None)]
// struct Args {
//     /// brokers
//     #[arg(short, long, default_value_t =String::from("localhost:9092"))]
//     brokers: String,
//
//     // ///  logging config
//     // #[arg(short, long, default_value_t =String::from("localhost:9092"))]
//     // log_conf: String,
//
//     ///  kafka topic name
//     #[arg(short, long, default_value_t=String::from("rust" ))]
//     topics: String,
//
//     ///  kafka group_id
//     #[arg(short, long , default_value_t =String::from("test.consumer.rust"))]
//     group_id: String,
//
//
// }

#[tokio::main]
pub async fn consumer() {
    println!("Hello from consumer!");
    // let matches = Args::parse();
    //
    // println!("Hello world {:?}",matches);
    //
    //
    // // setup_logger(true, matches.value_of("log-conf"));
    //
    // let (version_n, version_s) = get_rdkafka_version();
    // println!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);
    //
    // let topics = &matches.topics;
    // let brokers = &matches.brokers;
    // let group_id = &matches.group_id;
    //
    // // setup_logger(true, matches.log-conf);
    //
    //
    // println!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);

    let v = vec!["topic"];
    println!("starting consumer ...");

    // consume_and_print(brokers, group_id, &v).await
    consume_and_print("localhost:29092", "test.group.rust", &v).await
}

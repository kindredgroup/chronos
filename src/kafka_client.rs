// use anyhow::Result;
// use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
// use rdkafka::consumer::stream_consumer::StreamConsumer;
// use rdkafka::consumer::{ConsumerContext, Rebalance};
// use rdkafka::error::{KafkaError, KafkaResult};
// use rdkafka::message::BorrowedMessage;
// use rdkafka::producer::{FutureProducer, DefaultProducerContext};
// use rdkafka::{ClientContext, TopicPartitionList};

pub struct KafkaConsumerStruct {
    brokers: String,
    group_id: String,
    topic: String,
}
// pub struct Producer {
//     topic: String,
//     brokers: String,
// }

// pub trait KafkaClient {
//     fn connect(&self) -> Result<(), anyhow::Error>;
// }

use rdkafka::message::ToBytes;
use std::collections::HashMap;

// pub trait PublisherTrait {
//     async fn send(&self, msg: impl ToBytes, headers: HashMap<String, String>) -> Result<(), MyError>;
// }
pub trait KafkaConsumer {
    fn consume(
        &self,
        msg: impl ToBytes,
        headers: HashMap<String, String>,
    ) -> Result<(), anyhow::Error>;
    fn subscribe(&self);
    fn unsubscrbe(&self);
}

// pub struct KafkaPublisher {
//     producer: rdkafka::producer::FutureProducer<DefaultProducerContext>
// }

// impl KafkaPublisher {
//     pub fn new(config: ClientConfig) -> Self  {

//     }
// }
// // A context can be used to change the behavior of producers and consumers by adding callbacks
// // that will be executed by librdkafka.
// // This particular context sets up custom callbacks to log rebalancing events.
// struct CustomContext;

// impl ClientContext for CustomContext {}

// impl ConsumerContext for CustomContext {
//     fn pre_rebalance(&self, rebalance: &Rebalance) {
//         println!("Pre rebalance {:?}", rebalance);
//     }

//     fn post_rebalance(&self, rebalance: &Rebalance) {
//         println!("Post rebalance {:?}", rebalance);
//     }

//     fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
//         println!("Committing offsets: {:?}", result);
//     }
// }

// impl KafkaClient for Consumer {
//     fn connect(&self) -> Result<StreamConsumer<CustomContext>, anyhow::Error> {
//         println!("consumer client implementation");
//         let context = CustomContext;

//         // let group_id = "amn.test.rust";
//         // let brokers = "localhost:29092";
//         let consumer = ClientConfig::new()
//             .set("group.id", &self.group_id)
//             .set("bootstrap.servers", &self.brokers)
//             .set("enable.partition.eof", "false")
//             .set("session.timeout.ms", "6000")
//             .set("enable.auto.commit", "true")
//             //.set("statistics.interval.ms", "30000")
//             .set("auto.offset.reset", "beginning")
//             .set_log_level(RDKafkaLogLevel::Debug);

//         match consumer.create_with_context(context) {
//             Ok(v) => v,
//             Err(e) => {
//                 println!("cannot instantiate consumer {:?}", &e);

//                 e
//             }
//         };

//         println!("consumer client connected");
//     }
// }

// impl KafkaClient for Producer {
//     fn connect(&self) -> Result<(), anyhow::Error> {
//         let producer: &FutureProducer = &ClientConfig::new()
//             .set("bootstrap.servers", "localhost:29092")
//             .set("message.timeout.ms", "5000")
//             .create()?;

//         println!("producer  connected");
//         return Ok(producer);
//     }
// }

// impl Producer {
//     fn produce(&self, connection: String) {
//         println!(" start producing ")
//     }
// }
// impl Consumer {
//     pub fn consume(&self, connection: StreamConsumer<CustomContext>) -> Result<(), anyhow::Error> {
//         //Result<BorrowedMessage<'_>, KafkaError>{
//         connection.subscribe(&self.topic)?;

//         Ok(connection.recv());
//     }
// }

// // fn main(){
// //     let consumer = Consumer{
// //          group_id : String::from("amn.test.rust"),
// //          brokers : String::from("localhost:29092"),
// //         topic: String::from("inbox.topic"),
// //     };
// //     let con_connection = consumer.connect();
// //     consumer.consume(con_connection);

// //     let publisher = Producer{
// //         topic: String::from("outbox.topic"),
// //         brokers:String::from("localhost:29092"),
// //     };

// //     let pro_connection = publisher.connect();
// //     publisher.produce(pro_connection);

// // }

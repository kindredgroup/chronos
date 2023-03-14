// use anyhow::Result;
// use rdkafka::producer::{FutureProducer};
// use rdkafka::{ClientContext, TopicPartitionList};
// use rdkafka::consumer::{ConsumerContext, Rebalance};
// use rdkafka::consumer::stream_consumer::StreamConsumer;
// use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
// use rdkafka::error::KafkaResult;

// pub struct Consumer{
//     brokers:String,
//     group_id:String,
//     topic:String,
// }
// pub struct Producer{
//     topic:String,
//     brokers:String
// }

// pub trait KafkaClient{
//      fn connect(&self)->Result<T,anyhow::Error>;

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


// impl KafkaClient for Consumer{
//     fn connect(&self)->Result<StreamConsumer,anyhow::Error>{

//         println!("consumer client implementation");
//         let context = CustomContext;

//         // let group_id = "amn.test.rust";
//         // let brokers = "localhost:29092";
//         let consumer: StreamConsumer = ClientConfig::new()
//             .set("group.id", &self.group_id)
//             .set("bootstrap.servers", &self.brokers)
//             .set("enable.partition.eof", "false")
//             .set("session.timeout.ms", "6000")
//             .set("enable.auto.commit", "true")
//             //.set("statistics.interval.ms", "30000")
//             .set("auto.offset.reset", "beginning")
//             .set_log_level(RDKafkaLogLevel::Debug)
//             .create_with_context(context)?;
            
//             println!("consumer client connected");

//        return Ok(consumer)
//     }
// }

// impl KafkaClient for Producer{
//     fn connect(&self)->Result<&FutureProducer,anyhow::Error>{
//         let producer: &FutureProducer = &ClientConfig::new()
//         .set("bootstrap.servers", "localhost:29092")
//         .set("message.timeout.ms", "5000")
//         .create()?;

//         println!("producer  connected");
//         return Ok(producer)
//     }

// }

// impl Producer{
//    fn produce(&self,connection:String){
//     println!(" start producing ")
//    }
// }
// impl Consumer{
//    fn consume(&self,connection:String){
//     println!(" start consuming ")
//    }
// }

// fn main(){
//     let consumer = Consumer{
//          group_id : String::from("amn.test.rust"),
//          brokers : String::from("localhost:29092"),
//         topic: String::from("inbox.topic"),
//     };
//     let con_connection = consumer.connect();
//     consumer.consume(con_connection);


//     let publisher = Producer{
//         topic: String::from("outbox.topic"),
//         brokers:String::from("localhost:29092"),
//     };

//     let pro_connection = publisher.connect();
//     publisher.produce(pro_connection);

// }
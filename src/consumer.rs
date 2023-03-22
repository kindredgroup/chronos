use async_trait::async_trait;
use log::warn;
use rdkafka::Message;
use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{Consumer, ConsumerContext, Rebalance};

use rdkafka::message::BorrowedMessage;
use tokio::join;


const INPUT_HEADERS: [&str; 2] = ["chronosID", "chronosDeadline"];

pub fn consumer(group_id:String) -> Result<StreamConsumer<CustomContext>, anyhow::Error> {
    //->Result<(), Box<dyn std::error::Error>>  {
    println!("Hello from consumer!");
    let context = CustomContext;

    // let group_id = "amn.test.rust";
    let brokers = "localhost:9093";
    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        // .set("enable.auto.commit", "true")
        //.set("statistics.interval.ms", "30000")
        .set("auto.offset.reset", "beginning")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)?;

    return Ok(consumer);

}


// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
pub struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    // fn pre_rebalance(&self, rebalance: &Rebalance) {
    //     println!("Pre rebalance {:?}", rebalance);
    // }
    //
    // fn post_rebalance(&self, rebalance: &Rebalance) {
    //     println!("Post rebalance {:?}", rebalance);
    // }

    // fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
    //     println!("Committing offsets: {:?}", result);
    // }
}

// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;



pub struct ConsumerClient {
    pub(crate) client: Result<StreamConsumer<CustomContext>, anyhow::Error>,
    pub(crate) topics: Vec<&'static str>,
}

impl ConsumerClient {
    pub fn new(topics: Vec<&'static str>, group_id: String) -> ConsumerClient {
        Self { client:consumer(group_id) , topics }
    }

    pub async fn subsTopics (&self){
        let inst = if let Ok(inst) = &self.client { inst } else { todo!() };

        inst.subscribe(&self.topics).expect("consumer Subscribe to topic failed");

    }
   pub async fn consume_message(&self)->Result<BorrowedMessage, &anyhow::Error> {
       
        println!("route to consume");

        match &self.client {
            Ok(consumer) => {
                println!("success");
                let subs = consumer.subscription().expect("failed to fetch");
                println!("subs lists {:?}",subs );
                let  input_message: BorrowedMessage = consumer.recv().await.expect("message recv error");
                return Ok(input_message);
                
            },

            Err(e) => {
                println!("Error occurred");
                return Err(e)
            }
        }
        
    
      
    }
}

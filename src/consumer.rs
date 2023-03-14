use anyhow::Result;
use async_trait::async_trait;
use log::warn;
use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{Consumer, ConsumerContext, Rebalance};
use rdkafka::message::{Headers, Message};


#[derive(Debug)]
pub struct FireMessage {
    pub payload: String, //TODO: explore what type can be the payload
}
impl FireMessage {
    fn prepare_enq_fire_message(m: FireMessage) -> Self {
        //TODO: modulation for firing add extra identifiers

        Self { payload: m.payload }
    }
}
const INPUT_HEADERS: [&str; 2] = ["chronosID", "chronosDeadline"];

pub fn consumer() -> Result<StreamConsumer<CustomContext>, anyhow::Error> {
    //->Result<(), Box<dyn std::error::Error>>  {
    println!("Hello from consumer!");
    let context = CustomContext;

    let group_id = "amn.test.rust";
    let brokers = "localhost:9093";
    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        //.set("statistics.interval.ms", "30000")
        .set("auto.offset.reset", "beginning")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)?;

    return Ok(consumer);

    // let handle = tokio::spawn(async move { consume_and_print(consumer).await });

    // return consume_and_print(consumer).await;

    // handle.await.expect("TODO: panic message");
}

// #[tokio::main]
pub async fn consume_and_print(
    consumer: &StreamConsumer<CustomContext>,
    // sender: Sender<String>,
) -> Result<FireMessage, anyhow::Error> {
    let topics = vec!["input.topic"];
    println!("start to consume");

    consumer.subscribe(&topics)?;

    match consumer.recv().await {
        Err(e) => {
            warn!("Kafka error: {}", e);
            println!("Error to consume");
            return Err(anyhow::anyhow!("Kafka error: {}", e));
        }
        Ok(m) => {
            println!(" consume {:?}", m);
            let payload = match m.payload_view::<str>() {
                None => "",
                Some(Ok(s)) => s,
                Some(Err(e)) => {
                    println!("this is from error while consuming");
                    warn!("Error while deserializing message payload: {:?}", e);
                    ""
                }
            };
            println!(
                "key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                m.key(),
                payload,
                m.topic(),
                m.partition(),
                m.offset(),
                m.timestamp()
            );
            if let Some(headers) = m.headers() {
                if headers.count() == 2 {
                    for header in headers.iter() {
                        println!("  Header {:#?}: {:?}", &header.key, &header.value);
                        if INPUT_HEADERS.contains(&header.key) {
                            continue;
                        } else {
                            println!("Incompatible headers");
                            return Err(anyhow::anyhow!("Incompatible headers"));
                        }
                    }

                    //TODO: if all is good then prepare the enqueueing / firing message

                    let serialized = serde_json::to_string(&payload).unwrap();
                    println!("serialized = {}", serialized);

                    let consumed_message = FireMessage {
                        payload: String::from(serialized),
                    };
                    // sender.send(String::from(payload));
                    return Ok(FireMessage::prepare_enq_fire_message(consumed_message));

                    // TODO: commit once the fire is success
                    // consumer.commit_message(&m, CommitMode::Async).unwrap();
                } else {
                    // prinln!("Error")
                    return Err(anyhow::anyhow!(
                        "Error occurred while consuming the message"
                    ));
                }
            } else {
                // prinln!("Error")
                return Err(anyhow::anyhow!(
                    "Error occurred while consuming the message"
                ));
                //TODO: send to the Error or DLQ in case one of the headers is missing
            }
        }
    };
}

// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.
pub struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        println!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        println!("Post rebalance {:?}", rebalance);
    }

    // fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
    //     println!("Committing offsets: {:?}", result);
    // }
}

// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;

#[async_trait]
pub trait ConsumerMessages {
    async fn consume_message(x: &StreamConsumer<CustomContext>); //-> Result<BorrowedMessage, Error>;
}

pub struct ConsumerClient {
    pub(crate) client: Result<StreamConsumer<CustomContext>>,
}

impl ConsumerClient {
    pub async fn new() -> Self {
        Self { client: consumer() }
    }
}
#[async_trait]
impl ConsumerMessages for ConsumerClient {
    async fn consume_message(x: &StreamConsumer<CustomContext>) {
        // -> Result<BorrowedMessage, Error> {
        // todo!()
        // if let Ok(mess) =  x.recv().await{
        //     mess
        // }
        // return match x.recv().await {
        //     Err(e) => {
        //         warn!("Kafka error: {}", e);
        //         Err(anyhow::anyhow!("Kafka error: {}", e)).expect("TODO: panic message");
        //     }
        //     Ok(m) => println!("Message {:?}",m),
        // };
        println!("route to consume");

        consume_and_print(x).await.expect("TODO: panic message");
    }
}

// use crate::kafka_client::{Consumer, KafkaClient};

// fn subscribe(){
//     let consumer = Consumer{
//         group_id : String::from("amn.test.rust"),
//         brokers : String::from("localhost:29092"),
//        topic: String::from("inbox.topic"),
//     };
//     let con_connection = consumer.connect();
//     consumer.consume(con_connection);
// }

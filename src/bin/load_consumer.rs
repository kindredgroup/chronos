use chrono::{DateTime, Utc};
use chronos::consumer::{KafkaConsumer, MessageConsumer};
use chronos::runner::Runner;
use chronos::utils::{headers_check, required_headers, CHRONOS_ID, DEADLINE};
use futures::StreamExt;
use hdrhistogram::Histogram;
use log::kv::ToValue;
use rdkafka::message::Headers;
use rdkafka::Message;
use std::collections::HashMap;

struct Matrix {
    first_message: DateTime<Utc>,
    last_message: DateTime<Utc>,
    number_messages: i64,
}

#[tokio::main]
pub async fn main() {
    //Run main thread to produce outbox messages
    // let r = Runner {};
    // r.run().await;

    let topics = vec!["input.topic", "outbox.topic"];
    let kafka_consumer = KafkaConsumer::new(topics, "load.amn.test".to_string());
    kafka_consumer.subscribe().await;
    let mut hist_in = Histogram::<u64>::new(2).unwrap();

    let mut hist_out = Histogram::<u64>::new(2).unwrap();

    let mut input_time_records: HashMap<String, i64> = HashMap::new();
    let mut output_time_records: HashMap<String, i64> = HashMap::new();

    for n in 0..200 {
        // println!("n {}",n);
        // let mut counter = 0;
        // loop {
        //     counter = counter+1;
        //     println!("{}",counter+1);
        if let Ok(message) = kafka_consumer.consume_message().await {
            // println!("this is from the consumer {:?}", message.topic() );
            let m_headers = message.headers().expect("headers extraction failed");

            if headers_check(m_headers) {
                let m_timestamp = message.timestamp().to_millis().unwrap();
                let m_topic = message.topic();
                let headers = required_headers(&message).unwrap();

                let header = headers[CHRONOS_ID].to_owned();

                let key = format!("{}", &header);
                // println!("{:?} {:?} {:?}",m_topic, m_timestamp, headers);

                if &m_topic == &"input.topic" {
                    println!("{:?} - {:?}", key, m_topic);
                    input_time_records.insert(key, m_timestamp);
                    hist_in
                        .record(m_timestamp as u64)
                        .expect("TODO: panic message");
                } else {
                    println!("{:?} - {:?}", key, m_topic);
                    output_time_records.insert(key, m_timestamp);
                    hist_out
                        .record(m_timestamp as u64)
                        .expect("TODO: panic message");
                }
            } else {
                println!("invalid headers {:?}", required_headers(&message).unwrap());
                continue;
            }
        } else {
            println!("message read from consumer failed")
        }
    }

    // let mut combined_hash = HashMap::new();
    //      input_time_records.iter().map(|rec|{
    //          combined_hash.insert(rec.key,(rec.value,output_time_records.get(rec.key)))
    //      })
    // for v in hist.iter_recorded() {
    //     println!("{}'th percentile of data is {} with {} samples",
    //              v.percentile(), v.value_iterated_to(), v.count_at_value());
    // }

    // let diff_hist = Histogram::new()::<u64>();

    println!("collected data {:?}", input_time_records.len());
    println!("output collected data {:?}", output_time_records.len());
    let mut hist = Histogram::<u64>::new(2).unwrap();

    let something = input_time_records.iter().map(|rec| {
        let (key, value) = rec;
        let out_time = output_time_records.get(key).unwrap();
        let lag = out_time - value;
        hist.record(lag as u64).unwrap();
    });

    let in_count = hist_in.len();
    let out_count = hist_out.len();
    let in_time_diff = hist_in.max() - hist_in.min();
    let out_time_diff = hist_out.max() - hist_out.min();

    let in_rate = in_count / in_time_diff;
    let out_rate = out_count / out_time_diff;

    println!("in rate {in_rate} -- out rate {out_rate}");

    // // println!("number of Input messages::{:?} to number of output messages {}",input_time_records.len(), output_time_records.len());
    // //
    // // // sort_by_time(input_time_records)
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
}

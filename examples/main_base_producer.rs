// use std::{thread, time::Duration};
//
// use rdkafka::config::ClientConfig;
// use rdkafka::producer::{BaseProducer,BaseRecord,Producer};
//
//
// #[tokio::main]
// async fn main() {
//
//     let producer: BaseProducer = ClientConfig::new()
//         .set("bootstrap.servers", "localhost:9092")
//         // .set("security.protocol","SASL_SSL")
//         // .set("sasl.mechanism","PLAIN")
//         // .set("sasl.username","")
//         // .set("sasl.password","")
//         .create()
//         .expect("Producer creation error");
//
//     let response = producer.send(
//         BaseRecord::to("test")
//             .payload("this is the payload")
//             .key("and this is a key"),
//     ); //.expect("Failed to enqueue");
//
//     match response {
//         Ok(())=>{
//             println!("Success")
//         },
//         Err((e,_))=>{
//             eprintln!("Failed to publish on kafka {:?}", e);
//         }
//     }
//
//     //
//     // for _ in 0..10 {
//     //     producer.poll(Duration::from_millis(100));
//     // }
//     //
//     // // And/or flush the producer before dropping it.
//     // let result = producer.flush(Duration::from_secs(1));
//     // println!("producer result {:?}",result)
// }

use std::{thread, time::Duration};

use rdkafka::{
    producer::{BaseProducer, BaseRecord},
    ClientConfig,
};

fn main() {
    let producer: BaseProducer = ClientConfig::new()
        .set("bootstrap.servers", "something:888")
        //for auth
        // .set("security.protocol", "SASL_SSL")
        .set("sasl.mechanisms", "PLAIN")
        .set("sasl.username", "<update>")
        .set("sasl.password", "<update>")
        .create()
        .expect("invalid producer config");

    for i in 1..100 {
        println!("Base produce message");

        producer
            .send(
                BaseRecord::to("rust")
                    .key(&format!("key-{}", i))
                    .payload(&format!("value-{}", i)),
            )
            .expect("failed to send message");

        thread::sleep(Duration::from_secs(3));
    }
}

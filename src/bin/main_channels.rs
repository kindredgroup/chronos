// #![allow(unused)]

// use std::sync::Arc;
// // use std::thread::sleep;
// use tokio::time::sleep;

// use std::time::Duration;
// // use producer::publish;
// use tokio::sync::mpsc;
// use tokio::try_join;

// // use crate::consumer::{consume_and_print, consumer};
// // use crate::kafka_client::KafkaConsumer;
// // use crate::producer::producer;
// // use std::future::Future;

// // use std::sync::mpsc;
// // use std::sync::mpsc::{Receiver, Sender};

// // mod consumer;
// // mod layover;
// // mod pg_client;
// // mod producer;
// // mod scruitiny;

// #[tokio::main]
// async fn main() {
//     let (sender, mut receiver) = mpsc::channel(32);

//     let thread_safe_tx = Arc::new(sender).clone();

//     // let chronos_consumer = consumer().unwrap();

//     let c_handle = tokio::task::spawn(async move {
//         loop {
//             thread_safe_tx.send("this is a test").await;
//             // consume_and_print(&chronos_consumer, sender).await.unwrap();
//             sleep(Duration::from_secs(1));
//         }
//     });

//     let p_handle = tokio::task::spawn(async move {
//         loop {
//             let received = receiver.recv().await.unwrap();
//             println!("testing received {:?}", received);
//         }

//         // scruitiny::Srcuitiny::scuitinize(received).await;
//         // }
//     });

//     // try_join!(c_handle);
//     try_join!(c_handle, p_handle);
// }

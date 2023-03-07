#![allow(unused)]

use producer::publish;
use tokio::try_join;

use crate::consumer::{consume_and_print, consumer};
use crate::kafka_client::KafkaConsumer;
use crate::producer::producer;
// use std::future::Future;
//

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

mod consumer;
mod kafka_client;
mod producer;
mod scruitiny;
mod layover;
mod pg_client;

#[tokio::main]
async fn main() {
    let (sender, receiver) = mpsc::channel::<String>();

    let chronos_consumer = consumer().unwrap();

    let c_handle = tokio::spawn( async move  {

        consume_and_print(chronos_consumer, sender).await.unwrap();
    });

    // let chronos_producer = producer();
    let p_handle = tokio::spawn(async {
        scruitiny::Srcuitiny::scuitinize(receiver);
    });
    // let p_handle = tokio::spawn(async {
    //     publish(chronos_producer, receiver).await;
    // });

    try_join!(c_handle, p_handle);
}


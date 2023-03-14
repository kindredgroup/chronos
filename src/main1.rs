#![allow(unused)]

use producer::publish;
use tokio::try_join;

use crate::consumer::{consume_and_print, consumer};
use crate::kafka_client::KafkaConsumer;
use crate::producer::producer;
// use std::future::Future;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

mod consumer;
// mod layover;
mod pg_client;
mod producer;
// mod scruitiny;

#[tokio::main]
async fn main() {
    let (sender, receiver) = mpsc::channel::<String>();

    let chronos_consumer = consumer().unwrap();

    

    let c_handle = tokio::spawn(async {
        
        // loop{

            consume_and_print(&chronos_consumer, sender).await.unwrap();
        // }
    });

    // let p_handle = tokio::spawn(async move {
         let received  =  receiver.recv().unwrap();
        // prin
    tln!("tetsing received {:?}", received);

            // scruitiny::Srcuitiny::scuitinize(received).await;
        // }
    // });
   

    try_join!(c_handle);
    // try_join!(c_handle, p_handle);
}

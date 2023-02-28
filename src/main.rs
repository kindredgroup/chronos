#![allow(unused)]
use crate::consumer::{consumer};
use crate::producer::producer;
// use std::future::Future;
//

use std::sync::mpsc;
use std::thread;
// use std::thread;
//
mod consumer;
mod producer;

// // #[tokio::main]
// fn main() {
//     println!("welcome to kafka prosumer! 0.0.1");
//
//     let (rx, tx) = channel::<String>();
//
//     thread::spawn(move || match consumer() {
//         Ok(m) => {
//             println!("message that was consumed {:?}", m);
//             tx.send("Message consumed").unwrap();
//         }
//         Err(e) => println!("error from the consumer {:?}", e),
//     });
//
//     match rx.try_recv() {
//         Ok(m) => println!("checking the message rec {:?}", m),
//         Err(e) => println!("error in message rec {:?}", e),
//     }
//
//     // producer();
//     // consumer();
// }

// use tokio::sync::mpsc;
//
// #[tokio::main]
// async fn main() {
//     let (tx, mut rx) = mpsc::channel(32);
//     let tx2 = tx.clone();
//
//     tokio::spawn(async move {
//         tx.send("sending from first handle").await;
//     });
//
//     tokio::spawn(async move {
//         tx2.send("sending from second handle").await;
//     });
//
//     while let Some(message) = rx.recv().await {
//         println!("GOT = {}", message);
//     }
// }

fn main() {
    use std::sync::mpsc::channel;
    use std::thread;

    let (sender, receiver) = channel();

    let handle = thread::spawn(move || {
        sender.send(consumer()).unwrap();
    });

    // println!("This is the receiver{:?}", receiver.recv().unwrap());
    match receiver.recv().unwrap() {
        Ok(m) => {
            println!("received {:?}", &m);
            producer(m.payload);
        }
        Err(e) => println!("Error occurred while receiving {:?}", e),
    }

    handle.join().unwrap()
}

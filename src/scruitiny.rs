use anyhow::Ok;
use std::sync::mpsc::Receiver;

use crate::{producer::{publish, producer}, layover::into_layover};
pub struct Srcuitiny{

}

pub enum Outcomes{
    ToLayover(bool),
    ToPublish(bool),
    NoAction(bool)
}

impl Srcuitiny{
    pub fn scuitinize(rec:Receiver<String>)->Outcomes{
        match rec.recv().unwrap() {
            m => {
                println!("the payload from sender thread {:?}", m);
                let deadline = m;
                if(deadline == "now"){
                    let chronos_producer = producer();
                    let message = String::from("this is hard coded");
                    publish(chronos_producer,message);
                }else if (deadline == "later") {
                    into_layover()
                }
            }
        }

        todo!()

    }
}
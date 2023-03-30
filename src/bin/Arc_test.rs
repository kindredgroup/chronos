use std::sync::Arc;


trait DBOps {
    fn insert(&self);
}
#[derive(Debug)]
pub struct PG {}

impl PG{
    fn testing(&self){

    }
}

impl DBOps for PG {
   fn insert(&self) {
        println!("this is a PG Client")
    }
}

pub struct Receiver {
    client: Arc<Box<dyn DBOps  + Send + Sync >>,
}

impl Receiver {
    // pub fn new()->Self{
    //     Self{
    //         client: Arc::new(Box::new(PG{})),
    //     }
    // }
    pub fn recv(&self){

        &self.client.insert();
        print!("Recv and operate ");
    }
    pub fn test(&self){
        println!("this is a test");
    }
}


struct Runner {}

impl Runner {
    fn run(&self){
        let pg = PG{};
        let recv_inst = Receiver{
            client: Arc::new(Box::new(pg)),
        };

        recv_inst.recv();


    }

    // fn run(&self){
    //
    // }

}

fn main(){
    let runner = Runner{};
    runner.run()

}

// #[derive(Debug)]
// struct Rectangle {
//     width: u32,
//     height: u32,
// }
//
// impl Rectangle {
//     fn area(&self) -> u32 {
//         self.width * self.height
//     }
// }
//
// fn main() {
//     let rect1 = Rectangle {
//         width: 30,
//         height: 50,
//     };
//
//     println!(
//         "The area of the rectangle is {} square pixels.",
//         rect1.area()
//     );
// }
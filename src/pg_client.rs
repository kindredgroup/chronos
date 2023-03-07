use chrono::{DateTime, Utc};
use postgres::{Client, NoTls};

// // trait DB{
// //    fn connect_db(&self)-> Client;

// // }
#[derive(Debug)]
pub struct DbPg{
   pub connection_config: String
}

 impl  DbPg{
   pub fn connect_db(&self)-> Client{
        // let config  =  &self.connection_config
        let config = "host=localhost user=admin password=admin dbname=chronos_db";

        let mut client = Client::connect(config, NoTls).expect("connection creation failed");
        client

    }
}

#[derive(Debug)]
pub struct HangerColumns<'a> {
    pub id: &'a str,
    pub deadline: DateTime<Utc>,
    pub readied_at: DateTime<Utc>,
    pub readied_by: &'a str,
    //  message_headers :HashMap<&'a str, &'a str>,
    // message_headers: Vec<MessageHeaders>,
    pub message_key: &'a str,
    pub message_value: &'a str,
}

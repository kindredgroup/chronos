// use chrono::{DateTime, Utc};
// // use postgres::types::ToSql;
// // use std::collections::HashMap;
// use std::fmt::Debug;
// // use log::debug;
// // use serde_json::Map;

// fn main() {
//     use postgres::{Client, NoTls};
//     let utc: DateTime<Utc> = Utc::now();

//     let mut client = Client::connect(
//         "host=localhost user=admin password=admin dbname=chronos_db",
//         NoTls,
//     )
//     .expect("connection creation failed");

//     // let create_table = client.batch_execute(
//     //     "
//     //             CREATE TABLE hanger (
//     //                 id          varchar NOT NULL,
//     //                 deadline    timestamp NOT NULL DEFAULT NOW(),
//     //                 readied_at   timestamp,
//     //                 readied_by  uuid,
//     //                 message_headers JSON,
//     //                 message_key JSON,
//     //                 message_value text,
//     //                 PRIMARY KEY (id)

//     //             )
//     //         ",
//     // );

//     // match create_table {
//     //     Ok(m) => println!("Table created {:?}", m),
//     //     Err(e) => println!("Table creation failed {:?}", e),
//     // }

//     // #[derive(Debug)]
//     // struct MessageHeaders {
//     //     key: String,
//     //     value: String,
//     // }

//     // impl MessageHeaders {
//     //     fn new(key: &str, value: &str) -> Self {
//     //         Self {
//     //             key: key.to_string(),
//     //             value: value.to_string(),
//     //         }
//     //     }
//     // }

//     // impl ToSql for MessageHeaders {
//     //     fn to_sql(
//     //         &self,
//     //         out: &mut postgres::types::private::BytesMut,
//     //     ) -> Result<postgres::types::IsNull, Box<dyn Error + Sync + Send>>
//     //     where
//     //         Self: Sized,
//     //     {
//     //     }
//     // }

//     #[derive(Debug)]
//     struct HangerColumns<'a> {
//         id: &'a str,
//         deadline: DateTime<Utc>,
//         readied_at: DateTime<Utc>,
//         readied_by: &'a str,
//         //  message_headers :HashMap<&'a str, &'a str>,
//         message_headers: Vec<MessageHeaders>,
//         message_key: &'a str,
//         message_value: &'a str,
//     }

//     // impl HangerColumns<'_> {
//     //     fn new(&self) -> Self {
//     //         Self {
//     //             id: &self.id,
//     //             deadline: self.deadline,
//     //             readied_at: self.readied_at,
//     //             readied_by: &self.readied_by,
//     //             // message_headers: self.message_headers,
//     //             message_key: &self.message_key,
//     //             message_value: &self.message_value,
//     //         }
//     //     }
//     // }

//     // let entry_headers = Vec::new();
//     // entry_headers.push(MessageHeaders::new("key1", "value1"));
//     // entry_headers.push(MessageHeaders::new("key2", "value2"));
//     // let entry_header= HashMap::new();
//     // entry_header.insert(
//     //    "key1","value1"
//     // );
//     // entry_header.insert(
//     //    "key2","value2"
//     // );

//     let entries = HangerColumns {
//         id: "ID",
//         deadline: utc,
//         readied_at: utc,
//         readied_by: "a-node",
//         message_headers: entry_headers,
//         message_key: "key",
//         message_value: "value",
//     };

//     // let outfits = HangerColumns::new()

//     // impl ToSql for Outfits<'_>{
//     //     fn to_sql(&self, ty: &postgres::types::Type, out: &mut postgres::types::private::BytesMut) -> Result<postgres::types::IsNull, Box<dyn Error + Sync + Send>>
//     // where
//     //     Self: Sized {
//     //     todo!()
//     // }

//     //     fn accepts(ty: &postgres::types::Type) -> bool
//     // where
//     //     Self: Sized {
//     //     todo!()
//     // }

//     //     fn to_sql_checked(
//     //     &self,
//     //     ty: &postgres::types::Type,
//     //     out: &mut postgres::types::private::BytesMut,
//     // ) -> Result<postgres::types::IsNull, Box<dyn Error + Sync + Send>> {
//     //     todo!()
//     // }
//     // }

// //     client.execute(
// //     "INSERT INTO hanger (id, deadline, readied_at, readied_by, message_headers, message_key, message_value)
// //         VALUES ($1, $2 ,$3, $4, $5, $6, $7)",
// //     &[&entries.id, &entries.deadline,&entries.readied_at,&entries.readied_by,&entries.message_headers ,&entries.message_key,&entries.message_value],
// // ).expect("insert query failed");
//     client.execute(
//     "INSERT INTO hanger (id, deadline, readied_at, readied_by, message_key, message_value)
//         VALUES ($1, $2 ,$3, $4, $5, $6)",
//     &[&entries.id, &entries.deadline,&entries.readied_at,&entries.readied_by,&entries.message_key,&entries.message_value],
// ).expect("insert query failed");

//     for row in client
//         .query("SELECT id, deadline, FROM hanger", &[])
//         .expect("select failed")
//     {
//         let id: i32 = row.get(0);
//         let name: &str = row.get(1);
//         let data: Option<&[u8]> = row.get(2);

//         println!("found person: {} {} {:?}", id, name, data);
//     }
// }

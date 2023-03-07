use chrono::{DateTime, Utc};
use crate::pg_client::{DbPg, HangerColumns};

pub fn into_layover (){
    let config = String::from("host=localhost user=admin password=admin dbname=chronos_db");


    let db_pg = DbPg{
        connection_config:config
    };


    let pg_client =  db_pg.connect_db();

    let layover =  "INSERT INTO hanger (id, deadline, readied_at, readied_by, message_key, message_value)
    VALUES ($1, $2 ,$3, $4, $5, $6)";

    let utc: DateTime<Utc> = Utc::now();

    let entries = HangerColumns {
        id: "ID",
        deadline: utc,
        readied_at: utc,
        readied_by: "a-node",
        // message_headers: entry_headers,
        message_key: "key",
        message_value: "value",
    };
    let params =  &[&entries.id, &entries.deadline,&entries.readied_at,&entries.readied_by,&entries.message_key,&entries.message_value];
    pg_client.execute(layover, params).expect("insert failed");

}
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio_postgres::{Client, Error, NoTls, Row};
use uuid::Uuid;
use crate::persistence_store::PersistenceStore;

#[derive(Debug)]
pub struct PgDB {
    // pub connection_config: String,
    // pub client: Client,
}

impl PgDB {

    pub async fn connect(&self) -> Client {
        let (client, connection) = tokio_postgres::connect(
            "host=localhost user=admin password=admin dbname=chronos_db",
            NoTls,
        )
        .await.unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                println!("connection error: {}", e);
            }
        });
        client
    }
}

#[async_trait]
impl PersistenceStore for PgDB {
    async fn insert_to_delay(&self, params: &TableInsertRow) -> Result<u64, anyhow::Error> {
        let pg_client = self.connect().await;
        let insert_query =
            "INSERT INTO hanger (id, deadline,  message_headers, message_key, message_value)
                 VALUES ($1, $2 ,$3, $4, $5 )";

        let outcome = pg_client
            .execute(
                insert_query,
                &[
                    &params.id,
                    &params.deadline,
                    &params.message_headers,
                    &params.message_key,
                    &params.message_value,
                ],
            )
            .await?;

        Ok(outcome)
    }

    async fn delete_fired(&self, ids: &String) -> u64 {
        let pg_client = self.connect().await;

        println!("delete ids {:?}", ids);
        let delete_ids = ids.strip_suffix(",").unwrap().to_string();

        let sql = format!("DELETE FROM hanger WHERE id IN ({})", delete_ids);

        let response = pg_client.execute(&sql, &[]).await.expect("delete failed");
        response
    }

    async fn ready_to_fire(&self, params: &Vec<GetReady>) -> Vec<Row> {
        let pg_client = self.connect().await;

        println!("readying_update DB");
        let param = &params[0];

        //TODO: This query will need limit and the sort order
        let ready_query = format!( "UPDATE hanger SET readied_at= '{}', readied_by= '{}'  where id IN\
                                (SELECT ID FROM  hanger WHERE deadline < '{}' AND readied_at IS NULL LIMIT {})\
                                RETURNING id, deadline, readied_at, readied_by, message_headers, message_key, message_value;",&param.readied_at,
                                   &param.readied_by,
                                   &param.deadline,
                                   &param.limit);

        // println!("ready query {}", ready_query);

        let response = pg_client
            .query(&ready_query, &[])
            .await
            .expect("update failed");

        println!("redying success {:?}", &response);
        response

        // Ok(response)
    }

    async fn failed_to_fire(&self, delay_time: DateTime<Utc>) -> Vec<Row> {
        let pg_client = self.connect().await;

        let get_query = "SELECT * from hanger where readied_at < $1";
        let response = pg_client
            .query(get_query, &[&delay_time])
            .await
            .expect("get delayed messages query failed");

        response
    }

    async fn reset_to_init(&self,  to_init_list: &Vec<Row>) -> Vec<String> {

        let mut id_list = Vec::<String>::new();
        for row in to_init_list {
            let updated_row = TableRow {
                id: row.get("id"),
                deadline: row.get("deadline"),
                readied_at: row.get("readied_at"),
                readied_by: row.get("readied_by"),
                message_headers: row.get("message_headers"),
                message_key: row.get("message_key"),
                message_value: row.get("message_value"),
            };

            println!("logging failed to fire messages {}", updated_row.id);
            id_list.push(format!("'{}'", updated_row.id));
        }
        let ids_list = id_list.join(",");
        let pg_client = self.connect().await;

        let reset_query = format!(
            "UPDATE hanger SET readied_at=null , readied_by=null  WHERE id IN  ({})",
            ids_list
        );
        println!("reset query {}", reset_query);

        pg_client.execute(&reset_query, &[]).await.expect("reset to init query failed");

        id_list
    }
}

#[derive(Debug)]
pub struct TableInsertColumns<'a> {
    pub id: &'a str,
    pub deadline: DateTime<Utc>,
    pub message_headers: serde_json::Value,
    pub message_key: &'a str,
    pub message_value: serde_json::Value,
}

#[derive(Debug)]
pub struct TableRow<'a> {
    pub id: &'a str,
    pub deadline: DateTime<Utc>,
    pub readied_at: DateTime<Utc>,
    pub readied_by: Uuid,
    pub message_headers: serde_json::Value,
    pub message_key: &'a str,
    pub message_value: serde_json::Value,
}

#[derive(Debug)]
pub struct TableInsertRow<'a> {
    pub id: &'a str,
    pub deadline: DateTime<Utc>,
    pub message_headers: &'a serde_json::Value,
    pub message_key: &'a str,
    pub message_value: &'a serde_json::Value,
}

pub struct GetReady {
    pub readied_at: DateTime<Utc>,
    pub readied_by: Uuid,
    pub deadline: DateTime<Utc>,
    pub limit: i64,
    // pub order: &'a str,
}


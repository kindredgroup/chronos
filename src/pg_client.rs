use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio_postgres::{Client, Error, NoTls, Row};
use uuid::Uuid;

#[async_trait]
pub trait DBOps {
    async fn insert(db: &Client) -> Result<u64, anyhow::Error>;
    async fn queuing_fetch(pg_client: &Client, deadline: String, limit: u16) -> Vec<TableRow>;
    async fn delete_fired(pg_client: &Client, ids: &Vec<&str>)->u64;
    async fn readying_update(
        pg_client: &Client,
        params: Vec<GetReady>,
    ) -> Result<Vec<TableRow>, Error>;
}
#[derive(Debug)]
pub struct PgDB {
    pub connection_config: String,
}

impl PgDB {
    pub async fn new(&self) -> Client {
        // let config  =  &self.connection_config
        let config = "host=localhost user=admin password=admin dbname=chronos_db";

        let (client, connection) = tokio_postgres::connect(config, NoTls).await.unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        client
    }
}

#[async_trait]
impl DBOps for PgDB {
    async fn insert(pg_client: &Client) -> Result<u64, anyhow::Error> {
        let insert_query =
            "INSERT INTO hanger (id, deadline,  message_headers, message_key, message_value)
                 VALUES ($1, $2 ,$3, $4, $5, )";

        let utc: DateTime<Utc> = Utc::now();
        let id = Uuid::new_v4();

        let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;

        let Ok(headers) = serde_json::from_str(data) else { todo!() };

        let Ok(value) = serde_json::from_str(data) else { todo!() };

        let entries = TableInsertColumns {
            id: "ID8",
            deadline: utc,
            // readied_at: utc,
            // readied_by: id, // These are not required for the first insert but wil be needed in the subsequent updates
            message_headers: headers,
            message_key: "key",
            message_value: value,
        };

        let outcome = pg_client
            .execute(
                insert_query,
                &[
                    &entries.id,
                    &entries.deadline,
                    // &entries.readied_at,
                    // &entries.readied_by,
                    &entries.message_headers,
                    &entries.message_key,
                    &entries.message_value,
                ],
            )
            .await?;

        Ok(outcome)
    }

    async fn queuing_fetch(pg_client: &Client, deadline: String, limit: u16) -> Vec<TableRow> {
        let select_query = "select * from hanger as h
                    where h.deadline = $1 AND h.readied_at IS NULL ";

       let deadline = "2023-03-08 02:42:00.616449+00";

        let matching_rows = pg_client
            .query(select_query, &[&deadline])
            .await
            .expect("fetch failed");

        let values: Vec<TableRow> = matching_rows
            .into_iter()
            .map(|row| TableRow::from(row))
            .collect();
        println!("result for fetch {:?}", &values);
        values
    }

    async fn delete_fired(pg_client: &Client, ids: &Vec<&str>) -> u64 {
        let delete_query = "Delete from hanger where id In $1";

        let Ok(response) = pg_client.execute(delete_query, &[ids]).await;
        response
    }

    async fn readying_update(
        pg_client: &Client,
        params: Vec<GetReady>,
    ) -> Result<Vec<TableRow>, Error> {
        let param = &params[0];

        //TODO: This query will need limit and the sort order
        let ready_query = "UPDATE hanger set readied_at=clock_timestamp() , readied_by=$2 where id IN\
                                (SELECT ID FROM  hanger where deadline < $1 AND readied_at IS NULL LIMIT $3)\
                                RETURNING id, deadline, readied_at, readied_by, message_headers, message_key, message_value;";

        let Ok(response) = pg_client.query(ready_query, &[
            &param.deadline,
            &param.readied_by,
            &param.limit,
            &param.order
        ]).await;
        let mapped_rows = response
            .into_iter()
            .map(|row| TableRow::from(row))
            .collect();
        Ok(mapped_rows)
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

impl From<Row> for TableRow<'_> {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            deadline: row.get("deadline"),
            readied_at: row.get("readied_at"),
            readied_by: row.get("readied_by"),
            message_headers: row.get("message_headers"),
            message_key: row.get("message_key"),
            message_value: row.get("message_value"),
        }
    }
}

pub struct GetReady<'a> {
    // pub readied_at: DateTime<Utc>,
    pub readied_by: Uuid,
    pub deadline: DateTime<Utc>,
    pub limit: i16,
    pub order: &'a str,
}

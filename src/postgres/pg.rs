use async_trait::async_trait;
use chrono::{DateTime, Utc};
use deadpool_postgres::{Config, GenericClient, ManagerConfig, Object, Pool, PoolConfig, Runtime};
use log::error;
use serde_json::{json, Value};
use tokio_postgres::{NoTls, Row};
use uuid::Uuid;

use crate::postgres::config::PgConfig;
use crate::postgres::errors::PgError;


#[derive(Clone)]
pub struct Pg {
    pub pool: Pool,
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

// create postgres client
impl Pg {
    pub async fn new(pg_config: PgConfig) -> Result<Self, PgError> {
        let mut config = Config::new();
        config.dbname = Some(pg_config.database);
        config.user = Some(pg_config.user);
        config.password = Some(pg_config.password);
        config.host = Some(pg_config.host);
        config.port = Some(pg_config.port.parse::<u16>().expect("Failed to parse port to u16"));
        config.manager = Some(ManagerConfig {
            recycling_method: deadpool_postgres::RecyclingMethod::Fast,
        });
        config.pool = Some(PoolConfig::new(pg_config.pool_size));

        let pool = config.create_pool(Some(Runtime::Tokio1), NoTls).map_err(PgError::CreatePool)?;

        //test connection
        let _ = pool.get().await.map_err(PgError::GetClientFromPool)?;

        println!("pool.status: {:?}", pool.status());
        Ok(Pg { pool })
    }

    pub async fn get_client(&self) -> Result<Object, PgError> {
        // let client = self.pool.get().await.map_err(PgError::GetClientFromPool)?;
        match self.pool.get().await {
            Err(e) => {
                error!("error::: {:?}", e);
                Err(PgError::GetClientFromPool(e))
            }
            Ok(client) => Ok(client),
        }
        // println!("pool.status: {:?}",self.pool.status());
        // Ok(client)
    }
}

impl Pg {
    pub(crate) async fn insert_to_delay(&self, params: &TableInsertRow<'_>) -> Result<u64, PgError> {
        let pg_client =  self.get_client().await?;
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
            .await.expect("insert failed");

        Ok(outcome)
    }

    pub(crate) async fn delete_fired(&self, ids: &String) -> Result<u64,PgError> {
        let pg_client = self.get_client().await?;

        println!("delete ids {:?}", ids);
        let delete_ids = ids.strip_suffix(",").unwrap().to_string();

        let sql = format!("DELETE FROM hanger WHERE id IN ({})", delete_ids);

        let response = pg_client.execute(&sql, &[]).await.unwrap();
        Ok(response)
    }

    pub(crate) async fn ready_to_fire(&self, params: &Vec<GetReady>) -> Result<Vec<Row>, PgError> {
        let pg_client = self.get_client().await?;

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
        Ok(response)

        // Ok(response)
    }

    pub(crate) async fn failed_to_fire(&self, delay_time: DateTime<Utc>) -> Result<Vec<Row>, PgError> {
        let pg_client = self.get_client().await?;

        let get_query = "SELECT * from hanger where readied_at < $1";
        let response = pg_client
            .query(get_query, &[&delay_time])
            .await
            .expect("get delayed messages query failed");

        Ok(response)
    }

    pub(crate) async fn reset_to_init(&self, to_init_list: &Vec<Row>) -> Result<Vec<String>, PgError> {

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
        let pg_client = self.get_client().await?;

        let reset_query = format!(
            "UPDATE hanger SET readied_at=null , readied_by=null  WHERE id IN  ({})",
            ids_list
        );
        println!("reset query {}", reset_query);

        pg_client.execute(&reset_query, &[]).await.expect("reset to init query failed");

        Ok(id_list)
    }
}
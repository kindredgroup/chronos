use async_trait::async_trait;
use chrono::{DateTime, Utc};
use deadpool_postgres::{Config, GenericClient, ManagerConfig, Object, Pool, PoolConfig, Runtime};
use log::error;
use log::kv::Source;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio_postgres::types::ToSql;
use tokio_postgres::{Client, NoTls, Row};
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
#[derive(Debug)]
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
        // let (client, connection) = tokio_postgres::connect(
        //     "host=localhost user=postgres password=admin dbname=chronos",
        //     NoTls,
        // )
        //     .await.unwrap();
        // tokio::spawn(async move {
        //     if let Err(e) = connection.await {
        //         println!("connection error: {}", e);
        //     }
        // });
        // Ok(client)

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
        let get_client_instant = Instant::now();
        let pg_client = self.get_client().await?;
        let insert_query = "INSERT INTO hanger (id, deadline,  message_headers, message_key, message_value)
                 VALUES ($1, $2 ,$3, $4, $5 )";
        let query_execute_instant = Instant::now();
        let stmt = pg_client.prepare(insert_query).await?;
        let outcome = pg_client
            .execute(
                &stmt,
                &[
                    &params.id,
                    &params.deadline,
                    &params.message_headers,
                    &params.message_key,
                    &params.message_value,
                ],
            )
            .await
            .expect("insert failed");
        let time_elapsed = query_execute_instant.elapsed();
        if time_elapsed > Duration::from_millis(100) {
            println!("insert_to_delay query_execute_instant: {:?} ", time_elapsed);
        }

        Ok(outcome)
    }

    pub(crate) async fn delete_fired(&self, ids: &Vec<String>) -> Result<u64, PgError> {
        let query_execute_instant = Instant::now();
        let pg_client = self.get_client().await?;
        // let transaction = pg_client.transaction().await;

        // let delete_ids = ids.join(",");
        // let delete_ids = ids.strip_suffix(",").unwrap().to_string();
        // println!("delete ids {:?}", ids);

        let values_as_slice: Vec<_> = ids.iter().map(|x| x as &(dyn ToSql + Sync)).collect();

        let mut query: String = "DELETE FROM hanger WHERE id IN (".to_owned();
        for i in 0..ids.len() {
            query = query + "$" + (i + 1).to_string().as_str() + ",";
        }
        query = query.strip_suffix(",").unwrap().to_string();
        query = query + ")";
        // println!("query {}", query);
        //let sql = format!("DELETE FROM hanger WHERE id IN ({})", ids);
        let stmt = pg_client.prepare(query.as_str()).await?;
        let response = pg_client.execute(&stmt, &values_as_slice[..]).await?;
        let time_elapsed = query_execute_instant.elapsed();
        if time_elapsed > Duration::from_millis(100) {
            println!(" delete_fired query_execute_instant: {:?} ", time_elapsed);
        }
        Ok(response)
    }

    pub(crate) async fn ready_to_fire(&self, params: &Vec<GetReady>) -> Result<Vec<Row>, PgError> {
        let pg_client = self.get_client().await?;

        // println!("readying_update DB");
        let param = &params[0];

        // let ready_query = format!( "UPDATE hanger SET readied_at = '{}', readied_by= '{}'  where id IN\
        //                         (SELECT ID FROM  hanger WHERE deadline <= '{}' AND readied_at IS NULL LIMIT {})\
        //                         RETURNING id, deadline, readied_at, readied_by, message_headers, message_key, message_value;",&param.readied_at,
        //                            &param.readied_by,
        //                            &param.deadline,
        //                            &param.limit);
        let ready_query = "UPDATE hanger SET readied_at = $1, readied_by = $2 where deadline <= $3 AND readied_at IS NULL RETURNING id, deadline, readied_at, readied_by, message_headers, message_key, message_value";
        // println!("ready query {}", ready_query);
        let stmt = pg_client.prepare(ready_query).await?;
        let query_execute_instant = Instant::now();
        let response = pg_client
            .query(&stmt, &[&param.readied_at, &param.readied_by, &param.deadline])
            .await
            .expect("update failed");
        let time_elapsed = query_execute_instant.elapsed();
        if time_elapsed > Duration::from_millis(100) {
            println!(" ready_to_fire query_execute_instant: {:?} params: {:?}", time_elapsed, param);
        }
        // println!("redying success {:?}", &response);
        Ok(response)

        // Ok(response)
    }

    pub(crate) async fn failed_to_fire(&self, delay_time: DateTime<Utc>) -> Result<Vec<Row>, PgError> {
        let query_execute_instant = Instant::now();
        let pg_client = self.get_client().await?;

        let get_query = "SELECT * from hanger where readied_at > $1";
        let response = pg_client.query(get_query, &[&delay_time]).await.expect("get delayed messages query failed");
        let time_elapsed = query_execute_instant.elapsed();
        if time_elapsed > Duration::from_millis(100) {
            println!(" failed_to_fire query_execute_instant: {:?} ", time_elapsed);
        }
        Ok(response)
    }

    pub(crate) async fn reset_to_init(&self, to_init_list: &Vec<Row>) -> Result<Vec<String>, PgError> {
        let query_execute_instant = Instant::now();
        println!("to_init_list: {}", to_init_list.len());
        let mut id_list = Vec::<String>::new();
        for row in to_init_list {
            // let updated_row = TableRow {
            //     id: row.get("id"),
            //     deadline: row.get("deadline"),
            //     readied_at: row.get("readied_at"),
            //     readied_by: row.get("readied_by"),
            //     message_headers: row.get("message_headers"),
            //     message_key: row.get("message_key"),
            //     message_value: row.get("message_value"),
            // };

            // println!("logging failed to fire messages {}", updated_row.id);
            id_list.push(row.get("id"));
        }

        let pg_client = self.get_client().await?;

        // let reset_query = format!(
        //     "UPDATE hanger SET readied_at=null , readied_by=null  WHERE id IN  ({})",
        //     ids_list
        // );

        let values_as_slice: Vec<_> = id_list.iter().map(|x| x as &(dyn ToSql + Sync)).collect();

        let mut query: String = "UPDATE hanger SET readied_at=null , readied_by=null  WHERE id IN  (".to_owned();
        for i in 0..id_list.len() {
            query = query + "$" + (i + 1).to_string().as_str() + ",";
        }
        query = query.strip_suffix(",").unwrap().to_string();
        query = query + ")";
        // println!("reset query {}", query);
        let stmt = pg_client.prepare(query.as_str()).await?;

        pg_client.execute(&stmt, &values_as_slice[..]).await.expect("reset to init query failed");
        let time_elapsed = query_execute_instant.elapsed();
        if time_elapsed > Duration::from_millis(100) {
            println!(" reset_to_init query_execute_instant: {:?} ", time_elapsed);
        }
        Ok(id_list)
    }
}

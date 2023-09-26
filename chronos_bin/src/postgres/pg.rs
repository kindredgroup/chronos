use chrono::{DateTime, Utc};
use deadpool_postgres::{Config, GenericClient, ManagerConfig, Object, Pool, PoolConfig, Runtime, Transaction};
use log::error;
use std::time::{Duration, Instant};
use tokio_postgres::error::SqlState;
use tokio_postgres::types::ToSql;
use tokio_postgres::{NoTls, Row};
use uuid::Uuid;

use crate::postgres::config::PgConfig;
use crate::postgres::errors::PgError;

use tracing::{event, field, info_span, Span};

#[derive(Clone, Debug)]
pub struct Pg {
    pub pool: Pool,
    // pub pg_span: Span,
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
    // pub limit: i64,
    // pub order: &'a str,
}

struct PgTxn<'a> {
    txn: Transaction<'a>,
}

struct PgAccess {
    client: Object,
}

impl PgAccess {
    pub async fn get_txn(&mut self) -> PgTxn {
        let txn = self
            .client
            .build_transaction()
            .isolation_level(tokio_postgres::IsolationLevel::RepeatableRead)
            .start()
            .await
            .unwrap();
        PgTxn { txn }
    }
}

impl Pg {
    pub async fn new(pg_config: PgConfig) -> Result<Self, PgError> {
        // let pg_span = info_span!("pg_instance", error = field::Empty);
        // let _ = pg_span.enter();

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

        {
            //test connection
            let mut tmp_list: Vec<Object> = Vec::new();
            for _ in 1..=pg_config.pool_size {
                let client = match pool.get().await {
                    Ok(client) => {
                        event!(tracing::Level::INFO, "pg client created");
                        client
                    }
                    Err(e) => {
                        error!("error::: Cannot get client from the pool while setting transaction isolation level {:?}", &e);
                        event!(
                            tracing::Level::ERROR,
                            error = %e,
                            "error::: Cannot get client from the pool while setting transaction isolation",
                        );
                        return Err(PgError::GetClientFromPool(e));
                    }
                };
                let _ = client
                    .execute("SET SESSION CHARACTERISTICS AS TRANSACTION ISOLATION LEVEL REPEATABLE READ;", &[])
                    .await
                    .is_ok();
                tmp_list.push(client);
            }
        }

        for _ in 1..=pg_config.pool_size {
            let client = match pool.get().await {
                Ok(client) => client,
                Err(e) => {
                    error!("error::: Cannot get client from the pool {:?}", e);
                    return Err(PgError::GetClientFromPool(e));
                }
            };

            let rs = client.query_one("show transaction_isolation", &[]).await.unwrap();
            let value: String = rs.get(0);
            log::debug!("init: db-isolation-level: {}", value);
        }

        println!("pool.status: {:?}", pool.status());
        Ok(Pg { pool })
    }

    pub async fn get_client(&self) -> Result<Object, PgError> {
        match self.pool.get().await {
            Err(e) => {
                error!("error::: {:?}", e);
                event!(tracing::Level::ERROR,error=%e, "pg client creation error");
                Err(PgError::GetClientFromPool(e))
            }
            Ok(client) => {
                event!(tracing::Level::INFO, "pg client created");
                Ok(client)
            }
        }
    }
}

impl Pg {
    pub(crate) async fn insert_to_delay(&self, params: &TableInsertRow<'_>) -> Result<u64, PgError> {
        let delay_query_span = info_span!("insert_to_delay", params = field::Empty, query = field::Empty);
        let _ = delay_query_span.enter();

        let pg_client = self.get_client().await?;
        let mut pg_access = PgAccess { client: pg_client };
        let pg_txn: PgTxn = pg_access.get_txn().await;

        let insert_query = "INSERT INTO hanger (id, deadline,  message_headers, message_key, message_value)
        VALUES ($1, $2 ,$3, $4, $5 )";

        delay_query_span.record("query", insert_query);
        // delay_query_span.record("params", &params);

        let query_execute_instant = Instant::now();
        let stmt = pg_txn.txn.prepare(insert_query).await.unwrap();
        let outcome = pg_txn
            .txn
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
            .await;
        let time_elapsed = query_execute_instant.elapsed();
        if time_elapsed > Duration::from_millis(100) {
            println!("insert_to_delay query_execute_instant: {:?} ", time_elapsed);
        }

        if outcome.is_ok() {
            event!(tracing::Level::INFO, "insert_to_delay success");
            let cmt_rdy = pg_txn.txn.commit().await;
            if let Err(e) = cmt_rdy {
                error!("Unable to commit: {}. The original transaction updated: {} rows", e, outcome.unwrap());
                return Err(PgError::UnknownException(e));
            }
        }
        Ok(outcome.unwrap())
    }

    pub(crate) async fn delete_fired(&self, ids: &Vec<String>) -> Result<u64, String> {
        // let delete_query_span = info_span!("delete_fired", params = field::Empty, query = field::Empty);

        // let _ = delete_query_span.enter();

        // let query_execute_instant = Instant::now();
        let pg_client = self.get_client().await.expect("Failed to get client from pool");
        let mut pg_access = PgAccess { client: pg_client };
        let pg_txn: PgTxn = pg_access.get_txn().await;

        let values_as_slice: Vec<_> = ids.iter().map(|x| x as &(dyn ToSql + Sync)).collect();

        let mut query: String = "DELETE FROM hanger WHERE id IN (".to_owned();
        for i in 0..ids.len() {
            query = query + "$" + (i + 1).to_string().as_str() + ",";
        }
        query = query.strip_suffix(',').unwrap().to_string();
        query += ")";
        // println!("query {}", query);
        // delete_query_span.record("query", query.as_str());
        // delete_query_span.record("params", &values_as_slice);

        let stmt = pg_txn.txn.prepare(query.as_str()).await.unwrap();
        let response = pg_txn.txn.execute(&stmt, &values_as_slice).await;
        match response {
            Ok(resp) => {
                let cmt_rdy = pg_txn.txn.commit().await;
                if let Err(e) = cmt_rdy {
                    error!(
                        "delete_fired: Unable to commit: {}. The original transaction updated: {} rows",
                        e,
                        response.unwrap()
                    );
                    return Err(format!("Unable to commit: {}. The original transaction updated rows", e));
                }
                Ok(resp)
            }
            Err(e) => {
                let err_code = e.code();
                if err_code.is_some() {
                    let db_err = err_code.unwrap();
                    if db_err == &SqlState::T_R_SERIALIZATION_FAILURE {
                        error!("delete_fired: Unable to execute txn due to : {}", e);
                        return Err(format!("delete_fired: Unable to execute txn due to : {}", e));
                    }
                }
                Err(format!("delete_fired: Unknow exception {:?}", e))
            }
        }
    }

    pub(crate) async fn ready_to_fire(&self, param: &GetReady) -> Result<Vec<Row>, String> {
        // let ready_to_fire_query_span = info_span!("ready_to_fire", params = field::Empty, query = field::Empty);
        // let _ = ready_to_fire_query_span.enter();

        //TODO handle get client error gracefully
        let pg_client = self.get_client().await.expect("Unable to get client");
        let mut pg_access = PgAccess { client: pg_client };
        let pg_txn: PgTxn = pg_access.get_txn().await;

        let ready_query = "UPDATE hanger SET readied_at = $1, readied_by = $2 where deadline < $3 AND readied_at IS NULL RETURNING id, deadline, readied_at, readied_by, message_headers, message_key, message_value";
        // let stmt = pg_client.prepare(ready_query).await.expect("Unable to prepare query");
        // let query_execute_instant = Instant::now();
        // let response = pg_client
        //     .query(&stmt, &[&param.readied_at, &param.readied_by, &param.deadline])
        //     .await
        //     .expect("update failed");
        // let time_elapsed = query_execute_instant.elapsed();
        // if time_elapsed > Duration::from_millis(100) {
        //     println!(" ready_to_fire query_execute_instant: {:?} params: {:?}", time_elapsed, param);
        // }
        // println!("redying success {:?}", &response);
        // Ok(response)
        // println!("ready_to_fire query {}", &param.deadline);

        // ready_to_fire_query_span.record("query", ready_query);

        let stmt = pg_txn.txn.prepare(ready_query).await.expect("Unable to prepare query");
        let query_execute_instant = Instant::now();
        let response = pg_txn.txn.query(&stmt, &[&param.readied_at, &param.readied_by, &param.deadline]).await;

        match response {
            Ok(resp) => {
                let cmt_rdy = pg_txn.txn.commit().await;
                if let Err(e) = cmt_rdy {
                    error!("Unable to commit: {}. The original transaction updated: {:?} rows", e, resp);
                    return Err(format!(
                        "ready_to_fire: Unable to commit: {}. The original transaction updated: {:?} rows",
                        e, resp
                    ));
                }
                let time_elapsed = query_execute_instant.elapsed();
                if time_elapsed > Duration::from_millis(100) {
                    error!(" ready_to_fire query_execute_instant: {:?} params: {:?}", time_elapsed, param);
                }
                Ok(resp)
            }
            Err(e) => {
                let err_code = e.code();
                if err_code.is_some() {
                    let db_err = err_code.unwrap();
                    if db_err == &SqlState::T_R_SERIALIZATION_FAILURE {
                        error!("ready_to_fire: Unable to execute txn due to : {}", e);
                        return Err(format!("ready_to_fire: Unable to execute txn due to : {}", e));
                    }
                }
                error!("ready_to_fire: Unknow exception {:?}", e);
                Err(format!("ready_to_fire: Unknow exception {:?}", e))
            }
        }
    }

    pub(crate) async fn failed_to_fire(&self, delay_time: &DateTime<Utc>) -> Result<Vec<Row>, PgError> {
        let failed_to_fire_query_span = info_span!("failed_to_fire", params = field::Empty, query = field::Empty);
        let _ = failed_to_fire_query_span.enter();

        let query_execute_instant = Instant::now();
        let pg_client = self.get_client().await?;

        let get_query = "SELECT * from hanger where readied_at > $1 ORDER BY deadline DESC";
        let stmt = pg_client.prepare(get_query).await?;

        failed_to_fire_query_span.record("query", get_query);

        let response = pg_client.query(&stmt, &[&delay_time]).await.expect("get delayed messages query failed");
        let time_elapsed = query_execute_instant.elapsed();
        if time_elapsed > Duration::from_millis(100) {
            error!(" failed_to_fire query_execute_instant: {:?} ", time_elapsed);
        }
        Ok(response)
    }

    pub(crate) async fn reset_to_init(&self, to_init_list: &Vec<Row>) -> Result<Vec<String>, String> {
        let reset_to_init_query_span = info_span!("reset_to_init", params = field::Empty, query = field::Empty);
        let _ = reset_to_init_query_span.enter();

        let query_execute_instant = Instant::now();
        let mut id_list = Vec::<String>::new();
        for row in to_init_list {
            id_list.push(row.get("id"));
        }

        let pg_client = self.get_client().await.expect("Unable to get client");
        let mut pg_access = PgAccess { client: pg_client };
        let pg_txn: PgTxn = pg_access.get_txn().await;

        // let reset_query = format!(
        //     "UPDATE hanger SET readied_at=null , readied_by=null  WHERE id IN  ({})",
        //     ids_list
        // );

        let values_as_slice: Vec<_> = id_list.iter().map(|x| x as &(dyn ToSql + Sync)).collect();

        let mut query: String = "UPDATE hanger SET readied_at=null , readied_by=null  WHERE id IN  (".to_owned();
        for i in 0..id_list.len() {
            query = query + "$" + (i + 1).to_string().as_str() + ",";
        }
        query = query.strip_suffix(',').unwrap().to_string();
        query += ")";

        reset_to_init_query_span.record("query", query.as_str());
        // println!("reset query {}", query);
        let stmt = pg_txn.txn.prepare(query.as_str()).await.expect("Unable to prepare query");

        let response = pg_txn.txn.execute(&stmt, &values_as_slice[..]).await;

        match response {
            Ok(resp) => {
                let cmt_rdy = pg_txn.txn.commit().await;
                if let Err(e) = cmt_rdy {
                    error!("Unable to commit: {}. The original transaction updated: {:?} rows", e, resp);
                    return Err(format!("Unable to commit: {}. The original transaction updated: {:?} rows", e, resp));
                }
                let time_elapsed = query_execute_instant.elapsed();
                if time_elapsed > Duration::from_millis(100) {
                    error!(" ready_to_fire query_execute_instant: {:?} ", time_elapsed);
                }
                Ok(id_list)
            }
            Err(e) => {
                error!("reset_to_init: Unable to execute txn due to : {}", e);
                let err_code = e.code();
                if err_code.is_some() {
                    let db_err = err_code.unwrap();
                    if db_err == &SqlState::T_R_SERIALIZATION_FAILURE {
                        error!("reset_to_init: Unable to execute txn due to : {}", e);
                        return Err(format!("reset_to_init: Unable to execute txn due to : {}", e));
                    }
                }
                Err(format!("reset_to_init: Unknow exception {:?}", e))
            }
        }
    }
}

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio_postgres::{Error, Row};
use crate::pg_client::{ GetReady, TableInsertRow};

#[async_trait]
pub trait PersistenceStore {
    async fn insert_to_delay(&self, params: &TableInsertRow) -> Result<u64, anyhow::Error>;
    // async fn queuing_fetch(pg_client: &Client, deadline: String, limit: u16) -> Vec<TableRow>;
    async fn delete_fired(&self, ids: &String) -> u64;
    async fn ready_to_fire(&self, params: &Vec<GetReady>) -> Vec<Row>;
    async fn failed_to_fire(&self, delay_time: DateTime<Utc>) -> Vec<Row>;
    async fn reset_to_init(&self,  to_init_list: &Vec<Row>) -> Vec<String>;
}
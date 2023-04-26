use deadpool_postgres::{CreatePoolError, PoolError};
use serde_json::Value;
use thiserror::Error as ThisError;
use tokio_postgres::Error as TokioPostgresError;

#[derive(Debug, ThisError)]
pub enum PgError {
    // Deadpool errors
    #[error("Error creating pool ")]
    CreatePool(#[source] CreatePoolError),
    #[error("Error getting client from pool ")]
    GetClientFromPool(#[source] PoolError),
    // Tokio-postgres errors
    #[error("Unknown exception")]
    UnknownException(#[from] TokioPostgresError),


}

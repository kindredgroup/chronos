use anyhow::Error as anyhow;
use deadpool_postgres::{CreatePoolError, PoolError};
use thiserror::Error as ThisError;
use tokio_postgres::Error as TokioPostgresError;

#[derive(Debug, ThisError)]
pub enum PgError {
    // Deadpool errors
    #[error("Error creating pool ")]
    CreatePool(#[source] CreatePoolError),
    #[error("Error getting client from pool ")]
    GetClientFromPool(#[source] PoolError),
    #[error("Error could not serialize access due to concurrent update ")]
    ConcurrentTxn(#[source] anyhow),
    // Tokio-postgres errors
    #[error("Unknown exception")]
    UnknownException(#[from] TokioPostgresError),
}

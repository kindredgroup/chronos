use chronos_bin::postgres::{config::PgConfig, create_database};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let pg_config = PgConfig::from_env();
    Ok(create_database(pg_config).await?)
}

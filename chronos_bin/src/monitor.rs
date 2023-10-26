use crate::postgres::pg::Pg;
use crate::utils::config::ChronosConfig;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub struct FailureDetector {
    pub(crate) data_store: Arc<Pg>,
}

impl FailureDetector {
    pub async fn run(&self) {
        log::info!("Monitoring On!");
        loop {
            let _ = tokio::time::sleep(Duration::from_secs(ChronosConfig::from_env().monitor_db_poll)).await;

            let _ = &self.monitor_failed_fire_records().await;
        }
    }

    #[tracing::instrument(skip_all, fields(error))]
    async fn reset_to_init_db(&self, fetched_rows: &std::vec::Vec<tokio_postgres::Row>) {
        if !fetched_rows.is_empty() {
            if let Err(e) = &self.data_store.reset_to_init_db(fetched_rows).await {
                tracing::Span::current().record("error", e);
                log::error!("error in monitor reset_to_init {}", e);
            } else {
                log::debug!("reset_to_init_db success for {:?}", fetched_rows)
            }
        }
    }

    #[tracing::instrument(skip_all, fields(error, fail_to_fire_rows))]
    async fn monitor_failed_fire_records(&self) {
        match &self
            .data_store
            .failed_to_fire_db(&(Utc::now() - Duration::from_secs(ChronosConfig::from_env().fail_detect_interval)))
            .await
        {
            Ok(fetched_rows) => {
                tracing::Span::current().record("fail_to_fire_rows", fetched_rows.len());
                self.reset_to_init_db(fetched_rows).await;
            }
            Err(e) => {
                log::error!("error in monitor {}", e);
                tracing::Span::current().record("error", e.to_string());
            }
        }
    }
}

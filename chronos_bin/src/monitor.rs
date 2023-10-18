use crate::postgres::pg::Pg;
use crate::utils::config::ChronosConfig;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub struct FailureDetector {
    pub(crate) data_store: Arc<Pg>,
}

//Needs to accept the poll time
impl FailureDetector {
    // #[instrument]
    pub async fn run(&self) {
        log::info!("Monitoring On!");
        loop {
            // TODO multiple rows are fetched, what to track in the monitor?

            let _ = tokio::time::sleep(Duration::from_secs(ChronosConfig::from_env().monitor_db_poll)).await;

            let _ = &self.monitor_failed().await;
        }
    }
    #[tracing::instrument(skip_all, fields(message_key, error, monitoring_len))]
    async fn monitor_failed(&self) {
        match &self
            .data_store
            .failed_to_fire_db(&(Utc::now() - Duration::from_secs(ChronosConfig::from_env().fail_detect_interval)))
            .await
        {
            Ok(fetched_rows) => {
                if !fetched_rows.is_empty() {
                    if let Err(e) = &self.data_store.reset_to_init_db(fetched_rows).await {
                        tracing::Span::current().record("error", e);
                        println!("error in monitor reset_to_init {}", e);
                    }
                    tracing::Span::current().record("monitoring_len", fetched_rows.len());
                    // TODO Need to monitor the node that redied but never fired
                } else {
                    tracing::Span::current().record("monitoring_len", "empty");
                }
            }
            Err(e) => {
                println!("error in monitor {}", e);
            }
        }
    }
}

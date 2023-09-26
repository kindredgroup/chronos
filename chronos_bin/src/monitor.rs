use crate::postgres::pg::Pg;
use crate::utils::config::ChronosConfig;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error_span, event, field, info_span, instrument, span, trace, Level};

#[derive(Debug)]
pub struct FailureDetector {
    pub(crate) data_store: Arc<Pg>,
}

//Needs to accept the poll time
impl FailureDetector {
    // #[instrument]
    pub async fn run(&self) {
        // println!("Monitoring On!");
        trace!("Monitoring On!");
        loop {
            // TODO multiple rows are fetched, what to track in the monitor?
            let monitor_span = info_span!("failure_detector", records_len = field::Empty, exception = field::Empty);
            let _ = monitor_span.enter();

            let _ = tokio::time::sleep(Duration::from_secs(ChronosConfig::from_env().monitor_db_poll)).await; // sleep for 10sec

            trace!("failed_to_fire On!");
            match &self
                .data_store
                .failed_to_fire(&(Utc::now() - Duration::from_secs(ChronosConfig::from_env().fail_detect_interval)))
                .await
            {
                Ok(fetched_rows) => {
                    if !fetched_rows.is_empty() {
                        if let Err(e) = &self.data_store.reset_to_init(fetched_rows).await {
                            monitor_span.record("exception", e);
                            println!("error in monitor reset_to_init {}", e);
                        }
                        monitor_span.record("records_len", fetched_rows.len());
                    } else {
                        monitor_span.record("records_len", "empty");
                    }
                }
                Err(e) => {
                    println!("error in monitor {}", e);
                }
            }
        }
    }
}

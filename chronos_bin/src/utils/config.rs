#[derive(Debug, Clone)]
pub struct ChronosConfig {
    // pub random_delay: u64,
    pub monitor_db_poll: u64,
    pub processor_db_poll: u64,
    pub time_advance: u64,
    pub fail_detect_interval: u64,
}

impl ChronosConfig {
    pub fn from_env() -> ChronosConfig {
        ChronosConfig {
            // random_delay: env_var!("RANDOMNESS_DELAY").parse().unwrap(),
            monitor_db_poll: std::env::var("MONITOR_DB_POLL")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("Failed to parse MONITOR_DB_POLL!!"),
            processor_db_poll: std::env::var("PROCESSOR_DB_POLL")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("Failed to parse PROCESSOR_DB_POLL!!"),
            time_advance: std::env::var("TIMING_ADVANCE")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .expect("Failed to parse TIMING_ADVANCE!!"),
            fail_detect_interval: std::env::var("FAIL_DETECT_INTERVAL")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("Failed to parse FAIL_DETECT_INTERVAL!!"),
        }
    }
}

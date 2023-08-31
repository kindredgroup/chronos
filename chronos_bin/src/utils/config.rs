#[derive(Debug, Clone)]
pub struct ChronosConfig {
    // pub random_delay: u64,
    pub db_poll_interval: u64,
    pub time_advance: u64,
    pub fail_detect_interval: u64,
}

impl ChronosConfig {
    pub fn from_env() -> ChronosConfig {
        ChronosConfig {
            // random_delay: env_var!("RANDOMNESS_DELAY").parse().unwrap(),
            db_poll_interval: std::env::var("DB_POLL_INTERVAL")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("Failed to parse DB_POLL_INTERVAL!!"),
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

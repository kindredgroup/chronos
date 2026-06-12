mod custom_metrics;
mod otlp_collector;
pub mod prometheus;
pub mod register_telemetry;

#[cfg(test)]
#[path = "prometheus/tests.rs"]
mod prometheus_tests;

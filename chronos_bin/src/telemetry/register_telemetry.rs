use super::metrics;
use super::traces;
use opentelemetry::global;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug)]
pub enum TracesExporterType {
    Otlp,
    NoOp,
}

#[derive(Debug)]
pub enum MetricsExporterType {
    Prometheus,
    NoOp,
}

pub struct TelemetryCollector {
    pub traces_collector_type: TracesExporterType,
    pub metrics_collector_type: MetricsExporterType,
    pub service_name: String,
}

impl TelemetryCollector {
    // https://opentelemetry.io/docs/specs/otel/configuration/sdk-environment-variables/#exporter-selection
    pub fn new() -> Self {
        // Default for metrics and traces for chronos is NoOp
        let trace_exporter = std::env::var("OTEL_TRACES_EXPORTER").unwrap_or("none".to_string());
        let traces_collector_type = match trace_exporter.as_str() {
            "none" => TracesExporterType::NoOp,
            "otlp" => TracesExporterType::Otlp,
            _ => {
                println!("{} is an invalid trace exporter protocol, defaulting to none", trace_exporter);
                TracesExporterType::NoOp
            }
        };
        let metrics_exporter = std::env::var("OTEL_METRICS_EXPORTER").unwrap_or("none".to_string());
        let metrics_collector_type = match metrics_exporter.as_str() {
            "none" => MetricsExporterType::NoOp,
            "prometheus" => MetricsExporterType::Prometheus,
            _ => {
                println!("{} is an invalid metrics exporter protocol, defaulting to none", metrics_exporter);
                MetricsExporterType::NoOp
            }
        };
        let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or("chronos".to_string());
        TelemetryCollector {
            traces_collector_type,
            metrics_collector_type,
            service_name,
        }
    }

    pub fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // The tracer MUST be called first
        // Or we will always pass a no op tracer to the logger,
        // which might not be what you want if using OTLP
        // Will panic on fail
        self.register_tracing();
        // Will panic on fail
        self.register_logging();
        // We can start logging errors
        // Now we exit instead of panic
        self.register_metrics()?;
        Ok(())
    }

    fn register_tracing(&self) {
        match self.traces_collector_type {
            TracesExporterType::Otlp => traces::otlp_exporter::OtlpExporter::new(),
            TracesExporterType::NoOp => traces::noop_exporter::NoOpExporter::new(),
        };
    }

    fn register_logging(&self) {
        let otel_layer = tracing_opentelemetry::layer().with_tracer(global::tracer(self.service_name.clone()));
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        // This will fail if another logger has already been initialized
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .with(otel_layer)
            .try_init()
            .expect("tracing subscriber to global default logging subscriber");
    }

    fn register_metrics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self.metrics_collector_type {
            MetricsExporterType::Prometheus => metrics::prometheus_exporter::PrometheusExporter::new().init()?,
            // Do nothing, global meter will be no op
            MetricsExporterType::NoOp => {}
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::{assert_matches, env};

    const TESTING_ENV_VARS: &[&str] = &["OTEL_SERVICE_NAME", "OTEL_TRACES_EXPORTER", "OTEL_METRICS_EXPORTER"];

    // Could do something cooler with mutexes
    // So we can remove serial
    fn unset_vars(vars: &[&str]) {
        for e in vars.iter() {
            env::remove_var(e);
        }
    }

    #[test]
    #[serial]
    fn test_defaults() {
        unset_vars(TESTING_ENV_VARS);
        let c = TelemetryCollector::new();
        assert_eq!(c.service_name, "chronos".to_string());
        assert_matches!(c.metrics_collector_type, MetricsExporterType::NoOp);
        assert_matches!(c.traces_collector_type, TracesExporterType::NoOp);
        unset_vars(TESTING_ENV_VARS);
    }

    #[test]
    #[serial]
    fn test_invalid_metrics() {
        unset_vars(TESTING_ENV_VARS);
        env::set_var("OTEL_METRICS_EXPORTER", "a_very_invalid_exporter");
        let c = TelemetryCollector::new();
        assert_matches!(c.metrics_collector_type, MetricsExporterType::NoOp);
    }

    #[test]
    #[serial]
    fn test_invalid_tracer() {
        unset_vars(TESTING_ENV_VARS);
        env::set_var("OTEL_TRACES_EXPORTER", "a_very_invalid_exporter");
        let c = TelemetryCollector::new();
        assert_matches!(c.traces_collector_type, TracesExporterType::NoOp);
    }

    #[test]
    #[serial]
    fn test_otlp_trace_exporter() {
        unset_vars(TESTING_ENV_VARS);
        env::set_var("OTEL_TRACES_EXPORTER", "otlp");
        let c = TelemetryCollector::new();
        assert_matches!(c.traces_collector_type, TracesExporterType::Otlp);
    }

    #[test]
    #[serial]
    fn test_prometheus_exporter() {
        unset_vars(TESTING_ENV_VARS);
        env::set_var("OTEL_METRICS_EXPORTER", "prometheus");
        let c = TelemetryCollector::new();
        assert_matches!(c.metrics_collector_type, MetricsExporterType::Prometheus);
    }
}

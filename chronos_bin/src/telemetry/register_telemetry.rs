use super::metrics;
use super::traces;
use opentelemetry::global;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub enum TracesExporterType {
    Otlp,
    NoOp,
}

pub enum MetricsExporterType {
    Prometheus,
    NoOp,
}

pub struct TelemetryCollector {
    pub traces_collector_type: TracesExporterType,
    pub metrics_collector_type: MetricsExporterType,
    pub service_name: String,
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        TelemetryCollector {
            traces_collector_type: TracesExporterType::NoOp,
            metrics_collector_type: MetricsExporterType::NoOp,
            service_name: "chronos".to_string(),
        }
    }
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
                log::warn!("{} is an invalid trace exporter protocol, defaulting to none", trace_exporter);
                TracesExporterType::NoOp
            }
        };
        let metrics_exporter = std::env::var("OTEL_METRICS_EXPORTER").unwrap_or("none".to_string());
        let metrics_collector_type = match metrics_exporter.as_str() {
            "none" => MetricsExporterType::NoOp,
            "prometheus" => MetricsExporterType::Prometheus,
            _ => {
                log::warn!("{} is an invalid metrics exporter protocol, defaulting to none", metrics_exporter);
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
        // which might not be what you want if using OTLP etc
        // Will panic on fail
        self.register_tracing();
        // Will panic on fail
        self.register_logger();
        self.register_metrics()?;
        Ok(())
    }

    fn register_tracing(&self) {
        match self.traces_collector_type {
            TracesExporterType::Otlp => traces::otlp_exporter::OtlpExporter::new(),
            TracesExporterType::NoOp => traces::noop_exporter::NoOpExporter::new(),
        };
    }

    fn register_logger(&self) {
        let otel_layer = tracing_opentelemetry::layer().with_tracer(global::tracer(self.service_name.clone()));
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        // This will fail if another logger has already been initialized
        tracing_subscriber::registry()
            .with(filter)
            // forced stderr logs? How very cloud native of u
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

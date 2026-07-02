use super::metrics;
use super::traces;
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
        let mut f = false;
        if let Err(e) = self.register_metrics() {
            log::error!("failed to start metrics exporter with \"{}\"", e);
            f = true;
        }
        if let Err(e) = self.register_traces() {
            log::error!("failed to start tracer with \"{}\"", e);
            f = true;
        }
        // Start both, crash if either fail
        if f {
            return Err("failed to start telemetry".into());
        }
        Ok(())
    }

    fn register_traces(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tracer = match self.traces_collector_type {
            TracesExporterType::Otlp => traces::otlp_exporter::OtlpExporter::new()?.tracer(self.service_name.as_str()),
            TracesExporterType::NoOp => traces::noop_exporter::NoOpExporter::new()?.tracer(self.service_name.as_str()),
        };
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        let init_result = tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .with(otel_layer)
            .try_init();
        if let Err(e) = init_result {
            eprintln!("failed to initialize tracing subscriber: {e}");
            return Err(Box::new(e));
        };
        Ok(())
    }

    fn register_metrics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self.metrics_collector_type {
            MetricsExporterType::Prometheus => {
                let e = metrics::prometheus_exporter::PrometheusExporter::new()?;
                // ATM this can fail in the background,
                // And there is nothing to restart it
                // or kill the app
                tokio::spawn(async move {
                    if let Err(err) = e.start_web_server().await {
                        log::error!("prometheus server has stopped with error {}", err)
                    }
                });
            }
            MetricsExporterType::NoOp => {
                // Do nothing, global meter will be no op
            }
        };
        Ok(())
    }
}

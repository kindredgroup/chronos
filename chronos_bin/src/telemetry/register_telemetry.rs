use super::otlp_collector::OtlpTracer;
use super::prometheus::PrometheusExporter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub enum TracesExporterType {
    Otlp,
}

pub enum MetricsExporterType {
    Prometheus,
}

pub struct TelemetryCollector {
    pub traces_collector_type: TracesExporterType,
    pub metrics_collector_type: MetricsExporterType,
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        TelemetryCollector {
            traces_collector_type: TracesExporterType::Otlp,
            metrics_collector_type: MetricsExporterType::Prometheus,
        }
    }
}

impl TelemetryCollector {
    pub fn new(traces_collector_type: TracesExporterType, metrics_collector_type: MetricsExporterType) -> Self {
        TelemetryCollector {
            traces_collector_type,
            metrics_collector_type,
        }
    }

    pub fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut f = false;
        if let Err(e) = self.register_metrics() {
            log::error!("failed to start metrics exporter with {}", e);
            f = true;
        }
        if let Err(e) = self.register_traces() {
            log::error!("failed to start tracer with {}", e);
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
            TracesExporterType::Otlp => OtlpTracer::new(),
        };
        match tracer {
            Ok(tracer) => {
                let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer.tracer("chronos"));
                let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
                let init_result = tracing_subscriber::registry()
                    .with(filter)
                    .with(tracing_subscriber::fmt::layer())
                    .with(otel_layer)
                    .try_init();
                if let Err(e) = init_result {
                    eprintln!("failed to initialize tracing subscriber: {e}");
                    return Err(Box::new(e));
                }
            }
            Err(e) => {
                log::error!("error while initializing tracing {}", e);
                return Err(e);
            }
        }
        Ok(())
    }

    fn register_metrics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let exporter = match self.metrics_collector_type {
            MetricsExporterType::Prometheus => PrometheusExporter::new(),
        };
        // ATM simple match because we only expose Prom metrics
        match exporter {
            Ok(e) => {
                // ATM this can die in the back ground
                // Later we'll want to handle this more explicitly
                // (crash if crash)
                tokio::spawn(async move {
                    if let Err(err) = e.start_web_server().await {
                        log::error!("prometheus server has stopped with error {}", err)
                    }
                });
            }
            Err(e) => {
                log::error!("error initializing meter {}", e);
                return Err(e);
            }
        }
        Ok(())
    }
}

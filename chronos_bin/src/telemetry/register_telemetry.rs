use std::panic::{catch_unwind, AssertUnwindSafe};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use super::otlp_collector::OtlpCollector;

pub enum TelemetryCollectorType {
    Otlp,
}

pub struct TelemetryCollector {
    pub collector_type: TelemetryCollectorType,
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        TelemetryCollector {
            collector_type: TelemetryCollectorType::Otlp,
        }
    }
}

impl TelemetryCollector {
    pub fn new(collector_type: TelemetryCollectorType) -> Self {
        TelemetryCollector { collector_type }
    }

    pub fn register_traces(self) {
        let protocol = std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL").unwrap_or_else(|_| "grpc".to_string());
        let traces_endpoint = std::env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT").unwrap_or_else(|_| "<unset>".to_string());
        let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "<unset>".to_string());
        log::info!(
            "[telemetry] starting register_traces collector=Otlp protocol={} endpoint={} service={}",
            protocol,
            traces_endpoint,
            service_name
        );
        let tracer_result = catch_unwind(AssertUnwindSafe(|| {
            let otlp_collector = OtlpCollector::new();
            if protocol.to_lowercase().contains("grpc") {
                log::debug!("[telemetry] using grpc_collector_connect");
                otlp_collector.grpc_collector_connect()
            } else {
                log::debug!("[telemetry] using http_collector_connect");
                otlp_collector.http_collector_connect()
            }
        }));
        let tracer = match tracer_result {
            Ok(result) => result,
            Err(panic_payload) => {
                log::error!("[telemetry] PANIC while creating tracer: {:?}", panic_payload);
                return; // keep app alive, logs still work
            }
        };
        match tracer {
            Ok(tracer) => {
                log::debug!("[telemetry] tracer created successfully, initializing subscriber");
                let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
                let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
                let init_result = tracing_subscriber::registry()
                    .with(filter)
                    .with(tracing_subscriber::fmt::layer())
                    .with(otel_layer)
                    .try_init();
                if let Err(e) = init_result {
                    log::error!("[telemetry] subscriber init failed: {e}");
                } else {
                    log::debug!("[telemetry] subscriber init OK");
                }
            }
            Err(e) => {
                log::error!("[telemetry] tracer creation returned error: {e}");
                let mut source = e.source();
                while let Some(s) = source {
                    log::error!("[telemetry]   caused by: {s}");
                    source = s.source();
                }
            }
        }
    }
}

use opentelemetry_otlp::Protocol;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use super::{jaegar_backend::instrument_jaegar_pipleline, otlp_collector::OtlpCollector};

pub enum TelemetryCollectorType {
    Jaegar,
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
    pub fn new() -> Self {
        TelemetryCollector::default()
    }

    pub fn register_traces(self) {
        let tracer = match &self.collector_type {
            TelemetryCollectorType::Jaegar => instrument_jaegar_pipleline(),
            TelemetryCollectorType::Otlp => {
                let otlp_collector = OtlpCollector::new();
                otlp_collector.http_collector_connect(Protocol::HttpBinary)
            }
        };

        match tracer {
            Ok(tracer) => {
                //creating a layer for Otel
                let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

                //subscribing to tracing with opentelemetry
                match tracing_subscriber::registry().with(otel_layer).try_init() {
                    Ok(_) => {}
                    Err(e) => {
                        println!(" {}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("error while initializing tracing {}", e);
            }
        }
    }
}

use opentelemetry_otlp::Protocol;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

use super::{jaegar_backend::instrument_jaegar_pipleline, otlp_collector::OtlpCollector};

pub enum TelemetryCollectorType {
    Jaegar,
    Otlp,
}

pub struct TelemetryCollector {
    pub collector_type: TelemetryCollectorType,
    pub protocol: Protocol,
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        TelemetryCollector {
            collector_type: TelemetryCollectorType::Otlp,
            protocol: Protocol::HttpBinary,
        }
    }
}

impl TelemetryCollector {
    pub fn new(env_protocol: String, collector_type: TelemetryCollectorType) -> Self {
        let protocol = if env_protocol.to_lowercase().contains("grpc") {
            Protocol::Grpc
        } else {
            Protocol::HttpBinary
        };
        TelemetryCollector { collector_type, protocol }
    }

    pub fn register_traces(self) {
        let tracer = match &self.collector_type {
            TelemetryCollectorType::Jaegar => instrument_jaegar_pipleline(),
            TelemetryCollectorType::Otlp => match self.protocol {
                Protocol::Grpc => todo!(),
                Protocol::HttpBinary => {
                    let otlp_collector = OtlpCollector::new();
                    otlp_collector.http_collector_connect(Protocol::HttpBinary)
                }
            },
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

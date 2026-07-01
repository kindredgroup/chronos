use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{SdkTracerProvider, Tracer};

pub struct OtlpTracer {
    provider: SdkTracerProvider,
}

impl OtlpTracer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // All of this env var parsing should be removed in favour
        // of auto discovery by the OTEL SDK's.
        // Doing this ourselves open us up to spec drift
        // tl;dr
        // Install HTTP and gRPC exporters and let the clients pick through
        // the env vars.
        // I'll revisit later
        let endpoint = std::env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT")
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT not set"))?;
        let spanbuilder = opentelemetry_otlp::SpanExporter::builder();
        let protocol = std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL").unwrap_or_else(|_| "grpc".to_string());
        let exp = if protocol.to_lowercase().contains("grpc") {
            spanbuilder.with_tonic().with_endpoint(endpoint).build()?
        } else {
            spanbuilder.with_http().with_endpoint(endpoint).build()?
        };
        let provider = SdkTracerProvider::builder().with_batch_exporter(exp).build();
        global::set_text_map_propagator(TraceContextPropagator::new());
        global::set_tracer_provider(provider.clone());
        Ok(Self { provider })
    }
    pub fn tracer(&self, name: &'static str) -> Tracer {
        self.provider.tracer(name.to_string())
    }
}

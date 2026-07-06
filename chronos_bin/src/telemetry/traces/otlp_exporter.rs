use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;

pub struct OtlpExporter {}

impl OtlpExporter {
    pub fn new() {
        // Panic if fails to build when OTLP is required
        // Force the use of the batch exporter
        let provider = SdkTracerProvider::builder()
            .with_batch_exporter(opentelemetry_otlp::SpanExporter::builder().build().expect("OTLP span builder"))
            .build();
        global::set_text_map_propagator(TraceContextPropagator::new());
        global::set_tracer_provider(provider.clone());
    }
}

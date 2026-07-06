use opentelemetry::global;
use opentelemetry_sdk::trace::SdkTracerProvider;

pub struct NoOpExporter {}

impl NoOpExporter {
    pub fn new() {
        // This is redundant, keeping it for clarity
        let provider = SdkTracerProvider::builder().build();
        global::set_tracer_provider(provider.clone());
    }
}

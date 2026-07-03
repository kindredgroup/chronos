use opentelemetry::global;
use opentelemetry_sdk::trace::SdkTracerProvider;

pub struct NoOpExporter {}

impl NoOpExporter {
    pub fn new() {
        let provider = SdkTracerProvider::builder().build();
        global::set_tracer_provider(provider.clone());
    }
}

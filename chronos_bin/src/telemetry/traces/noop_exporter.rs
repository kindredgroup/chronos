use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::{SdkTracerProvider, Tracer};

pub struct NoOpExporter {
    provider: SdkTracerProvider,
}

impl NoOpExporter {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let provider = SdkTracerProvider::builder().build();
        // Not sure if we need to do this as global should be no op,
        // makes it clear for the reader tho
        global::set_tracer_provider(provider.clone());
        Ok(NoOpExporter { provider })
    }
    pub fn tracer(&self, name: &str) -> Tracer {
        self.provider.tracer(name.to_string())
    }
}

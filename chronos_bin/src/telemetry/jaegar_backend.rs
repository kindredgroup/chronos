use opentelemetry_api::trace::TraceError;
use opentelemetry_sdk::trace::Tracer;

pub fn instrument_jaegar_pipleline() -> Result<Tracer, TraceError> {
    let service_name = std::env::var("OTEL_SERVICE_NAME");
    if service_name.is_err() {
        std::env::set_var("OTEL_SERVICE_NAME", "chronos");
    }
    opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(format!("{:?}", service_name))
        .install_simple()
}

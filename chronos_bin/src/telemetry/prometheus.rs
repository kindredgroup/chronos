use std::env;
use std::io;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use tokio::task::JoinHandle;

use super::custom_metrics::register_custom_metrics;
use crate::utils::env::get_env_var_value;

pub(super) const DEFAULT_PROMETHEUS_HOST: &str = "localhost";
pub(super) const DEFAULT_PROMETHEUS_PORT: u16 = 9464;
pub(super) const PROMETHEUS_HOST_ENV: &str = "OTEL_EXPORTER_PROMETHEUS_HOST";
pub(super) const PROMETHEUS_PORT_ENV: &str = "OTEL_EXPORTER_PROMETHEUS_PORT";
pub(super) const OPENMETRICS_CONTENT_TYPE: &str = "application/openmetrics-text; version=1.0.0; charset=utf-8";

#[derive(Debug, Eq, PartialEq)]
pub(super) struct PrometheusConfig {
    pub(super) host: String,
    pub(super) port: u16,
}

impl PrometheusConfig {
    pub(super) fn from_env() -> Self {
        // get_env_var_value panics if the var is missing
        // We pre-check for the var to avoid a panic
        // I am using get_env_var_value because its a convention in the codebase
        // but this attempts to avoid the panic by hitting defaults.
        // If its prefer I can write my own get_env_var_value for this
        let host = if env::var_os(PROMETHEUS_HOST_ENV).is_some() {
            let value = get_env_var_value(PROMETHEUS_HOST_ENV).unwrap();
            if value.is_empty() {
                log::warn!("{PROMETHEUS_HOST_ENV} is empty using {DEFAULT_PROMETHEUS_HOST}");
                DEFAULT_PROMETHEUS_HOST.to_string()
            } else {
                value
            }
        } else {
            log::warn!("{PROMETHEUS_HOST_ENV} not found using {DEFAULT_PROMETHEUS_HOST}");
            DEFAULT_PROMETHEUS_HOST.to_string()
        };
        let port = if env::var_os(PROMETHEUS_PORT_ENV).is_some() {
            let value = get_env_var_value(PROMETHEUS_PORT_ENV).unwrap();
            match value.parse::<u16>() {
                Ok(port) => port,
                Err(error) => {
                    log::warn!("invalid {PROMETHEUS_PORT_ENV} value {value:?}: {error}; using {DEFAULT_PROMETHEUS_PORT}");
                    DEFAULT_PROMETHEUS_PORT
                }
            }
        } else {
            log::warn!("{PROMETHEUS_PORT_ENV} not found using {DEFAULT_PROMETHEUS_PORT}");
            DEFAULT_PROMETHEUS_PORT
        };
        Self { host, port }
    }

    fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

pub(super) fn create_registry() -> Registry {
    let mut registry = Registry::default();
    register_custom_metrics(&mut registry);
    return registry;
}

pub(super) fn app(registry: Arc<Registry>) -> Router {
    Router::new().route("/metrics", get(metrics_handler)).fallback(not_found).with_state(registry)
}

async fn metrics_handler(State(registry): State<Arc<Registry>>) -> Response {
    let mut body = String::new();
    match encode(&mut body, &registry) {
        Ok(()) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, OPENMETRICS_CONTENT_TYPE)
            .body(Body::from(body))
            .expect("valid metrics response"),
        Err(error) => {
            log::error!("failed to encode Prometheus metrics: {error}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn not_found(_uri: Uri) -> StatusCode {
    StatusCode::NOT_FOUND
}

pub async fn start_prometheus_server() -> io::Result<JoinHandle<()>> {
    let config = PrometheusConfig::from_env();
    let address = config.address();
    let listener = tokio::net::TcpListener::bind(&address).await?;
    let registry = Arc::new(create_registry());

    log::info!("Prometheus metrics server listening on {address}");

    Ok(tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app(registry)).await {
            log::error!("Prometheus metrics server stopped: {error}");
        }
    }))
}

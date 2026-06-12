use super::prometheus::{
    app, create_registry, PrometheusConfig, DEFAULT_PROMETHEUS_HOST, DEFAULT_PROMETHEUS_PORT, OPENMETRICS_CONTENT_TYPE, PROMETHEUS_HOST_ENV,
    PROMETHEUS_PORT_ENV,
};
use axum::body::to_bytes;
use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::Request;
use axum::http::StatusCode;
use prometheus_client::encoding::text::encode;
use serial_test::serial;
use std::env;
use std::sync::Arc;
use tower::ServiceExt;

fn clear_prometheus_env() {
    env::remove_var(PROMETHEUS_HOST_ENV);
    env::remove_var(PROMETHEUS_PORT_ENV);
}

#[test]
fn empty_registry_encodes_without_metrics() {
    let mut output = String::new();

    encode(&mut output, &create_registry()).unwrap();

    assert_eq!(output, "# EOF\n");
}

#[tokio::test]
async fn metrics_endpoint_returns_openmetrics_response() {
    let response = app(Arc::new(create_registry()))
        .oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], OPENMETRICS_CONTENT_TYPE);
    assert_eq!(to_bytes(response.into_body(), usize::MAX).await.unwrap(), "# EOF\n");
}

#[test]
#[serial]
fn config_uses_opentelemetry_defaults() {
    clear_prometheus_env();

    assert_eq!(
        PrometheusConfig::from_env(),
        PrometheusConfig {
            host: DEFAULT_PROMETHEUS_HOST.to_string(),
            port: DEFAULT_PROMETHEUS_PORT,
        }
    );
}

#[test]
#[serial]
fn config_uses_opentelemetry_overrides() {
    clear_prometheus_env();
    env::set_var(PROMETHEUS_HOST_ENV, "0.0.0.0");
    env::set_var(PROMETHEUS_PORT_ENV, "9090");

    assert_eq!(
        PrometheusConfig::from_env(),
        PrometheusConfig {
            host: "0.0.0.0".to_string(),
            port: 9090,
        }
    );

    clear_prometheus_env();
}

#[test]
#[serial]
fn empty_and_invalid_values_use_defaults() {
    clear_prometheus_env();
    env::set_var(PROMETHEUS_HOST_ENV, "");
    env::set_var(PROMETHEUS_PORT_ENV, "invalid");

    assert_eq!(
        PrometheusConfig::from_env(),
        PrometheusConfig {
            host: DEFAULT_PROMETHEUS_HOST.to_string(),
            port: DEFAULT_PROMETHEUS_PORT,
        }
    );

    clear_prometheus_env();
}

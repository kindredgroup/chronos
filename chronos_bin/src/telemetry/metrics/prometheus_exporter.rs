use http_body_util::Full;
use hyper::{body::Bytes, header::CONTENT_TYPE, service::service_fn, Method, Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use opentelemetry::global;
use opentelemetry_prometheus::exporter;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Encoder, Registry, TextEncoder};
use tokio::net::TcpListener;

// https://opentelemetry.io/docs/specs/otel/configuration/sdk-environment-variables/#prometheus-exporter
const OTEL_EXPORTER_PROMETHEUS_PORT: &str = "OTEL_EXPORTER_PROMETHEUS_PORT";
const OTEL_EXPORTER_PROMETHEUS_DEFAULT_PORT: u16 = 9464;
const OTEL_EXPORTER_PROMETHEUS_HOST: &str = "OTEL_EXPORTER_PROMETHEUS_HOST";
const OTEL_EXPORTER_PROMETHEUS_DEFAULT_HOST: &str = "localhost";

pub struct PrometheusExporter {}

impl PrometheusExporter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let registry = Registry::new();
        let provider = SdkMeterProvider::builder()
            .with_reader(exporter().with_registry(registry.clone()).build()?)
            .build();
        global::set_meter_provider(provider.clone());
        let r = registry.clone();
        // We should really watch this guy and make sure it doesn't die
        tokio::spawn(async move {
            if let Err(err) = start_web_server(r).await {
                log::error!("prometheus server has stopped with error {}", err)
            }
        });
        Ok(())
    }
}

async fn start_web_server(register: Registry) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let hostname = std::env::var(OTEL_EXPORTER_PROMETHEUS_HOST).unwrap_or(OTEL_EXPORTER_PROMETHEUS_DEFAULT_HOST.to_string());
    let port = std::env::var(OTEL_EXPORTER_PROMETHEUS_PORT)
        .ok()
        .and_then(|port| {
            port.parse::<u16>()
                .map_err(|e| {
                    log::warn!(
                        "failed to parse {} with {}, using default port {}",
                        OTEL_EXPORTER_PROMETHEUS_PORT,
                        e,
                        OTEL_EXPORTER_PROMETHEUS_DEFAULT_PORT.to_string()
                    )
                })
                .ok()
        })
        .unwrap_or(OTEL_EXPORTER_PROMETHEUS_DEFAULT_PORT);
    log::info!("starting prometheus server on {}:{}", hostname, port);
    let listener = TcpListener::bind((hostname, port)).await;
    match listener {
        Ok(l) => {
            while let Ok((stream, _addr)) = l.accept().await {
                if let Err(err) = Builder::new(TokioExecutor::new())
                    .serve_connection(TokioIo::new(stream), service_fn(|req| serve_req(req, register.clone())))
                    .await
                {
                    log::error!("error serving prometheus metics {err}")
                }
            }
        }
        Err(e) => {
            log::error!("error binding to prometheus port with {}", e);
            return Err("failed to bind prometheus address".into());
        }
    }
    log::error!("stopping prometheus server");
    Ok(())
}

async fn serve_req<B>(r: Request<B>, register: Registry) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let resp = match (r.method(), r.uri().path()) {
        (&Method::GET, "/metrics") => {
            let mut buffer = vec![];
            let encoder = TextEncoder::new();
            let reg = register.gather();
            let encode = encoder.encode(&reg, &mut buffer);
            match encode {
                Ok(_) => Response::builder()
                    .status(200)
                    .header(CONTENT_TYPE, encoder.format_type())
                    .body(Full::new(Bytes::from(buffer)))
                    .unwrap(),
                Err(err) => {
                    log::error!("error encoding prometheus metrics {}", err);
                    Response::builder()
                        .status(500)
                        .body(
                            Full::new("Internal Server Error".into()), // Panic if the HTTP lib tries
                                                                       // to build an invalid HTTP response
                        )
                        .unwrap()
                }
            }
        }
        _ => Response::builder()
            .status(404)
            .body(Full::new("Not found".into()))
            // Panic if the HTTP lib tries
            // to build an invalid HTTP response
            .unwrap(),
    };
    Ok(resp)
}

#[cfg(test)]
mod test {
    use super::*;
    use prometheus::TEXT_FORMAT;

    #[tokio::test]
    async fn test_serve_paths() {
        assert_eq!(
            serve_req(
                hyper::Request::builder().uri("/").method("GET").body(Full::<Bytes>::new("".into())).unwrap(),
                prometheus::Registry::new()
            )
            .await
            .unwrap()
            .status(),
            404
        );
        let req = serve_req(
            hyper::Request::builder()
                .uri("/metrics")
                .method("GET")
                .body(Full::<Bytes>::new("".into()))
                .unwrap(),
            prometheus::Registry::new(),
        )
        .await
        .unwrap();
        assert_eq!(req.status(), 200);
        assert!(req.headers().contains_key(CONTENT_TYPE) && req.headers()[CONTENT_TYPE] == TEXT_FORMAT);
    }
}

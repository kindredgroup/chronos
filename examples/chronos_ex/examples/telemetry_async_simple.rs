use tracing::info_span;
use tracing_subscriber::prelude::*;

struct B {}

impl B {
    pub async fn run(&self) {
        let span_sub_runer = info_span!("B");
        let _guard = span_sub_runer.enter();
        println!("B");
    }
}

struct A {}

impl A {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn run(&self) {
        let span_runer = info_span!("A");
        let _guard = span_runer.enter();
        println!("A");
        let sub_runner = B {};
        sub_runner.run().await;
    }
}

struct Runner {}

impl Runner {
    pub async fn run(&self) {
        let span_runer = info_span!("Runner");
        let _guard = span_runer.enter();
        println!("Runner");
        // A::new().run().await;
        let handler = tokio::task::spawn(async {
            println!("this is spawning");
            A::new().run().await;
        });
        // let handler_one = tokio::task::spawn(async {
        //     let runner = Runner {};
        //     runner.run().await;
        // });

        futures::future::join_all([handler]).await;
    }
}

#[tokio::main]
async fn main() {
    // env_logger::init();
    dotenv::dotenv().ok();

    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("test_async_tel")
        .install_simple()
        .expect("failed to install tracing");

    //creating a layer for Otel
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // let subscriber = Registry::default().with(otel_layer);

    //subscribing to tracing with opentelemetry
    match tracing_subscriber::registry().with(otel_layer).try_init() {
        Ok(_) => {}
        Err(e) => {
            println!("error while initializing tracing {}", e);
        }
    }

    let runner = Runner {};
    runner.run().await;
}

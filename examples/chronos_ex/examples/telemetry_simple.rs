use tracing::info_span;
use tracing_subscriber::prelude::*;

struct SubRunner {}

impl SubRunner {
    pub fn run(&self) {
        let span_sub_runer = info_span!("sub_hello_world");
        let _guard = span_sub_runer.enter();
        println!("Hello, under world!");
        let sub_runner = SubRunnerOne {};
        sub_runner.run();
    }
}
struct SubRunnerOne {}

impl SubRunnerOne {
    pub fn run(&self) {
        let span_sub_runer = info_span!("subone_hello_world");
        let _guard = span_sub_runer.enter();
        println!("Hello, sub one under world!");
    }
}

struct Runner {}

impl Runner {
    pub async fn run(&self) {
        let span_runer = info_span!("hello_world Runner");
        let _guard = span_runer.enter();
        println!("Hello, world!");
        let sub_runner = SubRunner {};
        sub_runner.run();

        // let sub_runner_one = SubRunnerOne {};
        // sub_runner_one.run();
    }
}

#[tokio::main]
async fn main() {
    // env_logger::init();
    dotenv::dotenv().ok();

    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("test_tel")
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

    let handler = tokio::task::spawn(async {
        println!("this is spawning");
        let runner = Runner {};
        runner.run().await;
    });
    // let handler_one = tokio::task::spawn(async {
    //     let runner = Runner {};
    //     runner.run().await;
    // });

    futures::future::join_all([handler]).await;

    // let runner = Runner {};
    // runner.run();
}

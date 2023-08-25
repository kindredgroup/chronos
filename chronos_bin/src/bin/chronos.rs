use chronos_bin::kafka::config::KafkaConfig;
use chronos_bin::kafka::consumer::KafkaConsumer;
use chronos_bin::kafka::producer::KafkaProducer;
use chronos_bin::postgres::config::PgConfig;
use chronos_bin::postgres::pg::Pg;
use chronos_bin::runner::Runner;
use log::debug;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenvy::dotenv().ok();

    let kafka_config = KafkaConfig::from_env();
    let pg_config = PgConfig::from_env();

    let kafka_consumer = KafkaConsumer::new(&kafka_config);
    let kafka_producer = KafkaProducer::new(&kafka_config);
    let data_store = Pg::new(pg_config).await.unwrap();

    let r = Runner {
        data_store: Arc::new(Box::new(data_store)),
        producer: Arc::new(Box::new(kafka_producer)),
        consumer: Arc::new(Box::new(kafka_consumer)),
    };

    debug!("debug logs starting chronos");

    r.run().await;
}

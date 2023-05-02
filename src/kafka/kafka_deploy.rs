use std::time::Duration;

use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    client::DefaultClientContext,
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
    types::RDKafkaErrorCode,
};
use thiserror::Error as ThisError;
use crate::kafka::config::KafkaConfig;

pub enum KafkaDeployStatus {
    TopicExists,
    TopicCreated,
}

#[derive(Debug, Clone, ThisError)]
pub enum KafkaDeployError {
    #[error("Failed to create topic {0} with kafka error code={1}")]
    TopicCreation(String, RDKafkaErrorCode),
    #[error(transparent)]
    KafkaError(#[from] KafkaError),
}

pub async fn create_topics(replication_factor: i32, num_of_partitions: i32) -> Result<(), KafkaDeployError> {
    let kafka_config = KafkaConfig::from_env();
    println!("kafka configs received from env... {kafka_config:#?}");

    let _ = create_topic(&kafka_config, kafka_config.in_topic.as_str(),replication_factor, num_of_partitions).await?;
    let _ = create_topic(&kafka_config, kafka_config.out_topic.as_str(),replication_factor, num_of_partitions).await?;
    Ok(())
}

async fn create_topic(kafka_config: &KafkaConfig, topic_name: &str, replication_factor: i32, num_of_partitions: i32) -> Result<KafkaDeployStatus, KafkaDeployError> {

    let consumer: StreamConsumer = kafka_config.build_consumer_config().create()?;
    let timeout = Duration::from_secs(1);
    let metadata = consumer
        .fetch_metadata(Some(topic_name), timeout)
        .expect("Fetching topic metadata failed");

    if !metadata.topics().is_empty() && !metadata.topics()[0].partitions().is_empty() {
        println!("Topic {topic_name} exists");
        Ok(KafkaDeployStatus::TopicExists)
    } else {
        println!("Topic does not exist, creating...");
        let topic = NewTopic {
            name: topic_name,
            num_partitions: num_of_partitions,
            replication: TopicReplication::Fixed(replication_factor),
            config: vec![("message.timestamp.type", "LogAppendTime")],
        };

        let opts = AdminOptions::new().operation_timeout(Some(timeout));

        let admin: AdminClient<DefaultClientContext> = kafka_config.build_consumer_config().create()?;

        let results = admin.create_topics(&[topic], &opts).await?;

        results[0]
            .as_ref()
            .map_err(|e| KafkaDeployError::TopicCreation(topic_name.to_string(), e.1))?;
        println!("Topic : {topic_name} created");
        Ok(KafkaDeployStatus::TopicCreated)
    }
}

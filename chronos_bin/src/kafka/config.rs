use crate::env_var;
use rdkafka::ClientConfig;
use std::collections::HashMap;

#[derive(Debug)]
pub struct KafkaConfig {
    pub host: Vec<String>,
    pub port: String,
    pub in_topic: String,
    pub out_topic: String,
    pub client_id: String,
    pub group_id: String,
    pub username: String,
    pub password: String,
    pub producer_config_overrides: HashMap<&'static str, &'static str>,
    pub consumer_config_overrides: HashMap<&'static str, &'static str>,
}

impl KafkaConfig {
    pub fn from_env() -> Self {
        KafkaConfig {
            host: env_var!("KAFKA_HOST", Vec<String>),
            port: env_var!("KAFKA_PORT"),
            in_topic: env_var!("KAFKA_IN_TOPIC"),
            out_topic: env_var!("KAFKA_OUT_TOPIC"),
            client_id: env_var!("KAFKA_CLIENT_ID"),
            group_id: env_var!("KAFKA_GROUP_ID"),
            username: env_var!("KAFKA_USERNAME"),
            password: env_var!("KAFKA_PASSWORD"),
            producer_config_overrides: HashMap::new(),
            consumer_config_overrides: HashMap::new(),
        }
    }

    pub fn set_overrides(
        &mut self,
        producer_config_overrides: HashMap<&'static str, &'static str>,
        consumer_config_overrides: HashMap<&'static str, &'static str>,
    ) {
        self.producer_config_overrides = producer_config_overrides;
        self.consumer_config_overrides = consumer_config_overrides;
    }

    pub fn build_consumer_config(&self) -> ClientConfig {
        let mut client_config = ClientConfig::new();

        let username = self.username.to_owned();
        let password = self.password.to_owned();
        let brokers = &self.host;
        let mut brokers = brokers.to_owned();
        for elm in brokers.iter_mut() {
            elm.push(':');
            elm.push_str(&self.port);
        }
        let brokers = brokers.join(",");
        let mut base_config = HashMap::from([
            ("group.id", self.group_id.as_str()),
            ("bootstrap.servers", brokers.as_str()),
            ("auto.offset.reset", "earliest"),
            ("socket.keepalive.enable", "true"),
            ("enable.auto.commit", "true"),
            ("auto.commit.interval.ms", "10"),
        ]);

        base_config.extend(&self.consumer_config_overrides);

        for (k, v) in base_config.into_iter() {
            client_config.set(k, v);
        }

        if !username.is_empty() && !password.is_empty() {
            client_config
                .set("security.protocol", "SASL_PLAINTEXT")
                .set("sasl.mechanisms", "SCRAM-SHA-512")
                .set("sasl.username", username)
                .set("sasl.password", password);
        }

        client_config
    }

    pub fn build_producer_config(&self) -> ClientConfig {
        let mut client_config = ClientConfig::new();

        let username = self.username.to_owned();
        let password = self.password.to_owned();
        let brokers = &self.host;
        let mut brokers = brokers.to_owned();
        for elm in brokers.iter_mut() {
            elm.push(':');
            elm.push_str(&self.port);
        }
        let brokers = brokers.join(",");
        let mut base_config = HashMap::from([
            ("message.timeout.ms", "30000"),
            ("bootstrap.servers", brokers.as_str()),
            ("message.send.max.retries", "100000"),
        ]);
        base_config.extend(&self.producer_config_overrides);
        for (k, v) in base_config.into_iter() {
            client_config.set(k, v);
        }
        if !username.is_empty() && !password.is_empty() {
            client_config
                .set("security.protocol", "SASL_PLAINTEXT")
                .set("sasl.mechanisms", "SCRAM-SHA-512")
                .set("sasl.username", username)
                .set("sasl.password", password);
        }

        client_config
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, env};

    use serial_test::serial;

    use super::*;

    fn set_env_var(key: &str, value: &str) {
        env::set_var(key, value)
    }

    fn unset_env_var(key: &str) {
        env::remove_var(key)
    }

    fn get_kafka_env_variables() -> HashMap<&'static str, &'static str> {
        let env_hashmap = [
            ("KAFKA_HOST", "broker1, broker2 "),
            ("KAFKA_PORT", "port"),
            ("KAFKA_IN_TOPIC", "some-topic"),
            ("KAFKA_OUT_TOPIC", "some-topic"),
            ("KAFKA_CLIENT_ID", "some-client-id"),
            ("KAFKA_GROUP_ID", "some-group-id"),
            ("KAFKA_USERNAME", ""),
            ("KAFKA_PASSWORD", ""),
        ];
        HashMap::from(env_hashmap)
    }

    fn build_test_kafka_config() -> KafkaConfig {
        KafkaConfig {
            host: vec!["broker1".to_string()],
            in_topic: "in_topic".to_owned(),
            out_topic: "out_topic".to_owned(),
            client_id: "client-id-1".to_string(),
            group_id: "group-id-1".to_string(),
            username: "user_name".to_owned(),
            password: "password".to_owned(),
            producer_config_overrides: Default::default(),
            consumer_config_overrides: Default::default(),
            port: "3000".to_string(),
        }
    }

    #[test]
    #[serial]
    fn test_from_env_gets_values_successully() {
        get_kafka_env_variables().iter().for_each(|(k, v)| {
            set_env_var(k, v);
        });

        let config = KafkaConfig::from_env();

        assert_eq!(config.client_id, "some-client-id");
        assert_eq!(config.host.len(), 2);

        get_kafka_env_variables().iter().for_each(|(k, _)| {
            unset_env_var(k);
        });
    }
    #[test]
    #[serial]
    #[should_panic(expected = "KAFKA_IN_TOPIC environment variable is not defined")]
    fn test_from_env_when_env_variable_not_found() {
        get_kafka_env_variables().iter().for_each(|(k, v)| {
            set_env_var(k, v);
        });

        unset_env_var("KAFKA_IN_TOPIC");

        let _config = KafkaConfig::from_env();

        get_kafka_env_variables().iter().for_each(|(k, _)| {
            unset_env_var(k);
        });
    }

    #[test]
    fn test_build_consumer_config_obj() {
        let config = build_test_kafka_config().build_consumer_config();
        assert_eq!(config.get("group.id").unwrap(), "group-id-1");
        assert_eq!(config.get("socket.keepalive.enable").unwrap(), "true");
        assert_eq!(config.get("sasl.username").unwrap(), "user_name");
    }
    #[test]
    fn test_passing_credentials_to_build_consumer_config() {
        let config = KafkaConfig {
            host: vec!["broker1".to_string()],
            in_topic: "consumer-topic-1".to_owned(),
            out_topic: "consumer-topic-2".to_owned(),
            client_id: "client-id-1".to_string(),
            group_id: "groud-id-1".to_string(),
            username: "user".to_string(),
            password: "password".to_string(),
            producer_config_overrides: Default::default(),
            consumer_config_overrides: Default::default(),
            port: "3000".to_string(),
        };
        let client_config = config.build_consumer_config();
        assert_eq!(client_config.get("auto.offset.reset").unwrap(), "earliest");
        assert_eq!(client_config.get("socket.keepalive.enable").unwrap(), "true");
        assert_eq!(client_config.get("enable.auto.commit").unwrap(), "true");
        assert_eq!(client_config.get("sasl.username").unwrap(), "user");
        assert_eq!(client_config.get("sasl.password").unwrap(), "password");
        assert_eq!(client_config.get("security.protocol").unwrap(), "SASL_PLAINTEXT");
    }

    #[test]
    fn test_build_producer_config_obj() {
        let config = build_test_kafka_config().build_producer_config();
        assert!(config.get("group.id").is_none());
        assert_eq!(config.get("sasl.username").unwrap(), "user_name");
        assert_eq!(config.get("sasl.password").unwrap(), "password");
        assert_eq!(config.get("message.timeout.ms").unwrap(), "30000");
        assert_eq!(config.get("message.send.max.retries").unwrap(), "100000");
    }
    #[test]
    fn test_passing_overrides() {
        let mut kafka_config = KafkaConfig {
            host: vec!["broker1".to_string()],
            in_topic: "in_topic".to_owned(),
            out_topic: "out_topic".to_owned(),
            client_id: "client-id-1".to_string(),
            group_id: "groud-id-1".to_string(),
            username: "user".to_string(),
            password: "password".to_string(),
            producer_config_overrides: Default::default(),
            consumer_config_overrides: Default::default(),
            port: "3000".to_string(),
        };
        let producer_override = HashMap::from([("message.timeout.ms", "10")]);
        let consumer_override = HashMap::from([("auto.offset.reset", "latest")]);
        kafka_config.set_overrides(producer_override, consumer_override);
        let producer_config = kafka_config.build_producer_config();
        assert_eq!(producer_config.get("sasl.username").unwrap(), "user");
        assert_eq!(producer_config.get("message.timeout.ms").unwrap(), "10");
        let consumer_config = kafka_config.build_consumer_config();
        assert_eq!(consumer_config.get("sasl.username").unwrap(), "user");
        assert_eq!(consumer_config.get("auto.offset.reset").unwrap(), "latest");
    }
}

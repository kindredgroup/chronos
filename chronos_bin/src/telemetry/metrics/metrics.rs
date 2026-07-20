use super::super::register_telemetry::DEFAULT_OTEL_SERVICE_NAME;
use opentelemetry::{
    global,
    metrics::{Counter, Histogram},
    KeyValue,
};
use std::sync::LazyLock;

struct Metrics {
    msg_consume_seconds: Histogram<f64>,
    msg_consume_latency_seconds: Histogram<f64>,
    msg_publish_seconds: Histogram<f64>,
    msg_jitter_seconds: Histogram<f64>,
    msg_resets: Counter<u64>,
}

impl Metrics {
    fn new() -> Self {
        let meter = global::meter(DEFAULT_OTEL_SERVICE_NAME);
        Self {
            // input consumption metrics
            msg_consume_seconds: meter
                .f64_histogram("msg.consume.service.time")
                .with_description("Service time after receiving a message from the input queue")
                .with_unit("s")
                .build(),
            msg_consume_latency_seconds: meter // Aka "lag time"
                .f64_histogram("msg.consume.latency")
                .with_description("Message latency on the input queue")
                .with_unit("s")
                .build(),
            msg_publish_seconds: meter
                .f64_histogram("msg.process.service.time")
                .with_description("The service time of \"message ready\" processing loop. Only increments when messages are found")
                .with_unit("s")
                .build(),
            msg_jitter_seconds: meter
                .f64_histogram("msg.jitter")
                .with_unit("s")
                .with_description("The delta between the desired published time and confirmed published time of messages to the output topic")
                .build(),
            msg_resets: meter.u64_counter("msg.reset").with_description("The count of messages reset").build(),
        }
    }
}

static METRICS: LazyLock<Metrics> = LazyLock::new(Metrics::new);

pub enum ConsumedMessageDestinations {
    KAFKA,
    DATABASE,
    DROPPED,
}

pub enum Status {
    SUCCESS,
    ERROR,
}

pub fn record_msg_consume(process_time: f64, destination: ConsumedMessageDestinations, status: Status) {
    let d = match destination {
        ConsumedMessageDestinations::DATABASE => "database",
        ConsumedMessageDestinations::KAFKA => "kafka",
        ConsumedMessageDestinations::DROPPED => "dropped",
    };
    let s = match status {
        Status::ERROR => "error",
        Status::SUCCESS => "success",
    };
    METRICS
        .msg_consume_seconds
        .record(process_time, &[KeyValue::new("destination", d), KeyValue::new("status", s)]);
}

pub fn record_msg_consume_latency(latency: f64, partition: i32) {
    METRICS
        .msg_consume_latency_seconds
        .record(latency, &[KeyValue::new("partition", partition.to_string())]);
}

use super::super::register_telemetry::DEFAULT_OTEL_SERVICE_NAME;
use opentelemetry::{global, metrics::Histogram, KeyValue};
use rdkafka::Message;
use std::sync::LazyLock;
use std::time::UNIX_EPOCH;

pub enum ConsumedMessageDestinations {
    KAFKA,
    DATABASE,
    DROPPED,
}
pub enum Status {
    SUCCESS,
    ERROR,
}

struct Metrics {
    msg_consume_seconds: Histogram<f64>,
    msg_consume_latency_seconds: Histogram<f64>,
    // Will add fr in the next PR
    //    msg_publish_seconds: Histogram<f64>,
    //    msg_jitter_seconds: Histogram<f64>,
    //    msg_resets: Counter<u64>,
}

impl Metrics {
    fn new() -> Self {
        let meter = global::meter(DEFAULT_OTEL_SERVICE_NAME);
        Self {
            // input consumption metrics
            msg_consume_seconds: meter
                .f64_histogram("msg.consume.service.time")
                .with_description("Service time after receiving a message from the input queue")
                .with_boundaries(vec![0.01, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 2.5, 5.0])
                .with_unit("s")
                .build(),
            msg_consume_latency_seconds: meter // Aka "lag time"
                .f64_histogram("msg.consume.latency")
                .with_description(
                    "Message latency on the input queue. Recorded from the start of the message handler function (does not include processing time)",
                )
                .with_boundaries(vec![0.01, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 2.5, 5.0])
                .with_unit("s")
                .build(),
            //            msg_publish_seconds: meter
            //                .f64_histogram("msg.process.service.time")
            //                .with_description("The service time of \"message ready\" processing loop. Only increments when messages are found")
            //                .with_unit("s")
            //                .build(),
            //            msg_jitter_seconds: meter
            //                .f64_histogram("msg.jitter")
            //                .with_unit("s")
            //                .with_description("The delta between the desired published time and confirmed published time of messages to the output topic")
            //                .build(),
            //            msg_resets: meter.u64_counter("msg.reset").with_description("The count of messages reset").build(),
        }
    }
}

static METRICS: LazyLock<Metrics> = LazyLock::new(Metrics::new);

fn record_msg_consume(process_time: f64, destination: &str, status: &str) {
    METRICS.msg_consume_seconds.record(
        process_time,
        &[
            KeyValue::new("destination", destination.to_string()),
            KeyValue::new("status", status.to_string()),
        ],
    );
}

fn record_msg_consume_latency(latency: f64, partition: i32) {
    METRICS
        .msg_consume_latency_seconds
        .record(latency, &[KeyValue::new("partition", partition.to_string())]);
}

pub fn record_consumer_metrics(
    start: &std::time::SystemTime,
    duration: &std::time::Duration,
    message: &rdkafka::message::BorrowedMessage<'_>,
    destination: &ConsumedMessageDestinations,
    status: &Status,
) {
    let d = match destination {
        ConsumedMessageDestinations::DATABASE => "database",
        ConsumedMessageDestinations::KAFKA => "kafka",
        ConsumedMessageDestinations::DROPPED => "dropped",
    };
    let s = match status {
        Status::ERROR => "error",
        Status::SUCCESS => "success",
    };
    record_msg_consume(duration.as_secs_f64(), d, s);
    // Requires error handling because we are using system time (time can go backwards!)
    match message.timestamp().to_millis() {
        Some(msg_ts) => match start.duration_since(UNIX_EPOCH) {
            Ok(dur) => {
                let delta_sec = dur.as_secs_f64() - (msg_ts / 1000) as f64;
                record_msg_consume_latency(delta_sec as f64, message.partition());
            }
            Err(e) => log::error!("metrics: system time error: {}", e),
        },
        None => {
            log::error!(
                "metrics: no message timestamp for message {} on partition {}",
                message.offset(),
                message.partition()
            );
        }
    }
}

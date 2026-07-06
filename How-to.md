# How to work with Chronos
- [How to run Chronos binary](#run-binary)
- [How to run Chronos in a docker container](#run-chronos-docker-image)
- [Environment Variables](#env-vars)

## Pre-requisites
For starting the delay queue process, Chronos expects a DB in Postgres and two topics one for input and other for publishing the messages after delay to be already created. The names of the topics and DB should be passed as env variables mentioned in [Env vars](#env-vars)
Input messages with headers
- chronosMessageId
- chronosDeadline

 will be processed for a delay depending on deadline header to be published on the output topic after the delay is acheived.

`Messages missing any of two above mentioned headers will be discarded.`
## Run Binary
1. Start Kafka brokers and Postgres server on local dev machine
2. Delete any existing .env file, use `make withenv RECIPE=run` 

## Run Chronos docker image 
Using [docker-compose](./docker-compose.yml) docker conatiner can host Chronos image with mentioned env variables for Kafka, PG and Chronos configuration variables.

Use `make withenv RECIPE=docker.up`

## ENV vars
All the required configurations for Chronos can be passed in environment variables mentioned below 

### Required Vars
|Env Var|Example Value| 
|----|----|
|KAFKA_HOST|"localhost"
|KAFKA_PORT|9093
|KAFKA_CLIENT_ID|"chronos"
|KAFKA_GROUP_ID|"chronos"
|KAFKA_IN_TOPIC|"chronos.in"
|KAFKA_OUT_TOPIC|"chronos.out"
|KAFKA_USERNAME|
|KAFKA_PASSWORD|
|PG_HOST|localhost
|PG_PORT|5432
|PG_USER|admin
|PG_PASSWORD|admin
|PG_DATABASE|chronos_db
|PG_POOL_SIZE|50

### Optional Vars
These values are set to fine tune performance Chrono in need, refer to [Chronos](./README.md)
|Env Var| Default Value|
|----|----|
| MONITOR_DB_POLL|5 sec
| PROCESSOR_DB_POLL|5 milli sec
| TIMING_ADVANCE|0 sec
| FAIL_DETECT_INTERVAL|10 sec
| HEALTHCHECK_FILE|healthcheck/chronos_healthcheck


## Observability

### Tracing

Chronos supports sending [batches](https://opentelemetry.io/docs/specs/otel/configuration/sdk-environment-variables/#batch-span-processor) of traces using OTLP over `grpc`, `http/json` or `http/protobuff`.

The recommended environment variable configuration for tracing is:

|Env var| Default Value|
|---|--|
|OTEL_SERVICE_NAME|chronos|
|OTEL_TRACES_EXPORTER|otlp|
|OTEL_EXPORTER_OTLP_ENDPOINT|http://{localhost\|OTEL_COLLECTOR_HOST}:{4317\|OTLP_GRPC_PORT}|
|OTEL_EXPORTER_OTLP_PROTOCOL|grpc|

OpenTelemetry span creation and exporting and can be "disabled" by setting:
```env
# Calls to start and end spans recordings are no-oped
OTEL_TRACES_SAMPLER="always_off"
# Calls to the trace exporter are no-oped
OTEL_EXPORTER_OTLP_PROTOCOL="none"
```

For more information on opentelemetry environment variables see the [opentelemetry_otlp crate docs](https://docs.rs/opentelemetry-otlp/0.32.0/opentelemetry_otlp/#environment-variables)

### Metrics

Chronos supports exporting metrics via Prometheus.\
To enable it, set `OTEL_METRICS_EXPORTER` to `prometheus`.\
[See the OpenTelemetry SDK environment variable specification for additional Prometheus configuration options](https://opentelemetry.io/docs/specs/otel/configuration/sdk-environment-variables/#prometheus-exporter).

### Logging

Logs are written to [`stderr`](https://en.wikipedia.org/wiki/Standard_streams#Standard_error_(stderr))

## Chronos Images 
Two images are published for each [RELEASE]( `https://github.com/kindredgroup/chronos/pkgs/container/chronos`)
- migrations image 
- chornos image 









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
<<<<<<< HEAD

### Required Vars
|Env Var|Example Value| 
|----|----|
|KAFKA_HOST|"localhost"
|KAFKA_PORT|9093
|  KAFKA_CLIENT_ID|"chronos"
|  KAFKA_GROUP_ID|"chronos"
|  KAFKA_IN_TOPIC|"chronos.in"
|  KAFKA_OUT_TOPIC|"chronos.out"
|  KAFKA_USERNAME|
|  KAFKA_PASSWORD|
|  PG_HOST|localhost
|  PG_PORT|5432
|  PG_USER|admin
|  PG_PASSWORD|admin
|  PG_DATABASE|chronos_db
|  PG_POOL_SIZE|50

### Optional Vars
These values are set to fine tune performance Chrono in need, refer to [Chronos](./README.md)
|Env Var| Default Value|
|----|----|
| MONITOR_DB_POLL|5 sec
| PROCESSOR_DB_POLL|5 milli sec
| TIMING_ADVANCE|0 sec
| FAIL_DETECT_INTERVAL|10 sec
=======
|Env Var|Example Value| Required|
|----|----|----|
|KAFKA_BROKERS|"localhost:9093"|True
|  KAFKA_CLIENT_ID|"chronos"|True
|  KAFKA_GROUP_ID|"chronos"|True
|  KAFKA_IN_TOPIC|"chronos.in"|True
|  KAFKA_OUT_TOPIC|"chronos.out"|True
|  KAFKA_USERNAME||True
|  KAFKA_PASSWORD||True
|  PG_HOST|localhost|True
|  PG_PORT|5432|True
|  PG_USER|admin|True
|  PG_PASSWORD|admin|True
|  PG_DATABASE|chronos_db|True
|  PG_POOL_SIZE|50|True
|NODE_ID|UUID|False
| DELAY_TIME|0|False
| RANDOMNESS_DELAY|100|False
| MONITOR_DB_POLL|5|False
| TIMING_ADVANCE|0|False
| FAIL_DETECT_INTERVAL|500|False
>>>>>>> 6107c18 (fix: handle retry and graceful close for all threads if one is stopped)


## Observability
At this time Chronos supports Http protocol based connectivity to the Otel collector. By providing following env variables for connecting to the Otel collector instance, traces will appear under the service name mentioned.
|Env var| Default Value|
|---|--|
|   OTEL_SERVICE_NAME|Chronos|
|   OTEL_TRACES_EXPORTER|otlp|
|   OTEL_EXPORTER_OTLP_TRACES_ENDPOINT|"http://localhost:4318/v1/traces"
|   OTEL_EXPORTER_OTLP_PROTOCOL|"http/json"

## Chronos Images 
Two images are published for each [RELEASE]( `https://github.com/kindredgroup/chronos/pkgs/container/chronos`)
- migrations image 
- chornos image 









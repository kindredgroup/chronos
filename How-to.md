# How to work with Chronos
- [How to run Chronos binary](#run-binary)
- [How to run Chronos in a docker container](#run-chronos-docker-image)

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
| DELAY_TIME|0|False
| RANDOMNESS_DELAY|100|False
| MONITOR_DB_POLL|5|False
| TIMING_ADVANCE|0|False
| FAIL_DETECT_INTERVAL|500|False



## Chronos Images 
Two images are published for each release to `https://github.com/kindredgroup/chronos/pkgs/container/chronos`
- migrations image 
- chornos image 









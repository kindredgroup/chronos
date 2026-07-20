#!/usr/bin/env bash

# Produces a message every 5 seconds
# tested in image:
# apache/kafka:4.1.1
while true
do
    for i in "-5" "+10";
    do
    echo "chronosMessageId:$(cat /proc/sys/kernel/random/uuid),chronosDeadline:$(date --date="$i seconds" --iso-8601=seconds) $(cat /proc/sys/kernel/random/uuid)::{\"msg\": \"I'm a msg!\"}" > /tmp/msg.$i ;
    /opt/kafka/bin/kafka-console-producer.sh \
        --topic "chronos.in" \
        --property "parse.key=true" \
        --property "parse.headers=true" \
        --property "key.separator=::" \
        --property "headers.delimiter= " \
        --bootstrap-server "kafka:9092" < /tmp/msg.$i ;
    done;
    sleep 10;
done;

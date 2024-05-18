#!/bin/bash

MASTER_PORT=7000
SLAVE_PORT=7001

./spawn_redis_server.sh --port $MASTER_PORT &
MASTER_PID=$!

redis-cli -p $MASTER_PORT PING

# ./spawn_redis_server.sh --port $SLAVE_PORT --replicaof localhost $MASTER_PORT &
# SLAVE_PID=$!
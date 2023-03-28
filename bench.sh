#!/bin/bash

# Run a simple benchmark, with only a single thread and single open connection.
wrk -t 1 -c 1 -d 10s http://localhost:8080/single/0
wrk -t 1 -c 1 -d 10s http://localhost:8080/single/3

wrk -t 1 -c 1 -d 10s http://localhost:8080/body/0
wrk -t 1 -c 1 -d 10s http://localhost:8080/body/1
wrk -t 1 -c 1 -d 10s http://localhost:8080/body/3

wrk -t 1 -c 1 -d 10s http://localhost:8080/delayed_body/0
wrk -t 1 -c 1 -d 10s http://localhost:8080/delayed_body/1
wrk -t 1 -c 1 -d 10s http://localhost:8080/delayed_body/3
echo "Done"
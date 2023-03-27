#!/bin/bash

# Run a simple benchmark, with only a single thread and single open connection.
wrk -t 1 -c 1 -d 20s http://localhost:8080/serve_dir/basic.html
wrk -t 1 -c 1 -d 20s http://localhost:8080/read/basic.html

wrk -t 1 -c 1 -d 20s http://localhost:8080/serve_dir/scout.webp
wrk -t 1 -c 1 -d 20s http://localhost:8080/read/scout.webp
echo "Done"
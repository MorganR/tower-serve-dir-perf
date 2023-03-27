#!/bin/bash

wrk -t 1 -c 1 -d 20s http://localhost:8080/serve_dir/basic.html > results.txt
wrk -t 1 -c 1 -d 20s http://localhost:8080/read/basic.html >> results.txt

wrk -t 1 -c 1 -d 20s http://localhost:8080/serve_dir/scout.webp >> results.txt
wrk -t 1 -c 1 -d 20s http://localhost:8080/read/scout.webp >> results.txt

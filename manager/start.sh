#!/bin/bash
if [ -z "$1" ]
  then
    echo "Missing daemon IP"
fi
RUST_LOG=info,cargo=warn,manager=trace,jsonrpc_core=trace,jsonrpc=trace cargo run -- -b 127.0.0.1:9000 -c 0.0.0.0:1336 -d $1:1338 -j 0.0.0.0:1337

#!/bin/bash

RUST_BACKTRACE=1 cargo build --release;
rm ~/.ts3client/plugins/libyamba_plugin.so
cp ./target/release/libyamba_plugin.so ~/.ts3client/plugins/
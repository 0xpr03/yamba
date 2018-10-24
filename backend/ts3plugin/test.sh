#!/bin/bash

RUST_BACKTRACE=1 cargo build;
rm ~/.ts3client/plugins/libyamba_plugin.so
cp ./target/debug/libyamba_plugin.so ~/.ts3client/plugins/
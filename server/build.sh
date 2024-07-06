#!/bin/bash
set -e

BASE_DIR=$(cd `dirname "$0"`; pwd)
cd $BASE_DIR

# valid command-line: RUSTFLAGS="--cfg tokio_unstable" && cargo build
# see:https://github.com/wl4g-collect/tokio-rs-console/tree/main/console-subscriber#enabling-tokio-instrumentation
RUSTFLAGS="--cfg tokio_unstable" && cargo build
#!/bin/bash

source "$(dirname "$0")/source.sh"
cargo run -p deluge-cli -- "$@"
